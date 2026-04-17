use rustc_ast::mut_visit::MutVisitor;
use std::collections::HashMap;




struct SymbolFinder {
    symbols: Vec<String>,
    idents: Vec<String>,
}

impl SymbolFinder {

    fn visit_token_stream (&mut self, ts: &rustc_ast::tokenstream::TokenStream) {
        for tree in ts.iter() {

            match tree {
                rustc_ast::tokenstream::TokenTree::Token(token, _) => {
                    match &token.kind {
                        rustc_ast::token::TokenKind::Literal(lit) => {
                            self.symbols.push(lit.symbol.as_str().to_string());
                        }

                        rustc_ast::token::TokenKind::Ident(name, _) => {
                            self.idents.push(name.as_str().to_string());
                        }

                        _ => {}
                    }
                }
                rustc_ast::tokenstream::TokenTree::Delimited(_,_,_, inner_tokenstream) => {
                    self.visit_token_stream(inner_tokenstream);
                }
                _ => {}
            }
        }
    }


}

//SymbolFinder finds symbols and Idents from an AST
impl MutVisitor for SymbolFinder {
    //For expressions the only special case we need is literals.
    //Identifiers are covered by visit_path as all identifiers are paths
    fn visit_expr(&mut self, expr: &mut rustc_ast::Expr) {
        match &mut expr.kind {
            rustc_ast::ExprKind::Lit(literal) => {
                self.symbols.push(literal.symbol.as_str().to_string());
            }
            rustc_ast::ExprKind::Field(_, ident) => {
                self.idents.push(ident.name.as_str().to_string());
            }
            _ => {}
        }

        rustc_ast::mut_visit::walk_expr(self, expr);
    }

    //Mac calls not stricly necessary, but nice for debug to be able to print in mock functions
    fn visit_mac_call(&mut self, node: &mut rustc_ast::MacCall) {
        self.visit_path(&mut node.path);
        self.visit_token_stream(&node.args.tokens);
        
    }

    // Will collect ALL identifiers(including keywords and types) but doing this doesn't seem to cause any problems

    fn visit_path_segment(&mut self, path: &mut rustc_ast::PathSegment) {
        self.idents.push(path.ident.name.as_str().to_string());
        rustc_ast::mut_visit::walk_path_segment(self, path);
    }

    fn visit_pat(&mut self, pat: &mut rustc_ast::Pat) {
        if let rustc_ast::PatKind::Ident(_, ident, _) = pat.kind {
            self.idents.push(ident.name.as_str().to_string())
        }
        rustc_ast::mut_visit::walk_pat(self, pat);
    }
}

struct SymbolFixer {
    symbols: Vec<String>,
    idents: Vec<String>,
    dict: HashMap<String, rustc_span::Symbol>,
}

impl SymbolFixer {
    fn visit_token_stream (&mut self, trees: &mut Vec<rustc_ast::tokenstream::TokenTree>) {
        for tree in trees {
            match tree {
                rustc_ast::tokenstream::TokenTree::Token(token, _) => {
                    match &mut token.kind {
                        rustc_ast::token::TokenKind::Literal(lit) => {
                            let string = self.symbols.remove(0);
                            lit.symbol = rustc_span::Symbol::intern(&string);
                        }

                        rustc_ast::token::TokenKind::Ident(name, _) => {
                            let ident = self.idents.remove(0);

                            match self.dict.get(&ident) {
                                Some(symb) => {
                                    *name = *symb;
                                }
                                None => {
                                    let symb = rustc_span::Symbol::intern(ident.as_str());
                                    self.dict.insert(ident, symb);
                                    *name = symb;
                                }
                            }
                        }

                        _ => {}
                    }
                }
                rustc_ast::tokenstream::TokenTree::Delimited(_,_,_, inner_tokenstream) => {
                    let mut inner_trees: Vec<_> = inner_tokenstream.iter().cloned().collect();
                    self.visit_token_stream(&mut inner_trees);
                    *inner_tokenstream = rustc_ast::tokenstream::TokenStream::new(inner_trees);

                }
                _ => {}
            }
        }
    }

}

