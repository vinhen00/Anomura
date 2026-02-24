#![feature(rustc_private)]

fn main() {
    env_logger::init();
    rustc_plugin::driver_main(driver_test::mock_discover_pass::MockDiscover);
}
