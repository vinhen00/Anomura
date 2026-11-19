//! Single-pass HIR-based analysis for the two-pass mock architecture.
//!
//! This module implements the **first pass** of the two-pass compilation strategy.
//! It runs `rustc` through full analysis to obtain the typed HIR, then walks the
//! public API surface — functions, structs, enums, traits, impls, and submodules —
//! collecting everything into a [`CrateApiModel`] using `TyCtxt` for fully-resolved
//! type information.
//!
//! All items are stored in a flat [`ItemStore`] and referenced by [`ItemId`].

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
        Self { crate_name, model: None }
    }

    pub fn into_model(self) -> CrateApiModel {
        self.model.expect("analysis pass did not produce a model — was after_analysis called?")
    }
}

impl rustc_driver::Callbacks for AnalysisPass {
    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &Compiler,
        tcx: TyCtxt<'tcx>,
    ) -> Compilation {
        log::debug!("AnalysisPass: collecting typed HIR API for crate '{}'", self.crate_name);

        let mut items = ItemStore::new();
        let root_id = collect_crate_api(tcx, &self.crate_name, &mut items);

        strip_module_prefixes(&mut items, root_id, &[]);

        let api = CrateApiModel { crate_name: self.crate_name.clone(), items, root: root_id };

        let (structs, enums, functions, traits, impls) = api.count_by_kind();
        log::debug!("AnalysisPass: collected {} structs, {} enums, {} traits, {} functions, {} impls, {} total items",
            structs, enums, traits, functions, impls, api.items.len());

        self.model = Some(api);
        Compilation::Stop
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Top-level collection
// ═══════════════════════════════════════════════════════════════════════════════

fn collect_crate_api<'tcx>(tcx: TyCtxt<'tcx>, crate_name: &str, items: &mut ItemStore) -> ItemId {
    let root_id = items.alloc(crate_name, Visibility::Public, ItemKind::Module(ModuleData { children: Vec::new() }));
    let mut children = Vec::new();

    // Pass 1: type defs, traits, functions, submodules
    for item_id in tcx.hir_crate_items(()).free_items() {
        let item = tcx.hir_node(item_id.hir_id());
        let rustc_hir::Node::Item(item) = item else { continue };
        let local_def_id = item.owner_id.def_id;
        let def_id = local_def_id.to_def_id();

        if tcx.local_parent(local_def_id) != rustc_hir::def_id::CRATE_DEF_ID { continue; }

        let is_impl = matches!(item.kind, rustc_hir::ItemKind::Impl(_));
        if !is_impl && !tcx.visibility(def_id).is_public() { continue; }

        match &item.kind {
            rustc_hir::ItemKind::Fn { sig, body, .. } => {
                if let Some(id) = collect_function(tcx, def_id, sig, *body, items) { children.push(id); }
            }
            rustc_hir::ItemKind::Struct(_ident, _generics, vd) => {
                let name = tcx.item_name(def_id).to_string();
                if let Some(id) = collect_struct(tcx, def_id, &name, vd, items) { children.push(id); }
            }
            rustc_hir::ItemKind::Enum(_ident, _generics, ed) => {
                let name = tcx.item_name(def_id).to_string();
                if let Some(id) = collect_enum(tcx, def_id, &name, ed, items) { children.push(id); }
            }
            rustc_hir::ItemKind::Trait { ident, items: ti, bounds, .. } => {
                let name = ident.name.to_string();
                if let Some(id) = collect_trait(tcx, def_id, &name, ti, bounds, items) { children.push(id); }
            }
            rustc_hir::ItemKind::Mod(_ident, md) => {
                let name = tcx.item_name(def_id).to_string();
                children.push(collect_module(tcx, &name, md, items));
            }
            _ => {}
        }
    }

    // Pass 2: impl blocks
    let (known_types, known_traits) = known_types_and_traits(items, &children);
    collect_impls_from_iter(
        tcx, items, &known_types, &known_traits, &mut children,
        IterSource::Root,
    );

    set_module_children(items, root_id, children);
    root_id
}

fn collect_module<'tcx>(tcx: TyCtxt<'tcx>, name: &str, mod_data: &rustc_hir::Mod<'tcx>, items: &mut ItemStore) -> ItemId {
    let mod_id = items.alloc(name, Visibility::Public, ItemKind::Module(ModuleData { children: Vec::new() }));
    let mut children = Vec::new();

    for &item_id in mod_data.item_ids {
        let item = tcx.hir_node(item_id.hir_id());
        let rustc_hir::Node::Item(item) = item else { continue };
        let def_id = item.owner_id.def_id.to_def_id();

        let is_impl = matches!(item.kind, rustc_hir::ItemKind::Impl(_));
        if !is_impl && !tcx.visibility(def_id).is_public() { continue; }

        match &item.kind {
            rustc_hir::ItemKind::Fn { sig, body, .. } => {
                if let Some(id) = collect_function(tcx, def_id, sig, *body, items) { children.push(id); }
            }
            rustc_hir::ItemKind::Struct(_ident, _generics, vd) => {
                let name = tcx.item_name(def_id).to_string();
                if let Some(id) = collect_struct(tcx, def_id, &name, vd, items) { children.push(id); }
            }
            rustc_hir::ItemKind::Enum(_ident, _generics, ed) => {
                let name = tcx.item_name(def_id).to_string();
                if let Some(id) = collect_enum(tcx, def_id, &name, ed, items) { children.push(id); }
            }
            rustc_hir::ItemKind::Trait { ident, items: ti, bounds, .. } => {
                let name = ident.name.to_string();
                if let Some(id) = collect_trait(tcx, def_id, &name, ti, bounds, items) { children.push(id); }
            }
            rustc_hir::ItemKind::Mod(_ident, sub) => {
                let name = tcx.item_name(def_id).to_string();
                children.push(collect_module(tcx, &name, sub, items));
            }
            _ => {}
        }
    }

    let (known_types, known_traits) = known_types_and_traits(items, &children);
    collect_impls_from_iter(
        tcx, items, &known_types, &known_traits, &mut children,
        IterSource::Submodule(mod_data),
    );

    set_module_children(items, mod_id, children);
    mod_id
}

