//! Single-pass HIR-based analysis for the two-pass mock architecture.
//!
//! This module implements the **first pass** of the two-pass compilation strategy.
//! It runs `rustc` through full analysis to obtain the typed HIR, then walks the
//! public API surface — functions, structs, enums, traits, impls, and submodules —
//! collecting everything into a [`CrateApiModel`] using `TyCtxt` for fully-resolved
//! type information.
//!
//! Unlike the previous AST-based approach that used `after_expansion`, this version
//! collects everything in `after_analysis`, giving access to resolved types via
//! `tcx.fn_sig()`, `tcx.type_of()`, etc. This eliminates the need for a separate
//! enrichment pass and avoids depending on `rustc_ast` / `rustc_ast_pretty`.
//!
//! Unlike [`CrateIntercept`](crate::crate_intercept::CrateIntercept), this pass
//! does **not** modify the HIR. It stops compilation after `after_analysis`,
//! producing only the model. The second pass (substitution) then uses that model
//! to rewrite function bodies and inject mock infrastructure.
//!
//! ## Usage
//!
//! ```ignore
//! let mut pass = AnalysisPass::new("my_crate".to_string());
//! rustc_driver::run_compiler(&args, &mut pass);
//! let api = pass.into_model();
//! ```

use rustc_hir;
use rustc_middle::ty::TyCtxt;
use rustc_driver::Compilation;
use rustc_interface::interface::Compiler;

use crate::crate_api::*;
use crate::generics_model::*;
use crate::type_repr::TypeRepr;

/// The analysis pass — collects the crate's public API from the typed HIR
/// without modifying it.
pub struct AnalysisPass {
    crate_name: String,
    model: Option<CrateApiModel>,
}

impl AnalysisPass {
    pub fn new(crate_name: String) -> Self {
        Self {
            crate_name,
            model: None,
        }
    }

    pub fn into_model(self) -> CrateApiModel {
        self.model
            .expect("analysis pass did not produce a model — was after_analysis called?")
    }
}

impl rustc_driver::Callbacks for AnalysisPass {
    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &Compiler,
        tcx: TyCtxt<'tcx>,
    ) -> Compilation {
        log::debug!(
            "AnalysisPass: collecting typed HIR API for crate '{}'",
            self.crate_name
        );

        let mut root = collect_crate_api(tcx, &self.crate_name);

        // Post-process: strip module path prefixes from type representations.
        // Types from TyCtxt use crate-relative paths (e.g., `a::nested::Inner`),
        // but code generated inside `pub mod a { pub mod nested { ... } }` needs
        // just `Inner`. Recursively strip matching prefixes at each module level.
        strip_module_prefixes(&mut root, &[]);

        let api = CrateApiModel {
            crate_name: self.crate_name.clone(),
            root,
        };

        log::debug!("AnalysisPass: collected API:");
        log::debug!("  {} structs", api.root.structs.len());
        log::debug!("  {} enums", api.root.enums.len());
        log::debug!("  {} traits", api.root.traits.len());
        log::debug!("  {} functions", api.root.functions.len());
        log::debug!("  {} impls", api.root.impls.len());
        log::debug!("  {} submodules", api.root.children.len());

        self.model = Some(api);
        Compilation::Stop
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Top-level collection — walk HIR free items and build the crate model
// ═══════════════════════════════════════════════════════════════════════════════

/// Walk all top-level HIR items and collect the public API into a [`ModuleModel`].
///
/// Uses a three-pass approach:
/// 1. Collect type definitions, traits, functions, and submodules from `free_items()`
/// 2. Collect impl blocks with filtering (inherent + local trait → regenerate,
///    external trait → record as TraitImplInfo)
/// 3. Enrich with `all_local_trait_impls` for comprehensive coverage
fn collect_crate_api<'tcx>(tcx: TyCtxt<'tcx>, crate_name: &str) -> ModuleModel {
    let mut module = ModuleModel::new(crate_name);

    // Pass 1: Collect type definitions, traits, functions, and submodules
    // Only collect items that are direct children of the crate root (not nested in submodules)
    for item_id in tcx.hir_crate_items(()).free_items() {
        let item = tcx.hir_node(item_id.hir_id());
        let rustc_hir::Node::Item(item) = item else { continue };
        let local_def_id = item.owner_id.def_id;
        let def_id = local_def_id.to_def_id();

        // Skip items nested inside submodules — those are collected recursively
        if tcx.local_parent(local_def_id) != rustc_hir::def_id::CRATE_DEF_ID {
            continue;
        }

        // Skip non-public items (except impl blocks, which are handled in pass 2)
        let is_impl = matches!(item.kind, rustc_hir::ItemKind::Impl(_));
        if !is_impl && !tcx.visibility(def_id).is_public() {
            continue;
        }

        match &item.kind {
            rustc_hir::ItemKind::Fn { sig, body, .. } => {
                if let Some(func) = collect_function(tcx, def_id, sig, *body) {
                    module.functions.push(func);
                }
            }
            rustc_hir::ItemKind::Struct(_ident, _generics, variant_data) => {
                let name = tcx.item_name(def_id).to_string();
                if let Some(s) = collect_struct(tcx, def_id, &name, variant_data) {
                    module.structs.push(s);
                }
            }
            rustc_hir::ItemKind::Enum(_ident, _generics, enum_def) => {
                let name = tcx.item_name(def_id).to_string();
                if let Some(e) = collect_enum(tcx, def_id, &name, enum_def) {
                    module.enums.push(e);
                }
            }
            rustc_hir::ItemKind::Trait { ident, items, bounds, .. } => {
                let name = ident.name.to_string();
                if let Some(t) = collect_trait(tcx, def_id, &name, items, bounds) {
                    module.traits.push(t);
                }
            }
            rustc_hir::ItemKind::Mod(_ident, mod_data) => {
                let name = tcx.item_name(def_id).to_string();
                let child = collect_module(tcx, &name, mod_data);
                module.children.push(child);
            }
            _ => {}
        }
    }

    // Pass 2: Collect impl blocks with filtering (iterating free_items again)
    collect_impls_for_root(tcx, &mut module);

    // Pass 3: Enrich with all_local_trait_impls for comprehensive coverage
    enrich_trait_impls_from_tyctxt(tcx, &mut module);

    module
}

