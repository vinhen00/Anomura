//! CrateIntercept — the `rustc_driver::Callbacks` implementation for `mock_crate!` targets.
//!
//! Walks the target crate's AST in `after_crate_root_parsing`, collects the full public API
//! into a `CrateApiModel`, generates mock infrastructure, and injects it back into the AST.

use rustc_ast as ast;
use rustc_driver::Compilation;
use rustc_interface::interface::Compiler;
use rustc_span::symbol::Symbol;
use rustc_span::FileName;

use crate::crate_api::*;
use crate::crate_mock_gen;

/// The CrateIntercept driver — collects the crate's public API and generates mock infrastructure.
#[derive(Debug)]
pub struct CrateIntercept {
    crate_name: String,
}

impl CrateIntercept {
    pub fn new(crate_name: String) -> Self {
        Self { crate_name }
    }

    /// Walk a list of items (module contents) and collect the public API.
    fn collect_module(&self, name: Symbol, items: &[Box<ast::Item>]) -> ModuleModel {
        let mut module = ModuleModel::new(name);

        for item in items {
            match &item.kind {
                ast::ItemKind::Fn(fn_data) if self.is_pub(&item.vis) => {
                    if let Some(func) = self.collect_function(fn_data) {
                        module.functions.push(func);
                    }
                }
                ast::ItemKind::Struct(ident, _generics, fields) if self.is_pub(&item.vis) => {
                    if let Some(s) = self.collect_struct(ident.name, fields) {
                        module.structs.push(s);
                    }
                }
                ast::ItemKind::Enum(ident, _generics, enum_def) if self.is_pub(&item.vis) => {
                    if let Some(e) = self.collect_enum(ident.name, &enum_def.variants) {
                        module.enums.push(e);
                    }
                }
                ast::ItemKind::Trait(trait_data) if self.is_pub(&item.vis) => {
                    if let Some(t) = self.collect_trait(trait_data) {
                        module.traits.push(t);
                    }
                }
                ast::ItemKind::Impl(impl_data) => {
                    if let Some(imp) = self.collect_impl(impl_data) {
                        module.impls.push(imp);
                    }
                }
                ast::ItemKind::Mod(_safety, ident, mod_kind) if self.is_pub(&item.vis) => {
                    if let ast::ModKind::Loaded(mod_items, ..) = mod_kind {
                        let child = self.collect_module(ident.name, mod_items);
                        module.children.push(child);
                    }
                }
                _ => {}
            }
        }

        module
    }

    fn is_pub(&self, vis: &ast::Visibility) -> bool {
        matches!(vis.kind, ast::VisibilityKind::Public)
    }

    fn collect_function(&self, fn_data: &ast::Fn) -> Option<FunctionModel> {
        let name = fn_data.ident.name;
        let params = self.collect_params(&fn_data.sig.decl.inputs);
        let return_type = self.collect_return_type(&fn_data.sig.decl.output);

        Some(FunctionModel {
            name,
            params,
            return_type,
        })
    }

    fn collect_struct(&self, name: Symbol, fields: &ast::VariantData) -> Option<StructModel> {
        let ast::VariantData::Struct { fields: field_list, .. } = fields else {
            // Skip tuple structs and unit structs for now
            return None;
        };

        let fields = field_list
            .iter()
            .map(|f| FieldModel {
                name: f.ident.map(|i| i.name).unwrap_or(Symbol::intern("_")),
                ty: f.ty.clone(),
                is_pub: matches!(f.vis.kind, ast::VisibilityKind::Public),
            })
            .collect();

        Some(StructModel { name, fields })
    }

    fn collect_enum(&self, name: Symbol, variants: &[ast::Variant]) -> Option<EnumModel> {
        let variants = variants
            .iter()
            .map(|v| {
                let fields = match &v.data {
                    ast::VariantData::Tuple(fields, ..) => {
                        fields.iter().map(|f| f.ty.clone()).collect()
                    }
                    ast::VariantData::Struct { fields, .. } => {
                        fields.iter().map(|f| f.ty.clone()).collect()
                    }
                    ast::VariantData::Unit(..) => Vec::new(),
                };
                VariantModel {
                    name: v.ident.name,
                    fields,
                }
            })
            .collect();

        Some(EnumModel { name, variants })
    }

    fn collect_trait(&self, trait_data: &ast::Trait) -> Option<TraitModel> {
        let name = trait_data.ident.name;
        let methods = trait_data
            .items
            .iter()
            .filter_map(|item| {
                if let ast::AssocItemKind::Fn(fn_data) = &item.kind {
                    Some(self.collect_method_sig(fn_data, &item.vis))
                } else {
                    None
                }
            })
            .collect();

        Some(TraitModel { name, methods })
    }

