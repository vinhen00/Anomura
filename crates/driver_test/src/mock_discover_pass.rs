use crate::{DISCOVER_TMP, Utf8Path};
use clap::Parser;
use interprocess::local_socket::traits::{Listener as _, Stream};
use interprocess::local_socket::{
    self, GenericFilePath, GenericNamespaced, Listener, ListenerOptions, NameType, ToFsName,
    ToNsName,
};
use itertools::Itertools;
use mockingbird::compile_mocks::CompileMocks;
use mockingbird::{MockedFun, compile_mocks};
use rustc_ast::PathSegment;
use rustc_ast::token::TokenKind::{self, Eof};
use rustc_ast::{MethodCall, visit::Visitor};
use rustc_interface::Config;
use rustc_parse::parser::{self};
use rustc_plugin::{CrateFilter, RustcPlugin, RustcPluginArgs, RustcPluginError, RustcWrapperType};
use rustc_session::parse::ParseSess;
use serde::{Deserialize, Serialize};

use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::process::exit;
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
    listener: Listener,
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
            listener,
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
        let mut callbacks = CompileMocks::new(Vec::new(), None, true);
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

    fn before_execution(&mut self) {}

    fn after_execution(&self) -> Result<DiscoverClientReturn, RustcPluginError> {
        let mut client_return = DiscoverClientReturn { mocked_fns: vec![] };
        let mut buffer = String::with_capacity(16192);
        self.listener
            .set_nonblocking(local_socket::ListenerNonblockingMode::Accept)
            .map_err(|_| {
                RustcPluginError::ClientReturnError("failed to set nonblockng state".to_string())
            })?;

        //receive a stream of expanded mock definitions
        while let Ok(mut conn) = self.listener.accept().map(BufReader::new) {
            conn.read_line(&mut buffer)?;
            //Do a single compilation of the file with concatinated mocks
            println!("before compile maccalls");
            let mocked_fns = compile_maccalls(&buffer);
            println!("after compile maccalls");

            client_return.mocked_fns.append(&mut mocked_fns.get_mocks());
            // Avoid holding up resources.
            drop(conn);

            // read_line keeps the line feed at the end.

            // Clear the buffer so that the next iteration will display new data
            // instead of messages stacking on top of one another.
            buffer.clear();
        }
        Ok(client_return)
    }
}

pub fn compile_maccalls(program: &str) -> CompileMocks {
    let mut mocked_funs = CompileMocks::new(Vec::new(), Some(String::from(program)), false);
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
pub fn send_back_results(compile_mocks: &CompileMocks) -> io::Result<()> {
    let Some(inline_result) = compile_mocks.get_inline() else {
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
    let serialized = serde_json::to_vec(&inline_result)?;
    conn.get_mut().write_all(&serialized)?;
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