/// Collect items from a submodule by iterating `mod_data.item_ids`.
fn collect_module<'tcx>(
    tcx: TyCtxt<'tcx>,
    name: &str,
    mod_data: &rustc_hir::Mod<'tcx>,
) -> ModuleModel {
    let mut module = ModuleModel::new(name);

    // Pass 1: Collect type definitions, traits, functions, and nested modules
    for &item_id in mod_data.item_ids {
        let item = tcx.hir_node(item_id.hir_id());
        let rustc_hir::Node::Item(item) = item else { continue };
        let def_id = item.owner_id.def_id.to_def_id();

        let is_impl = matches!(item.kind, rustc_hir::ItemKind::Impl(_));
        if !is_impl && !tcx.visibility(def_id).is_public() {
            continue;
        }

        match &item.kind {
            rustc_hir::ItemKind::Fn { sig, body, .. } => {
                if let Some(func) = collect_function(tcx, def_id, sig, *body) {
                    module.functions.push(func);
                }
            }
            rustc_hir::ItemKind::Struct(_ident, _generics, variant_data) => {
                let name = tcx.item_name(def_id).to_string();
                if let Some(s) = collect_struct(tcx, def_id, &name, variant_data) {
                    module.structs.push(s);
                }
            }
            rustc_hir::ItemKind::Enum(_ident, _generics, enum_def) => {
                let name = tcx.item_name(def_id).to_string();
                if let Some(e) = collect_enum(tcx, def_id, &name, enum_def) {
                    module.enums.push(e);
                }
            }
            rustc_hir::ItemKind::Trait { ident, items, bounds, .. } => {
                let name = ident.name.to_string();
                if let Some(t) = collect_trait(tcx, def_id, &name, items, bounds) {
                    module.traits.push(t);
                }
            }
            rustc_hir::ItemKind::Mod(_ident, sub_mod_data) => {
                let name = tcx.item_name(def_id).to_string();
                let child = collect_module(tcx, &name, sub_mod_data);
                module.children.push(child);
            }
            _ => {}
        }
    }

    // Pass 2: Collect impl blocks from this module's items
    collect_impls_for_submodule(tcx, &mut module, mod_data);

    // Pass 3: Enrich trait impls
    enrich_trait_impls_from_tyctxt(tcx, &mut module);

    module
}

// ═══════════════════════════════════════════════════════════════════════════════
// Item collection — functions, structs, enums, traits, impls
// ═══════════════════════════════════════════════════════════════════════════════

/// Collect a public function's signature from HIR + TyCtxt.
fn collect_function<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: rustc_hir::def_id::DefId,
    sig: &rustc_hir::FnSig<'tcx>,
    body_id: rustc_hir::BodyId,
) -> Option<FunctionModel> {
    let name = tcx.item_name(def_id).to_string();
    let fn_sig = tcx.fn_sig(def_id).skip_binder().skip_binder();

    // Get parameter names from body params
    let body = tcx.hir_body(body_id);
    let input_tys = fn_sig.inputs();
    let params: Vec<ParamModel> = body
        .params
        .iter()
        .zip(input_tys.iter())
        .map(|(param, ty)| {
            let param_name = match param.pat.kind {
                rustc_hir::PatKind::Binding(_, _, ident, _) => ident.name.to_string(),
                _ => "_".to_string(),
            };
            ParamModel {
                name: param_name,
                ty: ty_to_type_repr(&format!("{}", ty)),
            }
        })
        .collect();

    let return_type = if fn_sig.output().is_unit() {
        TypeRepr::unit()
    } else {
        ty_to_type_repr(&format!("{}", fn_sig.output()))
    };

    let is_unsafe = sig.header.is_unsafe();
    let is_async = sig.header.is_async();

    Some(FunctionModel {
        name,
        generics: collect_generics(tcx, def_id),
        params,
        return_type,
        is_async,
        is_unsafe,
        abi: None,
    })
}

/// Collect a struct's fields from HIR + TyCtxt.
fn collect_struct<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: rustc_hir::def_id::DefId,
    name: &str,
    variant_data: &rustc_hir::VariantData<'tcx>,
) -> Option<StructModel> {
    let fields = collect_variant_fields(tcx, variant_data);
    let trackable = fields.iter().any(|f| !f.is_pub);
    let repr = convert_repr(tcx, def_id);

    Some(StructModel {
        name: name.to_string(),
        generics: collect_generics(tcx, def_id),
        fields,
        trackable,
        repr,
        trait_impls: Vec::new(),
    })
}

