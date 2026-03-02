// Tested with nightly-2025-03-28

#![feature(rustc_private)]
mod expand_macro;
mod visitors;
mod compile_mocks;
mod function_intercept;
mod global_context;

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
use crate::global_context::{GlobalContext, MockFunction};


fn foo(cntxt: &mut GlobalContext, a: i32, b: String, c: i32){
    if let Some(boxed) = cntxt.get_mock("foo".to_string()) {
        if let Some(stats) = boxed.downcast_mut::<MockFunction<(i32, String, i32)>>(){
            stats.incr_count();
            stats.add_call((a,b,c));
        }
        
    }
    
}

fn main(){
    let mut cntxt = GlobalContext::new();

    let mut mock:MockFunction<(i32, String, i32)> = MockFunction::new(); 
    //mock.add_call((5, "Hello".to_string(), 8));
    let boxed = Box::new(mock);
    cntxt.insert_mock("foo".to_string(), boxed);


    foo(&mut cntxt, 1, "Soo".to_string(), 3);

    for i in 1..=10 {
        foo(&mut cntxt, i, "For".to_string(), i*2);
    }

    if let Some(boxed) = cntxt.get_mock("foo".to_string()) {
        if let Some(stats) = boxed.downcast_mut::<MockFunction<(i32, String, i32)>>(){
            let result = stats.get_call_list();
            for (i, j) in result {
                println!("Call {} had input {:#?}", i, j);
            }
        }
    }
}



fn main2() {
    let mut mocked_funs = CompileMocks::new(Vec::new(), None);
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
        &mut mocked_funs,
    );

    let mut insertion = FunctionIntercept::new(mocked_funs.get_mocks());
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




