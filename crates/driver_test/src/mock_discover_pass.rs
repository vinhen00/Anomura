use crate::{DISCOVER_TMP, Utf8Path};
use clap::Parser;
use interprocess::local_socket::traits::{Listener as _, Stream};
use interprocess::local_socket::{
    self, GenericFilePath, GenericNamespaced, ListenerOptions, NameType, ToFsName, ToNsName,
};
use itertools::Itertools;

use mockingbird::compile_mocks::CompileMocks;
use mockingbird::parse_mocks::ParseMocks;

use rustc_plugin::{
    CargoBuildCommand, CrateFilter, DefaultBuildCommand, RustcEnabledForNonFiltered, RustcPlugin,
    RustcPluginArgs, RustcPluginError, RustcWrapperType,
};

use serde::{Deserialize, Serialize};

use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::process::exit;
use std::thread::{self, JoinHandle};
use std::{borrow::Cow, env};
#[derive(Parser, Serialize, Deserialize, Clone)]
pub struct DiscoverPluginArgs {
    #[arg(short, long)]
    allcaps: bool,

    #[clap(last = true)]
    cargo_args: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CallBackMessage {
    NewMocks(String, Vec<String>),
    Done,
}

#[non_exhaustive]
pub struct DiscoverPlugin {
    channel_name: String,
    listener_handle: Option<JoinHandle<(Vec<String>, Vec<String>)>>,
}
impl DiscoverPlugin {
    pub fn new() -> Self {
        let tmp_dir = std::env::temp_dir();
        let (name_string, name) = if GenericNamespaced::is_supported() {
            let name_string = format!("{tmp_dir:#?}/tmp.sock");
            let name = name_string
                .clone()
                .to_ns_name::<GenericNamespaced>()
                .unwrap();
            (name_string, name)
        } else {
            let name_string = format!("/tmp/{}.sock", tmp_dir.display());
            let name = name_string.clone().to_fs_name::<GenericFilePath>().unwrap();
            (name_string, name)
        };
        println!("Init listener to {}", name_string);

        let listener = match ListenerOptions::new().name(name).create_sync() {
            Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
                // When a program that uses a file-type socket name terminates
                // its socket server without deleting the file, a "corpse socket"
                // remains, which can neither be connected to nor reused by a new
                // listener. Normally, Interprocess takes care of this on affected
                // platforms by deleting the socket file when the listener is
                // dropped. (This is vulnerable to all sorts of races and thus can
                // be disabled.)
                // In a real program, instead of leaving it up to the user
                // to perform cleanup, one would use the .try_overwrite(true)
                // listener option to try to replace the socket.
                eprintln!(
                    "Error: could not start server because the socket file is \
            occupied. Please check if {name_string} is in use by another \
            process and try again."
                );
                exit(1)
            }
            x => x.unwrap(),
        };

        let listener_handle: JoinHandle<(Vec<String>, Vec<String>)> = thread::spawn(move || {
            let mut all_mocked_fns = Vec::new();
            let mut crate_list = Vec::new();
            // listener
            //     .set_nonblocking(local_socket::ListenerNonblockingMode::Accept)
            //     .ok();
            //let timeout = std::time::Instant::now() + std::time::Duration::from_secs(10);
            while let Ok(conn) = listener.accept() {
                let mut buffer = String::with_capacity(16192);
                if let Ok(mut _reader) = BufReader::new(conn).read_line(&mut buffer)
                    && let Ok(deserial) = serde_json::from_str::<CallBackMessage>(&buffer)
                {
                    match deserial {
                        CallBackMessage::NewMocks(text, crates) => {
                            all_mocked_fns.push(text);
                            crate_list.extend(crates.iter().cloned());
                        }
                        CallBackMessage::Done => {
                            println!("got message done");
                            return (all_mocked_fns, crate_list);
                        }
                    }
                }
                //i think this just potentially messes thins up
                //std::thread::sleep(std::time::Duration::from_millis(10));
            }
            (all_mocked_fns, crate_list)
        });
        DiscoverPlugin {
            channel_name: name_string,
            listener_handle: Some(listener_handle),
        }
    }
}
impl Default for DiscoverPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct DiscoverClientReturn {
    pub mocked_fns: String,
    pub crate_list: Vec<String>,
}

