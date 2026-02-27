use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use proc_macro2::TokenStream;
use std::str::FromStr;
use rustc_ast_pretty::pprust;

use rustc_driver::{Compilation, run_compiler};
use rustc_interface::interface::{Compiler, Config};
use rustc_session::config::CrateType;

use crate::visitors::MockedFun;
use crate::expand_macro::expand_mock;


pub struct MockFileLoader{
    pub file: String
}

impl rustc_span::source_map::FileLoader for MockFileLoader {
    fn file_exists(&self, path: &std::path::Path) -> bool {
        path == std::path::Path::new(&self.file)
    }

    fn read_file(&self, path: &std::path::Path) -> std::io::Result<String> {
        if path == std::path::Path::new(&self.file) {
            let mut file = std::fs::File::open(format!("mockingbird/src/{}", self.file))?;
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


pub struct MockDefsLoader {
    pub mockdefs: String,
}

impl rustc_span::source_map::FileLoader for MockDefsLoader {
    fn file_exists(&self, path: &Path) -> bool {
        path == Path::new("main.rs")
    }

    fn read_file(&self, path: &Path) -> io::Result<String> {
        Ok(self.mockdefs.clone())
  
    }

    fn read_binary_file(&self, _path: &Path) -> io::Result<Arc<[u8]>> {
        Err(io::Error::other("oops"))
    }

    fn current_directory(&self) -> Result<PathBuf, std::io::Error> {
        Ok(PathBuf::from("."))
    }
}


pub fn extract_struct_name_from_impl (imp: rustc_ast::Impl) -> String {
    if let rustc_ast::TyKind::Path(_, path) = imp.self_ty.kind {
        match path.segments.last() {
            Some(seg) => {return seg.ident.name.as_str().to_string();}
            None => {return "".to_string();}
        }
    }
    return "".to_string();
}



pub struct CompileMocks {
    mocks: Vec<MockedFun>,
    inline: Option<String>,
}


impl CompileMocks {

    pub fn new(mocks: Vec<MockedFun>, inline: Option<String>) -> Self {
        CompileMocks{mocks, inline}
    }

    pub fn get_mocks(&self) -> Vec<MockedFun> {
        self.mocks.clone()
    }

    fn handleFn(&mut self, fn_data: &Box<rustc_ast::Fn>) {
        if fn_data.ident.name.as_str() != "main" {
            let mut foo = MockedFun::new(fn_data.clone());
            foo.collect_names();

            self.mocks.push(foo);
        }  
    }

    fn handleImpl(&mut self, impl_data: &rustc_ast::Impl) {
        let imp_name = extract_struct_name_from_impl(impl_data.clone());
        for imp_item in &impl_data.items {
            if let rustc_ast::AssocItemKind::Fn(fn_data) = &imp_item.kind {
                let mut foo = MockedFun::new(fn_data.clone());
                foo.collect_names();
                foo.set_name(format!("{}.{}", imp_name, foo.get_name()));
                self.mocks.push(foo);
            }
        }
    }

    fn handleMod(&mut self, mod_items: &rustc_ast::ModKind) {
        if let rustc_ast::ModKind::Loaded(items, _, _) = mod_items {
            for i in items {
                match &i.kind {
                    rustc_ast::ItemKind::Fn(fn_data) => {
                        self.handleFn(fn_data);
                    }
                    rustc_ast::ItemKind::Impl(impl_data) => {
                        self.handleImpl(impl_data);
                    }
                    rustc_ast::ItemKind::Mod(_,_,modData) => {
                        self.handleMod(modData);
                    }
                    _ => {}
                }
            }
        }
    }
    fn handleMacCall(&mut self, tokens: rustc_ast::tokenstream::TokenStream) {
        let syn_ts = TokenStream::from_str(&pprust::tts_to_string(&tokens))
        .expect("failed to parse token stream");
        //println!("{}", syn_ts);

        let result = expand_mock(syn_ts);
        println!("{}", result);
        self.compileMacCall(result.to_string());
    }

    fn compileMacCall(&mut self, program: String) {
        let mut mockedFuns = CompileMocks::new(Vec::new(), Some(program));
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
        for foo in mockedFuns.mocks{
            self.mocks.push(foo);
        }
    }



}


//Compile mocks is a compiler setting the compiles the file that the mocked functions reside in.
//Will grab all functions defined therein, and store them as a field in the mocks.
//Stops compilation when done
impl rustc_driver::Callbacks for CompileMocks {
    fn config(&mut self, config: &mut Config) {
        match &self.inline {
            Some(program) => { config.file_loader = Some(Box::new(MockDefsLoader{mockdefs: program.clone()}) );
}
            None => { config.file_loader = Some(Box::new(MockFileLoader{file: "mock_defs.rs".to_string()})); }
        }
        config.opts.crate_types = vec![CrateType::Executable];
        config.opts.search_paths.clear();
    }

    fn after_crate_root_parsing(
        &mut self,
        _compiler: &Compiler,
        krate: &mut rustc_ast::Crate,
    ) -> Compilation {
        // println!("{:#?}", krate);
        for item in &krate.items {
            match &item.kind {
                rustc_ast::ItemKind::Fn(fn_data) => {
                    self.handleFn(fn_data);
                }
                rustc_ast::ItemKind::Impl(impl_data) => {
                    self.handleImpl(impl_data);
                }
                rustc_ast::ItemKind::Mod(_,_,modData) => {
                    self.handleMod(modData);
                }
                rustc_ast::ItemKind::MacCall(macData) => {
                    let args = macData.args.clone();
                    let tokens = args.tokens;

                    self.handleMacCall(tokens);
                    
                    

                }
                _ => {}
            }

        }
        Compilation::Stop
    }


}
