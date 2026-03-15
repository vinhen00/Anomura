#![feature(rustc_private)]

use std::process::exit;

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
    // println!(
    //     "Mocking done found {} functions to mock",
    //     &res.mocked_fns.len()
    // );
    // for i in &res.mocked_fns {
    //     println!("mocked fn: {} in path: {}", i.get_name(), i.get_path());
    // }

    // let mut crates_containing_mocks: HashMap<String, Vec<MockedFun>> = HashMap::new();
    // for mock_fn in &res.mocked_fns {
    //     println!("mock fn path : {:?}", mock_fn.get_path());
    //     crates_containing_mocks
    //         .entry(mock_fn.get_path())
    //         .and_modify(|v| v.push(mock_fn.clone()))
    //         .or_insert(vec![mock_fn.clone()]);
    // }

    if let Err(e) = rustc_plugin::cli_main(driver_test::substitution_pass::SubstitutePlugin::new(
        res.mocked_fns,
    )) {
        eprintln!("substitute pass failed with error: {:?}", e);
        exit(1)
    }
}
