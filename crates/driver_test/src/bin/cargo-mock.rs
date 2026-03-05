#![feature(rustc_private)]

use std::{collections::HashMap, process::exit};

use driver_test::mock_discover_pass::MockFnCall;

fn main() {
    env_logger::init();
    let Some(res) = rustc_plugin::cli_main(driver_test::mock_discover_pass::DiscoverPlugin::new())
        .unwrap_or_else(|e| {
            eprintln!("discover pass failed with error {:?},", e);
            exit(1)
        })
    else {
        println!("no mocks found");
        return;
    };
    println!("got res: {:?}", res);

    let mut crates_containing_mocks: HashMap<String, Vec<MockFnCall>> = HashMap::new();
    /*   for mock_fn in &res.fn_calls {
        crates_containing_mocks
            .entry(mock_fn.path_segments[0].path.clone())
            .and_modify(|v| v.push(mock_fn.clone()))
            .or_default();
    }*/

    if let Err(e) = rustc_plugin::cli_main(driver_test::substitution_pass::SubstitutePlugin::new(
        crates_containing_mocks,
    )) {
        eprintln!("substitute pass failed with error: {:?}", e);
        exit(1)
    }
}
