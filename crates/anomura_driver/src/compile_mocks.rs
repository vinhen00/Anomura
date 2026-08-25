use std::io;
use std::path::Path;
use std::sync::Arc;

use rustc_driver::Compilation;
use rustc_interface::interface::{Compiler, Config};
use rustc_session::config::CrateType;

use crate::visitors::{MockObject, MockedFun, MockedStruct};

pub struct MockDefsLoader {
    pub mockdefs: String,
}

impl rustc_span::source_map::FileLoader for MockDefsLoader {
    fn file_exists(&self, path: &Path) -> bool {
        path == Path::new("main.rs")
    }

    fn read_file(&self, _path: &Path) -> io::Result<String> {
        Ok(self.mockdefs.clone())
    }

    fn read_binary_file(&self, _path: &Path) -> io::Result<Arc<[u8]>> {
        Err(io::Error::other("oops"))
    }
}

pub fn extract_struct_name_from_impl(imp: rustc_ast::Impl) -> Option<String> {
    let rustc_ast::TyKind::Path(_, path) = imp.self_ty.kind else {
        return None;
    };
    path.segments.last().map(|seg| seg.ident.to_string())
}

#[derive(Debug)]
pub struct CompileMocks {
    used_in_plugin: bool,
    mocks: Vec<MockObject>,
    program: String,
}

impl CompileMocks {
    pub fn new(mocks: Vec<MockObject>, program: String, used_in_plugin: bool) -> Self {
        CompileMocks {
            mocks,
            program,
            used_in_plugin,
        }
    }

    pub fn get_mocks(&self) -> Vec<MockObject> {
        self.mocks.clone()
    }

    fn handle_fn(&mut self, fn_data: &rustc_ast::Fn, path: String) {
        if fn_data.ident.name.as_str() != "main" {
            let mut mocked_fn = MockedFun::new(fn_data.clone(), path);
            mocked_fn.collect_names();

            self.mocks.push(MockObject::Function(mocked_fn));
        }
    }

    fn handle_impl(&mut self, impl_data: &rustc_ast::Impl) {
        let imp_name =
            extract_struct_name_from_impl(impl_data.clone()).expect("failed to parse struct");
        for imp_item in &impl_data.items {
            let mut path = "".to_string();
            for attr in &imp_item.attrs {
                path = extract_attribute_name(attr.clone());
            }
            if let rustc_ast::AssocItemKind::Fn(fn_data) = &imp_item.kind {
                let mut mocked_fn = MockedFun::new(*fn_data.clone(), path);
                mocked_fn.collect_names();
                mocked_fn.set_name(format!("{}.{}", imp_name, mocked_fn.get_name()));
                self.mocks.push(MockObject::Function(mocked_fn));
            }
        }
    }

    fn handle_mod(&mut self, mod_items: &rustc_ast::ModKind) {
        if let rustc_ast::ModKind::Loaded(items, ..) = mod_items {
            for i in items {
                let mut path = "".to_string();
                for attr in &i.attrs {
                    path = extract_attribute_name(attr.clone());
                }
                match &i.kind {
                    rustc_ast::ItemKind::Fn(fn_data) => {
                        self.handle_fn(fn_data, path);
                    }
                    rustc_ast::ItemKind::Impl(impl_data) => {
                        self.handle_impl(impl_data);
                    }
                    rustc_ast::ItemKind::Mod(_, _, mod_data) => {
                        self.handle_mod(mod_data);
                    }
                    rustc_ast::ItemKind::Struct(name, _, fields) => {
                        self.handle_struct(name.as_str().into(), fields.clone(), path);
                    }
                    _ => {}
                }
            }
        }
    }

    fn handle_struct(&mut self, name: String, fields: rustc_ast::VariantData, path: String) {
        let mut mock_struct = MockedStruct::new(name, fields, path);
        mock_struct.collect_names();

        self.mocks.push(MockObject::Struct(mock_struct));
    }
}

fn extract_attribute_name(atr: rustc_ast::Attribute) -> String {
    let mut path = "".to_string();
    if let rustc_ast::AttrKind::Normal(norm) = atr.kind
        && let rustc_ast::AttrArgs::Delimited(del_args) = norm.item.args
    {
        for token in del_args.tokens.iter() {
            if let rustc_ast::tokenstream::TokenTree::Token(tok, _) = token
                && let rustc_ast::token::TokenKind::Ident(name, _) = tok.kind
            {
                if !path.is_empty() {
                    path = format!("{}::{}", path, name.as_str());
                } else {
                    path = name.as_str().to_string()
                }
                //println!("TOK: {:#?}", path);
            }
        }
    }
    println!("Found path {}", path);
    path
}

//Compile mocks is a compiler setting the compiles the file that the mocked functions reside in.
//Will grab all functions defined therein, and store them as a field in the mocks.
//Stops compilation when done
impl rustc_driver::Callbacks for CompileMocks {
    fn config(&mut self, config: &mut Config) {
        config.file_loader = Some(Box::new(MockDefsLoader {
            mockdefs: self.program.clone(),
        }));

        if !self.used_in_plugin {
            config.opts.crate_types = vec![CrateType::Executable];
            config.opts.search_paths.clear();
        }
    }

    fn after_crate_root_parsing(
        &mut self,
        _compiler: &Compiler,
        krate: &mut rustc_ast::Crate,
    ) -> Compilation {
        //we are using it within a
        // if self.used_in_plugin {
        //     self.visit_crate(krate);
        //     return Compilation::Stop;
        // }
        for item in &krate.items {
            let mut path = "".to_string();
            for attr in &item.attrs {
                path = extract_attribute_name(attr.clone());
            }
            // println!("checking item {:?}", item.kind.ident());
            match &item.kind {
                rustc_ast::ItemKind::Fn(fn_data) => {
                    self.handle_fn(fn_data, path);
                }
                rustc_ast::ItemKind::Impl(impl_data) => {
                    self.handle_impl(impl_data);
                }
                rustc_ast::ItemKind::Mod(_, _, mod_data) => {
                    self.handle_mod(mod_data);
                }
                rustc_ast::ItemKind::Struct(name, _, fields) => {
                    self.handle_struct(name.as_str().into(), fields.clone(), path)
                }
                _ => {}
            }
        }
        Compilation::Stop
    }
}

// impl<'a> Visitor<'a> for CompileMocks {
//     #[doc = r" The result type of the `visit_*` methods. Can be either `()`,"]
//     #[doc = r" or `ControlFlow<T>`."]
//     type Result = ();
//     fn visit_mac_call(&mut self, node: &'_ rustc_ast::MacCall) -> Self::Result {
//         self.handle_maccall(node);
//     }
// }
