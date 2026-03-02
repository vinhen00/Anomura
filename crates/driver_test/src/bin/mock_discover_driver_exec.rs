#![feature(rustc_private)]

use driver_test::mock_discover_pass::DiscoverPlugin;
use rustc_plugin::RustcPlugin;

fn main() {
    env_logger::init();
    DiscoverPlugin::driver_main();
}