fn set_module_children(items: &mut ItemStore, mod_id: ItemId, children: Vec<ItemId>) {
    if let ItemKind::Module(ref mut m) = items.get_mut(mod_id).kind {
        m.children = children;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Item collection
// ═══════════════════════════════════════════════════════════════════════════════

fn collect_function<'tcx>(
    tcx: TyCtxt<'tcx>, def_id: rustc_hir::def_id::DefId,
    sig: &rustc_hir::FnSig<'tcx>, body_id: rustc_hir::BodyId, items: &mut ItemStore,
) -> Option<ItemId> {
    let name = tcx.item_name(def_id).to_string();
    let fn_sig = tcx.fn_sig(def_id).skip_binder().skip_binder();
    let body = tcx.hir_body(body_id);

    let params: Vec<ParamModel> = body.params.iter()
        .zip(fn_sig.inputs().iter())
        .map(|(param, ty)| {
            let pname = match param.pat.kind {
                rustc_hir::PatKind::Binding(_, _, ident, _) => ident.name.to_string(),
                _ => "_".to_string(),
            };
            ParamModel { name: pname, ty: ty_to_type_repr(&format!("{}", ty)) }
        })
        .collect();

    let return_type = if fn_sig.output().is_unit() { TypeRepr::unit() } else { ty_to_type_repr(&format!("{}", fn_sig.output())) };

    Some(items.alloc(&name, Visibility::Public, ItemKind::Function(FunctionData {
        generics: collect_generics(tcx, def_id),
        qualifiers: collect_fn_qualifiers(&sig.header),
        params,
        return_type,
    })))
}

fn collect_struct<'tcx>(
    tcx: TyCtxt<'tcx>, def_id: rustc_hir::def_id::DefId, name: &str,
    variant_data: &rustc_hir::VariantData<'tcx>, items: &mut ItemStore,
) -> Option<ItemId> {
    let (kind, field_ids) = collect_fields(tcx, variant_data, items);
    let trackable = field_ids.iter().any(|&fid| !items.get(fid).is_pub());
    let repr = convert_repr(tcx, def_id);

    let sid = items.alloc(name, Visibility::Public, ItemKind::Struct(StructData {
        generics: collect_generics(tcx, def_id),
        kind,
        fields: field_ids,
        trackable,
    }));

    if let Some(repr_info) = repr {
        items.get_mut(sid).attrs.push(Attribute::Repr(repr_info));
    }
    Some(sid)
}

fn collect_fields<'tcx>(
    tcx: TyCtxt<'tcx>, variant_data: &rustc_hir::VariantData<'tcx>, items: &mut ItemStore,
) -> (StructKind, Vec<ItemId>) {
    match variant_data {
        rustc_hir::VariantData::Struct { fields, .. } => {
            let ids = fields.iter().map(|field| {
                let vis = if tcx.visibility(field.def_id.to_def_id()).is_public() { Visibility::Public } else { Visibility::Private };
                let ty = tcx.type_of(field.def_id).skip_binder();
                items.alloc(field.ident.name.to_string(), vis, ItemKind::Field(FieldData { ty: ty_to_type_repr(&format!("{}", ty)) }))
            }).collect();
            (StructKind::Plain, ids)
        }
        rustc_hir::VariantData::Tuple(fields, _, _) => {
            let ids = fields.iter().enumerate().map(|(i, field)| {
                let vis = if tcx.visibility(field.def_id.to_def_id()).is_public() { Visibility::Public } else { Visibility::Private };
                let ty = tcx.type_of(field.def_id).skip_binder();
                items.alloc(format!("{}", i), vis, ItemKind::Field(FieldData { ty: ty_to_type_repr(&format!("{}", ty)) }))
            }).collect();
            (StructKind::Tuple, ids)
        }
        rustc_hir::VariantData::Unit(..) => (StructKind::Unit, Vec::new()),
    }
}

