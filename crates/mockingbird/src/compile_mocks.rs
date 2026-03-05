use std::io;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use proc_macro2::TokenStream;
use rustc_ast::visit::Visitor;
use rustc_ast_pretty::pprust;
use std::str::FromStr;

use rustc_driver::{Compilation, run_compiler};
use rustc_interface::interface::{Compiler, Config};
use rustc_session::config::CrateType;

use crate::expand_macro::{expand_mock_fn, expand_mock_method};
use crate::visitors::MockedFun;

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
    mocks: Vec<MockedFun>,
    pub inline: Option<String>,
}

impl CompileMocks {
    pub fn new(mocks: Vec<MockedFun>, inline: Option<String>, used_in_plugin: bool) -> Self {
        CompileMocks {
            mocks,
            inline,
            used_in_plugin,
        }
    }

    pub fn get_mocks(&self) -> Vec<MockedFun> {
        self.mocks.clone()
    }

    pub fn get_inline(&self) -> Option<String> {
        self.inline.clone()
    }

    fn handle_fn(&mut self, fn_data: &rustc_ast::Fn) {
        if fn_data.ident.name.as_str() != "main" {
            let mut mocked_fn = MockedFun::new(fn_data.clone());
            mocked_fn.collect_names();

            self.mocks.push(mocked_fn);
        }
    }

    fn handle_impl(&mut self, impl_data: &rustc_ast::Impl) {
        let imp_name =
            extract_struct_name_from_impl(impl_data.clone()).expect("failed to parse struct");
        for imp_item in &impl_data.items {
            if let rustc_ast::AssocItemKind::Fn(fn_data) = &imp_item.kind {
                let mut mocked_fn = MockedFun::new(*fn_data.clone());
                mocked_fn.collect_names();
                mocked_fn.set_name(format!("{}.{}", imp_name, mocked_fn.get_name()));
                self.mocks.push(mocked_fn);
            }
        }
    }

    fn handle_mod(&mut self, mod_items: &rustc_ast::ModKind) {
        if let rustc_ast::ModKind::Loaded(items, ..) = mod_items {
            for i in items {
                match &i.kind {
                    rustc_ast::ItemKind::Fn(fn_data) => {
                        self.handle_fn(fn_data);
                    }
                    rustc_ast::ItemKind::Impl(impl_data) => {
                        self.handle_impl(&impl_data);
                    }
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
    pub fn compile_maccalls(&mut self, program: &str) {
        let mut mocked_funs = CompileMocks::new(Vec::new(), Some(String::from(program)), false);
        run_compiler(
            &[
                "ignored".to_string(),
                "mock_defs.rs".to_string(),
                "--crate-type".to_string(),
                "bin".to_string(),
                "-o".to_string(),
                "./target/mocked_main".to_string(),
            ],
            &mut mocked_funs,
        );
        for mocked_fn in mocked_funs.mocks {
            self.mocks.push(mocked_fn);
        }
    }

    fn handle_maccall(&mut self, mac_call: &rustc_ast::MacCall) {
        println!("handle_maccall");

        let args = mac_call.args.clone();
        let tokens = args.tokens;
        let result;
        let syn_ts = TokenStream::from_str(&pprust::tts_to_string(&tokens))
            .expect("failed to parse token stream");

        if let Some(path) = mac_call.path.segments.last() {
            match path.ident.name.as_str() {
                "mock_fn" => result = expand_mock_fn(syn_ts).to_string(),
                "mock_method" => result = expand_mock_method(syn_ts).to_string(),
                _ => return,
            }
        } else {
            return;
        }

        println!("result: {:?}", result);

        match &self.inline {
            Some(program) => {
                self.inline = Some(format!("{program}\n {result}"));
                println!("second pass: {:?}", self.inline);
            }

            None => {
                println!("first pass result: {:?}", result);
                self.inline = Some(result)
            }
        }
    }
}

//Compile mocks is a compiler setting the compiles the file that the mocked functions reside in.
//Will grab all functions defined therein, and store them as a field in the mocks.
//Stops compilation when done
impl rustc_driver::Callbacks for CompileMocks {
    fn config(&mut self, config: &mut Config) {
        match &self.inline {
            Some(program) => {
                config.file_loader = Some(Box::new(MockDefsLoader {
                    mockdefs: program.clone(),
                }));
            }
            None if !self.used_in_plugin => {
                config.file_loader = Some(Box::new(MockFileLoader {
                    file: "mock_defs.rs".to_string(),
                }));
            }
            None => (),
        }
        if !self.used_in_plugin {
            config.opts.crate_types = vec![CrateType::Executable];
            config.opts.search_paths.clear();
        }
    }

    fn after_crate_root_parsing(
        &mut self,
        compiler: &Compiler,
        krate: &mut rustc_ast::Crate,
    ) -> Compilation {
        let mut run_once = false;
        if self.inline.is_none() {
            run_once = true
        }


        //we are using it within a
        if self.used_in_plugin {
            self.visit_crate(krate);
            return Compilation::Stop;
        }
        for item in &krate.items {
            println!("checking item {:?}", item.kind.ident());
            match &item.kind {
                rustc_ast::ItemKind::Fn(fn_data) => {
                    if run_once {
                        println!("parsing fn first time")
                    } else {
                        println!("parsing fn second time")
                    }
                    self.handle_fn(fn_data);
                }
                rustc_ast::ItemKind::Impl(impl_data) => {
                    if run_once {
                        println!("parsing impl first time")
                    } else {
                        println!("parsing impl second time")
                    }
                    self.handle_impl(impl_data);
                }
                rustc_ast::ItemKind::Mod(_, _, mod_data) => {
                    if run_once {
                        println!("parsing mod first time")
                    } else {
                        println!("parsing mod second time")
                    }
                    self.handle_mod(mod_data);
                }
                rustc_ast::ItemKind::MacCall(mac_data) => {
                    if run_once {
                        println!("parsing maccall first time")
                    } else {
                        println!("parsing maccall second time")
                    }
                    self.handle_maccall(mac_data);
                }
                _ => {}
            }
        }

        if let Some(program) = &self.get_inline()
            && run_once
            && !self.used_in_plugin
        {
            self.compile_maccalls(program)
        }
        Compilation::Stop
    }
}

impl<'a> Visitor<'a> for CompileMocks {
    #[doc = r" The result type of the `visit_*` methods. Can be either `()`,"]
    #[doc = r" or `ControlFlow<T>`."]
    type Result = ();
    fn visit_mac_call(&mut self, node: &'_ rustc_ast::MacCall) -> Self::Result {
        self.handle_maccall(node);
    }
}
