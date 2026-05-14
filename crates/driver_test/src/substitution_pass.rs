use std::{
    borrow::Cow,
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use crate::{SUBSTITUTION_MOCK_PATHS, Utf8Path};

use mockingbird::{MACRO_MOCK_MAP, MockedFun, compile_mocks::CompileMocks, parse_mocks::MacroMap};

use itertools::Itertools;
use mockingbird::function_intercept::FunctionIntercept;
use rustc_plugin::{
    CrateFilter, PluginResult, RustcEnabledForNonFiltered, RustcPlugin, RustcPluginArgs,
    RustcWrapperType,
};
use serde::{Deserialize, Serialize};

#[derive(clap::Parser, Serialize, Deserialize)]
pub struct SubstitutePluginArgs {
    #[clap(last = true)]
    cargo_args: Vec<String>,
}

#[non_exhaustive]
pub struct SubstitutePlugin {
    program: String,
    crate_list: Vec<String>,
}

pub fn mock_map_from_program(
    program: String,
    with_macro_map: bool,
) -> (Option<MacroMap>, HashMap<String, Vec<MockedFun>>) {
    let mut callbacks = CompileMocks::new(Vec::new(), program.clone(), true, with_macro_map);
    rustc_driver::run_compiler(
        &[
            "ignored".to_string(),
            "mock_defs.rs".to_string(),
            "--crate-type".to_string(),
            "bin".to_string(),
            "-o".to_string(),
            "./target/mocked_main".to_string(),
        ],
        &mut callbacks,
    );

    let mut crate_mock_map: HashMap<String, Vec<MockedFun>> = HashMap::new();
    for mock_fn in &callbacks.get_mocks() {
        log::debug!("mock fn path : {:?}", mock_fn.get_path());
        crate_mock_map
            .entry(mock_fn.get_path_as_string())
            .and_modify(|v| v.push(mock_fn.clone()))
            .or_insert(vec![mock_fn.clone()]);
    }
    log::debug!("mock map keys: {:?}", crate_mock_map.keys());
    (callbacks.macro_map, crate_mock_map)
}

impl SubstitutePlugin {
    pub fn new(program: String, crate_list: Vec<String>) -> Self {
        Self {
            program,
            crate_list,
        }
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
        let crate_filters = self.crate_list.clone();
        //only execute driver on crates containing mocks
        println!("crate filters: {:?}", crate_filters);
        println!("here we are");
        let filter = CrateFilter::RunOnCrates(crate_filters);
        if let CrateFilter::RunOnCrates(filt) = &filter {
            println!("{:#?}", filt);
        }
        RustcPluginArgs {
            args: Some(args),
            filter,
            wrapper_type: RustcWrapperType::RustcWrapper,
            rustc_enabled_for_non_filtered: RustcEnabledForNonFiltered::Yes,
            default_build_command: None,
        }
    }

    fn run(
        crate_name: String,
        compiler_args: Vec<String>,
        plugin_args: &Vec<String>,
    ) -> rustc_interface::interface::Result<()> {
        log::debug!("Test");
        let program = std::env::var(SUBSTITUTION_MOCK_PATHS)
            .expect("should always be available at this point");
        let (_, mut mock_map) = mock_map_from_program(program, false);
        let mocks = mock_map.remove(&crate_name).expect("should exist");
        let mut callbacks = FunctionIntercept::new(mocks);
        log::debug!("runnin sugstitution plugin for crate {crate_name}");
        log::debug!("plugin_args: {:?}", plugin_args);

        //link against context
        let l_index = compiler_args
            .iter()
            .enumerate()
            .find(|(_, e)| *e == "-L")
            .map(|(i, _)| i)
            .unwrap();
        let dependency_args = compiler_args[l_index + 1].split("=").collect_vec();
        assert!(dependency_args[0] == "dependency");
        let dependency_path = dependency_args[1].to_string();
        let context_path = get_context_meta_file(Path::new(&dependency_path))
            .expect("context .rmeta path not found");
        let mut compiler_args = compiler_args;
        compiler_args.insert(l_index + 2, "--extern".into());
        compiler_args.insert(
            l_index + 3,
            format!("context={}", context_path.to_string_lossy()),
        );

        log::debug!("sub new compiler args: {:?}", compiler_args);
        rustc_driver::run_compiler(&compiler_args, &mut callbacks);
        Ok(())
    }

    fn modify_cargo(&self, cargo: &mut std::process::Command, args: &Vec<String>) {
        println!("cargo args: {:?}", &args);
        let (macro_map, mocked_funs) = mock_map_from_program(self.program.clone(), true);

        //to be used by macros to fetch type information for expectations and sequences.
        let macro_map = serde_json::to_string(&macro_map).unwrap();
        cargo.env(MACRO_MOCK_MAP, macro_map);
        cargo.env(SUBSTITUTION_MOCK_PATHS, self.program.clone());
        cargo.args(args);
    }

    fn before_execution(&mut self) {}

    fn after_execution(&mut self) -> PluginResult<()> {
        Ok(())
    }
}
fn get_context_meta_file(folder_path: &Path) -> Option<PathBuf> {
    let Ok(entries) = std::fs::read_dir(folder_path) else {
        return None;
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();

        if name_str.starts_with("libcontext") && name_str.ends_with(".rmeta") {
            return Some(entry.path());
        }
    }
    None
}