/// Collect fields from a VariantData, using TyCtxt for resolved types.
fn collect_variant_fields<'tcx>(
    tcx: TyCtxt<'tcx>,
    variant_data: &rustc_hir::VariantData<'tcx>,
) -> Vec<FieldModel> {
    match variant_data {
        rustc_hir::VariantData::Struct { fields, .. } => {
            fields
                .iter()
                .map(|field| {
                    let field_def_id = field.def_id.to_def_id();
                    let ty = tcx.type_of(field.def_id).skip_binder();
                    let is_pub = tcx.visibility(field_def_id).is_public();
                    FieldModel {
                        name: field.ident.name.to_string(),
                        ty: ty_to_type_repr(&format!("{}", ty)),
                        is_pub,
                    }
                })
                .collect()
        }
        rustc_hir::VariantData::Tuple(fields, _, _) => {
            fields
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    let field_def_id = field.def_id.to_def_id();
                    let ty = tcx.type_of(field.def_id).skip_binder();
                    let is_pub = tcx.visibility(field_def_id).is_public();
                    FieldModel {
                        name: format!("{}", i),
                        ty: ty_to_type_repr(&format!("{}", ty)),
                        is_pub,
                    }
                })
                .collect()
        }
        rustc_hir::VariantData::Unit(..) => Vec::new(),
    }
}

/// Collect an enum's variants from HIR + TyCtxt.
fn collect_enum<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: rustc_hir::def_id::DefId,
    name: &str,
    enum_def: &rustc_hir::EnumDef<'tcx>,
) -> Option<EnumModel> {
    let variants = enum_def
        .variants
        .iter()
        .map(|variant| {
            let variant_name = variant.ident.name.to_string();
            let fields = match &variant.data {
                rustc_hir::VariantData::Tuple(fields, _, _) => {
                    fields
                        .iter()
                        .map(|field| {
                            let ty = tcx.type_of(field.def_id).skip_binder();
                            ty_to_type_repr(&format!("{}", ty))
                        })
                        .collect()
                }
                rustc_hir::VariantData::Struct { fields, .. } => {
                    fields
                        .iter()
                        .map(|field| {
                            let ty = tcx.type_of(field.def_id).skip_binder();
                            ty_to_type_repr(&format!("{}", ty))
                        })
                        .collect()
                }
                rustc_hir::VariantData::Unit(..) => Vec::new(),
            };
            VariantModel {
                name: variant_name,
                fields,
            }
        })
        .collect();

    Some(EnumModel {
        name: name.to_string(),
        generics: collect_generics(tcx, def_id),
        variants,
        repr: None,
        trait_impls: Vec::new(),
    })
}

/// Collect a trait definition from HIR + TyCtxt.
fn collect_trait<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: rustc_hir::def_id::DefId,
    name: &str,
    trait_item_ids: &[rustc_hir::TraitItemId],
    bounds: &[rustc_hir::GenericBound<'tcx>],
) -> Option<TraitModel> {
    // Build the pretty-printer annotation for type printing
    let hir_ctx: &dyn rustc_hir::intravisit::HirTyCtxt<'_> = &tcx;
    let ann: &dyn rustc_hir_pretty::PpAnn = &hir_ctx;

    let methods = trait_item_ids
        .iter()
        .filter_map(|&trait_item_id| {
            let trait_item = tcx.hir_trait_item(trait_item_id);
            if let rustc_hir::TraitItemKind::Fn(ref fn_sig, _) = trait_item.kind {
                let method_def_id = trait_item.owner_id.def_id.to_def_id();
                let method_name = tcx.item_name(method_def_id).to_string();
                let ty_fn_sig = tcx.fn_sig(method_def_id).skip_binder().skip_binder();
                let receiver = detect_receiver(tcx, method_def_id, ty_fn_sig);

                // Collect params (skip self)
                let arg_idents = tcx.fn_arg_idents(method_def_id);
                let input_tys = ty_fn_sig.inputs();
                let has_self = !matches!(receiver, ReceiverKind::None);
                let skip = if has_self { 1 } else { 0 };

                let params: Vec<ParamModel> = arg_idents
                    .iter()
                    .skip(skip)
                    .zip(input_tys.iter().skip(skip))
                    .map(|(ident_opt, ty)| {
                        let param_name = ident_opt
                            .map(|i| i.name.to_string())
                            .unwrap_or_else(|| "_".to_string());
                        ParamModel {
                            name: param_name,
                            ty: ty_to_type_repr(&format!("{}", ty)),
                        }
                    })
                    .collect();

                let return_type = if ty_fn_sig.output().is_unit() {
                    TypeRepr::unit()
                } else {
                    ty_to_type_repr(&format!("{}", ty_fn_sig.output()))
                };

                Some(MethodSigModel {
                    name: method_name,
                    generics: collect_generics(tcx, method_def_id),
                    receiver,
                    params,
                    return_type,
                    is_pub: true,
                    is_async: fn_sig.header.is_async(),
                    is_unsafe: fn_sig.header.is_unsafe(),
                })
            } else {
                None
            }
        })
        .collect();

    // Collect associated types from the trait definition
    let associated_types: Vec<AssociatedTypeModel> = trait_item_ids
        .iter()
        .filter_map(|&trait_item_id| {
            let trait_item = tcx.hir_trait_item(trait_item_id);
            if let rustc_hir::TraitItemKind::Type(type_bounds, default_ty) = &trait_item.kind {
                let type_name = trait_item.ident.name.to_string();
                let bounds: Vec<TypeRepr> = type_bounds.iter().filter_map(|b| match b {
                    rustc_hir::GenericBound::Trait(poly) => {
                        let segments = &poly.trait_ref.path.segments;
                        let path: Vec<String> = segments.iter()
                            .map(|s| s.ident.name.to_string())
                            .collect();
                        if path.is_empty() { return None; }
                        let args = if let Some(last_seg) = segments.last() {
                            extract_generic_args_from_segment(tcx, last_seg)
                        } else {
                            vec![]
                        };
                        Some(TypeRepr::with_args(path, args))
                    }
                    _ => None,
                }).collect();
                let default = default_ty.map(|ty| {
                    ty_to_type_repr(&rustc_hir_pretty::ty_to_string(ann, ty))
                });
                Some(AssociatedTypeModel {
                    name: type_name,
                    bounds,
                    default,
                })
            } else {
                None
            }
        })
        .collect();

    Some(TraitModel {
        name: name.to_string(),
        generics: collect_generics(tcx, def_id),
        supertraits: collect_supertraits(tcx, bounds),
        methods,
        associated_types,
    })
}

