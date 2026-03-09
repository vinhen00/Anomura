use crate::{Utf8Path, DISCOVER_TMP};
use clap::Parser;
use interprocess::local_socket::traits::{Listener as _, Stream};
use interprocess::local_socket::{
    self, GenericFilePath, GenericNamespaced, Listener, ListenerOptions, NameType, ToFsName,
    ToNsName,
};
use itertools::Itertools;

use mockingbird::compile_mocks::CompileMocks;
use mockingbird::parse_mocks::ParseMocks;
use mockingbird::{compile_mocks, MockedFun};

use rustc_ast::token::TokenKind::{self, Eof};
use rustc_ast::PathSegment;
use rustc_ast::{visit::Visitor, MethodCall};
use rustc_interface::Config;
use rustc_parse::parser::{self};
use rustc_plugin::{CrateFilter, RustcPlugin, RustcPluginArgs, RustcPluginError, RustcWrapperType};
use rustc_session::parse::ParseSess;
use serde::{Deserialize, Serialize};

use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;
use std::{borrow::Cow, env};
#[derive(Parser, Serialize, Deserialize, Clone)]
pub struct DiscoverPluginArgs {
    #[arg(short, long)]
    allcaps: bool,

    #[clap(last = true)]
    cargo_args: Vec<String>,
}

