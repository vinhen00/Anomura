#![feature(rustc_private)]

use driver_test::PrintAllItemsPlugin;
use rustc_plugin::RustcPlugin;

fn main() {
    env_logger::init();
    PrintAllItemsPlugin::driver_main();
}