fn collect_enum<'tcx>(
    tcx: TyCtxt<'tcx>, def_id: rustc_hir::def_id::DefId, name: &str,
    enum_def: &rustc_hir::EnumDef<'tcx>, items: &mut ItemStore,
) -> Option<ItemId> {
    let variant_ids: Vec<ItemId> = enum_def.variants.iter().map(|variant| {
        let vname = variant.ident.name.to_string();
        let (kind, field_ids): (StructKind, Vec<ItemId>) = match &variant.data {
            rustc_hir::VariantData::Tuple(fields, _, _) => (StructKind::Tuple, fields.iter().enumerate().map(|(i, f)| {
                let ty = tcx.type_of(f.def_id).skip_binder();
                items.alloc(format!("{}", i), Visibility::Public, ItemKind::Field(FieldData { ty: ty_to_type_repr(&format!("{}", ty)) }))
            }).collect()),
            rustc_hir::VariantData::Struct { fields, .. } => (StructKind::Plain, fields.iter().map(|f| {
                let ty = tcx.type_of(f.def_id).skip_binder();
                items.alloc(f.ident.name.to_string(), Visibility::Public, ItemKind::Field(FieldData { ty: ty_to_type_repr(&format!("{}", ty)) }))
            }).collect()),
            rustc_hir::VariantData::Unit(..) => (StructKind::Unit, Vec::new()),
        };
        items.alloc(vname, Visibility::Public, ItemKind::Variant(VariantData { kind, fields: field_ids }))
    }).collect();

    Some(items.alloc(name, Visibility::Public, ItemKind::Enum(EnumData {
        generics: collect_generics(tcx, def_id),
        variants: variant_ids,
    })))
}

fn collect_trait<'tcx>(
    tcx: TyCtxt<'tcx>, def_id: rustc_hir::def_id::DefId, name: &str,
    trait_item_ids: &[rustc_hir::TraitItemId],
    bounds: &[rustc_hir::GenericBound<'tcx>],
    items: &mut ItemStore,
) -> Option<ItemId> {
    let hir_ctx: &dyn rustc_hir::intravisit::HirTyCtxt<'_> = &tcx;
    let ann: &dyn rustc_hir_pretty::PpAnn = &hir_ctx;

    let mut method_ids = Vec::new();
    let mut assoc_type_ids = Vec::new();

    for &tid in trait_item_ids {
        let ti = tcx.hir_trait_item(tid);
        match &ti.kind {
            rustc_hir::TraitItemKind::Fn(fn_sig, _) => {
                let mdid = ti.owner_id.def_id.to_def_id();
                let mname = tcx.item_name(mdid).to_string();
                let ty_fn_sig = tcx.fn_sig(mdid).skip_binder().skip_binder();
                let receiver = detect_receiver(tcx, mdid, ty_fn_sig);
                let (params, return_type) = collect_fn_params_and_ret(tcx, mdid, ty_fn_sig, receiver.has_self());
                method_ids.push(items.alloc(&mname, Visibility::Public, ItemKind::Method(MethodData {
                    generics: collect_generics(tcx, mdid),
                    qualifiers: collect_fn_qualifiers(&fn_sig.header),
                    receiver, params, return_type,
                })));
            }
            rustc_hir::TraitItemKind::Type(type_bounds, default_ty) => {
                let tname = ti.ident.name.to_string();
                let bounds: Vec<Bound> = type_bounds.iter().filter_map(|b| hir_bound_to_bound(tcx, b)).collect();
                let default = default_ty.map(|ty| ty_to_type_repr(&rustc_hir_pretty::ty_to_string(ann, ty)));
                assoc_type_ids.push(items.alloc(&tname, Visibility::Public, ItemKind::AssociatedType(AssociatedTypeData { bounds, ty: default })));
            }
            _ => {}
        }
    }

    Some(items.alloc(name, Visibility::Public, ItemKind::Trait(TraitData {
        generics: collect_generics(tcx, def_id),
        supertraits: collect_supertraits(tcx, bounds),
        methods: method_ids,
        associated_types: assoc_type_ids,
    })))
}

