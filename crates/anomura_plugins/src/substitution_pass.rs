use std::{
    borrow::Cow,
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use crate::{MOCK_CRATE_TARGETS_ENV, SUBSTITUTION_MOCK_PATHS, Utf8Path};

use anomura_driver::{MockObject, compile_mocks::CompileMocks};

use itertools::Itertools;
use anomura_driver::function_intercept::FunctionIntercept;
use anomura_driver::crate_intercept::CrateIntercept;
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
    mock_crate_targets: Vec<String>,
}

pub fn mock_map_from_program(program: String) -> HashMap<String, Vec<MockObject>> {
    let mut callbacks = CompileMocks::new(Vec::new(), program.clone(), true);
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

    let mut crate_mock_map: HashMap<String, Vec<MockObject>> = HashMap::new();
    for mock in &callbacks.get_mocks() {
        println!("mock fn path : {:?}", mock.get_path());
        crate_mock_map
            .entry(mock.get_path())
            .and_modify(|v| v.push(mock.clone()))
            .or_insert(vec![mock.clone()]);
    }
    println!("mock map keys: {:?}", crate_mock_map.keys());
    crate_mock_map
}

impl SubstitutePlugin {
    pub fn new(program: String, crate_list: Vec<String>, mock_crate_targets: Vec<String>) -> Self {
        Self {
            program: program.clone(),
            crate_list,
            mock_crate_targets,
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
        // Check if this crate is a mock_crate target
        let mock_crate_targets: Vec<String> = std::env::var(MOCK_CRATE_TARGETS_ENV)
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        let is_mock_crate_target = mock_crate_targets.contains(&crate_name);

        // Link against context crate (needed for both paths)
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

        if is_mock_crate_target {
            println!("Running CrateIntercept for mock_crate target: {crate_name}");
            let mut callbacks = CrateIntercept::new(crate_name.clone());
            log::debug!("crate_intercept compiler args: {:?}", compiler_args);
            rustc_driver::run_compiler(&compiler_args, &mut callbacks);
        } else {
            println!("Running FunctionIntercept for crate: {crate_name}");
            let program = std::env::var(SUBSTITUTION_MOCK_PATHS)
                .expect("should always be available at this point");
            let mut mock_map = mock_map_from_program(program);
            let mocks = mock_map.remove(&crate_name).expect("should exist");
            let mut callbacks = FunctionIntercept::new(mocks);
            println!("plugin_args: {:?}", plugin_args);
            log::debug!("sub new compiler args: {:?}", compiler_args);
            rustc_driver::run_compiler(&compiler_args, &mut callbacks);
        }

        Ok(())
    }

    fn modify_cargo(&self, cargo: &mut std::process::Command, args: &Vec<String>) {
        println!("cargo args: {:?}", &args);
        cargo.env(SUBSTITUTION_MOCK_PATHS, self.program.clone());
        if !self.mock_crate_targets.is_empty() {
            cargo.env(MOCK_CRATE_TARGETS_ENV, self.mock_crate_targets.join(","));
        }
        // Forward debug env vars
        if let Ok(v) = std::env::var("DUMP_AST") { cargo.env("DUMP_AST", v); }
        if let Ok(v) = std::env::var("AST_WRITE") { cargo.env("AST_WRITE", v); }
        // Propagate the user's working directory so the driver can resolve relative paths
        if let Ok(cwd) = std::env::current_dir() {
            cargo.env("ANOMURA_CWD", cwd);
        }
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
