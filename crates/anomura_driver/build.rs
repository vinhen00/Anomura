/// Detect rustc version and emit cfg flags for API compatibility.
///
/// - `rustc_has_compiler_entrypoint`: rustc_driver has `compiler_entrypoint` (1.100+)
///   instead of `run_compiler`
/// - `rustc_has_structures_crate_type`: `CrateType` is in `rustc_structures` (1.100+)
///   instead of being re-exported from `rustc_session`
/// - `rustc_tyref_three_fields`: `TyKind::Ref` has 3 fields (1.100+)
///   instead of 2 fields with `MutTy`
fn main() {
    let version = rustc_version();
    if version >= (1, 100) {
        println!("cargo:rustc-cfg=rustc_has_compiler_entrypoint");
        println!("cargo:rustc-cfg=rustc_has_structures_crate_type");
        println!("cargo:rustc-cfg=rustc_tyref_three_fields");
    }
}

fn rustc_version() -> (u32, u32) {
    let output = std::process::Command::new(
        std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()),
    )
    .arg("--version")
    .output()
    .expect("failed to run rustc --version");
    let version_str = String::from_utf8(output.stdout).unwrap();
    // Parse "rustc X.Y.Z ..." or "rustc X.Y.Z-nightly ..."
    let version_part = version_str
        .split_whitespace()
        .nth(1)
        .expect("unexpected rustc --version output");
    let mut parts = version_part.split('.');
    let major: u32 = parts.next().unwrap().parse().unwrap();
    let minor: u32 = parts.next().unwrap().parse().unwrap();
    (major, minor)
}
