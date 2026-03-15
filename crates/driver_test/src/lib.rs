//! A Rustc plugin that prints out the name of all items in a crate.

#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_parse;
extern crate rustc_session;
use rustc_plugin::Utf8Path;

pub mod mock_discover_pass;
pub mod substitution_pass;
///Used to set channel name for interprocess callbacks from driver instances in MockDiscoverPass
pub const DISCOVER_TMP: &str = "DISCOVER_BOOTSTRAP";

pub const SELECTED_CRATES: &[&str] = &["memchr", "serde_json"];
//Environment variable for mocks definitions in rustc
pub const SUBSTITUTION_MOCK_PATHS: &str = "SUBSTITUTION_MOCK_PATHS";