    fn collect_impl(&self, impl_data: &ast::Impl) -> Option<ImplModel> {
        // Get the self type name
        let self_type_name = self.extract_type_name(&impl_data.self_ty)?;

        // Get the trait name if this is a trait impl
        let trait_name = impl_data.of_trait.as_ref().and_then(|trait_header| {
            trait_header.trait_ref.path.segments.last().map(|seg| seg.ident.name)
        });

        // Skip external trait impls for now (Debug, Clone, etc.)
        // TODO: support these once we handle the receiver/signature properly
        if trait_name.is_some() {
            return None;
        }

        let methods = impl_data
            .items
            .iter()
            .filter_map(|item| {
                if let ast::AssocItemKind::Fn(fn_data) = &item.kind {
                    // Only collect public methods for inherent impls;
                    // for trait impls, collect all methods
                    if trait_name.is_some() || self.is_pub(&item.vis) {
                        Some(self.collect_method_sig(fn_data, &item.vis))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        Some(ImplModel {
            self_type_name,
            trait_name,
            methods,
        })
    }

    fn collect_method_sig(&self, fn_data: &ast::Fn, vis: &ast::Visibility) -> MethodSigModel {
        let name = fn_data.ident.name;
        let receiver = self.extract_receiver(&fn_data.sig.decl.inputs);
        let params = self.collect_method_params(&fn_data.sig.decl.inputs);
        let return_type = self.collect_return_type(&fn_data.sig.decl.output);
        let is_pub = self.is_pub(vis);

        MethodSigModel {
            name,
            receiver,
            params,
            return_type,
            is_pub,
        }
    }

    fn collect_params(&self, inputs: &[ast::Param]) -> Vec<ParamModel> {
        inputs
            .iter()
            .filter_map(|param| {
                let name = self.extract_param_name(&param.pat)?;
                Some(ParamModel {
                    name,
                    ty: param.ty.clone(),
                })
            })
            .collect()
    }

    /// Collect method params, skipping the self receiver.
    fn collect_method_params(&self, inputs: &[ast::Param]) -> Vec<ParamModel> {
        inputs
            .iter()
            .filter(|param| !param.is_self())
            .filter_map(|param| {
                let name = self.extract_param_name(&param.pat)?;
                Some(ParamModel {
                    name,
                    ty: param.ty.clone(),
                })
            })
            .collect()
    }

    fn extract_receiver(&self, inputs: &[ast::Param]) -> ReceiverKind {
        let Some(first) = inputs.first() else {
            return ReceiverKind::None;
        };

        if !first.is_self() {
            return ReceiverKind::None;
        }

        // Determine if it's &self, &mut self, or self
        match &first.ty.kind {
            ast::TyKind::Ref(_, mut_ty) => {
                if mut_ty.mutbl == ast::Mutability::Mut {
                    ReceiverKind::RefMut
                } else {
                    ReceiverKind::Ref
                }
            }
            _ => ReceiverKind::Owned,
        }
    }

    fn collect_return_type(&self, output: &ast::FnRetTy) -> Option<Box<ast::Ty>> {
        match output {
            ast::FnRetTy::Ty(ty) => Some(ty.clone()),
            ast::FnRetTy::Default(..) => None,
        }
    }

    fn extract_type_name(&self, ty: &ast::Ty) -> Option<Symbol> {
        if let ast::TyKind::Path(_, path) = &ty.kind {
            path.segments.last().map(|seg| seg.ident.name)
        } else {
            None
        }
    }

    fn extract_param_name(&self, pat: &ast::Pat) -> Option<Symbol> {
        match &pat.kind {
            ast::PatKind::Ident(_, ident, _) => Some(ident.name),
            _ => Some(Symbol::intern("_")),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Phase B: Code generation and AST injection
    // ═══════════════════════════════════════════════════════════════════════════

    /// Apply mock bodies to all functions and methods in the crate.
    /// Follows the FunctionIntercept two-pass pattern:
    /// 1. Clone originals (renamed to _original)
    /// 2. Replace bodies with mock dispatch
    fn apply_mock_bodies(
        &self,
        compiler: &Compiler,
        krate: &mut ast::Crate,
        api: &CrateApiModel,
    ) {
        let mut fn_count = 0;

        // Replace function and method bodies in place
        for item in krate.items.iter_mut() {
            match &mut item.kind {
                ast::ItemKind::Fn(fn_data) if self.is_pub(&item.vis) => {
                    let name = fn_data.ident.name.as_str().to_string();
                    if let Some(func_model) = api.root.functions.iter().find(|f| f.name.as_str() == name) {
                        self.replace_fn_body(compiler, fn_data, &api.crate_name, func_model);
                        fn_count += 1;
                    }
                }
                ast::ItemKind::Impl(impl_data) => {
                    self.handle_impl_methods(compiler, impl_data, api);
                }
                ast::ItemKind::Mod(_safety, _ident, mod_kind) => {
                    self.handle_mod_mock_bodies(compiler, mod_kind, api);
                }
                _ => {}
            }
        }

        println!("CrateIntercept: replaced {} function bodies", fn_count);
    }

    /// Replace a function's body with the mock dispatch code.
    /// Replaces the entire Fn (sig + body) to avoid span/SyntaxContext mismatches
    /// between the parsed mock body and the original function params.
    fn replace_fn_body(
        &self,
        compiler: &Compiler,
        fn_data: &mut ast::Fn,
        crate_name: &str,
        func: &FunctionModel,
    ) {
        let mock_source = crate_mock_gen::gen_mock_fn_body(crate_name, func);

        // Parse the generated source as a complete function
        if let Some(parsed_fn) = self.parse_fn_item(compiler, &mock_source) {
            // Replace sig and body entirely (keeps the original item's visibility/attrs)
            fn_data.sig = parsed_fn.sig;
            fn_data.body = parsed_fn.body;
        } else {
            eprintln!("CrateIntercept: failed to parse mock body for fn {}", func.name);
        }
    }

    /// Handle mock body replacement for methods within an impl block.
    fn handle_impl_methods(
        &self,
        compiler: &Compiler,
        impl_data: &mut ast::Impl,
        api: &CrateApiModel,
    ) {
        let Some(self_type_name) = self.extract_type_name(&impl_data.self_ty) else {
            return;
        };

        // Find the matching ImplModel in the API
        let trait_name = impl_data.of_trait.as_ref().and_then(|h| {
            h.trait_ref.path.segments.last().map(|s| s.ident.name)
        });

        let Some(impl_model) = api.root.impls.iter().find(|i| {
            i.self_type_name == self_type_name && i.trait_name == trait_name
        }) else {
            return;
        };

        let struct_name = self_type_name.as_str().to_string();

        // Replace method bodies
        for assoc_item in impl_data.items.iter_mut() {
            if let ast::AssocItemKind::Fn(fn_data) = &mut assoc_item.kind {
                let method_name = fn_data.ident.name.as_str().to_string();
                if let Some(method_model) = impl_model.methods.iter().find(|m| m.name.as_str() == method_name) {
                    let mock_source = crate_mock_gen::gen_mock_method_body(
                        &api.crate_name,
                        &struct_name,
                        method_model,
                    );
                    if let Some(parsed_fn) = self.parse_fn_item(compiler, &mock_source) {
                        fn_data.sig = parsed_fn.sig;
                        fn_data.body = parsed_fn.body;
                    } else {
                        eprintln!("CrateIntercept: failed to parse mock body for method {}.{}", struct_name, method_name);
                    }
                }
            }
        }
    }

    /// Recursively handle modules.
    fn handle_mod_mock_bodies(
        &self,
        compiler: &Compiler,
        mod_kind: &mut ast::ModKind,
        api: &CrateApiModel,
    ) {
        if let ast::ModKind::Loaded(items, ..) = mod_kind {
            // For now, handle functions in submodules with the root API
            // TODO: properly match submodule functions to child ModuleModels
            for item in items.iter_mut() {
                if let ast::ItemKind::Impl(impl_data) = &mut item.kind {
                    self.handle_impl_methods(compiler, impl_data, api);
                }
            }
        }
    }

    /// Parse a generated function source string and extract the full Fn data.
    fn parse_fn_item(&self, compiler: &Compiler, source: &str) -> Option<Box<ast::Fn>> {
        let psess = &compiler.sess.psess;
        let filename = FileName::Custom("mock_generated".to_string());

        let mut parser = match rustc_parse::new_parser_from_source_str(
            psess,
            filename,
            source.to_string(),
        ) {
            Ok(parser) => parser,
            Err(diags) => {
                for d in diags {
                    d.cancel();
                }
                eprintln!("CrateIntercept: parse error for source:\n{}", source);
                return None;
            }
        };

        // Parse as a single item (function)
        match parser.parse_item(rustc_parse::parser::ForceCollect::No) {
            Ok(Some(item)) => {
                if let ast::ItemKind::Fn(fn_data) = item.kind {
                    Some(fn_data)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl rustc_driver::Callbacks for CrateIntercept {
    fn after_crate_root_parsing(
        &mut self,
        compiler: &Compiler,
        krate: &mut ast::Crate,
    ) -> Compilation {
        println!("CrateIntercept: collecting API for crate '{}'", self.crate_name);

        // Phase A: Collect the crate's public API
        let root = self.collect_module(
            Symbol::intern(&self.crate_name),
            &krate.items,
        );

        let api = CrateApiModel {
            crate_name: self.crate_name.clone(),
            root,
        };

        println!("CrateIntercept: collected API:");
        println!("  {} structs", api.root.structs.len());
        println!("  {} enums", api.root.enums.len());
        println!("  {} traits", api.root.traits.len());
        println!("  {} functions", api.root.functions.len());
        println!("  {} impls", api.root.impls.len());
        println!("  {} submodules", api.root.children.len());

        // Phase B: Generate mock dispatch bodies and inject into AST
        self.apply_mock_bodies(compiler, krate, &api);

        Compilation::Continue
    }
}
