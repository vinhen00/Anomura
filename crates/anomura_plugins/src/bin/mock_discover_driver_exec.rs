#![feature(rustc_private)]

use anomura_plugins::mock_discover_pass::DiscoverPlugin;
use rustc_plugin::RustcPlugin;

use std::process::ExitCode;

fn main() -> ExitCode {
    env_logger::init();
    DiscoverPlugin::driver_main()
}