/// Collect an impl block from HIR + TyCtxt.
fn collect_impl<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: rustc_hir::def_id::DefId,
    impl_data: &rustc_hir::Impl<'tcx>,
) -> Option<ImplModel> {
    // Build the pretty-printer annotation for type printing
    let hir_ctx: &dyn rustc_hir::intravisit::HirTyCtxt<'_> = &tcx;
    let ann: &dyn rustc_hir_pretty::PpAnn = &hir_ctx;

    // Get the self type
    let self_ty = tcx.type_of(def_id).skip_binder();
    let self_type_name = extract_type_name_from_ty(&format!("{}", self_ty));
    if self_type_name.is_empty() {
        return None;
    }
    let self_type = TypeRepr::simple(&self_type_name);

    // Get the trait name if this is a trait impl
    let (trait_name, trait_ref_type) = if impl_data.of_trait.is_some() {
        let trait_ref = tcx.impl_trait_ref(def_id);
        let inner = trait_ref.skip_binder();
        let trait_def_id = inner.def_id;
        let name = tcx.item_name(trait_def_id).to_string();
        let tr = TypeRepr::simple(&name);
        (Some(name), Some(tr))
    } else {
        (None, None)
    };

    let is_trait_impl = trait_name.is_some();

    // Collect methods from impl items
    let methods = impl_data
        .items
        .iter()
        .filter_map(|&impl_item_id| {
            let impl_item = tcx.hir_impl_item(impl_item_id);
            if let rustc_hir::ImplItemKind::Fn(ref _fn_sig, _body_id) = impl_item.kind {
                let method_def_id = impl_item.owner_id.def_id.to_def_id();
                collect_method_from_impl(tcx, method_def_id, &self_type_name, is_trait_impl)
            } else {
                None
            }
        })
        .collect();

    // Collect associated type bindings from impl items (e.g., `type Item = u32;`)
    let associated_types: Vec<AssociatedTypeBinding> = impl_data
        .items
        .iter()
        .filter_map(|&impl_item_id| {
            let impl_item = tcx.hir_impl_item(impl_item_id);
            if let rustc_hir::ImplItemKind::Type(ty) = &impl_item.kind {
                let type_name = impl_item.ident.name.to_string();
                let ty_str = rustc_hir_pretty::ty_to_string(ann, ty);
                Some(AssociatedTypeBinding {
                    name: type_name,
                    ty: ty_to_type_repr(&ty_str),
                })
            } else {
                None
            }
        })
        .collect();

    Some(ImplModel {
        self_type,
        self_type_name,
        trait_ref: trait_ref_type,
        trait_name,
        generics: collect_generics(tcx, def_id),
        methods,
        associated_types,
    })
}

/// Collect a method signature from an impl block.
fn collect_method_from_impl<'tcx>(
    tcx: TyCtxt<'tcx>,
    method_def_id: rustc_hir::def_id::DefId,
    _self_type_name: &str,
    is_trait_impl: bool,
) -> Option<MethodSigModel> {
    let method_name = tcx.item_name(method_def_id).to_string();
    let fn_sig = tcx.fn_sig(method_def_id).skip_binder().skip_binder();
    let receiver = detect_receiver(tcx, method_def_id, fn_sig);

    // Get parameter names
    let arg_idents = tcx.fn_arg_idents(method_def_id);
    let input_tys = fn_sig.inputs();
    let has_self = !matches!(receiver, ReceiverKind::None);
    let skip = if has_self { 1 } else { 0 };

    let params: Vec<ParamModel> = arg_idents
        .iter()
        .skip(skip)
        .zip(input_tys.iter().skip(skip))
        .map(|(ident_opt, ty)| {
            let param_name = ident_opt
                .map(|i| i.name.to_string())
                .unwrap_or_else(|| "_".to_string());
            ParamModel {
                name: param_name,
                ty: ty_to_type_repr(&format!("{}", ty)),
            }
        })
        .collect();

    let return_type = if fn_sig.output().is_unit() {
        TypeRepr::unit()
    } else {
        ty_to_type_repr(&format!("{}", fn_sig.output()))
    };

    // For inherent impls, only public methods; for trait impls, all methods
    let is_pub = if is_trait_impl {
        true
    } else {
        tcx.visibility(method_def_id).is_public()
    };

    Some(MethodSigModel {
        name: method_name,
        generics: collect_generics(tcx, method_def_id),
        receiver,
        params,
        return_type,
        is_pub,
        is_async: false,
        is_unsafe: false,
    })
}

