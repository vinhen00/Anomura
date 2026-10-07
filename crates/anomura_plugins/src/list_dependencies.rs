use std::collections::{HashMap, HashSet};

use cargo_metadata::{DependencyKind, MetadataCommand, Package, PackageId};

/// Compute the set of crate **names** that must be compiled with normal `rustc`
/// (not the discover driver) during the discover pass.
///
/// The discover driver intercepts compilation to find mock definitions, but it
/// cannot correctly compile:
///   - proc-macro crates (and their transitive dependencies)
///   - build-script dependencies (and their transitive dependencies)
///   - the `context` crate and its transitive dependencies (needed intact for
///     linking during the substitution pass)
///
/// This function dynamically computes that full set so we don't need a fragile
/// hardcoded list.
pub fn list_crates_to_build_normally() -> HashSet<String> {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("Failed to run `cargo metadata`");

    let resolve = metadata.resolve.as_ref().expect("Missing resolve graph");

    // Index packages by id for quick lookup
    let packages: HashMap<&PackageId, &Package> =
        metadata.packages.iter().map(|p| (&p.id, p)).collect();

    // Seeds: package IDs whose entire transitive dependency tree must be built normally.
    let mut seeds: HashSet<&PackageId> = HashSet::new();

    // ─── 1. Proc-macro crates ─────────────────────────────────────────────
    // Any crate that produces a proc-macro artifact cannot be compiled through
    // the discover driver. Collect them as seeds.
    for pkg in &metadata.packages {
        if pkg.targets.iter().any(|t| t.is_proc_macro()) {
            log::debug!("proc-macro seed: {}", pkg.name);
            seeds.insert(&pkg.id);
        }
    }

    // ─── 2. Build dependencies ────────────────────────────────────────────
    // For every package in the resolve graph, collect any dependency that is
    // used only at build time (`DependencyKind::Build`).
    for node in &resolve.nodes {
        for dep in &node.deps {
            let is_build_dep = dep
                .dep_kinds
                .iter()
                .any(|k| k.kind == DependencyKind::Build);
            if is_build_dep {
                log::debug!("build-dep seed: {} (from {})",
                    packages.get(&dep.pkg).map(|p| p.name.as_str()).unwrap_or("?"),
                    packages.get(&node.id).map(|p| p.name.as_str()).unwrap_or("?"),
                );
                seeds.insert(&dep.pkg);
            }
        }
    }

    // ─── 3. The `context` crate ───────────────────────────────────────────
    // The context crate must be compiled normally so its .rmeta / .rlib is
    // available for linking in the substitution pass.
    for pkg in &metadata.packages {
        if pkg.name == "context" {
            log::debug!("context seed: {}", pkg.name);
            seeds.insert(&pkg.id);
        }
    }

    // ─── 4. Expand seeds to full transitive closure ───────────────────────
    let mut result: HashSet<String> = HashSet::new();
    let mut visited: HashSet<&PackageId> = HashSet::new();
    let mut to_visit: Vec<&PackageId> = seeds.iter().copied().collect();

    while let Some(current_id) = to_visit.pop() {
        if !visited.insert(current_id) {
            continue;
        }

        // Add crate name to the result set.
        // Normalize: cargo_metadata uses hyphens (e.g. "proc-macro2") but rustc
        // uses underscores (e.g. "proc_macro2") in --crate-name. The driver
        // matches against the rustc name, so we must normalize here.
        if let Some(pkg) = packages.get(current_id) {
            result.insert(pkg.name.to_string().replace('-', "_"));
        }

        // Walk all normal + build dependencies (the deps this crate needs to compile)
        if let Some(node) = resolve.nodes.iter().find(|n| &n.id == current_id) {
            for dep in &node.deps {
                let is_compile_dep = dep
                    .dep_kinds
                    .iter()
                    .any(|k| k.kind == DependencyKind::Normal || k.kind == DependencyKind::Build);

                if is_compile_dep && !visited.contains(&dep.pkg) {
                    to_visit.push(&dep.pkg);
                }
            }
        }
    }

    log::debug!("total crates to build normally: {} entries", result.len());
    result
}
