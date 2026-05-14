#![feature(rustc_private)]
pub mod compile_mocks;
mod expand_macro;
pub mod function_intercept;
pub mod parse_mocks;
mod visitors;

pub const MACRO_MOCK_MAP: &str = "MACRO_MOCK_MAP";
extern crate rustc_ast;
extern crate rustc_ast_pretty;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_driver_impl;
extern crate rustc_error_codes;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_parse;
extern crate rustc_passes;
extern crate rustc_serialize;
extern crate rustc_session;
extern crate rustc_span;

pub use visitors::MockedFun;
