use rustc_driver::Compilation;
use rustc_interface::interface::Compiler;

//use rustc_plugin::{CrateFilter, PluginResult, RustcPlugin, RustcPluginArgs, RustcWrapperType};

use crate::compile_mocks;
use crate::visitors::{MockObject, MockedFun, MockedStruct};

//These function will be used when creating impl for Food_original
fn _extract_ident(ty: &rustc_ast::Ty) -> Option<String> {
    match &ty.kind {
        rustc_ast::TyKind::Path(_, path) => path.segments.last().map(|seg| seg.ident.to_string()),
        _ => None,
    }
}

fn _set_ident_name(ty: &mut rustc_ast::Ty, new_name: String) {
    if let rustc_ast::TyKind::Path(_, path) = &mut ty.kind {
        if let Some(seg) = path.segments.last_mut() {
            seg.ident.name = rustc_span::symbol::Symbol::intern(&new_name);
        }
    }
}

#[derive(Debug)]
pub struct FunctionIntercept {
    mockfun: Vec<MockedFun>,
    mockobj: Vec<MockedStruct>,
}

impl FunctionIntercept {
    pub fn new(mocks: Vec<MockObject>) -> Self {
        let mut mockfun = Vec::new();
        let mut mockobj = Vec::new();

        for mock in mocks {
            match mock {
                MockObject::Function(m) => {
                    mockfun.push(m);
                }
                MockObject::Struct(m) => mockobj.push(m),
            }
        }

        FunctionIntercept { mockfun, mockobj }
    }
    // checks if name is included in the list of mocks
    fn check_name(&self, name: String) -> bool {
        for mock in &self.mocks {
            if name == mock.get_name() {
                return true;
            }
        }
        false
    }
    fn copy_fun(&self, item: Box<rustc_ast::Item>) -> Box<rustc_ast::Item> {
        // clone original function and give at new identifier
        let mut original_function = item.clone();
        let rustc_ast::ItemKind::Fn(fn_data) = &mut original_function.kind else {
            unreachable!("original_function can impossibly have another kind");
        };
        let new_name = format!("{}_original", fn_data.ident.name.as_str());
        fn_data.ident.name = rustc_span::Symbol::intern(&new_name);

        original_function
    }

    fn replace_fun(&mut self, fun: &mut rustc_ast::Fn) {
        for mock in &mut self.mockfun {
            //println!("Mocking {}", mock.get_name());
            if fun.ident.name.as_str() == mock.get_name().as_str() {
                //println!("Mocking {}", mock.get_name());
                mock.resolve_names();
                //println!("TEST: {:#?}", mock.get_body());

                fun.sig.decl = mock.get_sig().decl;
                fun.body = Some(mock.get_body());
            }
        }
    }

    fn copy_method(
        &self,
        item: &mut Box<rustc_ast::AssocItem>,
        imp_name: String,
    ) -> Option<rustc_ast::AssocItem> {
        let rustc_ast::AssocItemKind::Fn(fn_data) = &item.kind else {
            return None;
        };
        let method_name = format!("{}.{}", imp_name, fn_data.ident.name.as_str());
        //Dont copy constructors of the class
        if let rustc_ast::FnRetTy::Ty(ty) = &fn_data.sig.decl.output {
            match &ty.kind {
                rustc_ast::TyKind::ImplicitSelf => return None,
                rustc_ast::TyKind::Path(_, path) => {
                    let name = path.segments.last().unwrap().ident.name.as_str();
                    println!("Type of {} is {}", fn_data.ident.name.as_str(), name);
                    if name == "Self" || name == imp_name.as_str() {
                        return None;
                    }
                }
                _ => {} // Do nothing
            }
        }

        //Check that function exists in list of mocks
        if self.check_name_fun(method_name) {
            let mut original_function = item.clone();
            if let rustc_ast::AssocItemKind::Fn(fn_data) = &mut original_function.kind {
                let new_name = format!("{}_original", fn_data.ident.name.as_str());
                fn_data.ident.name = rustc_span::Symbol::intern(&new_name);
            }
            return Some(*original_function);
        }
        return None;
    }

    fn replace_method(&mut self, meth: &mut rustc_ast::Fn, imp_name: String) {
        for mock in &mut self.mockfun {
            let method_name = format!("{}.{}", imp_name, meth.ident.name.as_str());

            if method_name == mock.get_name().as_str() {
                //println!("Mocking method {}", mock.get_name());
                mock.resolve_names();
                //println!("{:#?}", mock.get_body());
                meth.sig.decl = mock.get_sig().decl;
                meth.body = Some(mock.get_body());
            }
        }
    }

