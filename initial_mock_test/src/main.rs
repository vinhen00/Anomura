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

struct MyFileLoader{file: String}

impl rustc_span::source_map::FileLoader for MyFileLoader {
    fn file_exists(&self, path: &std::path::Path) -> bool {
        path == std::path::Path::new(&self.file)
    }

    fn read_file(&self, path: &std::path::Path) -> std::io::Result<String> {
        if path == std::path::Path::new(&self.file) {
            let mut file = std::fs::File::open(format!("src/{}", self.file))?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            Ok(contents)
        } else {
            Err(std::io::Error::other("Could not open file"))
        }
    }

    fn read_binary_file(&self, _path: &std::path::Path) -> std::io::Result<std::sync::Arc<[u8]>> {
        Err(std::io::Error::other("Could not open file"))
    }

    fn current_directory(&self) -> Result<std::path::PathBuf, std::io::Error> {
        Ok(std::path::PathBuf::from("."))
    }
}
struct SymbolFinder{
    symbols: Vec<String>,
    idents: Vec<String>,
}

//SymbolFinder finds symbols and Idents from an AST
impl MutVisitor for SymbolFinder { 

    //For expressions the only special case we need is literals. 
    //Identifiers are covered by visit_path as all identifiers are paths
    fn visit_expr(&mut self, expr: &mut rustc_ast::Expr) {
        if let rustc_ast::ExprKind::Lit(literal) = &mut expr.kind {
            self.symbols.push(literal.symbol.as_str().to_string());
        }
        rustc_ast::mut_visit::walk_expr(self, expr);
    }

    //Mac calls not stricly necessary, but nice for debug to be able to print in mock functions
    fn visit_mac_call(&mut self, node: &mut rustc_ast::MacCall) {
        self.visit_path(&mut node.path);
        for tree in node.args.tokens.iter() {
            if let rustc_ast::tokenstream::TokenTree::Token(token, _) = tree {
                if let rustc_ast::token::TokenKind::Literal(lit) = &token.kind {
                    if let rustc_ast::token::LitKind::Str = lit.kind {
                        self.symbols.push(lit.symbol.as_str().to_string());
                    }
                }
            }
        }
    }

    // Will collect ALL identifiers(including keywords and types) but doing this doesn't seem to cause any problems
    fn visit_path(&mut self, path: &mut rustc_ast::Path) {
        for i in &path.segments {
            self.idents.push(i.ident.name.as_str().to_string())
        }
        rustc_ast::mut_visit::walk_path(self, path);
    }

