#![feature(rustc_private)]

use std::process::exit;

fn main() {
    env_logger::init();
    let Some(res) = rustc_plugin::cli_main(anomura_plugins::mock_discover_pass::DiscoverPlugin::new())
        .unwrap_or_else(|e| {
            eprintln!("discover pass failed with error {:?},", e);
            exit(1)
        })
    else {
        println!("no mocks found");
        return;
    };

    println!("CRATE LIST: {:#?}", res.crate_list);
    if !res.mock_crate_targets.is_empty() {
        println!("MOCK_CRATE TARGETS: {:#?}", res.mock_crate_targets);
    }

    if let Err(e) = rustc_plugin::cli_main(anomura_plugins::substitution_pass::SubstitutePlugin::new(
        res.mocked_fns,
        res.crate_list,
        res.mock_crate_targets,
    )) {
        eprintln!("substitute pass failed with error: {:?}", e);
        exit(1)
    }
}