/// Detect the receiver kind (`&self`, `&mut self`, `self`, or none) for a method.
///
/// Uses `fn_arg_idents` to check if the first parameter is named `self`, then
/// inspects the type signature to determine mutability/reference.
fn detect_receiver<'tcx>(
    tcx: TyCtxt<'tcx>,
    method_def_id: rustc_hir::def_id::DefId,
    fn_sig: rustc_middle::ty::FnSig<'tcx>,
) -> ReceiverKind {
    let arg_idents = tcx.fn_arg_idents(method_def_id);
    let first_arg = match arg_idents.first() {
        Some(Some(ident)) => ident,
        _ => return ReceiverKind::None,
    };

    if first_arg.name.as_str() != "self" {
        return ReceiverKind::None;
    }

    // The first input type tells us the receiver kind
    let Some(first_ty) = fn_sig.inputs().first() else {
        return ReceiverKind::None;
    };

    let ty_str = format!("{}", first_ty);
    if ty_str.starts_with("&mut ") {
        ReceiverKind::RefMut
    } else if ty_str.starts_with('&') {
        ReceiverKind::Ref
    } else {
        ReceiverKind::Owned
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Impl collection — two-pass filtering for known types/traits
// ═══════════════════════════════════════════════════════════════════════════════

/// Collect impl blocks from the crate root (free_items), filtering by known types/traits.
fn collect_impls_for_root<'tcx>(tcx: TyCtxt<'tcx>, module: &mut ModuleModel) {
    let known_types: std::collections::HashSet<String> = module
        .structs
        .iter()
        .map(|s| s.name.clone())
        .chain(module.enums.iter().map(|e| e.name.clone()))
        .collect();
    let known_traits: std::collections::HashSet<String> = module
        .traits
        .iter()
        .map(|t| t.name.clone())
        .collect();

    let mut external_trait_impls: Vec<(String, TraitImplInfo)> = Vec::new();

    for item_id in tcx.hir_crate_items(()).free_items() {
        let item = tcx.hir_node(item_id.hir_id());
        let rustc_hir::Node::Item(item) = item else { continue };

        // Only process impls that are direct children of the crate root
        let local_def_id = item.owner_id.def_id;
        if tcx.local_parent(local_def_id) != rustc_hir::def_id::CRATE_DEF_ID {
            continue;
        }

        if let rustc_hir::ItemKind::Impl(impl_data) = &item.kind {
            let def_id = local_def_id.to_def_id();
            process_impl_item(
                tcx,
                def_id,
                impl_data,
                &known_types,
                &known_traits,
                &mut module.impls,
                &mut external_trait_impls,
            );
        }
    }

    apply_external_trait_impls(&mut module.structs, &mut module.enums, external_trait_impls);
}

/// Collect impl blocks from a submodule's items.
fn collect_impls_for_submodule<'tcx>(
    tcx: TyCtxt<'tcx>,
    module: &mut ModuleModel,
    mod_data: &rustc_hir::Mod<'tcx>,
) {
    let known_types: std::collections::HashSet<String> = module
        .structs
        .iter()
        .map(|s| s.name.clone())
        .chain(module.enums.iter().map(|e| e.name.clone()))
        .collect();
    let known_traits: std::collections::HashSet<String> = module
        .traits
        .iter()
        .map(|t| t.name.clone())
        .collect();

    let mut external_trait_impls: Vec<(String, TraitImplInfo)> = Vec::new();

    for &item_id in mod_data.item_ids {
        let item = tcx.hir_node(item_id.hir_id());
        let rustc_hir::Node::Item(item) = item else { continue };

        if let rustc_hir::ItemKind::Impl(impl_data) = &item.kind {
            let def_id = item.owner_id.def_id.to_def_id();
            process_impl_item(
                tcx,
                def_id,
                impl_data,
                &known_types,
                &known_traits,
                &mut module.impls,
                &mut external_trait_impls,
            );
        }
    }

    apply_external_trait_impls(&mut module.structs, &mut module.enums, external_trait_impls);
}

/// Process a single impl item: classify it and either add to impls or external_trait_impls.
fn process_impl_item<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: rustc_hir::def_id::DefId,
    impl_data: &rustc_hir::Impl<'tcx>,
    known_types: &std::collections::HashSet<String>,
    known_traits: &std::collections::HashSet<String>,
    impls: &mut Vec<ImplModel>,
    external_trait_impls: &mut Vec<(String, TraitImplInfo)>,
) {
    if let Some(imp) = collect_impl(tcx, def_id, impl_data) {
        let type_is_known = known_types.contains(&imp.self_type_name);
        if !type_is_known {
            log::debug!(
                "AnalysisPass: skipping impl for unknown/non-public type '{}'",
                imp.self_type_name
            );
            return;
        }

        match &imp.trait_name {
            None => {
                // Inherent impl for a known type — always include for regeneration
                impls.push(imp);
            }
            Some(trait_name) => {
                if known_traits.contains(trait_name) {
                    // Trait impl for a known public trait — include for regeneration
                    impls.push(imp);
                } else {
                    // External trait impl — record for trait_impls but don't regenerate
                    log::debug!(
                        "AnalysisPass: recording external trait impl {} for {} (not regenerated)",
                        trait_name,
                        imp.self_type_name
                    );
                    external_trait_impls.push((
                        imp.self_type_name.clone(),
                        TraitImplInfo {
                            trait_path: trait_name.clone(),
                            is_auto: false,
                            is_blanket: false,
                            is_derived: false,
                        },
                    ));
                }
            }
        }
    }
}

/// Apply collected external trait impl info to struct/enum models.
fn apply_external_trait_impls(
    structs: &mut [StructModel],
    enums: &mut [EnumModel],
    external_trait_impls: Vec<(String, TraitImplInfo)>,
) {
    for (type_name, trait_info) in external_trait_impls {
        if let Some(s) = find_struct_mut(structs, &type_name) {
            s.trait_impls.push(trait_info);
        } else if let Some(e) = find_enum_mut(enums, &type_name) {
            e.trait_impls.push(trait_info);
        }
    }
}

