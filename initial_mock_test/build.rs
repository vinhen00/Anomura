#![feature(rustc_private)]
use std::{
    ops::Deref,
    path::{Path, PathBuf},
    process::Command,
};
pub fn main() {}

fn target_libdir(rustc: &Path) -> PathBuf {
    let output = Command::new(rustc)
        .args(["--print", "target-libdir"])
        .output()
        .expect("failed to run rustc --print target-libdir");
    let libdir = String::from_utf8(output.stdout).unwrap();
    PathBuf::from(libdir.trim())
}
