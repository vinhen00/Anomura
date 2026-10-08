use std::{
    borrow::Cow,
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use crate::{MOCK_CRATE_TARGETS_ENV, SUBSTITUTION_MOCK_PATHS, Utf8Path};

use anomura_driver::{MockObject, compile_mocks::CompileMocks};

use anomura_driver::crate_intercept::CrateIntercept;
use anomura_driver::function_intercept::FunctionIntercept;
use rustc_plugin::{
    CargoBuildCommand, CrateFilter, DefaultBuildCommand, PluginResult, RustcEnabledForNonFiltered,
    RustcPlugin, RustcPluginArgs, RustcWrapperType,
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
    rustc_driver::compiler_entrypoint(
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
        let args: Vec<String> = env::args().skip(2).filter(|a| !a.is_empty()).collect();
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

        // Extract the cargo subcommand (check, test, build, etc.) from args so
        // cli_main places it in the correct position (before flags).
        // If no recognized command is found, fall back to check.
        use std::str::FromStr;
        let build_cmd = args
            .iter()
            .find_map(|a| CargoBuildCommand::from_str(a).ok())
            .unwrap_or(CargoBuildCommand::Check);
        // Remove the build command from args so it isn't duplicated
        let args: Vec<String> = args
            .into_iter()
            .filter(|a| CargoBuildCommand::from_str(a).is_err())
            .collect();

        RustcPluginArgs {
            args: Some(args),
            filter,
            wrapper_type: RustcWrapperType::RustcWrapper,
            rustc_enabled_for_non_filtered: RustcEnabledForNonFiltered::Yes,
            default_build_command: Some(DefaultBuildCommand::Default(build_cmd)),
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

        // Link against context crate (needed for both paths).
        // The context rmeta path is resolved in modify_cargo() and passed via env var.
        let context_path = std::env::var(crate::CONTEXT_RMETA_PATH_ENV)
            .expect("CONTEXT_RMETA_PATH env var not set — was modify_cargo() called?");
        let mut compiler_args = compiler_args;
        compiler_args.push("--extern".into());
        compiler_args.push(format!("context={}", context_path));

        // Suppress warnings for mocked crates — their bodies are generated code
        // with synthetic source locations (e.g. <mock_gen_N>) that tools like
        // rust-analyzer cannot resolve to real files.
        compiler_args.push("--cap-lints=allow".into());

        if is_mock_crate_target {
            println!("Running CrateIntercept for mock_crate target: {crate_name}");
            let mut callbacks = CrateIntercept::new(crate_name.clone());
            log::debug!("crate_intercept compiler args: {:?}", compiler_args);
            rustc_driver::compiler_entrypoint(&compiler_args, &mut callbacks);
        } else {
            println!("Running FunctionIntercept for crate: {crate_name}");
            let program = std::env::var(SUBSTITUTION_MOCK_PATHS)
                .expect("should always be available at this point");
            let mut mock_map = mock_map_from_program(program);
            let mocks = mock_map.remove(&crate_name).expect("should exist");
            let mut callbacks = FunctionIntercept::new(mocks);
            println!("plugin_args: {:?}", plugin_args);
            log::debug!("sub new compiler args: {:?}", compiler_args);
            rustc_driver::compiler_entrypoint(&compiler_args, &mut callbacks);
        }

        Ok(())
    }

    fn modify_cargo(&self, cargo: &mut std::process::Command, args: &Vec<String>) {
        println!("cargo args: {:?}", &args);
        cargo.env(SUBSTITUTION_MOCK_PATHS, self.program.clone());
        if !self.mock_crate_targets.is_empty() {
            cargo.env(MOCK_CRATE_TARGETS_ENV, self.mock_crate_targets.join(","));
        }

        // Resolve the context crate's .rmeta path from the plugin target directory
        // and pass it to the driver via env var, so the driver can inject
        // `--extern context=<path>` into compiler args regardless of whether
        // cargo passes `-L` flags.
        let metadata = cargo_metadata::MetadataCommand::new()
            .no_deps()
            .other_options(["--all-features".to_string(), "--offline".to_string()])
            .exec()
            .expect("cargo metadata failed");
        let plugin_subdir = format!("plugin-{}", rustc_plugin::CHANNEL);
        let build_dir = metadata
            .target_directory
            .join(plugin_subdir)
            .join("debug")
            .join("build");
        if let Some(context_path) = find_context_rmeta(build_dir.as_std_path()) {
            // Also add the context output directory as a dependency search path.
            // Rewritten crates (e.g. fns) depend on context at the rmeta level,
            // and rustc needs -L to resolve transitive dependencies when loading
            // their metadata from downstream crates (e.g. mocks).
            if let Some(context_dir) = context_path.parent() {
                let existing_flags = std::env::var("RUSTFLAGS").unwrap_or_default();
                let new_flags =
                    format!("{} -L dependency={}", existing_flags, context_dir.display());
                cargo.env("RUSTFLAGS", new_flags.trim());
            }
            cargo.env(crate::CONTEXT_RMETA_PATH_ENV, context_path);
        }

        // Forward debug env vars
        if let Ok(v) = std::env::var("DUMP_AST") {
            cargo.env("DUMP_AST", v);
        }
        if let Ok(v) = std::env::var("AST_WRITE") {
            cargo.env("AST_WRITE", v);
        }
        // Propagate the user's working directory so the driver can resolve relative paths
        if let Ok(cwd) = std::env::current_dir() {
            cargo.env("ANOMURA_CWD", cwd);
        }
        // Filter out empty arguments (e.g. from rust-analyzer passing "" in its command)
        let sanitized: Vec<&String> = args.iter().filter(|a| !a.is_empty()).collect();
        cargo.args(sanitized);
    }

    fn before_execution(&mut self) {}

    fn after_execution(&mut self) -> PluginResult<()> {
        Ok(())
    }
}
/// Search the plugin build directory for the context crate's .rmeta file.
///
/// The rustc_plugin output layout is `<build_dir>/context/<hash>/out/libcontext-<hash>.rmeta`.
fn find_context_rmeta(build_dir: &Path) -> Option<PathBuf> {
    let context_dir = build_dir.join("context");
    let entries = std::fs::read_dir(&context_dir).ok()?;
    for entry in entries.flatten() {
        let out_dir = entry.path().join("out");
        if let Ok(files) = std::fs::read_dir(&out_dir) {
            for file in files.flatten() {
                let name = file.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("libcontext") && name_str.ends_with(".rmeta") {
                    return Some(file.path());
                }
            }
        }
    }
    None
}
