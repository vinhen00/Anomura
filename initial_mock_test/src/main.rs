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

impl rustc_span::source_map::FileLoader for MyFileLoader {
    fn file_exists(&self, path: &std::path::Path) -> bool {
        path == std::path::Path::new("mock_test.rs") || path == std::path::Path::new("mock_defs.rs")
    }

    fn read_file(&self, path: &std::path::Path) -> std::io::Result<String> {
        if path == std::path::Path::new("mock_test.rs") {
            let mut file = std::fs::File::open("src/mock_test.rs")?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            Ok(contents)
        } else if path == std::path::Path::new("mock_defs.rs") {
            let mut file = std::fs::File::open("src/mock_defs.rs")?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            Ok(contents)
        } else {
            Err(std::io::Error::other("oops"))
        }
    }

    fn read_binary_file(&self, _path: &std::path::Path) -> std::io::Result<std::sync::Arc<[u8]>> {
        Err(std::io::Error::other("oops"))
    }

    fn current_directory(&self) -> Result<std::path::PathBuf, std::io::Error> {
        Ok(std::path::PathBuf::from("."))
    }
}
// MutVisitor to replace function calls from "dub" to "mocked_dub"
struct MockReplacer{
    mocklist: Vec<String>,
}

impl MutVisitor for MockReplacer {
    fn visit_expr(&mut self, expr: &mut rustc_ast::Expr) {
        if let rustc_ast::ExprKind::Call(func, _args) = &mut expr.kind {
            if let rustc_ast::ExprKind::Path(_, path) = &mut func.kind {
                if let Some(seg) = path.segments.first_mut() {
                    let funcname = seg.ident.name.to_string();
                    if self.mocklist.contains(&funcname) {
                        let mockname = format!("mocked_{}", funcname);
                        seg.ident = Ident::new(rustc_span::Symbol::intern(&mockname), seg.ident.span);
                    }
                }
            }
        }
        rustc_ast::mut_visit::walk_expr(self, expr);
    }
}


struct CompileMocks {
    mocks: Vec<(rustc_span::symbol::Ident, std::boxed::Box<rustc_ast::Block>)>,
}

impl rustc_driver::Callbacks for CompileMocks {
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
        for item in &krate.items {
            if let rustc_ast::ItemKind::Fn(fn_data) = &item.kind {
                if let Some(block) = &fn_data.body {
                    let body = block;
                    let id = fn_data.ident;
                    if id.name.as_str() != "main" {
                        println!("Found function {}", id.name);
                        self.mocks.push((id.clone(), block.clone()));
                        println!("{:#?}", fn_data.sig)

                    }

                }        
            }

        }
        Compilation::Stop
    }
}

struct FunctionIntercept{
    mocks: Vec<(rustc_span::symbol::Ident, std::boxed::Box<rustc_ast::Block>)>,
}

impl rustc_driver::Callbacks for FunctionIntercept {
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
        for item in &mut krate.items {
            if let rustc_ast::ItemKind::Fn(fn_data) = &mut item.kind {
                for (ident, block) in &self.mocks{
                    println!("{:#?} compared to {:#?}", ident, fn_data.ident);
                    if fn_data.ident.name.as_str() == ident.name.as_str() {
                        println!("Mocking {:#?}", fn_data.ident);
                        fn_data.body = Some(block.clone());
                    }
                }
            }

        }
        Compilation::Continue
    }
}

fn main() {
    let mut mockedFuns = CompileMocks {mocks: Vec::new()};
    run_compiler(
        &[
            "ignored".to_string(),
            "mock_defs.rs".to_string(),
            "--crate-type".to_string(),
            "bin".to_string(),
            "-o".to_string(),
            "./target/mocked_main".to_string(),
        ],
        &mut mockedFuns,
    );
    let mut insertion = FunctionIntercept {mocks: mockedFuns.mocks};
    run_compiler(
        &[
            "ignored".to_string(),
            "mock_test.rs".to_string(),
            "--crate-type".to_string(),
            "bin".to_string(),
            "-o".to_string(),
            "./target/mocked_main".to_string(),
        ],
        &mut insertion,
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