//SymbolFixer will walk through an AST and fix all Identifiers and Symbols
//This is to avoid name resolution errors as literals and idents get
//stored as adresses which've been overwritten in the second compilation
//SymbolFixer recreates these adresses with the proper names
impl MutVisitor for SymbolFixer {
    fn visit_expr(&mut self, expr: &mut rustc_ast::Expr) {
        match &mut expr.kind {
            rustc_ast::ExprKind::Lit(literal) => {
                let string = self.symbols.remove(0);
                literal.symbol = rustc_span::Symbol::intern(&string); //Create new symbol in registry
            }
            rustc_ast::ExprKind::Field(_, id) => {
                let ident = self.idents.remove(0);
                match self.dict.get(&ident) {
                    Some(symb) => {
                        id.name = *symb;
                    }
                    None => {
                        let symb = rustc_span::Symbol::intern(ident.as_str());
                        self.dict.insert(ident, symb);
                        id.name = symb;
                    }
                }
            }
            _ => {}
        }
        rustc_ast::mut_visit::walk_expr(self, expr);
    }

    //Reconstructing MacCalls is a headache as the tokenstream only has private fields and non mutable methods
    //What we do instead is to just to clone the immutables and create a new tokenstream where we fix them
    fn visit_mac_call(&mut self, node: &mut rustc_ast::MacCall) {
        self.visit_path(&mut node.path);
        let mut trees: Vec<_> = node.args.tokens.iter().cloned().collect();
        self.visit_token_stream(&mut trees);
        node.args.tokens = rustc_ast::tokenstream::TokenStream::new(trees);
    }

    //Both path and pat work by creating a new entry into the dictionary the first them we encounter an identifier
    //Next time we encounter them we lookup the value from the dict
    fn visit_path_segment(&mut self, path: &mut rustc_ast::PathSegment) {
        let name = self.idents.remove(0);
        match self.dict.get(&name) {
            Some(symb) => {
                path.ident.name = *symb;
            }
            None => {
                let symb = rustc_span::Symbol::intern(name.as_str());
                self.dict.insert(name, symb);
                path.ident.name = symb;
            }
        }
        //println!("Fixed path named: {}", path.ident.name);

        rustc_ast::mut_visit::walk_path_segment(self, path);
    }

    fn visit_pat(&mut self, pat: &mut rustc_ast::Pat) {
        if let rustc_ast::PatKind::Ident(_, ident, _) = &mut pat.kind {
            let name = self.idents.remove(0);
            match self.dict.get(&name) {
                Some(symb) => {
                    ident.name = *symb;
                }
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
#[derive(Clone, Debug)]
pub struct MockedFun {
    name: String,
    path: String,
    sig: rustc_ast::FnSig,
    body: Box<rustc_ast::Block>,
    symbols: Vec<String>,
    idents: Vec<String>,
}

//encodes using pretty printing, this kinda sucks but it might work out idfk
impl MockedFun {
    pub fn new(fun: rustc_ast::Fn, path: String) -> MockedFun {
        //println!("{:#?}", fun);
        let name = fun.ident.as_str().to_string();
        match fun.body {
            Some(body) => MockedFun {
                name,
                path,
                sig: fun.sig,
                body,
                symbols: Vec::new(),
                idents: Vec::new(),
            },
            None => {
                panic!("Mocked fun missing body, this shouldnt ever happen")
            }
        }
    }

    pub fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    pub fn get_path(&self) -> String {
        self.path.clone()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_sig(&self) -> rustc_ast::FnSig {
        self.sig.clone()
    }

    pub fn get_body(&self) -> Box<rustc_ast::Block> {
        self.body.clone()
    }

    // This fn creates a visitor that visits the mock function and collects all symbols and identifiers
    pub fn collect_names(&mut self) {
        let mut visitor = SymbolFinder {
            symbols: Vec::new(),
            idents: Vec::new(),
        };
       visitor.visit_fn_decl(&mut self.sig.decl);
        visitor.visit_block(&mut self.body);
        self.symbols = visitor.symbols;
        self.idents = visitor.idents;
    }

    // This fn creates a visitor that visits the mocked function and resolves all the symbols and identifiers
    // It is meant to be called when in the second compilation context

    //???? how does this work? you don't
    pub fn resolve_names(&mut self) {
        let mut visitor = SymbolFixer {
            symbols: self.symbols.clone(),
            idents: self.idents.clone(),
            dict: HashMap::new(),
        };
        visitor.visit_fn_decl(&mut self.sig.decl);
        visitor.visit_block(&mut self.body);
    }


}
