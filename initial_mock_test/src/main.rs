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

use std::collections::HashMap;


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
    idents: Vec<String>,
}

//Will find all symbols and save as string
impl MutVisitor for SymbolFinder { 
    fn visit_expr(&mut self, expr: &mut rustc_ast::Expr) {
        if let rustc_ast::ExprKind::Lit(literal) = &mut expr.kind {
            self.symbols.push(literal.symbol.as_str().to_string());

            
        }
        rustc_ast::mut_visit::walk_expr(self, expr);
    }

    fn visit_mac_call(&mut self, node: &mut rustc_ast::MacCall) {
        self.visit_path(&mut node.path);
        for tree in node.args.tokens.iter() {
            if let rustc_ast::tokenstream::TokenTree::Token(token, _) = tree {
                if let rustc_ast::token::TokenKind::Literal(lit) = &token.kind {
                    if let rustc_ast::token::LitKind::Str = lit.kind {
                        //println!("Found symbol in MacCall: {}", lit.symbol.as_str().to_string());
                        self.symbols.push(lit.symbol.as_str().to_string());
                    }
                }
            }
        }
    }

    fn visit_path(&mut self, path: &mut rustc_ast::Path) {
        for i in &path.segments {
            println!("Found ident {:#?}", i.ident);
            self.idents.push(i.ident.name.as_str().to_string())
        }

        rustc_ast::mut_visit::walk_path(self, path);
    }

    fn visit_pat(&mut self, pat: &mut rustc_ast::Pat) {
        if let rustc_ast::PatKind::Ident(_, ident, _) = pat.kind {
            println!("Found ident {:#?}", ident);
            self.idents.push(ident.name.as_str().to_string())  
        }
        rustc_ast::mut_visit::walk_pat(self, pat);
    }
}


struct SymbolFixer{
    symbols: Vec<String>,
    idents: Vec<String>,
    dict: HashMap<String, rustc_span::Symbol>,
}

//Will find all symbols and fix their strings
impl MutVisitor for SymbolFixer { 
    fn visit_expr(&mut self, expr: &mut rustc_ast::Expr) {
        if let rustc_ast::ExprKind::Lit(literal) = &mut expr.kind {
            let string = self.symbols.remove(0);
            //println!("Have symbol: {}", string);
            literal.symbol = rustc_span::Symbol::intern(&string);
            
        }
        rustc_ast::mut_visit::walk_expr(self, expr);
    }
    fn visit_mac_call(&mut self, node: &mut rustc_ast::MacCall) {
        self.visit_path(&mut node.path);
        let mut trees: Vec<_> = node.args.tokens.iter().cloned().collect();
        for tree in &mut trees {
            if let rustc_ast::tokenstream::TokenTree::Token(token, _) = tree {
                if let rustc_ast::token::TokenKind::Literal(lit) = &mut token.kind {
                    if let rustc_ast::token::LitKind::Str = lit.kind {
                        if let string = self.symbols.remove(0) {
                            //println!("Restoring symbol in MacCall: {}", string);
                            lit.symbol = rustc_span::Symbol::intern(&string);
                        }
                    }
                }
            }
        }
        node.args.tokens = rustc_ast::tokenstream::TokenStream::new(trees);
    }
    fn visit_path(&mut self, path: &mut rustc_ast::Path) {
        for i in &mut path.segments {
            let mut name = self.idents.remove(0);
            println!("Fixing ident {}", name);
            match self.dict.get(&name) {
                Some(symb) => {i.ident.name = *symb;}
                None => {
                    let symb = rustc_span::Symbol::intern(name.as_str());
                    self.dict.insert(name, symb);
                    i.ident.name = symb;
                }
            }
        }
        rustc_ast::mut_visit::walk_path(self, path);
    }

    fn visit_pat(&mut self, pat: &mut rustc_ast::Pat) {
        if let rustc_ast::PatKind::Ident(_, ident, _) = &mut pat.kind {
            let mut name = self.idents.remove(0);
            println!("Fixing ident {}", name);
            match self.dict.get(&name) {
                Some(symb) => {ident.name = *symb;}
                None => {
                    let symb = rustc_span::Symbol::intern(name.as_str());
                    self.dict.insert(name, symb);
                    ident.name = symb;
                }
            } 
        }
        rustc_ast::mut_visit::walk_pat(self, pat);
    }
}


struct CompileMocks {
    mocks: Vec<(String, std::boxed::Box<rustc_ast::Block>)>,
    symbols: Vec<String>,
    idents: Vec<String>,
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
        let mut visitor = SymbolFinder{ symbols: Vec::new(), idents: Vec::new() };
        for item in &krate.items {
            if let rustc_ast::ItemKind::Fn(fn_data) = &item.kind {
                if let Some(block) = &fn_data.body {
                    let body = block;
                    let id = fn_data.ident;
                    if id.name.as_str() != "main" {
                        println!("Compiling mock {}", fn_data.ident.name.as_str());
                        visitor.visit_fn_decl(&mut fn_data.sig.decl.clone());
                        visitor.visit_block(&mut *body.clone());
                        self.symbols = visitor.symbols.clone();
                        self.idents = visitor.idents.clone();
                        self.mocks.push((id.name.as_str().to_string(), block.clone()));
                        // println!("{:#?}", item);

                    }

                }        
            }

        }
        Compilation::Stop
    }
}

struct FunctionIntercept{
    mocks: Vec<(String, std::boxed::Box<rustc_ast::Block>)>,
    symbols: Vec<String>,
    idents: Vec<String>,
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
        //println!("{:#?}", krate);

        let mut visitor = SymbolFixer {symbols: self.symbols.clone(), idents: self.idents.clone(), dict: HashMap::new()};
        for item in &mut krate.items {
            if let rustc_ast::ItemKind::Fn(fn_data) = &mut item.kind {
                for (ident, block) in &self.mocks{
                    println!("Looked into {}, compared to {}", fn_data.ident.name.as_str(), ident);
                    if fn_data.ident.name.as_str().to_string() == *ident {
                        println!("Mocking {}", fn_data.ident.name.as_str());
                        fn_data.body = Some(block.clone());
                        match &mut fn_data.body {
                            Some(body) => { //once told me
                                visitor.visit_fn_decl(&mut fn_data.sig.decl.clone());
                                visitor.visit_block(body);
                                println!("{:#?}", fn_data);

                            }
                            None => {}

                        }

                    }
                }            
            }
        }
        //println!("{:#?}", krate);

        Compilation::Continue
    }
}

fn main() {
    let mut mockedFuns = CompileMocks {mocks: Vec::new(), symbols: Vec::new(), idents: Vec::new()};
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

    let mut insertion = FunctionIntercept {mocks: mockedFuns.mocks.clone(), symbols: mockedFuns.symbols.clone(), idents: mockedFuns.idents.clone()};
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