fn collect_impl_item<'tcx>(
    tcx: TyCtxt<'tcx>, def_id: rustc_hir::def_id::DefId,
    impl_data: &rustc_hir::Impl<'tcx>, items: &mut ItemStore,
) -> Option<ItemId> {
    let hir_ctx: &dyn rustc_hir::intravisit::HirTyCtxt<'_> = &tcx;
    let ann: &dyn rustc_hir_pretty::PpAnn = &hir_ctx;

    let self_ty = tcx.type_of(def_id).skip_binder();
    let self_ty_str = format!("{}", self_ty);
    let self_type_name = extract_type_name_from_ty(&self_ty_str);
    if self_type_name.is_empty() { return None; }

    let (trait_name, trait_ref_type) = if impl_data.of_trait.is_some() {
        let trait_ref = tcx.impl_trait_ref(def_id);
        let inner = trait_ref.skip_binder();
        let trait_def_id = inner.def_id;
        let name = tcx.item_name(trait_def_id).to_string();
        let segs: Vec<String> = tcx.def_path_str(trait_def_id).split("::").map(|s| s.to_string()).collect();
        let args: Vec<crate::type_repr::GenericArg> = inner.args.iter().skip(1)
            .map(|arg| crate::type_repr::GenericArg::Type(ty_to_type_repr(&format!("{}", arg))))
            .collect();
        let tr = if args.is_empty() { TypeRepr::simple(&tcx.def_path_str(trait_def_id)) } else { TypeRepr::with_args(segs, args) };
        (Some(name), Some(tr))
    } else {
        (None, None)
    };

    let is_trait_impl = trait_name.is_some();
    let mut method_ids = Vec::new();
    let mut assoc_type_ids = Vec::new();

    for &ii_id in impl_data.items {
        let ii = tcx.hir_impl_item(ii_id);
        match &ii.kind {
            rustc_hir::ImplItemKind::Fn(..) => {
                let mdid = ii.owner_id.def_id.to_def_id();
                if let Some(mid) = collect_method_from_impl(tcx, mdid, is_trait_impl, items) {
                    method_ids.push(mid);
                }
            }
            rustc_hir::ImplItemKind::Type(ty) => {
                let tname = ii.ident.name.to_string();
                let ty_str = rustc_hir_pretty::ty_to_string(ann, ty);
                assoc_type_ids.push(items.alloc(&tname, Visibility::Public,
                    ItemKind::AssociatedType(AssociatedTypeData { bounds: Vec::new(), ty: Some(ty_to_type_repr(&ty_str)) })));
            }
            _ => {}
        }
    }

    Some(items.alloc(
        format!("impl_{}", self_type_name), Visibility::Public,
        ItemKind::Impl(ImplData {
            self_type: ty_to_type_repr(&self_ty_str), self_type_name, trait_ref: trait_ref_type, trait_name,
            generics: collect_generics(tcx, def_id), methods: method_ids, associated_types: assoc_type_ids,
        }),
    ))
}

fn collect_method_from_impl<'tcx>(
    tcx: TyCtxt<'tcx>, method_def_id: rustc_hir::def_id::DefId,
    is_trait_impl: bool, items: &mut ItemStore,
) -> Option<ItemId> {
    let name = tcx.item_name(method_def_id).to_string();
    let fn_sig = tcx.fn_sig(method_def_id).skip_binder().skip_binder();
    let receiver = detect_receiver(tcx, method_def_id, fn_sig);
    let (params, return_type) = collect_fn_params_and_ret(tcx, method_def_id, fn_sig, receiver.has_self());

    // Extract qualifiers from HIR
    let qualifiers = method_def_id.as_local()
        .and_then(|local_id| {
            match tcx.hir_node_by_def_id(local_id) {
                rustc_hir::Node::ImplItem(ii) => {
                    if let rustc_hir::ImplItemKind::Fn(ref hir_sig, _) = ii.kind {
                        Some(collect_fn_qualifiers(&hir_sig.header))
                    } else { None }
                }
                _ => None,
            }
        })
        .unwrap_or_default();

    let vis = if is_trait_impl || tcx.visibility(method_def_id).is_public() { Visibility::Public } else { Visibility::Private };

    Some(items.alloc(&name, vis, ItemKind::Method(MethodData {
        generics: collect_generics(tcx, method_def_id),
        qualifiers,
        receiver, params, return_type,
    })))
}

/// Shared helper to collect function/method params and return type.
fn collect_fn_params_and_ret<'tcx>(
    tcx: TyCtxt<'tcx>, def_id: rustc_hir::def_id::DefId,
    fn_sig: rustc_middle::ty::FnSig<'tcx>, has_self: bool,
) -> (Vec<ParamModel>, TypeRepr) {
    let arg_idents = tcx.fn_arg_idents(def_id);
    let skip = if has_self { 1 } else { 0 };
    let params = arg_idents.iter().skip(skip)
        .zip(fn_sig.inputs().iter().skip(skip))
        .map(|(ident_opt, ty)| ParamModel {
            name: ident_opt.map(|i| i.name.to_string()).unwrap_or_else(|| "_".to_string()),
            ty: ty_to_type_repr(&format!("{}", ty)),
        })
        .collect();
    let ret = if fn_sig.output().is_unit() { TypeRepr::unit() } else { ty_to_type_repr(&format!("{}", fn_sig.output())) };
    (params, ret)
}

/// Extract function qualifiers (async, unsafe, const, ABI) from a HIR function header.
fn collect_fn_qualifiers(header: &rustc_hir::FnHeader) -> FnQualifiers {
    let abi_name = header.abi.name();
    let abi = if abi_name == "Rust" { None } else { Some(abi_name.to_string()) };
    FnQualifiers {
        is_async: header.is_async(),
        is_unsafe: header.is_unsafe(),
        is_const: matches!(header.constness, rustc_hir::Constness::Const { .. }),
        abi,
    }
}

