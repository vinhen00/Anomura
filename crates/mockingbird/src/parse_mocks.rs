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



#[derive(Debug)]
pub struct ParseMocks {
    used_in_plugin: bool,
    pub program: String,
}

impl ParseMocks {
    pub fn new( used_in_plugin: bool ) -> Self {
        ParseMocks {
            program: String::new(),
            used_in_plugin,
        }
    }


    pub fn get_program(&self) -> String {
        self.program.clone()
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

        self.program.push_str(&result);
    }

}




//Compile mocks is a compiler setting the compiles the file that the mocked functions reside in.
//Will grab all functions defined therein, and store them as a field in the mocks.
//Stops compilation when done
impl rustc_driver::Callbacks for ParseMocks {
    fn config(&mut self, config: &mut Config) {
        if !self.used_in_plugin {
            config.file_loader = Some(Box::new(MockFileLoader {
                file: "mock_defs.rs".to_string(),
            }));
            config.opts.crate_types = vec![CrateType::Executable];
            config.opts.search_paths.clear();
        }
    }

    fn after_crate_root_parsing(
        &mut self,
        compiler: &Compiler,
        krate: &mut rustc_ast::Crate,
    ) -> Compilation {

        //we are using it within a
        if self.used_in_plugin {
            self.visit_crate(krate);
            return Compilation::Stop;
        }
        for item in &krate.items {
            println!("checking item {:?}", item.kind.ident());
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

impl<'a> Visitor<'a> for ParseMocks {
    #[doc = r" The result type of the `visit_*` methods. Can be either `()`,"]
    #[doc = r" or `ControlFlow<T>`."]
    type Result = ();
    fn visit_mac_call(&mut self, node: &'_ rustc_ast::MacCall) -> Self::Result {
        self.handle_maccall(node);
    }
}
