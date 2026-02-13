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
struct SymbolFinder{
    symbols: Vec<String>,
}

//Will find all symbols and save as string
impl MutVisitor for SymbolFinder { 
    fn visit_expr(&mut self, expr: &mut rustc_ast::Expr) {
        if let rustc_ast::ExprKind::Lit(literal) = &mut expr.kind {
            match &literal.kind {
                rustc_ast::token::LitKind::Str => {
                    println!("Found symbol in literal: {}", literal.symbol.as_str().to_string());
                    self.symbols.push(literal.symbol.as_str().to_string());
                }
                _ => {} 
            }
            
        }
        rustc_ast::mut_visit::walk_expr(self, expr);
    }

    fn visit_mac_call(&mut self, node: &mut rustc_ast::MacCall) {
        for tree in node.args.tokens.iter() {
            if let rustc_ast::tokenstream::TokenTree::Token(token, _) = tree {
                if let rustc_ast::token::TokenKind::Literal(lit) = &token.kind {
                    if let rustc_ast::token::LitKind::Str = lit.kind {
                        println!("Found symbol in MacCall: {}", lit.symbol.as_str().to_string());
                        self.symbols.push(lit.symbol.as_str().to_string());
                    }
                }
            }
        }
    }
}

struct SymbolFixer{
    symbols: Vec<String>,
}

//Will find all symbols and fix their strings
impl MutVisitor for SymbolFixer { 
    fn visit_expr(&mut self, expr: &mut rustc_ast::Expr) {
        if let rustc_ast::ExprKind::Lit(literal) = &mut expr.kind {
            match &literal.kind {
                rustc_ast::token::LitKind::Str => {
                    let string = self.symbols.remove(0);
                    println!("Have symbol: {}", string);
                    literal.symbol = rustc_span::Symbol::intern(&string);
                    }
                    
                
                _ => {} 
            }
            
        }
        rustc_ast::mut_visit::walk_expr(self, expr);
    }
    fn visit_mac_call(&mut self, node: &mut rustc_ast::MacCall) {
        let mut trees: Vec<_> = node.args.tokens.iter().cloned().collect();
        for tree in &mut trees {
            if let rustc_ast::tokenstream::TokenTree::Token(token, _) = tree {
                if let rustc_ast::token::TokenKind::Literal(lit) = &mut token.kind {
                    if let rustc_ast::token::LitKind::Str = lit.kind {
                        if let string = self.symbols.remove(0) {
                            println!("Restoring symbol in MacCall: {}", string);
                            lit.symbol = rustc_span::Symbol::intern(&string);
                        }
                    }
                }
            }
        }
        node.args.tokens = rustc_ast::tokenstream::TokenStream::new(trees);
    }
}


struct CompileMocks {
    mocks: Vec<(rustc_span::symbol::Ident, std::boxed::Box<rustc_ast::Block>)>,
    symbols: Vec<String>,
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
        let mut visitor = SymbolFinder{ symbols: Vec::new() };
        for item in &krate.items {
            if let rustc_ast::ItemKind::Fn(fn_data) = &item.kind {
                if let Some(block) = &fn_data.body {
                    let body = block;
                    let id = fn_data.ident;
                    if id.name.as_str() != "main" {
                        visitor.visit_block(&mut *body.clone());
                        self.symbols = visitor.symbols.clone();
                        self.mocks.push((id.clone(), block.clone()));
                        //println!("{:#?}", block);

                    }

                }        
            }

        }
        Compilation::Stop
    }
}

struct FunctionIntercept{
    mocks: Vec<(rustc_span::symbol::Ident, std::boxed::Box<rustc_ast::Block>)>,
    symbols: Vec<String>,

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
        let mut visitor = SymbolFixer {symbols: self.symbols.clone()};
        for item in &mut krate.items {
            if let rustc_ast::ItemKind::Fn(fn_data) = &mut item.kind {
                for (ident, block) in &self.mocks{
                    if fn_data.ident.name.as_str() == ident.name.as_str() {
                        fn_data.body = Some(block.clone());
                        match &mut fn_data.body {
                            Some(body) => { 
                                visitor.visit_block(body);
                                //println!("{:#?}", body);

                            }
                            None => {}

                        }

                    }
                }
            }

        }
        Compilation::Continue
    }
}

fn main() {
    let mut mockedFuns = CompileMocks {mocks: Vec::new(), symbols: Vec::new()};
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

    let mut insertion = FunctionIntercept {mocks: mockedFuns.mocks.clone(), symbols: mockedFuns.symbols.clone()};
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