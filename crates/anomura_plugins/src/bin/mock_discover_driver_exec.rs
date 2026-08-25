#![feature(rustc_private)]

use anomura_plugins::mock_discover_pass::DiscoverPlugin;
use rustc_plugin::RustcPlugin;

fn main() {
    env_logger::init();
    DiscoverPlugin::driver_main();
}