    fn visit_pat(&mut self, pat: &mut rustc_ast::Pat) {
        if let rustc_ast::PatKind::Ident(_, ident, _) = pat.kind {
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

//SymbolFixer will walk through an AST and fix all Identifiers and Symbols
impl MutVisitor for SymbolFixer { 
    fn visit_expr(&mut self, expr: &mut rustc_ast::Expr) {
        if let rustc_ast::ExprKind::Lit(literal) = &mut expr.kind {
            let string = self.symbols.remove(0);
            literal.symbol = rustc_span::Symbol::intern(&string); //Create new symbol in registry
        }
        rustc_ast::mut_visit::walk_expr(self, expr);
    }

    //Reconstructing MacCalls is a headache as the tokenstream only has private fields and non mutable methods
    //What we do instead is to just to clone the immutables and create a new tokenstream where we fix them
    fn visit_mac_call(&mut self, node: &mut rustc_ast::MacCall) {
        self.visit_path(&mut node.path);
        let mut trees: Vec<_> = node.args.tokens.iter().cloned().collect();
        for tree in &mut trees {
            if let rustc_ast::tokenstream::TokenTree::Token(token, _) = tree {
                if let rustc_ast::token::TokenKind::Literal(lit) = &mut token.kind {
                    if let rustc_ast::token::LitKind::Str = lit.kind {
                        if let string = self.symbols.remove(0) {
                            lit.symbol = rustc_span::Symbol::intern(&string);
                        }
                    }
                }
            }
        }
        node.args.tokens = rustc_ast::tokenstream::TokenStream::new(trees);
    }

    //Both path and pat work by creating a new entry into the dictionary the first them we encounter an identifier
    //Next time we encounter them we lookup the value from the dict
    fn visit_path(&mut self, path: &mut rustc_ast::Path) {
        for i in &mut path.segments {
            let mut name = self.idents.remove(0);
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

// MockedFun is a struct representing a single mocked function and all the information needed to transfer it
// Symbols is a list of all symbols encountered in order, Idents is a list of all identifiers
//
// We need them because literals and identifiers are stored in a compilation context
// and when we switch compiler that data disappears
struct MockedFun {
    name: String,
    sig: rustc_ast::FnSig,
    body: Box<rustc_ast::Block>,
    symbols: Vec<String>,
    idents: Vec<String>,
}

impl MockedFun {

    fn new(foo: Box<rustc_ast::Fn>) -> MockedFun{
        let name = foo.ident.as_str().to_string();
        match foo.body {
            Some(body) => {
                MockedFun { name, sig: foo.sig , body: body, symbols: Vec::new(), idents: Vec::new() }
            }
            None => {panic!()}   
        }
    }

    // This fn creates a visitor that visits the mock function and collects all symbols and identifiers
    fn collect_names(&mut self) {
        let mut visitor = SymbolFinder{symbols: Vec::new(), idents: Vec::new()};
        visitor.visit_fn_decl(&mut self.sig.decl);
        visitor.visit_block(&mut self.body);
        self.symbols = visitor.symbols;
        self.idents = visitor.idents;
    }
    // This fn creates a visitor that visits the mocked function and resolves all the symbols and identifiers
    // It is meant to be called when in the second compilation context
    fn resolve_names(&mut self) {
        let mut visitor = SymbolFixer{symbols: self.symbols.clone(), idents: self.idents.clone(), dict: HashMap::new()};
        visitor.visit_fn_decl(&mut self.sig.decl);
        visitor.visit_block(&mut self.body);
    }
}



struct CompileMocks {
    mocks: Vec<MockedFun>,
}

//Compile mocks is a compiler setting the compiles the file that the mocked functions reside in.
//Will grab all functions defined therein, and store them as a field in the mocks.
//Stops compilation when done
impl rustc_driver::Callbacks for CompileMocks {
    fn config(&mut self, config: &mut Config) {
        config.file_loader = Some(Box::new(MyFileLoader{file: "mock_defs.rs".to_string()}));
        config.opts.crate_types = vec![CrateType::Executable];
        config.opts.search_paths.clear();
    }

    fn after_crate_root_parsing(
        &mut self,
        _compiler: &Compiler,
        krate: &mut rustc_ast::Crate,
    ) -> Compilation {
        for item in &krate.items {
            if let rustc_ast::ItemKind::Fn(fn_data) = &item.kind {
                if fn_data.ident.name.as_str() != "main" {
                    let mut foo = MockedFun::new(fn_data.clone());
                    foo.collect_names();
                    self.mocks.push(foo);
                }        
            }
        }
        Compilation::Stop
    }
}

struct FunctionIntercept{
    mocks: Vec<MockedFun>,
}

impl FunctionIntercept {
    fn checkName(&mut self, name: String ) -> bool {
        for i in &self.mocks {
            if name == i.name {
                return true;
            }
        }
        return false;
    }
}

//Function_intercept is a compiler setting that compiles the target file and replaces the function body of the functions that have a mocked variant
impl rustc_driver::Callbacks for FunctionIntercept {
    fn config(&mut self, config: &mut Config) {
        config.file_loader = Some(Box::new(MyFileLoader{file: "mock_test.rs".to_string()}));
        config.opts.crate_types = vec![CrateType::Executable];
        config.opts.search_paths.clear();
    }

    fn after_crate_root_parsing(
        &mut self,
        _compiler: &Compiler,
        krate: &mut rustc_ast::Crate,
    ) -> Compilation {
        //First create copies of all functions that will be mocked 
        let mut function_originals: Vec<Box<rustc_ast::Item>> = Vec::new();
        for item in &mut krate.items{
            if let rustc_ast::ItemKind::Fn(fn_data) = &item.kind {
                if self.checkName(fn_data.ident.name.as_str().to_string()) {
                    let mut original_function = item.clone();
                    if let rustc_ast::ItemKind::Fn(fn_data) = &mut original_function.kind {
                        let new_name = format!("{}_original", fn_data.ident.name.as_str());
                        fn_data.ident.name = rustc_span::Symbol::intern(&new_name);
                    }
                    function_originals.push(original_function);
                }
            }
        }
        //Then replace the original with their mocked variants
        for item in &mut krate.items {
            if let rustc_ast::ItemKind::Fn(fn_data) = &mut item.kind {
                for foo in &mut self.mocks{
                    if fn_data.ident.name.as_str() == foo.name.as_str() {
                        println!("Mocking {}", foo.name);
                        foo.resolve_names();
                        fn_data.sig.decl = foo.sig.decl.clone();
                        fn_data.body = Some(foo.body.clone());
                    }
                }            
            }
        }
        for func in function_originals {
            krate.items.push(func);
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