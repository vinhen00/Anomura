use std::collections::{HashMap, HashSet};

use cargo_metadata::{DependencyKind, MetadataCommand, Package, PackageId};

pub fn list_transitive_build_dependencies() -> HashSet<PackageId> {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("Failed to run `cargo metadata`");

    let packages: HashMap<&PackageId, &Package> =
        metadata.packages.iter().map(|p| (&p.id, p)).collect();
    let root = metadata.root_package().expect("should be some root");

    // =========================================================================
    // STEP 1: Find all normal and build dependencies of root (B0..Bn)
    // =========================================================================
    let mut b_crates = HashSet::new();
    let resolve = metadata.resolve.as_ref().expect("Missing resolve graph");

    if let Some(node) = resolve.nodes.iter().find(|n| n.id == root.id) {
        for dep in &node.deps {
            // We consider standard and build dependencies as part of A's dependencies
            let is_normal_or_build = dep
                .dep_kinds
                .iter()
                .any(|k| k.kind == DependencyKind::Normal || k.kind == DependencyKind::Build);

            if is_normal_or_build {
                b_crates.insert(&dep.pkg);
            }
        }
        log::debug!("b_crates: {b_crates:?}");
    }

    // =========================================================================
    // STEP 2: Extract all build dependencies of B0..Bn (C0..Cn)
    // =========================================================================
    let mut c_crates = HashSet::new();

    for b_id in &b_crates {
        if let Some(node) = resolve.nodes.iter().find(|n| n.id == **b_id) {
            for dep in &node.deps {
                let has_build_dep = dep
                    .dep_kinds
                    .iter()
                    .any(|k| k.kind == DependencyKind::Build);

                if has_build_dep {
                    c_crates.insert(&dep.pkg);
                }
            }
        }
    }

    log::debug!("c_crates: {c_crates:?}");
    // =========================================================================
    // STEP 3: Find ALL transitive dependencies (normal + build) of C0..Cn
    // =========================================================================
    let mut all_transitive_from_c: HashSet<PackageId> = HashSet::new();
    let mut to_visit: Vec<&PackageId> = c_crates.iter().copied().collect();

    while let Some(current_id) = to_visit.pop() {
        if let Some(node) = resolve.nodes.iter().find(|n| n.id == *current_id) {
            for dep in &node.deps {
                // For C0..Cn's transitive dependencies, we match both normal and build kinds
                let is_valid_edge = dep
                    .dep_kinds
                    .iter()
                    .any(|k| k.kind == DependencyKind::Normal || k.kind == DependencyKind::Build);

                if is_valid_edge && !all_transitive_from_c.contains(&dep.pkg) {
                    all_transitive_from_c.insert(dep.pkg.clone());
                    to_visit.push(&dep.pkg); // Push to continue walking the tree
                }
            }
        }
    }

    all_transitive_from_c
} //println!("runnin main driver");
