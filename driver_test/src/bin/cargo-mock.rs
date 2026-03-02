#![feature(rustc_private)]

use std::process::exit;

fn main() {
    env_logger::init();
    let res = rustc_plugin::cli_main(driver_test::mock_discover_pass::DiscoverPlugin::new())
        .unwrap_or_else(|e| {
            eprintln!("discover pass failed with error {:?},", e);
            exit(1)
        });
    println!("got res: {:?}", res);
}