/// Enrich trait impl information using `all_local_trait_impls` for comprehensive coverage.
///
/// This catches trait impls that might be missed by the item-by-item scan, such as
/// auto trait impls or impls in other modules.
fn enrich_trait_impls_from_tyctxt<'tcx>(tcx: TyCtxt<'tcx>, module: &mut ModuleModel) {
    let all_impls = tcx.all_local_trait_impls(());

    for (trait_def_id, impl_local_def_ids) in all_impls.iter() {
        let trait_name = tcx.item_name(*trait_def_id).to_string();

        for &impl_local_def_id in impl_local_def_ids {
            let impl_def_id = impl_local_def_id.to_def_id();
            let self_ty = tcx.type_of(impl_def_id).skip_binder();
            let type_name = extract_type_name_from_ty(&format!("{}", self_ty));

            // Check if this type is one of our known structs/enums
            let already_has = |trait_impls: &[TraitImplInfo]| -> bool {
                trait_impls.iter().any(|ti| ti.trait_path == trait_name)
            };

            if let Some(s) = find_struct_mut(&mut module.structs, &type_name) {
                if !already_has(&s.trait_impls) {
                    log::debug!(
                        "AnalysisPass: enriching struct {} with trait impl {}",
                        type_name,
                        trait_name
                    );
                    s.trait_impls.push(TraitImplInfo {
                        trait_path: trait_name.clone(),
                        is_auto: false,
                        is_blanket: false,
                        is_derived: false,
                    });
                }
            } else if let Some(e) = find_enum_mut(&mut module.enums, &type_name) {
                if !already_has(&e.trait_impls) {
                    log::debug!(
                        "AnalysisPass: enriching enum {} with trait impl {}",
                        type_name,
                        trait_name
                    );
                    e.trait_impls.push(TraitImplInfo {
                        trait_path: trait_name.clone(),
                        is_auto: false,
                        is_blanket: false,
                        is_derived: false,
                    });
                }
            }
        }
    }

    // Recursively enrich child modules
    for child in &mut module.children {
        enrich_trait_impls_from_tyctxt(tcx, child);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Type conversion and repr helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Bridge: convert a Ty string representation to [`TypeRepr::simple`].
///
/// This wraps the formatted type string in a single-segment `TypeRepr::Path`.
/// For most types this produces correct source output; complex types with generics
/// or references will still render correctly because the string representation
/// from `format!("{}", ty)` is valid Rust syntax.
fn ty_to_type_repr(ty_str: &str) -> TypeRepr {
    if ty_str == "()" {
        TypeRepr::unit()
    } else {
        TypeRepr::simple(ty_str)
    }
}

/// Convert `ReprOptions` from `adt_def` to our [`ReprInfo`] model.
///
/// Returns `None` for default Rust repr with no modifiers.
fn convert_repr<'tcx>(
    tcx: TyCtxt<'tcx>,
    def_id: rustc_hir::def_id::DefId,
) -> Option<ReprInfo> {
    let adt = tcx.adt_def(def_id);
    let repr = adt.repr();

    let layout = if repr.c() {
        PrimaryLayout::C
    } else if repr.transparent() {
        PrimaryLayout::Transparent
    } else if let Some(int_ty) = repr.int {
        PrimaryLayout::Int(format!("{:?}", int_ty).to_lowercase())
    } else {
        PrimaryLayout::Rust
    };

    let alignment = if repr.packed() {
        let pack_val = repr.pack.map(|a| a.bytes() as u32).unwrap_or(1);
        Some(AlignmentModifier::Packed(pack_val))
    } else if let Some(align) = repr.align {
        Some(AlignmentModifier::Align(align.bytes() as u32))
    } else {
        None
    };

    let info = ReprInfo { layout, alignment };
    if info.is_default() {
        None
    } else {
        Some(info)
    }
}

/// Extract a simple type name from a formatted Ty string.
///
/// Strips generic parameters and path prefixes to get just the type name.
/// e.g. `"MyStruct<T>"` → `"MyStruct"`, `"crate::Foo"` → `"Foo"`
fn extract_type_name_from_ty(ty_str: &str) -> String {
    // Strip generic params
    let base = if let Some(idx) = ty_str.find('<') {
        &ty_str[..idx]
    } else {
        ty_str
    };
    // Take last path segment
    if let Some(idx) = base.rfind("::") {
        base[idx + 2..].to_string()
    } else {
        base.to_string()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helper functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Find a mutable reference to a struct by name.
fn find_struct_mut<'a>(structs: &'a mut [StructModel], name: &str) -> Option<&'a mut StructModel> {
    structs.iter_mut().find(|s| s.name == name)
}

/// Find a mutable reference to an enum by name.
fn find_enum_mut<'a>(enums: &'a mut [EnumModel], name: &str) -> Option<&'a mut EnumModel> {
    enums.iter_mut().find(|e| e.name == name)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Generics and supertrait collection from HIR
// ═══════════════════════════════════════════════════════════════════════════════

