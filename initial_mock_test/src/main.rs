// Tested with nightly-2025-03-28

#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_ast_pretty;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_error_codes;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::fs::File;
use std::io::Read;

use rustc_ast::mut_visit::MutVisitor;
use rustc_ast_pretty::pprust::item_to_string;
use rustc_driver::{Compilation, run_compiler};
use rustc_interface::interface::{Compiler, Config};
use rustc_middle::ty::TyCtxt;
use rustc_session::config::CrateType;
use rustc_span::symbol::Ident;

struct MyFileLoader;

// MutVisitor to replace function calls from "dub" to "mocked_dub"
struct DubReplacer;

impl MutVisitor for DubReplacer {
    fn visit_expr(&mut self, expr: &mut rustc_ast::Expr) {
        if let rustc_ast::ExprKind::Call(func, _args) = &mut expr.kind {
            if let rustc_ast::ExprKind::Path(_, path) = &mut func.kind {
                if let Some(seg) = path.segments.first_mut() {
                    if seg.ident.name.as_str() == "dub" {
                        seg.ident = Ident::new(rustc_span::Symbol::intern("mocked_dub"), seg.ident.span);
                    }
                }
            }
        }
        rustc_ast::mut_visit::walk_expr(self, expr);
    }
}

impl rustc_span::source_map::FileLoader for MyFileLoader {
    fn file_exists(&self, path: &Path) -> bool {
        path == Path::new("mock_test.rs")
    }

    fn read_file(&self, path: &Path) -> io::Result<String> {
        if path == Path::new("mock_test.rs") {
            let mut file = File::open("src/mock_test.rs")?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            Ok(contents)
        } else {
            Err(io::Error::other("oops"))
        }
    }

    fn read_binary_file(&self, _path: &Path) -> io::Result<Arc<[u8]>> {
        Err(io::Error::other("oops"))
    }

    fn current_directory(&self) -> Result<PathBuf, std::io::Error> {
        Ok(PathBuf::from("."))
    }
}

struct MyCallbacks;

impl rustc_driver::Callbacks for MyCallbacks {
    fn config(&mut self, config: &mut Config) {
        config.file_loader = Some(Box::new(MyFileLoader));
        config.opts.crate_types = vec![CrateType::Executable];
        // Set output directory
        config.opts.search_paths.clear();
    }

    fn after_crate_root_parsing(
        &mut self,
        _compiler: &Compiler,
        krate: &mut rustc_ast::Crate,
    ) -> Compilation {
        // Print original AST
        println!("=== BEFORE MODIFICATIONS ===");
        for item in &krate.items {
            println!("{}", item_to_string(&item));
        }

        // Replace dub with mocked_dub in the AST
        let mut replacer = DubReplacer;
        replacer.visit_crate(krate);

        // Print modified AST to verify changes
        println!("\n=== AFTER MODIFICATIONS ===");
        for item in &krate.items {
            println!("{}", item_to_string(&item));
        }

        Compilation::Continue
    }

    fn after_analysis(&mut self, _compiler: &Compiler, tcx: TyCtxt<'_>) -> Compilation {
        // Iterate over the top-level items in the crate, looking for the main function.
        for id in tcx.hir_free_items() {
            let item = &tcx.hir_item(id);
            // Use pattern-matching to find a specific node inside the main function.
            if let rustc_hir::ItemKind::Fn { body, .. } = item.kind {
                let expr = &tcx.hir_body(body).value;
                if let rustc_hir::ExprKind::Block(block, _) = expr.kind {
                    for stmt in block.stmts {   //loop through stmts in main
                        if let rustc_hir::StmtKind::Let(let_stmt) = stmt.kind {
                            if let Some(expr) = let_stmt.init { //find let stmts
                                let hir_id = expr.hir_id;
                                let def_id = item.hir_id().owner.def_id;
                                let ty = tcx.typeck(def_id).node_type(hir_id);
                                if let rustc_hir::ExprKind::Call(func,_args) = expr.kind { //only check call expr
                                    if let rustc_hir::ExprKind::Path(rustc_hir::QPath::Resolved(_,path)) = func.kind {
                                        let func_ident = path.segments[0].ident;
                                        let func_name = func_ident.name.as_str();
                                        if func_name == "mocked_dub" {
                                            println!("{func_ident} has been mocked");
                                        }
                                        println!("{expr:#?}: {ty:?}");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Compilation::Continue
    }
}

fn main() {
    run_compiler(
        &[
            "ignored".to_string(),
            "mock_test.rs".to_string(),
            "--crate-type".to_string(),
            "bin".to_string(),
            "-o".to_string(),
            "./target/mocked_main".to_string(),
        ],
        &mut MyCallbacks,
    );
    
    // Run the compiled executable
    println!("\n=== RUNNING COMPILED PROGRAM ===");
    let output = Command::new("./target/mocked_main")
        .output();
    
    match output {
        Ok(output) => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            if !output.stderr.is_empty() {
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            println!("Failed to run executable: {}", e);
        }
    }
}