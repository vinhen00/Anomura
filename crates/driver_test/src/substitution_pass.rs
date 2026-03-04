use std::{borrow::Cow, collections::HashMap, env};

use crate::mock_discover_pass::MockFnCall;
use crate::{SUBSTITUTION_MOCK_PATHS, Utf8Path};
use clap::Parser;
use itertools::Itertools;
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
    crate_mock_map: HashMap<String, Vec<MockFnCall>>,
}
impl SubstitutePlugin {
    pub fn new(crate_mock_map: HashMap<String, Vec<MockFnCall>>) -> Self {
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

        let filter = CrateFilter::RunOnCrates(self.crate_mock_map.keys().cloned().collect_vec());
        RustcPluginArgs {
            args,
            filter,
            wrapper_type: RustcWrapperType::RustcWrapper,
            rustc_enabled_for_non_filtered: true,
            default_build_command: None,
        }
    }

    fn run(
        compiler_args: Vec<String>,
        plugin_args: Self::Args,
    ) -> rustc_interface::interface::Result<()> {
        let mut callbacks = SubstitutePluginCallback::default();
        println!("compiler_args: {:?}", plugin_args.cargo_args);

        rustc_driver::run_compiler(&compiler_args, &mut callbacks);
        Ok(())
    }

    fn modify_cargo(&self, cargo: &mut std::process::Command, args: &Self::Args) {
        println!("cargo args: {:?}", &args.cargo_args);
        cargo.args(&args.cargo_args);
        let serialized = serde_json::to_string(&self.crate_mock_map)
            .expect("serialization of crate_mock_map failed");
        cargo.env(SUBSTITUTION_MOCK_PATHS, serialized);
    }

    fn before_execution(&mut self) {}

    fn after_execution(&self) -> PluginResult<()> {
        Ok(())
    }
}
#[derive(Default)]
pub struct SubstitutePluginCallback {
    mock_fns: Vec<MockFnCall>,
}

impl rustc_driver::Callbacks for SubstitutePluginCallback {
    fn after_crate_root_parsing(
        &mut self,
        compiler: &rustc_interface::interface::Compiler,
        krate: &mut rustc_ast::Crate,
    ) -> rustc_driver::Compilation {
        let source_name = compiler
            .sess
            .io
            .input
            .source_name()
            .into_local_path()
            .expect("should be able to cast");
        println!(
            "In crate root parse for substitute plugin with source name: {:?}",
            source_name
        );

        let mocks = if let Ok(mock_map_serialized) = std::env::var(SUBSTITUTION_MOCK_PATHS)
            && let Ok(mut mock_map) =
                serde_json::from_str::<HashMap<String, Vec<MockFnCall>>>(&mock_map_serialized)
            && let Some(mocks) = mock_map.remove(&source_name.to_str().unwrap().to_string())
        {
            mocks
        } else {
            panic!(
                "environment variable {:?} not found, when it should be set",
                SUBSTITUTION_MOCK_PATHS
            )
        };
        //send messages to main cargo process with mocks found.
        rustc_driver::Compilation::Stop
    }
}
