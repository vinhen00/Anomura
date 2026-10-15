#![feature(rustc_private)]

/// Detect rustc version and emit cfg flags for API compatibility.
///
/// - `rustc_has_compiler_entrypoint`: rustc_driver has `compiler_entrypoint` (1.100+)
///   instead of `run_compiler`
fn main() {
    rustc_plugin::build_main();

    let version = rustc_version();
    if version >= (1, 100) {
        println!("cargo:rustc-cfg=rustc_has_compiler_entrypoint");
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
    let version_part = version_str
        .split_whitespace()
        .nth(1)
        .expect("unexpected rustc --version output");
    let mut parts = version_part.split('.');
    let major: u32 = parts.next().unwrap().parse().unwrap();
    let minor: u32 = parts.next().unwrap().parse().unwrap();
    (major, minor)
}
