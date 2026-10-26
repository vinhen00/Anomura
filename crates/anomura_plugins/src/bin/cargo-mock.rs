#![feature(rustc_private)]

use std::process::exit;

fn main() {
    env_logger::init();
    let Some(res) = rustc_plugin::cli_main(anomura_plugins::mock_discover_pass::DiscoverPlugin::new())
        .unwrap_or_else(|e| {
            log::error!("discover pass failed with error {:?},", e);
            exit(1)
        })
    else {
        log::debug!("no mocks found");
        return;
    };

    log::debug!("CRATE LIST: {:#?}", res.crate_list);
    if !res.mock_crate_targets.is_empty() {
        log::debug!("MOCK_CRATE TARGETS: {:#?}", res.mock_crate_targets);
    }

    if let Err(e) = rustc_plugin::cli_main(anomura_plugins::substitution_pass::SubstitutePlugin::new(
        res.mocked_fns,
        res.crate_list,
        res.mock_crate_targets,
    )) {
        log::error!("substitute pass failed with error: {:?}", e);
        exit(1)
    }
}
