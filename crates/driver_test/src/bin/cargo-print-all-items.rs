#![feature(rustc_private)]

fn main() {
    env_logger::init();
    println!("hello world");
    rustc_plugin::cli_main(driver_test::PrintAllItemsPlugin);
}