fn detect_receiver<'tcx>(
    tcx: TyCtxt<'tcx>, method_def_id: rustc_hir::def_id::DefId,
    _fn_sig: rustc_middle::ty::FnSig<'tcx>,
) -> Receiver {
    let Some(local_id) = method_def_id.as_local() else { return Receiver::None; };

    // Get the HIR FnDecl to access ImplicitSelfKind and input types
    let hir_decl: Option<&rustc_hir::FnDecl<'tcx>> = match tcx.hir_node_by_def_id(local_id) {
        rustc_hir::Node::ImplItem(ii) => {
            if let rustc_hir::ImplItemKind::Fn(ref sig, _) = ii.kind { Some(sig.decl) } else { None }
        }
        rustc_hir::Node::TraitItem(ti) => {
            if let rustc_hir::TraitItemKind::Fn(ref sig, _) = ti.kind { Some(sig.decl) } else { None }
        }
        rustc_hir::Node::Item(item) => {
            if let rustc_hir::ItemKind::Fn { sig, .. } = &item.kind { Some(sig.decl) } else { None }
        }
        _ => None,
    };
    let Some(decl) = hir_decl else { return Receiver::None; };

    match decl.implicit_self() {
        rustc_hir::ImplicitSelfKind::Imm | rustc_hir::ImplicitSelfKind::Mut => Receiver::Owned,
        rustc_hir::ImplicitSelfKind::RefImm => {
            // Check for explicit lifetime: &'a self
            extract_ref_lifetime(tcx, decl)
                .map(Receiver::RefLifetime)
                .unwrap_or(Receiver::Ref)
        }
        rustc_hir::ImplicitSelfKind::RefMut => {
            // Check for explicit lifetime: &'a mut self
            extract_ref_lifetime(tcx, decl)
                .map(Receiver::RefMutLifetime)
                .unwrap_or(Receiver::RefMut)
        }
        rustc_hir::ImplicitSelfKind::None => {
            // Could be a static method, or an arbitrary self type (self: Pin<&mut Self>)
            let arg_idents = tcx.fn_arg_idents(method_def_id);
            let has_self = arg_idents.first().is_some_and(|i| i.is_some_and(|i| i.name.as_str() == "self"));
            if has_self {
                // Arbitrary self type — extract from HIR
                let hir_ctx: &dyn rustc_hir::intravisit::HirTyCtxt<'_> = &tcx;
                let ann: &dyn rustc_hir_pretty::PpAnn = &hir_ctx;
                if let Some(first_input) = decl.inputs.first() {
                    let ty_str = rustc_hir_pretty::ty_to_string(ann, first_input);
                    Receiver::Arbitrary(ty_to_type_repr(&ty_str))
                } else {
                    Receiver::Owned
                }
            } else {
                Receiver::None
            }
        }
    }
}