impl RustcPlugin<DiscoverClientReturn> for DiscoverPlugin {
    fn version(&self) -> Cow<'static, str> {
        env!("CARGO_PKG_VERSION").into()
    }

    fn driver_name(&self) -> Cow<'static, str> {
        "mock_discover_driver_exec".into()
    }

    fn args(&self, _target_dir: &Utf8Path) -> rustc_plugin::RustcPluginArgs {
        let args = env::args().skip(2).collect_vec();
        args.iter()
            .for_each(|a| log::debug!("discover arg: {:?}", a));

        RustcPluginArgs {
            args: Some(args),
            filter: CrateFilter::OnlyWorkspace,
            wrapper_type: RustcWrapperType::RustcWrapper,
            rustc_enabled_for_non_filtered: RustcEnabledForNonFiltered::Only(vec![
                "context".into(),
            ]),
            default_build_command: Some(DefaultBuildCommand::Override(CargoBuildCommand::Build)),
        }
    }

    fn run(
        _crate_name: String,
        compiler_args: Vec<String>,
        plugin_args: &Vec<String>,
    ) -> rustc_interface::interface::Result<()> {
        let mut callbacks = ParseMocks::new(true);
        println!("compiler_args: {:?}", plugin_args);
        rustc_driver::run_compiler(&compiler_args, &mut callbacks);
        println!("got callbacks {:?}", callbacks);
        let _ = send_back_results(&callbacks).inspect_err(|e| {
            eprintln!(
                "callback failed to send back result, got error message {:?}",
                e
            );
        });
        Ok(())
    }

    fn modify_cargo(&self, cargo: &mut std::process::Command, args: &Vec<String>) {
        cargo.env(DISCOVER_TMP, &self.channel_name);
        cargo.args(args);
    }

    fn before_execution(&mut self) {}

    fn after_execution(&mut self) -> Result<DiscoverClientReturn, RustcPluginError> {
        let name = if GenericNamespaced::is_supported() {
            self.channel_name
                .clone()
                .to_ns_name::<GenericNamespaced>()?
        } else {
            self.channel_name.clone().to_fs_name::<GenericFilePath>()?
        };

        //tell the server that we are done looking for messages sent from rustc_driver instances
        println!("Sending Done");
        let mut conn: BufWriter<local_socket::Stream> = BufWriter::new(Stream::connect(name)?);
        let json = serde_json::to_string(&CallBackMessage::Done).unwrap();
        conn.write_all(json.as_bytes())?;
        conn.write_all(b"\n")?;
        conn.flush()?;

        let listener_handle = self.listener_handle.take().expect(
            "after_execution in DiscoverPlugin should contain a listener_handle to be consumed",
        );

        let mocked_fns = listener_handle.join().map_err(|e| {
            RustcPluginError::ClientReturnError(format!("listener process returned : {:?}", e))
        })?;

        let program = mocked_fns.0.join("");
        let crate_list = mocked_fns.1;

        println!("After_execution: Found {} mocks", mocked_fns.0.len());
        Ok(DiscoverClientReturn {
            mocked_fns: program,
            crate_list,
        })
    }
}

pub fn compile_maccalls(program: &str) -> CompileMocks {
    let mut mocked_funs = CompileMocks::new(Vec::new(), program.to_string(), false);
    rustc_driver::run_compiler(
        &["ignored".to_string(), "anything".to_string()],
        &mut mocked_funs,
    );
    mocked_funs
}

pub fn send_back_results(parse_mocks: &ParseMocks) -> io::Result<()> {
    let cratelist = parse_mocks.get_crates();
    let mocks = parse_mocks.get_program();

    let inline_result = CallBackMessage::NewMocks(mocks, cratelist);


    let name_str = std::env::var(DISCOVER_TMP)
        .expect("there should be a discover tmp env var created in the main cargo command");
    let name = if GenericNamespaced::is_supported() {
        name_str.clone().to_ns_name::<GenericNamespaced>()?
    } else {
        name_str.clone().to_fs_name::<GenericFilePath>()?
    };

    println!("Sending mock {:?} to {}", inline_result, name_str);
    let mut conn: BufWriter<local_socket::Stream> = BufWriter::new(Stream::connect(name)?);
    let json = serde_json::to_string(&inline_result)?;
    conn.write_all(json.as_bytes())?;
    conn.write_all(b"\n")?;
    conn.flush()?;
    Ok(())
}
