use std::fs;
use syn::{visit::Visit, File, Macro};
use proc_macro2::TokenStream;


pub struct MockCollector {
    pub mock_fns: Vec<TokenStream>,
    pub mock_methods: Vec<TokenStream>,
}

impl<'ast> Visit<'ast> for MockCollector {
    fn visit_macro(&mut self, node: &'ast Macro) {
        if let Some(ident) = node.path.get_ident() {
            match ident.to_string().as_str() {
                "mock_fn" => {
                    self.mock_fns.push(node.tokens.clone());
                }
                "mock_method" => {
                    self.mock_methods.push(node.tokens.clone());
                }
                _ => {}
            }
        }

        syn::visit::visit_macro(self, node);
    }
}

pub fn collect_from_file(path: &str) -> syn::Result<MockCollector> {
    let content = fs::read_to_string(path)
        .map_err(|e| syn::Error::new(proc_macro2::Span::call_site(), e.to_string()))?;
    let ast: File = syn::parse_file(&content)?;

    let mut collector = MockCollector {
        mock_fns: Vec::new(),
        mock_methods: Vec::new(),
    };

    collector.visit_file(&ast);
    Ok(collector)
}