/// Collect generic parameters, bounds, and where clauses for any item via its DefId.
///
/// Looks up the HIR node for the item and extracts generics from it. Works for
/// functions, structs, enums, traits, impls, trait items, and impl items.
/// Uses `rustc_hir_pretty::ty_to_string` for proper type formatting.
fn collect_generics<'tcx>(tcx: TyCtxt<'tcx>, def_id: rustc_hir::def_id::DefId) -> GenericsModel {
    let Some(local_id) = def_id.as_local() else {
        return GenericsModel::empty();
    };

    // Look up HIR generics for this item
    let hir_generics = match tcx.hir_node_by_def_id(local_id) {
        rustc_hir::Node::Item(item) => match &item.kind {
            rustc_hir::ItemKind::Fn { generics, .. } => Some(*generics),
            rustc_hir::ItemKind::Struct(_, generics, _) => Some(*generics),
            rustc_hir::ItemKind::Enum(_, generics, _) => Some(*generics),
            rustc_hir::ItemKind::Trait { generics, .. } => Some(*generics),
            rustc_hir::ItemKind::Impl(impl_data) => Some(impl_data.generics),
            _ => None,
        },
        rustc_hir::Node::TraitItem(item) => Some(item.generics),
        rustc_hir::Node::ImplItem(item) => Some(item.generics),
        _ => None,
    };

    let Some(hir_gen) = hir_generics else {
        return GenericsModel::empty();
    };

    // Build the pretty-printer annotation from TyCtxt
    let hir_ctx: &dyn rustc_hir::intravisit::HirTyCtxt<'_> = &tcx;
    let ann: &dyn rustc_hir_pretty::PpAnn = &hir_ctx;

    let mut lifetimes = Vec::new();
    let mut type_params = Vec::new();
    let mut const_params = Vec::new();
    let mut where_clauses = Vec::new();

    for param in hir_gen.params {
        // Collect trait bounds for this parameter
        let bounds: Vec<TypeRepr> = hir_gen
            .bounds_for_param(param.def_id)
            .flat_map(|bp| {
                bp.bounds.iter().filter_map(|b| match b {
                    rustc_hir::GenericBound::Trait(poly) => {
                        let segments = &poly.trait_ref.path.segments;
                        let path: Vec<String> = segments
                            .iter()
                            .map(|s| s.ident.name.to_string())
                            .collect();
                        if path.is_empty() {
                            return None;
                        }
                        let args = if let Some(last_seg) = segments.last() {
                            extract_generic_args_from_segment(tcx, last_seg)
                        } else {
                            vec![]
                        };
                        Some(TypeRepr::with_args(path, args))
                    }
                    _ => None,
                })
            })
            .collect();

        // Collect lifetime bounds separately
        let lifetime_bounds: Vec<String> = hir_gen
            .bounds_for_param(param.def_id)
            .flat_map(|bp| {
                bp.bounds.iter().filter_map(|b| match b {
                    rustc_hir::GenericBound::Outlives(lt) => {
                        Some(format!("'{}", lt.ident.name))
                    }
                    _ => None,
                })
            })
            .collect();

        match &param.kind {
            rustc_hir::GenericParamKind::Lifetime { .. } => {
                lifetimes.push(LifetimeParam {
                    name: format!("'{}", param.name.ident().name),
                    bounds: lifetime_bounds,
                });
            }
            rustc_hir::GenericParamKind::Type { default, .. } => {
                let default_type = default.map(|hir_ty| {
                    ty_to_type_repr(&rustc_hir_pretty::ty_to_string(ann, hir_ty))
                });
                type_params.push(TypeParam {
                    name: param.name.ident().name.to_string(),
                    bounds,
                    default: default_type,
                });
            }
            rustc_hir::GenericParamKind::Const { ty, .. } => {
                const_params.push(ConstParam {
                    name: param.name.ident().name.to_string(),
                    ty: ty_to_type_repr(&rustc_hir_pretty::ty_to_string(ann, ty)),
                });
            }
        }
    }

    // Collect where clause predicates
    for pred in hir_gen.predicates {
        match pred.kind {
            rustc_hir::WherePredicateKind::BoundPredicate(bound_pred) => {
                let pred_bounds: Vec<TypeRepr> = bound_pred
                    .bounds
                    .iter()
                    .filter_map(|b| match b {
                        rustc_hir::GenericBound::Trait(poly) => {
                            let segments = &poly.trait_ref.path.segments;
                            let path: Vec<String> = segments
                                .iter()
                                .map(|s| s.ident.name.to_string())
                                .collect();
                            if path.is_empty() {
                                return None;
                            }
                            let args = if let Some(last_seg) = segments.last() {
                                extract_generic_args_from_segment(tcx, last_seg)
                            } else {
                                vec![]
                            };
                            Some(TypeRepr::with_args(path, args))
                        }
                        rustc_hir::GenericBound::Outlives(lt) => {
                            // Represent lifetime bounds as simple TypeRepr
                            // e.g., 'static becomes TypeRepr::simple("'static")
                            Some(TypeRepr::simple(&format!("'{}", lt.ident.name)))
                        }
                        _ => None,
                    })
                    .collect();

                if !pred_bounds.is_empty() {
                    let bounded_ty_str = rustc_hir_pretty::ty_to_string(ann, bound_pred.bounded_ty);
                    where_clauses.push(WherePredicate::TypeBound {
                        ty: ty_to_type_repr(&bounded_ty_str),
                        bounds: pred_bounds,
                    });
                }
            }
            rustc_hir::WherePredicateKind::RegionPredicate(region_pred) => {
                let lifetime_bounds: Vec<String> = region_pred.bounds.iter().filter_map(|b| {
                    match b {
                        rustc_hir::GenericBound::Outlives(lt) => Some(format!("'{}", lt.ident.name)),
                        _ => None,
                    }
                }).collect();
                if !lifetime_bounds.is_empty() {
                    where_clauses.push(WherePredicate::LifetimeBound {
                        lifetime: format!("'{}", region_pred.lifetime.ident.name),
                        bounds: lifetime_bounds,
                    });
                }
            }
            _ => {
                // Other predicate kinds (e.g., equality predicates) are unstable
                // or rarely used; skip for now.
            }
        }
    }

    GenericsModel {
        lifetimes,
        type_params,
        const_params,
        where_clauses,
    }
}

