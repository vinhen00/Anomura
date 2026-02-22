use std::{borrow::Cow, env};

use crate::Utf8Path;
use clap::Parser;
use rustc_plugin::{CrateFilter, RustcPlugin, RustcPluginArgs, RustcWrapperType};
use serde::{Deserialize, Serialize};
#[derive(Parser, Serialize, Deserialize, Clone)]
pub struct MockDiscoverArgs {
    #[arg(short, long)]
    allcaps: bool,

    #[clap(last = true)]
    cargo_args: Vec<String>,
}

pub struct MockDiscover {}

impl RustcPlugin for MockDiscover {
    type Args = MockDiscoverArgs;

    fn version(&self) -> Cow<'static, str> {
        env!("CARGO_PKG_VERSION").into()
    }

    fn driver_name(&self) -> Cow<'static, str> {
        "mock discover".into()
    }

    fn args(&self, target_dir: &Utf8Path) -> rustc_plugin::RustcPluginArgs<Self::Args> {
        let args = MockDiscoverArgs::parse_from(env::args().skip(1));
        args.cargo_args
            .iter()
            .for_each(|a| log::debug!("discover arg: {:?}", a));

        let filter = CrateFilter::OnlyWorkspace;
        RustcPluginArgs {
            args,
            filter,
            wrapper_type: RustcWrapperType::RustcWorkspaceWrapper,
        }
    }

    fn run(
        compiler_args: Vec<String>,
        plugin_args: Self::Args,
    ) -> rustc_interface::interface::Result<()> {
        todo!()
    }
}

impl rustc_driver::Callbacks for MockDiscover {}
