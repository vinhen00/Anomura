// Tested with nightly-2025-03-28

#![feature(rustc_private)]
mod expand_macro;
mod visitors;
mod compile_mocks;
mod function_intercept;

extern crate rustc_ast;
extern crate rustc_ast_pretty;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_error_codes;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use std::process::Command;

use rustc_driver::run_compiler;



use crate::compile_mocks::CompileMocks;
use crate::function_intercept::FunctionIntercept;






fn main() {
    // let program = run_cargo_expand().unwrap();
    // println!("{}",program);
    let mut mockedFuns = CompileMocks::new(Vec::new(), None);
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
        &mut mockedFuns,
    );

    let mut insertion = FunctionIntercept::new(mockedFuns.get_mocks());
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
    let output = Command::new("./target/mocked_main")
        .output();
    
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