/// Extract an explicit lifetime from the first parameter's HIR type (for &'a self / &'a mut self).
/// Returns None for elided lifetimes.
fn extract_ref_lifetime<'tcx>(tcx: TyCtxt<'tcx>, decl: &rustc_hir::FnDecl<'tcx>) -> Option<String> {
    let first_ty = decl.inputs.first()?;
    if let rustc_hir::TyKind::Ref(lifetime, _, _) = &first_ty.kind {
        let name = lifetime.ident.name.to_string();
        // Elided lifetimes show up as "'_" — don't emit those
        if name == "'_" || name == "''" { return None; }
        let name = if name.starts_with('\'') { name } else { format!("'{name}") };
        Some(name)
    } else {
        None
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Impl collection with filtering
// ═══════════════════════════════════════════════════════════════════════════════

fn known_types_and_traits(items: &ItemStore, children: &[ItemId]) -> (std::collections::HashSet<String>, std::collections::HashSet<String>) {
    let mut types = std::collections::HashSet::new();
    let mut traits = std::collections::HashSet::new();
    for &id in children {
        let item = items.get(id);
        match &item.kind {
            ItemKind::Struct(_) | ItemKind::Enum(_) => { types.insert(item.name.clone()); }
            ItemKind::Trait(_) => { traits.insert(item.name.clone()); }
            _ => {}
        }
    }
    (types, traits)
}

/// Where to iterate for impl blocks.
enum IterSource<'a, 'tcx> {
    Root,
    Submodule(&'a rustc_hir::Mod<'tcx>),
}

fn collect_impls_from_iter<'a, 'tcx>(
    tcx: TyCtxt<'tcx>, items: &mut ItemStore,
    known_types: &std::collections::HashSet<String>,
    known_traits: &std::collections::HashSet<String>,
    children: &mut Vec<ItemId>,
    source: IterSource<'a, 'tcx>,
) {
    // Collect def_ids of impl items first, then process them
    let impl_def_ids: Vec<(rustc_hir::def_id::DefId, &rustc_hir::Impl<'tcx>)> = match source {
        IterSource::Root => {
            tcx.hir_crate_items(()).free_items().filter_map(|item_id| {
                let item = tcx.hir_node(item_id.hir_id());
                let rustc_hir::Node::Item(item) = item else { return None };
                let local = item.owner_id.def_id;
                if tcx.local_parent(local) != rustc_hir::def_id::CRATE_DEF_ID { return None; }
                if let rustc_hir::ItemKind::Impl(impl_data) = &item.kind {
                    Some((local.to_def_id(), impl_data as &rustc_hir::Impl<'tcx>))
                } else { None }
            }).collect()
        }
        IterSource::Submodule(mod_data) => {
            mod_data.item_ids.iter().filter_map(|&item_id| {
                let item = tcx.hir_node(item_id.hir_id());
                let rustc_hir::Node::Item(item) = item else { return None };
                if let rustc_hir::ItemKind::Impl(impl_data) = &item.kind {
                    Some((item.owner_id.def_id.to_def_id(), impl_data as &rustc_hir::Impl<'tcx>))
                } else { None }
            }).collect()
        }
    };

    for (def_id, impl_data) in impl_def_ids {
        if let Some(impl_id) = collect_impl_item(tcx, def_id, impl_data, items) {
            let item = items.get(impl_id);
            let ItemKind::Impl(ref imp) = item.kind else { continue };
            let stn = imp.self_type_name.clone();
            let tn = imp.trait_name.clone();

            if !known_types.contains(&stn) {
                log::debug!("AnalysisPass: skipping impl for unknown type '{}'", stn);
                continue;
            }
            match &tn {
                None => children.push(impl_id),
                Some(t) if known_traits.contains(t) => children.push(impl_id),
                Some(t) => {
                    log::debug!("AnalysisPass: skipping external trait impl {} for {}", t, stn);
                }
            }
        }
    }
}

// (enrich_trait_impls removed — TraitImplInfo no longer used)

// ═══════════════════════════════════════════════════════════════════════════════
// Type helpers
// ═══════════════════════════════════════════════════════════════════════════════

fn ty_to_type_repr(ty_str: &str) -> TypeRepr {
    if ty_str == "()" { TypeRepr::unit() } else { TypeRepr::simple(ty_str) }
}

fn convert_repr<'tcx>(tcx: TyCtxt<'tcx>, def_id: rustc_hir::def_id::DefId) -> Option<ReprInfo> {
    let adt = tcx.adt_def(def_id);
    let repr = adt.repr();
    let layout = if repr.c() { PrimaryLayout::C }
        else if repr.transparent() { PrimaryLayout::Transparent }
        else if let Some(int_ty) = repr.int { PrimaryLayout::Int(format!("{:?}", int_ty).to_lowercase()) }
        else { PrimaryLayout::Rust };
    let alignment = if repr.packed() { Some(AlignmentModifier::Packed(repr.pack.map(|a| a.bytes() as u32).unwrap_or(1))) }
        else if let Some(align) = repr.align { Some(AlignmentModifier::Align(align.bytes() as u32)) }
        else { None };
    let info = ReprInfo { layout, alignment };
    if info.is_default() { None } else { Some(info) }
}

fn extract_type_name_from_ty(ty_str: &str) -> String {
    let base = if let Some(idx) = ty_str.find('<') { &ty_str[..idx] } else { ty_str };
    if let Some(idx) = base.rfind("::") { base[idx + 2..].to_string() } else { base.to_string() }
}

/// Convert a HIR generic bound to a `Bound`.
fn hir_bound_to_bound<'tcx>(tcx: TyCtxt<'tcx>, b: &rustc_hir::GenericBound<'tcx>) -> Option<Bound> {
    match b {
        rustc_hir::GenericBound::Trait(poly) => {
            let segments = &poly.trait_ref.path.segments;
            let path: Vec<String> = segments.iter().map(|s| s.ident.name.to_string()).collect();
            if path.is_empty() { return None; }
            let args = segments.last().map(|s| extract_generic_args_from_segment(tcx, s)).unwrap_or_default();
            let trait_path = TypeRepr::with_args(path, args);

            let for_lifetimes = ForLifetimes(
                poly.bound_generic_params.iter().filter_map(|p| {
                    if let rustc_hir::GenericParamKind::Lifetime { .. } = &p.kind {
                        let raw = p.name.ident().name.to_string();
                        Some(if raw.starts_with('\'') { raw } else { format!("'{}", raw) })
                    } else {
                        None
                    }
                }).collect()
            );

            let is_maybe = matches!(poly.modifiers, rustc_hir::TraitBoundModifiers { polarity: rustc_hir::BoundPolarity::Maybe(_), .. });

            Some(Bound::Trait(TraitBound { path: trait_path, for_lifetimes, is_maybe }))
        }
        rustc_hir::GenericBound::Outlives(lt) => {
            let raw = lt.ident.name.to_string();
            let name = if raw.starts_with('\'') { raw } else { format!("'{}", raw) };
            Some(Bound::Lifetime(name))
        }
        _ => None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Generics collection — builds GenericParams with declaration-order Vec<GenericParam>
// ═══════════════════════════════════════════════════════════════════════════════

fn collect_generics<'tcx>(tcx: TyCtxt<'tcx>, def_id: rustc_hir::def_id::DefId) -> GenericParams {
    let Some(local_id) = def_id.as_local() else { return GenericParams::empty(); };
    let hir_generics = match tcx.hir_node_by_def_id(local_id) {
        rustc_hir::Node::Item(item) => match &item.kind {
            rustc_hir::ItemKind::Fn { generics, .. } => Some(*generics),
            rustc_hir::ItemKind::Struct(_, generics, _) => Some(*generics),
            rustc_hir::ItemKind::Enum(_, generics, _) => Some(*generics),
            rustc_hir::ItemKind::Trait { generics, .. } => Some(*generics),
            rustc_hir::ItemKind::Impl(d) => Some(d.generics),
            _ => None,
        },
        rustc_hir::Node::TraitItem(item) => Some(item.generics),
        rustc_hir::Node::ImplItem(item) => Some(item.generics),
        _ => None,
    };
    let Some(hir_gen) = hir_generics else { return GenericParams::empty(); };

    let hir_ctx: &dyn rustc_hir::intravisit::HirTyCtxt<'_> = &tcx;
    let ann: &dyn rustc_hir_pretty::PpAnn = &hir_ctx;

    // Build params in declaration order (single Vec<GenericParam>)
    let mut params = Vec::new();

    for param in hir_gen.params {
        match &param.kind {
            rustc_hir::GenericParamKind::Lifetime { .. } => {
                let raw = param.name.ident().name.to_string();
                let name = if raw.starts_with('\'') { raw } else { format!("'{}", raw) };
                let lifetime_bounds: Vec<String> = hir_gen.bounds_for_param(param.def_id).flat_map(|bp| {
                    bp.bounds.iter().filter_map(|b| match b {
                        rustc_hir::GenericBound::Outlives(lt) => {
                            let raw = lt.ident.name.to_string();
                            Some(if raw.starts_with('\'') { raw } else { format!("'{}", raw) })
                        }
                        _ => None,
                    })
                }).collect();
                params.push(GenericParam::Lifetime(LifetimeParam { name, bounds: lifetime_bounds }));
            }
            rustc_hir::GenericParamKind::Type { default, .. } => {
                let bounds: Vec<Bound> = hir_gen.bounds_for_param(param.def_id).flat_map(|bp| {
                    bp.bounds.iter().filter_map(|b| hir_bound_to_bound(tcx, b))
                }).collect();
                let default_type = default.map(|hir_ty| ty_to_type_repr(&rustc_hir_pretty::ty_to_string(ann, hir_ty)));
                params.push(GenericParam::Type(TypeParam { name: param.name.ident().name.to_string(), bounds, default: default_type }));
            }
            rustc_hir::GenericParamKind::Const { ty, default, .. } => {
                let default_val = default.map(|d| {
                    match d.kind {
                        rustc_hir::ConstArgKind::Anon(anon) => {
                            let body = tcx.hir_body(anon.body);
                            rustc_hir_pretty::expr_to_string(ann, body.value)
                        }
                        rustc_hir::ConstArgKind::Path(ref qpath) => {
                            rustc_hir_pretty::qpath_to_string(ann, qpath)
                        }
                        _ => format!("{:?}", d.kind),
                    }
                });
                params.push(GenericParam::Const(ConstParam {
                    name: param.name.ident().name.to_string(),
                    ty: ty_to_type_repr(&rustc_hir_pretty::ty_to_string(ann, ty)),
                    default: default_val,
                }));
            }
        }
    }

    // Where clauses
    let mut where_clauses = Vec::new();
    for pred in hir_gen.predicates {
        match pred.kind {
            rustc_hir::WherePredicateKind::BoundPredicate(bp) => {
                let bounds: Vec<Bound> = bp.bounds.iter().filter_map(|b| hir_bound_to_bound(tcx, b)).collect();
                if !bounds.is_empty() {
                    let for_lifetimes = ForLifetimes(
                        bp.bound_generic_params.iter().filter_map(|p| {
                            if let rustc_hir::GenericParamKind::Lifetime { .. } = &p.kind {
                                let raw = p.name.ident().name.to_string();
                                Some(if raw.starts_with('\'') { raw } else { format!("'{}", raw) })
                            } else {
                                None
                            }
                        }).collect()
                    );
                    let ty_str = rustc_hir_pretty::ty_to_string(ann, bp.bounded_ty);
                    where_clauses.push(WherePredicate::TypeBound { for_lifetimes, ty: ty_to_type_repr(&ty_str), bounds });
                }
            }
            rustc_hir::WherePredicateKind::RegionPredicate(rp) => {
                let lbs: Vec<String> = rp.bounds.iter().filter_map(|b| match b {
                    rustc_hir::GenericBound::Outlives(lt) => {
                        let raw = lt.ident.name.to_string();
                        Some(if raw.starts_with('\'') { raw } else { format!("'{}", raw) })
                    }
                    _ => None,
                }).collect();
                if !lbs.is_empty() {
                    let raw = rp.lifetime.ident.name.to_string();
                    let lifetime = if raw.starts_with('\'') { raw } else { format!("'{}", raw) };
                    where_clauses.push(WherePredicate::LifetimeBound { lifetime, bounds: lbs });
                }
            }
            _ => {}
        }
    }

    GenericParams { params, where_clauses }
}

fn extract_generic_args_from_segment<'tcx>(tcx: TyCtxt<'tcx>, segment: &rustc_hir::PathSegment<'tcx>) -> Vec<crate::type_repr::GenericArg> {
    use crate::type_repr::GenericArg as GA;
    let Some(args) = segment.args else { return vec![]; };
    let hir_ctx: &dyn rustc_hir::intravisit::HirTyCtxt<'_> = &tcx;
    let ann: &dyn rustc_hir_pretty::PpAnn = &hir_ctx;
    let mut result = Vec::new();
    for arg in args.args {
        match arg {
            rustc_hir::GenericArg::Type(ty) => {
                result.push(GA::Type(ty_to_type_repr(&rustc_hir_pretty::ty_to_string(ann, ty.as_unambig_ty()))));
            }
            rustc_hir::GenericArg::Lifetime(lt) => {
                let name = lt.ident.name.to_string();
                if name != "'_" { result.push(GA::Lifetime(crate::type_repr::Lifetime::new(name.trim_start_matches('\'')))); }
            }
            rustc_hir::GenericArg::Const(ct) => {
                let const_str = match ct.as_unambig_ct().kind {
                    rustc_hir::ConstArgKind::Anon(anon) => {
                        let body = tcx.hir_body(anon.body);
                        rustc_hir_pretty::expr_to_string(ann, body.value)
                    }
                    rustc_hir::ConstArgKind::Path(ref qpath) => {
                        rustc_hir_pretty::qpath_to_string(ann, qpath)
                    }
                    _ => format!("{:?}", ct.as_unambig_ct().kind),
                };
                result.push(GA::Const(const_str));
            }
            _ => {}
        }
    }
    for c in args.constraints {
        match &c.kind {
            rustc_hir::AssocItemConstraintKind::Equality { term } => {
                if let rustc_hir::Term::Ty(ty) = term {
                    result.push(GA::Binding(c.ident.name.to_string(), ty_to_type_repr(&rustc_hir_pretty::ty_to_string(ann, ty))));
                }
            }
            rustc_hir::AssocItemConstraintKind::Bound { bounds } => {
                let bound_strs: Vec<String> = bounds.iter()
                    .filter_map(|b| hir_bound_to_bound(tcx, b))
                    .map(|b| b.to_source())
                    .collect();
                if !bound_strs.is_empty() {
                    result.push(GA::BoundConstraint(c.ident.name.to_string(), bound_strs.join(" + ")));
                }
            }
        }
    }
    result
}

fn collect_supertraits<'tcx>(tcx: TyCtxt<'tcx>, bounds: &[rustc_hir::GenericBound<'tcx>]) -> Vec<Bound> {
    bounds.iter().filter_map(|b| hir_bound_to_bound(tcx, b)).collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Post-processing: strip module path prefixes
// ═══════════════════════════════════════════════════════════════════════════════

fn strip_module_prefixes(items: &mut ItemStore, module_id: ItemId, current_path: &[&str]) {
    let prefix = if current_path.is_empty() { String::new() } else { format!("{}::", current_path.join("::")) };

    let children: Vec<ItemId> = match &items.get(module_id).kind {
        ItemKind::Module(m) => m.children.clone(),
        _ => return,
    };

    if !prefix.is_empty() {
        for &cid in &children { strip_item_prefixes(items, cid, &prefix); }
    }

    // Recurse into child modules
    for &cid in &children {
        let name = items.get(cid).name.clone();
        if let ItemKind::Module(_) = &items.get(cid).kind {
            let mut path: Vec<&str> = current_path.to_vec();
            path.push(&name);
            strip_module_prefixes(items, cid, &path);
        }
    }
}

fn strip_item_prefixes(items: &mut ItemStore, id: ItemId, prefix: &str) {
    match items.get(id).kind.clone() {
        ItemKind::Function(mut f) => {
            strip_prefix(&mut f.return_type, prefix);
            for p in &mut f.params { strip_prefix(&mut p.ty, prefix); }
            items.get_mut(id).kind = ItemKind::Function(f);
        }
        ItemKind::Struct(s) => {
            for &fid in &s.fields { strip_field_prefix(items, fid, prefix); }
        }
        ItemKind::Enum(e) => {
            for &vid in &e.variants {
                if let ItemKind::Variant(ref v) = items.get(vid).kind {
                    let fids = v.fields.clone();
                    for fid in fids { strip_field_prefix(items, fid, prefix); }
                }
            }
        }
        ItemKind::Trait(t) => {
            for &mid in &t.methods { strip_method_prefixes(items, mid, prefix); }
        }
        ItemKind::Impl(mut imp) => {
            strip_prefix(&mut imp.self_type, prefix);
            let mids = imp.methods.clone();
            items.get_mut(id).kind = ItemKind::Impl(imp);
            for mid in mids { strip_method_prefixes(items, mid, prefix); }
        }
        _ => {}
    }
}

fn strip_field_prefix(items: &mut ItemStore, fid: ItemId, prefix: &str) {
    if let ItemKind::Field(ref mut f) = items.get_mut(fid).kind {
        strip_prefix(&mut f.ty, prefix);
    }
}

fn strip_method_prefixes(items: &mut ItemStore, mid: ItemId, prefix: &str) {
    if let ItemKind::Method(ref m) = items.get(mid).kind {
        let mut m = m.clone();
        strip_prefix(&mut m.return_type, prefix);
        for p in &mut m.params { strip_prefix(&mut p.ty, prefix); }
        items.get_mut(mid).kind = ItemKind::Method(m);
    }
}

fn strip_prefix(ty: &mut TypeRepr, prefix: &str) {
    if let TypeRepr::Path { first, rest, .. } = ty {
        if rest.is_empty() && first.starts_with(prefix) {
            *first = first[prefix.len()..].to_string();
        }
    }
}
