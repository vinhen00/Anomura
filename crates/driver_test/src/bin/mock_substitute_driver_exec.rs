#![feature(rustc_private)]

use driver_test::substitution_pass;

use rustc_plugin::RustcPlugin;
pub fn main() {
    env_logger::init();
    //substitution_pass::SubstitutePlugin::driver_main();
}
