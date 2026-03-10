use std::{borrow::Cow, collections::HashMap, env};

use crate::mock_discover_pass::MockFnCall;
use crate::Utf8Path;

use mockingbird::MockedFun;
use mockingbird::function_intercept;

use clap::Parser;
use itertools::Itertools;
use mockingbird::function_intercept::FunctionIntercept;
use rustc_plugin::{CrateFilter, PluginResult, RustcPlugin, RustcPluginArgs, RustcWrapperType};
use serde::{Deserialize, Serialize};

#[derive(clap::Parser, Serialize, Deserialize)]
pub struct SubstitutePluginArgs {
    #[arg(short, long)]
    allcaps: bool,

    #[clap(last = true)]
    cargo_args: Vec<String>,
}

#[non_exhaustive]
pub struct SubstitutePlugin {
    crate_mock_map: HashMap<String, Vec<MockedFun>>,
}
impl SubstitutePlugin {
    pub fn new(crate_mock_map: HashMap<String, Vec<MockedFun>>) -> Self {
        Self { crate_mock_map }
    }
}

impl RustcPlugin for SubstitutePlugin {
    type Args = SubstitutePluginArgs;

    fn version(&self) -> Cow<'static, str> {
        env!("CARGO_PKG_VERSION").into()
    }

    fn driver_name(&self) -> Cow<'static, str> {
        "mock_substitute_driver_exec".into()
    }

    fn args(&self, _target_dir: &Utf8Path) -> rustc_plugin::RustcPluginArgs<Self::Args> {
        let args = SubstitutePluginArgs::parse_from(env::args().skip(1));
        args.cargo_args
            .iter()
            .for_each(|a| log::debug!("discover arg: {:?}", a));

        //Hashset to skip duplicates
        //only execute driver on crates containing mocks
        let filter = CrateFilter::RunOnCrates(self.crate_mock_map.keys().cloned().collect_vec());
        if let CrateFilter::RunOnCrates(filt) = &filter {
            println!("{:#?}", filt);
        }
        //let filter = CrateFilter::OnlyWorkspace;
        RustcPluginArgs {
            args,
            filter,
            wrapper_type: RustcWrapperType::RustcWrapper,
            rustc_enabled_for_non_filtered: true,
            default_build_command: None,
        }
    }

    fn run(
        crate_name: String,
        compiler_args: Vec<String>,
        plugin_args: Self::Args,
    ) -> rustc_interface::interface::Result<()> {
        //println!("{:#?}", plugin_args.filter);
        //let mut mockfuns: Vec<MockedFun>;
        // match self.crate_mock_map.get(crate_name) {
        //     Some(mocks) => { mockfuns = mocks }
        //     None => { mockfuns = Vec::new() }
        // }
        let mut callbacks = FunctionIntercept::new(Vec::new());
        println!("runnin sugstitution plugin for crate {crate_name}");
        println!("compiler_args: {:?}", plugin_args.cargo_args);

        let result = rustc_driver::run_compiler(&compiler_args, &mut callbacks);
        println!("{:#?}", result);
        Ok(())
    }

    fn modify_cargo(&self, cargo: &mut std::process::Command, args: &Self::Args) {
        println!("cargo args: {:?}", &args.cargo_args);
        cargo.args(&args.cargo_args);
    }

    fn before_execution(&mut self) {}

    fn after_execution(&self) -> PluginResult<()> {
        Ok(())
    }
}
// #[derive(Default)]
// pub struct SubstitutePluginCallback {
//     mock_fns: Vec<MockFnCall>,
// }

// impl rustc_driver::Callbacks for SubstitutePluginCallback {
//     fn after_crate_root_parsing(
//         &mut self,
//         compiler: &rustc_interface::interface::Compiler,
//         krate: &mut rustc_ast::Crate,
//     ) -> rustc_driver::Compilation {
//         //send messages to main cargo process with mocks found.
//         rustc_driver::Compilation::Stop
//     }
// }