/// Extract generic arguments from a HIR path segment.
///
/// Handles positional type/lifetime/const args and associated type constraint
/// bindings (e.g., `Item = u32` in `Iterator<Item = u32>`).
fn extract_generic_args_from_segment<'tcx>(
    tcx: TyCtxt<'tcx>,
    segment: &rustc_hir::PathSegment<'tcx>,
) -> Vec<crate::type_repr::GenericArg> {
    use crate::type_repr::GenericArg as GA;

    let Some(args) = segment.args else { return vec![]; };

    let hir_ctx: &dyn rustc_hir::intravisit::HirTyCtxt<'_> = &tcx;
    let ann: &dyn rustc_hir_pretty::PpAnn = &hir_ctx;

    let mut result = Vec::new();

    // Process positional args (types, lifetimes, consts)
    for arg in args.args {
        match arg {
            rustc_hir::GenericArg::Type(ty) => {
                let ty_str = rustc_hir_pretty::ty_to_string(ann, ty.as_unambig_ty());
                result.push(GA::Type(ty_to_type_repr(&ty_str)));
            }
            rustc_hir::GenericArg::Lifetime(lt) => {
                let name = lt.ident.name.to_string();
                if name != "'_" {
                    result.push(GA::Lifetime(crate::type_repr::Lifetime::new(
                        name.trim_start_matches('\''),
                    )));
                }
            }
            rustc_hir::GenericArg::Const(ct) => {
                result.push(GA::Const(format!("{:?}", ct)));
            }
            _ => {}
        }
    }

    // Process associated type constraints (bindings like Item = u32)
    for constraint in args.constraints {
        match constraint.kind {
            rustc_hir::AssocItemConstraintKind::Equality { ref term } => {
                match term {
                    rustc_hir::Term::Ty(ty) => {
                        let ty_str = rustc_hir_pretty::ty_to_string(ann, ty);
                        result.push(GA::Binding(
                            constraint.ident.name.to_string(),
                            ty_to_type_repr(&ty_str),
                        ));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    result
}

/// Collect supertraits from the bounds on a trait definition.
fn collect_supertraits<'tcx>(tcx: TyCtxt<'tcx>, bounds: &[rustc_hir::GenericBound<'tcx>]) -> Vec<TypeRepr> {
    bounds
        .iter()
        .filter_map(|b| match b {
            rustc_hir::GenericBound::Trait(poly) => {
                let segments = &poly.trait_ref.path.segments;
                let path: Vec<String> = segments
                    .iter()
                    .map(|s| s.ident.name.to_string())
                    .collect();
                if path.is_empty() {
                    return None;
                }
                let args = if let Some(last_seg) = segments.last() {
                    extract_generic_args_from_segment(tcx, last_seg)
                } else {
                    vec![]
                };
                Some(TypeRepr::with_args(path, args))
            }
            _ => None,
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Post-processing: strip module path prefixes from type representations
// ═══════════════════════════════════════════════════════════════════════════════

/// Recursively strip module path prefixes from all type representations in the model.
///
/// At each module level, types that start with the current module path get their
/// prefix stripped. For example, inside `mod a { mod nested { ... } }`, a type
/// `a::nested::Inner` becomes just `Inner`.
fn strip_module_prefixes(module: &mut ModuleModel, current_path: &[&str]) {
    // Build the prefix string for this module level (e.g., "a::nested::")
    let prefix = if current_path.is_empty() {
        String::new()
    } else {
        format!("{}::", current_path.join("::"))
    };

    // Strip prefixes from all type representations in this module's items
    if !prefix.is_empty() {
        for func in &mut module.functions {
            strip_prefix_from_type(&mut func.return_type, &prefix);
            for param in &mut func.params {
                strip_prefix_from_type(&mut param.ty, &prefix);
            }
        }
        for s in &mut module.structs {
            for field in &mut s.fields {
                strip_prefix_from_type(&mut field.ty, &prefix);
            }
        }
        for e in &mut module.enums {
            for variant in &mut e.variants {
                for field_ty in &mut variant.fields {
                    strip_prefix_from_type(field_ty, &prefix);
                }
            }
        }
        for t in &mut module.traits {
            for method in &mut t.methods {
                strip_prefix_from_type(&mut method.return_type, &prefix);
                for param in &mut method.params {
                    strip_prefix_from_type(&mut param.ty, &prefix);
                }
            }
        }
        for imp in &mut module.impls {
            strip_prefix_from_type(&mut imp.self_type, &prefix);
            for method in &mut imp.methods {
                strip_prefix_from_type(&mut method.return_type, &prefix);
                for param in &mut method.params {
                    strip_prefix_from_type(&mut param.ty, &prefix);
                }
            }
        }
    }

    // Recurse into child modules with the extended path
    for child in &mut module.children {
        let child_name = child.name.clone();
        let mut child_path: Vec<&str> = current_path.to_vec();
        child_path.push(&child_name);
        strip_module_prefixes(child, &child_path);
    }
}

/// Strip a module path prefix from a TypeRepr if it's a simple path that starts
/// with the prefix.
///
/// For example, with prefix "a::nested::", the type "a::nested::Inner" becomes "Inner".
fn strip_prefix_from_type(ty: &mut TypeRepr, prefix: &str) {
    // Currently TypeRepr::simple stores the entire type string as a single path segment.
    // We check if that segment starts with the prefix and strip it.
    if let TypeRepr::Path { first, rest, .. } = ty {
        if rest.is_empty() {
            // Single-segment path (from ty_to_type_repr bridge) — the whole type string
            // is in `first`. Check if it starts with the module prefix.
            if first.starts_with(prefix) {
                *first = first[prefix.len()..].to_string();
            }
        }
        // For multi-segment paths (when we have proper structural TypeRepr),
        // we'd strip matching leading segments. TODO for later.
    }
}