    fn handle_impl(&mut self, item: &mut rustc_ast::Item) {
        let mut method_originals = Vec::new();

        // Item will always be impl
        if let rustc_ast::ItemKind::Impl(imp) = &mut item.kind {
            let imp_name = compile_mocks::extract_struct_name_from_impl(imp.clone())
                .expect("expected struct name in {:?imp}");

            //Save original methods
            for item in imp.items.iter_mut() {
                if let Some(method_copy) = self.copy_method(item, imp_name.clone()) {
                    method_originals.push(method_copy);
                }
            }

            // Replace methods
            for imp_item in imp.items.iter_mut() {
                if let rustc_ast::AssocItemKind::Fn(fn_data) = &mut imp_item.kind {
                    self.replace_method(fn_data, imp_name.clone());
                }
            }

            //Push originals back
            for i in method_originals {
                imp.items.push(Box::new(i));
            }
        }
    }

    fn handle_mod(&mut self, module: &mut rustc_ast::ModKind) {
        let mut function_originals: Vec<Box<rustc_ast::Item>> = Vec::new();

        if let rustc_ast::ModKind::Loaded(items, ..) = module {
            for item in items.iter() {
                match &item.kind {
                    rustc_ast::ItemKind::Fn(fn_data)
                        if self.check_name_fun(fn_data.ident.name.as_str().to_string()) =>
                    {
                        function_originals.push(self.copy_fun(item.clone()));
                    }
                    // rustc_ast::ItemKind::Struct(name, _, _)
                    // if self.check_name_stro(name.as_str().into()) => {
                    //     function_originals.push(self.copy_struct(item.clone()))
                    // }
                    _ => {}
                }
            }

            for item in items.iter_mut() {
                match &mut item.kind {
                    rustc_ast::ItemKind::Fn(fn_data) => {
                        self.replace_fun(fn_data);
                    }
                    rustc_ast::ItemKind::Impl(_) => {
                        self.handle_impl(item);
                    }
                    rustc_ast::ItemKind::Mod(_, _, module) => {
                        self.handle_mod(module);
                    }
                    _ => {}
                }
            }
        }
    }

    fn handle_struct(&mut self, item: &mut rustc_ast::Item) {
        if let rustc_ast::ItemKind::Struct(name, gens, old_fields) = item.clone().kind {
            for mock in &mut self.mockobj {
                if mock.get_name() == name.as_str().to_string() {
                    mock.resolve_names();

                    let mut combined_fields = old_fields.clone();
                    let rustc_ast::VariantData::Struct {
                        fields: combined, ..
                    } = &mut combined_fields
                    else {
                        panic!("variant data should always be a struct");
                    };
                    let rustc_ast::VariantData::Struct {
                        fields: incoming, ..
                    } = &mut mock.get_fields()
                    else {
                        panic!("variant data should always be a struct");
                    };

                    for incoming_field in incoming {
                        let mut exists = true;
                        for cur_field in &mut *combined {
                            if incoming_field.ident.unwrap().name == cur_field.ident.unwrap().name {
                                exists = false
                            }
                        }
                        if exists {
                            combined.push(incoming_field.clone())
                        }
                    }

                    item.kind = rustc_ast::ItemKind::Struct(name, gens.clone(), combined_fields);
                }
            }
        }
    }

    fn _copy_struct(&mut self, item: Box<rustc_ast::Item>) -> Box<rustc_ast::Item> {
        // clone original function and give at new identifier
        let mut original_function = item.clone();
        let rustc_ast::ItemKind::Struct(name, _gens, _fields) = &mut original_function.kind else {
            unreachable!("original_function can impossibly have another kind");
        };
        let new_name = format!("{}_original", name.as_str());
        name.name = rustc_span::Symbol::intern(&new_name);

        original_function
    }
}

//Function_intercept is a compiler setting that compiles the target file and replaces the function body of the functions that have a mocked variant
impl rustc_driver::Callbacks for FunctionIntercept {
    fn after_crate_root_parsing(
        &mut self,
        _compiler: &Compiler,
        krate: &mut rustc_ast::Crate,
    ) -> Compilation {
        println!("Started compilation");
        //println!("Have Krate {:#?}", krate);

        //First create copies of all functions that will be mocked
        let mut function_originals: Vec<Box<rustc_ast::Item>> = Vec::new();

        for item in &krate.items {
            match &item.kind {
                rustc_ast::ItemKind::Fn(fn_data)
                    if self.check_name_fun(fn_data.ident.name.as_str().to_string()) =>
                {
                    function_originals.push(self.copy_fun(item.clone()));
                }
                // rustc_ast::ItemKind::Struct(name, _, _)
                // if self.check_name_stro(name.as_str().into()) => {
                //     function_originals.push(self.copy_struct(item.clone()))
                // }
                _ => {}
            }
        }

        //Then replace the original with their mocked variants
        for item in &mut krate.items {
            match &mut item.kind {
                rustc_ast::ItemKind::Fn(fn_data) => {
                    self.replace_fun(fn_data);
                }
                rustc_ast::ItemKind::Impl(_) => {
                    self.handle_impl(item);
                }
                rustc_ast::ItemKind::Mod(_, _, module) => {
                    self.handle_mod(module);
                }
                _ => {}
            }
        }
        for func in function_originals {
            krate.items.push(func);
        }
        //println!("{:#?}", krate);
        Compilation::Continue
    }
}
