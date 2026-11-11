//! CrateIntercept — the `rustc_driver::Callbacks` implementation for `mock_crate!` targets.
//!
//! In the two-pass architecture, CrateIntercept is the second pass (generative).
//! It receives a pre-collected `CrateApiModel` from the `AnalysisPass` and uses it
//! in `after_crate_root_parsing` to generate mock infrastructure and inject it into the AST.

use rustc_ast as ast;
use rustc_driver::Compilation;
use rustc_interface::interface::Compiler;
use rustc_parse::lexer::StripTokens;
use rustc_parse::parser::AllowConstBlockItems;
use rustc_span::FileName;

use crate::crate_api::*;
use crate::crate_mock_gen;

/// The CrateIntercept driver — receives a pre-collected API model and generates mock infrastructure.
#[derive(Debug)]
pub struct CrateIntercept {
    crate_name: String,
    /// The API model collected by the AnalysisPass (first pass).
    api: CrateApiModel,
}

impl CrateIntercept {
    pub fn new(crate_name: String, api: CrateApiModel) -> Self {
        Self { crate_name, api }
    }

    /// Parse a source string containing multiple items and inject them into the crate.
    fn inject_source_items(&self, compiler: &Compiler, krate: &mut ast::Crate, source: &str) {
        let psess = &compiler.sess.psess;
        let filename = FileName::Custom("mock_convenience_api".to_string());

        let mut parser = match rustc_parse::new_parser_from_source_str(
            psess,
            filename,
            source.to_string(),
            StripTokens::Nothing,
        ) {
            Ok(parser) => parser,
            Err(diags) => {
                for d in diags {
                    d.cancel();
                }
                log::error!("CrateIntercept: failed to parse convenience API source");
                return;
            }
        };

        let mut injected = 0;
        loop {
            match parser.parse_item(rustc_parse::parser::ForceCollect::No, AllowConstBlockItems::Yes) {
                Ok(Some(item)) => {
                    krate.items.push(item);
                    injected += 1;
                }
                Ok(None) => break, // end of input
                Err(diag) => {
                    diag.cancel();
                    log::error!("CrateIntercept: error parsing convenience API item");
                    break;
                }
            }
        }
        log::debug!("CrateIntercept: injected {} convenience API items", injected);
    }
}

impl rustc_driver::Callbacks for CrateIntercept {
    fn after_crate_root_parsing(
        &mut self,
        compiler: &Compiler,
        krate: &mut ast::Crate,
    ) -> Compilation {
        log::debug!("CrateIntercept: regenerating crate '{}' from analysis model", self.crate_name);

        log::debug!("CrateIntercept: API from AnalysisPass:");
        log::debug!("  {} structs", self.api.root.structs.len());
        log::debug!("  {} enums", self.api.root.enums.len());
        log::debug!("  {} traits", self.api.root.traits.len());
        log::debug!("  {} functions", self.api.root.functions.len());
        log::debug!("  {} impls", self.api.root.impls.len());
        log::debug!("  {} submodules", self.api.root.children.len());

        let dump_ast = std::env::var("DUMP_AST").is_ok();
        let ast_write = std::env::var("AST_WRITE").ok();

        // Step 1: Retain only scaffolding items
        let before_count = krate.items.len();
        krate.items.retain(|item| matches!(&item.kind,
            ast::ItemKind::Use(..)
            | ast::ItemKind::TyAlias(..)
            | ast::ItemKind::Const(..)
            | ast::ItemKind::Static(..)
            | ast::ItemKind::ForeignMod(..)
            | ast::ItemKind::MacroDef(..)
            | ast::ItemKind::ExternCrate(..)
        ));
        log::debug!("CrateIntercept: retained {} scaffolding items (removed {} items)",
            krate.items.len(), before_count - krate.items.len());

        // Step 2: Generate all items from the model (definitions + mock dispatch + convenience API)
        let source = crate_mock_gen::gen_crate_from_model(&self.api);
        if !source.is_empty() {
            self.inject_source_items(compiler, krate, &source);
        }

        log::debug!("CrateIntercept: regenerated crate has {} total items", krate.items.len());

        // Step 3: Dump the final AST if requested
        if dump_ast || ast_write.is_some() {
            let mut dump_buf = String::new();
            for item in krate.items.iter() {
                dump_buf.push_str(&rustc_ast_pretty::pprust::item_to_string(item));
                dump_buf.push_str("\n\n");
            }

            if dump_ast {
                log::debug!("\n=== DUMP_AST: regenerated items ===\n{}\n=== end ===", dump_buf);
            }

            if let Some(path) = &ast_write {
                let resolved = if std::path::Path::new(path).is_absolute() {
                    std::path::PathBuf::from(path)
                } else {
                    let base = std::env::var("ANOMURA_CWD")
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
                    base.join(path)
                };
                if let Err(e) = std::fs::write(&resolved, &dump_buf) {
                    log::error!("CrateIntercept: failed to write AST to {}: {}", resolved.display(), e);
                } else {
                    log::debug!("CrateIntercept: wrote regenerated AST to {}", resolved.display());
                }
            }
        }

        Compilation::Continue
    }
}
