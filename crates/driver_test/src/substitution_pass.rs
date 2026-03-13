use std::{borrow::Cow, collections::HashMap, env};

use crate::Utf8Path;

use mockingbird::{MockedFun, compile_mocks::CompileMocks};

use itertools::Itertools;
use mockingbird::function_intercept::FunctionIntercept;
use rustc_plugin::{CrateFilter, PluginResult, RustcPlugin, RustcPluginArgs, RustcWrapperType};
use serde::{Deserialize, Serialize};



#[derive(clap::Parser, Serialize, Deserialize)]
pub struct SubstitutePluginArgs {
    #[clap(last = true)]
    cargo_args: Vec<String>,
}

#[non_exhaustive]
pub struct SubstitutePlugin {
    program: String,
    crate_mock_map: HashMap<String, Vec<MockedFun>>,
}
impl SubstitutePlugin {
    pub fn new(program: String)-> Self {
        let mut callbacks = CompileMocks::new(Vec::new(), program.clone(), true);
        rustc_driver::run_compiler(&[], &mut callbacks);

        let mut crate_mock_map: HashMap<String, Vec<MockedFun>> = HashMap::new();
        for mock_fn in &callbacks.get_mocks() {
            println!("mock fn path : {:?}", mock_fn.get_path());
            crate_mock_map
                .entry(mock_fn.get_path())
                .and_modify(|v| v.push(mock_fn.clone()))
                .or_insert(vec![mock_fn.clone()]);
        }

        Self {program, crate_mock_map }
    }
}

impl RustcPlugin for SubstitutePlugin {
    fn version(&self) -> Cow<'static, str> {
        env!("CARGO_PKG_VERSION").into()
    }

    fn driver_name(&self) -> Cow<'static, str> {
        "mock_substitute_driver_exec".into()
    }

    fn args(&self, _target_dir: &Utf8Path) -> rustc_plugin::RustcPluginArgs {
        let args = env::args().skip(2).collect_vec();
        args.iter()
            .for_each(|a| log::debug!("discover arg: {:?}", a));

        //Hashset to skip duplicates
        let crate_filters = self.crate_mock_map.keys().cloned().collect_vec();
        //only execute driver on crates containing mocks
        println!("crate filters: {:?}", crate_filters);
        let filter = CrateFilter::RunOnCrates(crate_filters);
        if let CrateFilter::RunOnCrates(filt) = &filter {
            println!("{:#?}", filt);
        }
        //let filter = CrateFilter::OnlyWorkspace;
        RustcPluginArgs {
            args: Some(args),
            filter,
            wrapper_type: RustcWrapperType::RustcWrapper,
            rustc_enabled_for_non_filtered: true,
            default_build_command: None,
        }
    }

    fn run(
        crate_name: String,
        compiler_args: Vec<String>,
        plugin_args: &Vec<String>,
    ) -> rustc_interface::interface::Result<()> {
        let mut callbacks = FunctionIntercept::new(Vec::new());
        println!("runnin sugstitution plugin for crate {crate_name}");
        println!("plugin_args: {:?}", plugin_args);

        let result = rustc_driver::run_compiler(&compiler_args, &mut callbacks);
        println!("{:#?}", result);
        Ok(())
    }

    fn modify_cargo(&self, cargo: &mut std::process::Command, args: &Vec<String>) {
        println!("cargo args: {:?}", &args);
        cargo.args(args);
    }

    fn before_execution(&mut self) {}

    fn after_execution(&mut self) -> PluginResult<()> {
        Ok(())
    }
}
