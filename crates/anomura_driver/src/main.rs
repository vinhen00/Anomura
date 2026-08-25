// Tested with nightly-2025-03-28

#![feature(rustc_private)]
use std::process::Command;
extern crate rustc_driver;
use anomura_driver::{
    compile_mocks::CompileMocks, function_intercept::FunctionIntercept, parse_mocks::ParseMocks,
};
use rustc_driver::run_compiler;

fn main() {
    let mut expanded_macros = ParseMocks::new(false);
    run_compiler(
        &[
            "ignored".to_string(),
            "mock_defs.rs".to_string(),
            "--crate-type".to_string(),
            "bin".to_string(),
            "-o".to_string(),
            "./target/mocked_main".to_string(),
            "--extern".to_string(),
            "mock_macro=./target/debug/mock_macro.dll".to_string(),
            "-L".to_string(),
            "dependency=./target/debug".to_string(),
        ],
        &mut expanded_macros,
    );

    let mut mocked_funs = CompileMocks::new(Vec::new(), expanded_macros.get_program(), false);
    run_compiler(
        &[
            "ignored".to_string(),
            "mock_defs.rs".to_string(),
            "--crate-type".to_string(),
            "bin".to_string(),
            "-o".to_string(),
            "./target/mocked_main".to_string(),
        ],
        &mut mocked_funs,
    );

    let mut insertion = FunctionIntercept::new(mocked_funs.get_mocks());
    //dbg!(&insertion);
    run_compiler(
        &[
            "ignored".to_string(),
            "mock_test.rs".to_string(),
            "--crate-type".to_string(),
            "bin".to_string(),
            "-o".to_string(),
            "./target/mocked_main".to_string(),
        ],
        &mut insertion,
    );

    // Run the compiled executable
    println!("\n=== RUNNING COMPILED PROGRAM ===");
    let output = Command::new("./target/mocked_main").output();

    match output {
        Ok(output) => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
            if !output.stderr.is_empty() {
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            println!("Failed to run executable: {}", e);
        }
    }
}
