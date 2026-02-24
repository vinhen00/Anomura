use std::{borrow::Cow, env};

use crate::Utf8Path;
use clap::Parser;
use rustc_ast::token::TokenKind::{self, Comma, Eof};
use rustc_ast::{Expr, MethodCall, Path, visit::Visitor};
use rustc_parse::parser::{self, ExpTokenPair, TokenType};
use rustc_plugin::{CrateFilter, RustcPlugin, RustcPluginArgs, RustcWrapperType};
use rustc_session::parse::ParseSess;
use serde::{Deserialize, Serialize};
#[derive(Parser, Serialize, Deserialize, Clone)]
pub struct MockDiscoverArgs {
    #[arg(short, long)]
    allcaps: bool,

    #[clap(last = true)]
    cargo_args: Vec<String>,
}

pub struct MockDiscover;

impl RustcPlugin for MockDiscover {
    type Args = MockDiscoverArgs;

    fn version(&self) -> Cow<'static, str> {
        env!("CARGO_PKG_VERSION").into()
    }

    fn driver_name(&self) -> Cow<'static, str> {
        "mock_discover_driver_exec".into()
    }

    fn args(&self, _target_dir: &Utf8Path) -> rustc_plugin::RustcPluginArgs<Self::Args> {
        let args = MockDiscoverArgs::parse_from(env::args().skip(1));
        args.cargo_args
            .iter()
            .for_each(|a| log::debug!("discover arg: {:?}", a));

        let filter = CrateFilter::OnlyWorkspace;
        RustcPluginArgs {
            args,
            filter,
            wrapper_type: RustcWrapperType::RustcWrapper,
        }
    }

    fn run(
        compiler_args: Vec<String>,
        plugin_args: Self::Args,
    ) -> rustc_interface::interface::Result<()> {
        let mut callbacks = MockDiscoverCallback::default();
        println!("compiler_args: {:?}", plugin_args.cargo_args);
        rustc_driver::run_compiler(&compiler_args, &mut callbacks);

        Ok(())
    }

    fn modify_cargo(&self, cargo: &mut std::process::Command, args: &Self::Args) {
        println!("cargo args: {:?}", &args.cargo_args);
        cargo.args(&args.cargo_args);
    }
}
#[derive(Default)]
pub struct MockDiscoverCallback {
    mock_fns: Vec<Path>,
}

impl rustc_driver::Callbacks for MockDiscoverCallback {
    fn after_crate_root_parsing(
        &mut self,
        compiler: &rustc_interface::interface::Compiler,
        krate: &mut rustc_ast::Crate,
    ) -> rustc_driver::Compilation {
        println!("in crate root parse");
        let psess = &compiler.sess.psess;
        let mut visitor = MockVisitor::new(&compiler.sess.psess);
        visitor.visit_crate(krate);
        rustc_driver::Compilation::Stop
    }
}

pub struct MockVisitor<'a> {
    psess: &'a ParseSess,
    fn_calls: Vec<(Box<Expr>, Vec<Box<Expr>>)>,
    method_calls: Vec<Box<MethodCall>>,
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
            let mut parser = rustc_parse::parser::Parser::new(&self.psess, tokens.clone(), None)
                .recovery(parser::Recovery::Allowed);

            while parser.token != Eof {
                if let Ok(expr) = parser.parse_expr() {
                    match expr.kind {
                        rustc_ast::ExprKind::Call(fn_ident_expr, args) => {
                            println!(
                                "with function call {:?} with args: {:?}",
                                fn_ident_expr, args
                            );
                            self.fn_calls.push((fn_ident_expr, args.into()));
                        }
                        rustc_ast::ExprKind::MethodCall(method_call) => {
                            println!("with method call: {:?}", method_call);
                            self.method_calls.push(method_call);
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
