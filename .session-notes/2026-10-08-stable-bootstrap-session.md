# Session Notes: Stable 1.99 RUSTC_BOOTSTRAP=1 Compatibility

## Date: 2026-10-08

## What was accomplished

### Nightly migration (complete ✅)
The project was fully migrated from `nightly-2025-08-20` to `nightly-2026-10-06` with all 16 tests passing. Key changes:
- rustc_plugin bumped to 0.15.0
- Consolidated rust-toolchain.toml into single root file
- run_compiler → compiler_entrypoint
- Driver main() returns ExitCode
- Upstream API fixes: CrateType, FileLoader, TyKind::Ref, StripTokens, AllowConstBlockItems
- Removed proc_macro2/quote from context crate (ToTokens impl was redundant)
- Context rmeta resolution via cargo_metadata + env var (CONTEXT_RMETA_PATH_ENV)
- RUSTFLAGS -L for transitive dependency resolution
- Fixed EmptyCallbacks in rustc_plugin (clear output_types to prevent stale artifacts with -Z embed-metadata=no)

### Stable 1.99 cfg compatibility (in progress 🔶)

#### Completed:
1. **Version detection** — build.rs files in `anomura_driver`, `anomura_plugins`, and `rustc_plugin` detect rustc version and emit cfg flags:
   - `rustc_has_compiler_entrypoint` (>= 1.100)
   - `rustc_has_structures_crate_type` (>= 1.100)
   - `rustc_tyref_three_fields` (>= 1.100)

2. **API differences handled with cfg:**
   - `run_compiler` vs `compiler_entrypoint` — helper function `rustc_run()` in both `anomura_plugins/src/lib.rs` and `rustc_plugin/src/lib.rs`
   - `CrateType` import — cfg-gated between `rustc_structures` and `rustc_session::config`
   - `TyKind::Ref` — cfg-gated pattern match (2 fields with MutTy vs 3 separate fields)
   - `anomura_driver/src/main.rs` — cfg-gated imports with `as rustc_run` alias

3. **Verified:** Both `cargo check` (nightly) and `RUSTC_BOOTSTRAP=1 cargo +stable check` compile successfully.

4. **Plugin installed on stable** successfully with `RUSTC_BOOTSTRAP=1 cargo +stable install`.

#### Current blocker:
Running `RUSTC_BOOTSTRAP=1 cargo +stable mock test` fails with:
```
CONTEXT_RMETA_PATH env var not set — was modify_cargo() called?: NotPresent
```

**Root cause:** `modify_cargo()` in `substitution_pass.rs` uses `rustc_plugin::CHANNEL` to construct the path to the plugin build directory:
```rust
let plugin_subdir = format!("plugin-{}", rustc_plugin::CHANNEL);
let build_dir = metadata.target_directory.join(plugin_subdir).join("debug").join("build");
```

`CHANNEL` is set from `rust-toolchain.toml` which says `nightly-2026-10-06`. But when running on stable, the target directory that `cli_main` creates also uses `CHANNEL`. The context rmeta IS built (the discover pass succeeded), but `find_context_rmeta()` may fail because:
1. The build dir path is `plugin-nightly-2026-10-06` which is correct (cli_main uses the same CHANNEL)
2. BUT the context crate might not have been compiled yet at the time `modify_cargo()` runs — the discover pass builds it, but `modify_cargo()` runs at the START of the substitution pass before cargo builds anything

**Wait — actually, the discover pass should have already built context.** The real issue might be simpler: the `find_context_rmeta()` function searches `build_dir/context/<hash>/out/` but on stable the output layout might differ. Need to check what files exist after the discover pass on stable.

## Next steps

1. **Debug the CONTEXT_RMETA_PATH issue on stable:**
   - Check if the discover pass produces context artifacts on stable
   - Check if the target directory layout is the same on stable
   - The `find_context_rmeta()` function in `substitution_pass.rs` scans `<build_dir>/context/<hash>/out/` for `libcontext*.rmeta` — verify this matches stable's layout
   - Might just need to run with `RUST_LOG=debug` to see where things go wrong

2. **If the output layout differs on stable:**
   - May need to adjust `find_context_rmeta()` to handle both layouts
   - Or the issue might be that `modify_cargo()` runs cargo_metadata which uses the workspace rust-toolchain.toml (nightly), but the actual cargo invocation uses +stable

3. **After fixing the runtime:**
   - Run `RUSTC_BOOTSTRAP=1 cargo +stable mock test` to verify all 16 tests pass
   - Document the stable usage in README

## Files modified for stable compatibility

### anomura:
- `crates/anomura_driver/build.rs` (new) — version detection, emits 3 cfg flags
- `crates/anomura_driver/src/lib.rs` — cfg-gated `extern crate rustc_structures`
- `crates/anomura_driver/src/main.rs` — cfg-gated imports for rustc_run
- `crates/anomura_driver/src/compile_mocks.rs` — cfg-gated CrateType import
- `crates/anomura_driver/src/crate_intercept.rs` — cfg-gated TyKind::Ref match
- `crates/anomura_plugins/build.rs` — added version detection
- `crates/anomura_plugins/src/lib.rs` — added `rustc_run()` helper
- `crates/anomura_plugins/src/substitution_pass.rs` — uses `crate::rustc_run()`
- `crates/anomura_plugins/src/mock_discover_pass.rs` — uses `crate::rustc_run()`
- `crates/anomura_plugins/src/main_sub.rs` — uses `crate::rustc_run()`

### rustc_plugin (local):
- `crates/rustc_plugin/build.rs` — added version detection
- `crates/rustc_plugin/src/lib.rs` — added `rustc_run()` helper
- `crates/rustc_plugin/src/driver.rs` — uses `crate::rustc_run()`

## Important note
The workspace Cargo.toml is currently set to use the LOCAL rustc_plugin path (not git). This is needed for the cfg changes to take effect:
```toml
#rustc_plugin = { git = "...", version = "0.15.0-nightly-2026-10-06" }
rustc_plugin = { version = "0.15.0-nightly-2026-10-06", path = "../rustc_plugin/crates/rustc_plugin" }
```
