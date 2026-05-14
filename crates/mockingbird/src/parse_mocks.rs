use std::{collections::HashMap, io::Read};

use proc_macro2::TokenStream;
use quote::quote;
use rustc_ast::token::TokenKind;
use rustc_ast::tokenstream::TokenTree;
use rustc_ast::visit::Visitor;
use rustc_ast_pretty::pprust;
use rustc_span::symbol::Symbol;
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;
use syn::{Ident, parse_quote};

use rustc_driver::Compilation;
use rustc_interface::interface::Compiler;

use crate::{
    MockedFun,
    expand_macro::{expand_mock_fn, expand_mock_method},
};

pub struct MockFileLoader {
    pub file: String,
}

impl rustc_span::source_map::FileLoader for MockFileLoader {
    fn file_exists(&self, path: &std::path::Path) -> bool {
        path == std::path::Path::new(&self.file)
    }

    fn read_file(&self, path: &std::path::Path) -> std::io::Result<String> {
        if path == std::path::Path::new(&self.file) {
            let mut file =
                std::fs::File::open(format!("crates/mockingbird/test_files/{}", self.file))?;
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

    // fn current_directory(&self) -> Result<std::path::PathBuf, std::io::Error> {
    //     Ok(std::path::PathBuf::from("."))
    // }
}
/// Used in macro expansion for deriving types in expectations and sequences, etc
#[derive(Serialize, Deserialize, Debug)]
pub struct MacroMap {
    map: HashMap<String, Vec<MacroTypeDataErased>>,
}
///Holds data used in `MacroMap`
#[derive(Serialize, Deserialize, Debug)]
pub struct MacroTypeDataErased {
    /// contains path postfixed with definition name
    path: String,
    input_idents: Vec<String>,
    input_types: Vec<String>,
    return_type: String,
}
pub struct MacroTypeData {
    pub path: syn::Path,
    pub input_idents: Vec<Ident>,
    pub input_types: Vec<syn::Type>,
    pub return_type: syn::Type,
}
impl MacroMap {
    pub fn new(fns: Vec<MockedFun>) -> Self {
        let mut map: HashMap<String, Vec<MacroTypeDataErased>> = [].into();
        log::debug!("HERE");
        for m in fns {
            let input_idents = m.get_idents().iter().map(String::clone).collect();
            let input_types = m
                .input_types()
                .iter()
                .map(|t| quote! {#t}.to_string())
                .collect::<Vec<String>>();
            let return_type = {
                let ret = m.return_type();
                quote! {#ret}.to_string()
            };
            log::debug!("there");
            let path_with_name = format!("{}::{}", m.get_path_as_string(), m.get_name());
            let type_data = MacroTypeDataErased {
                path: path_with_name,
                input_idents,
                input_types,
                return_type,
            };
            map.entry(m.get_path().get_crate())
                .and_modify(|v| v.push(type_data))
                .or_insert(vec![]);
        }
        MacroMap { map }
    }
    pub fn make_useful(self) -> HashMap<String, Vec<MacroTypeData>> {
        let mut new_map = HashMap::new();
        for (key, data) in self.map {
            let mut new_vec = vec![];
            for d in data {
                let path = d.path;
                let new_path: syn::Path = parse_quote!( #path );
                let new_idents = d
                    .input_idents
                    .into_iter()
                    .map(|i| parse_quote!(#i))
                    .collect::<Vec<syn::Ident>>();
                let new_types = d
                    .input_types
                    .into_iter()
                    .map(|t| parse_quote!(#t))
                    .collect::<Vec<syn::Type>>();
                let ret = d.return_type;
                let new_return: syn::Type = parse_quote!(#ret);
                let new_data = MacroTypeData {
                    path: new_path,
                    input_idents: new_idents,
                    input_types: new_types,
                    return_type: new_return,
                };
                new_vec.push(new_data);
            }
            new_map.insert(key, new_vec);
        }
        new_map
    }
}

#[derive(Debug)]
pub struct ParseMocks {
    used_in_plugin: bool,
    pub program: String,
    pub crates: Vec<String>,
}

impl ParseMocks {
    pub fn new(used_in_plugin: bool) -> Self {
        ParseMocks {
            program: String::new(),
            used_in_plugin,
            crates: Vec::new(),
        }
    }

    pub fn get_program(&self) -> String {
        self.program.clone()
    }

    pub fn get_crates(&self) -> Vec<String> {
        self.crates.clone()
    }

    fn handle_mod(&mut self, mod_items: &rustc_ast::ModKind) {
        if let rustc_ast::ModKind::Loaded(items, ..) = mod_items {
            for i in items {
                match &i.kind {
                    rustc_ast::ItemKind::MacCall(mac_data) => {
                        self.handle_maccall(mac_data);
                    }
                    rustc_ast::ItemKind::Mod(_, _, mod_data) => {
                        self.handle_mod(&mod_data);
                    }
                    _ => {}
                }
            }
        }
    }

    ///This runs a new compilation process inside the callback function for the original compilation process
    ///This new compilation compiles the expanded macros and saves
    fn handle_maccall(&mut self, mac_call: &rustc_ast::MacCall) {
        // println!("handle_maccall");

        let args = mac_call.args.clone();
        let tokens = args.tokens;
        let result;
        let syn_ts = TokenStream::from_str(&pprust::tts_to_string(&tokens))
            .expect("failed to parse token stream");

        if let Some(path) = mac_call.path.segments.last() {
            match path.ident.name.as_str() {
                "mock_fn" => {
                    let (res, mocked_fun) = expand_mock_fn(syn_ts);
                    result = res.to_string();
                }
                "mock_method" => result = expand_mock_method(syn_ts).to_string(),
                _ => return,
            }
        } else {
            return;
        }

        //println!("result: {:?}", result);

        self.program.push_str(&result);
    }
}

//Compile mocks is a compiler setting the compiles the file that the mocked functions reside in.
//Will grab all functions defined therein, and store them as a field in the mocks.
//Stops compilation when done
impl rustc_driver::Callbacks for ParseMocks {
    fn after_crate_root_parsing(
        &mut self,
        _compiler: &Compiler,
        krate: &mut rustc_ast::Crate,
    ) -> Compilation {
        //println!("{:#?}", krate);
        //we are using it within a
        if self.used_in_plugin {
            self.visit_crate(krate);
            return Compilation::Stop;
        }
        for item in &krate.items {
            //println!("checking item {:?}", item.kind.ident());
            match &item.kind {
                rustc_ast::ItemKind::Mod(_, _, mod_data) => {
                    self.handle_mod(mod_data);
                }
                rustc_ast::ItemKind::MacCall(mac_data) => {
                    self.handle_maccall(mac_data);
                }
                _ => {}
            }
        }

        Compilation::Stop
    }
}

fn extract_path_value(mac_call: &rustc_ast::MacCall) -> Option<Symbol> {
    if let Some(path) = mac_call.path.segments.last() {
        match path.ident.name.as_str() {
            "mock_fn" => {
                let tokens: Vec<&TokenTree> = mac_call.args.tokens.iter().collect();
                let TokenTree::Token(val_tok, _) = tokens[0] else {
                    return None;
                };
                let TokenKind::Ident(value, _) = val_tok.kind else {
                    return None;
                };
                return Some(value);
            }
            "mock_method" => {
                let tokens: Vec<&TokenTree> = mac_call.args.tokens.iter().collect();
                let TokenTree::Token(val_tok, _) = tokens[0] else {
                    return None;
                };
                let TokenKind::Ident(value, _) = val_tok.kind else {
                    return None;
                };
                return Some(value);
            }
            _ => return None,
        }
    } else {
        return None;
    }
}

impl<'a> Visitor<'a> for ParseMocks {
    #[doc = r" The result type of the `visit_*` methods. Can be either `()`,"]
    #[doc = r" or `ControlFlow<T>`."]
    type Result = ();
    fn visit_mac_call(&mut self, node: &'_ rustc_ast::MacCall) -> Self::Result {
        if let Some(path) = extract_path_value(node) {
            self.crates.push(path.as_str().to_string());
        }
        self.handle_maccall(node);
    }
}
