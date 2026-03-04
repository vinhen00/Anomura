use rustc_driver::Compilation;
use rustc_interface::interface::{Compiler, Config};
use rustc_session::config::CrateType;

use crate::compile_mocks;
use crate::visitors;

#[derive(Debug)]
pub struct FunctionIntercept {
    mocks: Vec<visitors::MockedFun>,
}

impl FunctionIntercept {
    pub fn new(mocks: Vec<visitors::MockedFun>) -> Self {
        FunctionIntercept { mocks }
    }
    // checks if name is included in the list of mocks
    fn check_name(&mut self, name: String) -> bool {
        for mock in &self.mocks {
            if name == mock.get_name() {
                return true;
            }
        }
        false
    }
}

//Function_intercept is a compiler setting that compiles the target file and replaces the function body of the functions that have a mocked variant
impl rustc_driver::Callbacks for FunctionIntercept {
    fn config(&mut self, config: &mut Config) {
        config.file_loader = Some(Box::new(compile_mocks::MockFileLoader {
            file: "mock_test.rs".to_string(),
        }));
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
        let mut method_originals = Vec::new();

        for item in &mut krate.items {
            if let rustc_ast::ItemKind::Fn(fn_data) = &item.kind
                && self.check_name(fn_data.ident.name.as_str().to_string())
            {
                // clone original function and give at new identifier
                let mut original_function = item.clone();
                let rustc_ast::ItemKind::Fn(fn_data) = &mut original_function.kind else {
                    unreachable!("original_function can impossibly have another kind");
                };
                let new_name = format!("{}_original", fn_data.ident.name.as_str());
                fn_data.ident.name = rustc_span::Symbol::intern(&new_name);

                function_originals.push(original_function);
            }
        }
        //Then replace the original with their mocked variants
        for item in &mut krate.items {
            match &mut item.kind {
                rustc_ast::ItemKind::Fn(fn_data) => {
                    for mock in &mut self.mocks {
                        if fn_data.ident.name.as_str() == mock.get_name().as_str() {
                            println!("Mocking {}", mock.get_name());
                            mock.resolve_names();
                            //println!("{:#?}", foo.get_body());

                            fn_data.sig.decl = mock.get_sig().decl;
                            fn_data.body = Some(mock.get_body());
                        }
                    }
                }
                rustc_ast::ItemKind::Impl(imp) => {
                    let imp_name = compile_mocks::extract_struct_name_from_impl(imp.clone())
                        .expect("expected struct name in {:?imp}");

                    //Save original method
                    for item in &mut imp.items {
                        if let rustc_ast::AssocItemKind::Fn(fn_data) = &item.kind {
                            let method_name =
                                format!("{}.{}", imp_name, fn_data.ident.name.as_str());
                            if self.check_name(method_name) {
                                let mut original_function = item.clone();
                                if let rustc_ast::AssocItemKind::Fn(fn_data) =
                                    &mut original_function.kind
                                {
                                    let new_name =
                                        format!("{}_original", fn_data.ident.name.as_str());
                                    fn_data.ident.name = rustc_span::Symbol::intern(&new_name);
                                }
                                method_originals.push(original_function);
                            }
                        }
                    }

                    //Replace method
                    for imp_item in &mut imp.items {
                        if let rustc_ast::AssocItemKind::Fn(fn_data) = &mut imp_item.kind {
                            for mock in &mut self.mocks {
                                let method_name =
                                    format!("{}.{}", imp_name, fn_data.ident.name.as_str());
                                if method_name == mock.get_name().as_str() {
                                    println!("Mocking method {}", mock.get_name());
                                    mock.resolve_names();
                                    //println!("{:#?}", mock.get_body());
                                    fn_data.sig.decl = mock.get_sig().decl;
                                    fn_data.body = Some(mock.get_body());
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
            for i in method_originals {
                if let rustc_ast::ItemKind::Impl(imp) = &mut item.kind {
                    //println!("Pushing method {:#?}", i);
                    imp.items.push(i)
                }
            }
            method_originals = Vec::new();
        }
        for func in function_originals {
            krate.items.push(func);
        }
        //println!("{:#?}", krate);
        Compilation::Continue
    }
}