#[non_exhaustive]
pub struct DiscoverPlugin {
    channel_name: String,
    listener: Option<Listener>,
    collected_mocks: Arc<Mutex<Vec<MockedFun>>>,
}
impl DiscoverPlugin {
    pub fn new() -> Self {
        let tmp_dir = std::env::temp_dir();
        let (name_string, name) = if GenericNamespaced::is_supported() {
            let name_string = format!("{}.sock", tmp_dir.display());
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
        DiscoverPlugin {
            channel_name: name_string,
            listener: Some(listener),
            collected_mocks: Arc::new(Mutex::new(Vec::new())),
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
    pub mocked_fns: Vec<MockedFun>,
}

impl RustcPlugin<DiscoverClientReturn> for DiscoverPlugin {
    type Args = DiscoverPluginArgs;

    fn version(&self) -> Cow<'static, str> {
        env!("CARGO_PKG_VERSION").into()
    }

    fn driver_name(&self) -> Cow<'static, str> {
        "mock_discover_driver_exec".into()
    }

    fn args(&self, _target_dir: &Utf8Path) -> rustc_plugin::RustcPluginArgs<Self::Args> {
        let args = DiscoverPluginArgs::parse_from(env::args().skip(1));
        args.cargo_args
            .iter()
            .for_each(|a| log::debug!("discover arg: {:?}", a));

        let filter = CrateFilter::OnlyWorkspace;
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
        let mut callbacks = ParseMocks::new(true);
        println!("compiler_args: {:?}", plugin_args.cargo_args);
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

    fn modify_cargo(&self, cargo: &mut std::process::Command, args: &Self::Args) {
        println!("cargo args: {:?}", &args.cargo_args);
        cargo.env(DISCOVER_TMP, &self.channel_name);
        cargo.args(&args.cargo_args);
    }

    fn before_execution(&mut self) {
        // Start a background thread to listen for connections during cargo execution
        let mocks = self.collected_mocks.clone();
        let listener = self.listener.take().expect("listener should exist");
        thread::spawn(move || {
            listener.set_nonblocking(local_socket::ListenerNonblockingMode::Accept).ok();
            let timeout = std::time::Instant::now() + std::time::Duration::from_secs(10);
            while std::time::Instant::now() < timeout {
                if let Ok(mut conn) = listener.accept() {
                    let mut buffer = String::with_capacity(16192);
                    if let Ok(mut reader) = BufReader::new(conn).read_line(&mut buffer) {
                        if let Ok(deserial) = serde_json::from_str::<String>(&buffer) {
                            let fns = compile_maccalls(&deserial);
                            if let Ok(mut m) = mocks.lock() {
                                m.append(&mut fns.get_mocks());
                            }
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        });
    }

    fn after_execution(&self) -> Result<DiscoverClientReturn, RustcPluginError> {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let mocked_fns = self.collected_mocks.lock().unwrap().clone();
        Ok(DiscoverClientReturn { mocked_fns })
    }
}

pub fn compile_maccalls(program: &str) -> CompileMocks {
    let mut mocked_funs = CompileMocks::new(Vec::new(), program.to_string(), false);
    //println!("Mockedfuns: {:#?}", mocked_funs.inline);
    rustc_driver::run_compiler(
        &["ignored".to_string(), "anything".to_string()],
        &mut mocked_funs,
    );
    mocked_funs
}
#[derive(Default)]
pub struct DiscoverPluginCallback {
    mock_fns: Vec<MockFnCall>,
}
pub fn send_back_results(parse_mocks: &ParseMocks) -> io::Result<()> {
    let inline_result = parse_mocks.get_program() else {
        return Err(io::Error::other("no mocks to unpack"));
    };

    let name_str = std::env::var(DISCOVER_TMP)
        .expect("there should be a discover tmp env var created in the main cargo command");
    let name = if GenericNamespaced::is_supported() {
        name_str.clone().to_ns_name::<GenericNamespaced>()?
    } else {
        name_str.clone().to_fs_name::<GenericFilePath>()?
    };

    let mut conn: BufWriter<local_socket::Stream> = BufWriter::new(Stream::connect(name)?);
    let json = serde_json::to_string(&inline_result)?;
    conn.write_all(json.as_bytes())?;
    conn.write_all(b"\n")?;
    conn.flush()?;
    Ok(())
}

impl rustc_driver::Callbacks for DiscoverPluginCallback {
    fn after_crate_root_parsing(
        &mut self,
        compiler: &rustc_interface::interface::Compiler,
        krate: &mut rustc_ast::Crate,
    ) -> rustc_driver::Compilation {
        println!("in crate root parse");
        let mut visitor = MockVisitor::new(&compiler.sess.psess);
        visitor.visit_crate(krate);
        self.mock_fns = visitor.fn_calls;
        //send messages to main cargo process with mocks found.
        rustc_driver::Compilation::Stop
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MockPathSegment {
    pub path: String,
}
impl MockPathSegment {
    pub fn new(path: PathSegment) -> Self {
        MockPathSegment {
            path: path.ident.as_str().to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MockFnCall {
    pub path_segments: Vec<MockPathSegment>,
}
pub struct MockMethodCalls {}

pub struct MockVisitor<'a> {
    psess: &'a ParseSess,
    fn_calls: Vec<MockFnCall>,
    method_calls: Vec<MethodCall>,
}
impl<'a> MockVisitor<'a> {
    pub fn new(psess: &'a ParseSess) -> Self {
        MockVisitor {
            psess,
            fn_calls: vec![],
            method_calls: vec![],
        }
    }
}

impl<'a> Visitor<'a> for MockVisitor<'a> {
    #[doc = r" The result type of the `visit_*` methods. Can be either `()`,"]
    #[doc = r" or `ControlFlow<T>`."]
    type Result = ();
    fn visit_mac_call(&mut self, node: &'_ rustc_ast::MacCall) -> Self::Result {
        let Some(elem) = node.path.segments.iter().last() else {
            log::error!("failed to find last segment");
            return;
        };

        if elem.ident.as_str() == "mock" {
            println!("found mock {node:?}");
            let tokens = &node.args.tokens;
            let mut parser = rustc_parse::parser::Parser::new(self.psess, tokens.clone(), None)
                .recovery(parser::Recovery::Allowed);

            while parser.token != Eof {
                if let Ok(expr) = parser.parse_expr() {
                    match expr.kind {
                        rustc_ast::ExprKind::Call(expr, args) => {
                            println!("with function call {:?} with args: {:?}", &expr, args);
                            let rustc_ast::ExprKind::Path(_, path) = expr.kind else {
                                eprintln!("function identifier must be a path");
                                exit(1);
                            };
                            self.fn_calls.push(MockFnCall {
                                path_segments: path
                                    .segments
                                    .into_iter()
                                    .map(MockPathSegment::new)
                                    .collect_vec(),
                            });
                        }
                        rustc_ast::ExprKind::MethodCall(method_call) => {
                            println!("with method call: {:?}", method_call);
                            self.method_calls.push(*method_call);
                        }
                        _ => (),
                    }
                }
                while parser.token.kind == TokenKind::Comma {
                    parser.bump();
                }
            }
        }
    }
}
