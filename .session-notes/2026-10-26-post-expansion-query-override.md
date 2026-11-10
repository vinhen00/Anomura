# Post-Expansion Query Override — Session Notes (2026-10-26)

## Summary

Scaffolded and wired up the `resolver_for_lowering_raw` query override in `CrateIntercept`, moving from the pre-expansion `after_crate_root_parsing` callback to a post-expansion interception point. The pipeline is functional but panics on DUMMY_NODE_ID — the next step is implementing the `NodeIdAssigner` MutVisitor.

## What Was Done

### 1. Verified rustc APIs on nightly-2026-10-06

Confirmed all required types and functions exist:
- `DEFAULT_QUERY_PROVIDERS.queries.resolver_for_lowering_raw` — `fn(TyCtxt<'tcx>, ()) -> (&'tcx Steal<ResolverAstLowering<'tcx>>, &'tcx Steal<ast::Crate>, &'tcx ResolverGlobalCtxt)`
- `config.override_queries` — `Option<fn(&Session, &mut Providers)>`
- `providers.queries.resolver_for_lowering_raw` is assignable
- `Steal` at `rustc_data_structures::steal::Steal`
- `ResolverAstLowering` at `rustc_middle::middle::resolve::ResolverAstLowering`
- `ResolverGlobalCtxt` at `rustc_middle::middle::resolve::ResolverGlobalCtxt`
- `ParseSess` at `rustc_session::parse::ParseSess`
- `tcx.crate_name(LOCAL_CRATE)` returns `Symbol` — **no thread-local needed** for passing crate name
- `tcx.sess.psess` gives direct access to `ParseSess`
- `tcx.arena.alloc(Steal::new(...))` works for re-wrapping

### 2. Implemented query override scaffold

- Added `anomura_resolver_for_lowering` function — delegates to original provider, steals results, re-wraps
- `CrateIntercept::config()` installs the override via `config.override_queries`
- Removed `after_crate_root_parsing` entirely (no empty callback needed)
- Removed `parse_counter` field from `CrateIntercept` (replaced with global `AtomicU32`)

### 3. Extracted all methods to standalone functions

All methods that were on `impl CrateIntercept` are now module-level functions:

**API collection (pure, no parse session needed):**
- `collect_module(name, items) -> ModuleModel`
- `collect_function`, `collect_struct`, `collect_enum`, `collect_trait`, `collect_impl`
- `collect_method_sig`, `collect_params`, `collect_method_params`
- `extract_receiver`, `extract_type_name`, `extract_param_name`
- `collect_return_type`, `fetch_derives`, `is_pub`

**AST mutation (take `&ParseSess`):**
- `apply_mock_bodies(psess, krate, api)`
- `replace_fn_body(psess, fn_data, prefix, func)`
- `handle_impl_methods(psess, impl_data, module, path)`
- `handle_mod_mock_bodies(psess, mod_kind, path, child_module)`
- `modify_struct_fields(psess, fields, trackable)`

**Parsing helpers (take `&ParseSess`):**
- `parse_fn_item(psess, source) -> Option<Box<ast::Fn>>`
- `parse_type(psess, ty_str) -> Option<Box<ast::Ty>>`
- `parse_struct_field(psess, field_str) -> Option<ast::FieldDef>`
- `inject_source_items(psess, krate, source)`

### 4. Wired into the query override

`anomura_resolver_for_lowering` now:
1. Delegates to original provider (full expansion + resolution)
2. Steals the expanded AST and resolver
3. Collects API from the expanded AST (sees macro-generated items!)
4. Calls `apply_mock_bodies(tcx.sess.psess, &mut krate, &api)` — replaces bodies + injects convenience API
5. Re-wraps in Steal and returns

### Test Results

From a clean build (`rm -rf target/plugin-nightly-2026-10-06`):
- **API collection works**: 37 top-level items in expanded AST
- **Body replacement works**: 7 function bodies replaced
- **Convenience API injection works**: 196 items injected
- **PANICS** in `rustc_lint/src/early.rs:344` — the early lint pass encounters nodes with `DUMMY_NODE_ID`

This is the exact issue predicted in section 4.2 of `post_expansion_substitution.md`:
> "All parsed replacement code (from `new_parser_from_source_str`) gets `DUMMY_NODE_ID` on every node. Post-expansion, `DUMMY_NODE_ID` causes hard panics during HIR lowering."

The panic happens even earlier — in the lint pass, not HIR lowering — because the lint infrastructure also expects valid NodeIds.

## Next Steps

### Immediate: Implement NodeIdAssigner MutVisitor

As designed in the doc (section 4.2), implement a `MutVisitor` that walks all injected/replaced AST nodes and replaces `DUMMY_NODE_ID` with fresh IDs from `resolver.next_node_id`:

```rust
struct NodeIdAssigner<'a, 'tcx> {
    resolver: &'a mut ResolverAstLowering<'tcx>,
}

impl MutVisitor for NodeIdAssigner<'_, '_> {
    fn visit_id(&mut self, id: &mut NodeId) {
        if *id == ast::DUMMY_NODE_ID {
            let fresh = self.resolver.next_node_id;
            self.resolver.next_node_id = NodeId::from_u32(fresh.as_u32() + 1);
            *id = fresh;
        }
    }
}
```

Run this over the entire crate after all mutations, before re-wrapping in Steal.

Key insight from earlier discussion: **DefId registration may not be a big issue** because the only external crate referenced in injected items is the `context` crate (fully qualified paths like `context::run_mock`, `context::MockId`). No complex `partial_res_map` entries should be needed.

### Then: DefId registration for injected items (section 4.3)

Each new top-level item needs a `LocalDefId` via `tcx.create_def()` and an entry in `resolver.owners`. This may or may not be needed — need to test empirically after NodeId assignment.

### Then: Verify all 22 tests pass

### Then: Remove `fetch_derives` workaround (expanded AST has real `impl Default` blocks)

### Then: Add macro-generated item tests

## Files Modified

- `/home/eli/anomura/crates/anomura_driver/src/crate_intercept.rs` — complete rewrite (810 lines)

## Key Design Decisions

- **No thread-local**: `tcx.crate_name(LOCAL_CRATE)` provides the crate name directly inside the query
- **Global AtomicU32 for parse counter**: simpler than passing state through the query function pointer
- **CrateIntercept struct reduced to just `crate_name`**: `parse_counter` field removed
- **All logic in standalone functions**: makes them callable from the query override without needing `&self`
