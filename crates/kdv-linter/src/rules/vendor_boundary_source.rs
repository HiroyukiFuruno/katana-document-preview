use crate::diagnostics::{KdvLintError, Violation};
use crate::span::SpanOps;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

use super::vendor_boundary::{VendorScope, is_allowed_source_ref, is_vendor_ref};

pub(super) struct VendorBoundarySourceRule;

impl VendorBoundarySourceRule {
    pub(super) fn check(root: &Path) -> Result<Vec<Violation>, KdvLintError> {
        let mut violations = Vec::new();
        for scope in &SOURCE_SCOPES {
            let source_root = root.join(scope.path);
            violations.extend(SourceScanner::scan(*scope, &source_root)?);
        }
        Ok(violations)
    }
}

const SOURCE_SCOPES: [VendorScope; 2] = [
    VendorScope::core("crates/katana-document-viewer/src"),
    VendorScope::core("tools/kdv-storybook/src"),
];

struct SourceScanner;

impl SourceScanner {
    fn scan(scope: VendorScope, source_root: &Path) -> Result<Vec<Violation>, KdvLintError> {
        let mut violations = Vec::new();
        if !source_root.exists() {
            return Ok(violations);
        }
        for entry in WalkBuilder::new(source_root).standard_filters(true).build() {
            let Ok(entry) = entry else {
                continue;
            };
            let path = entry.path();
            if !is_rust_file(path) {
                continue;
            }
            violations.extend(Self::scan_file(scope, path)?);
        }
        Ok(violations)
    }

    fn scan_file(scope: VendorScope, path: &Path) -> Result<Vec<Violation>, KdvLintError> {
        let source = std::fs::read_to_string(path).map_err(|source| KdvLintError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let syntax = syn::parse_file(&source).map_err(|source| {
            let location = SpanOps::start(source.span());
            KdvLintError::RustParse {
                path: path.to_path_buf(),
                line: location.line,
                column: location.column,
                message: source.to_string(),
            }
        })?;
        let mut visitor = VendorSourceVisitor::new(scope, path);
        visitor.visit_file(&syntax);
        Ok(visitor.into_violations())
    }
}

struct VendorSourceVisitor {
    path: PathBuf,
    scope: VendorScope,
    violations: Vec<Violation>,
}

impl VendorSourceVisitor {
    fn new(scope: VendorScope, path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            scope,
            violations: Vec::new(),
        }
    }

    fn into_violations(self) -> Vec<Violation> {
        self.violations
    }

    fn check_ref(&mut self, name: &str, span: proc_macro2::Span) {
        if !is_vendor_ref(name) || is_allowed_source_ref(self.scope, &self.path, name) {
            return;
        }
        let location = SpanOps::start(span);
        self.violations.push(Violation::new(
            self.path.clone(),
            location.line,
            location.column,
            "vendor-boundary-source",
            source_message(self.scope, name),
        ));
    }

    fn check_use_tree(&mut self, tree: &syn::UseTree) {
        match tree {
            syn::UseTree::Path(path) => self.check_ref(&path.ident.to_string(), path.ident.span()),
            syn::UseTree::Name(name) => self.check_ref(&name.ident.to_string(), name.ident.span()),
            syn::UseTree::Rename(rename) => {
                self.check_ref(&rename.ident.to_string(), rename.ident.span());
            }
            syn::UseTree::Group(group) => {
                for item in &group.items {
                    self.check_use_tree(item);
                }
            }
            syn::UseTree::Glob(_) => {}
        }
    }
}

impl<'ast> Visit<'ast> for VendorSourceVisitor {
    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        self.check_use_tree(&node.tree);
        syn::visit::visit_item_use(self, node);
    }

    fn visit_item_extern_crate(&mut self, node: &'ast syn::ItemExternCrate) {
        self.check_ref(&node.ident.to_string(), node.ident.span());
        syn::visit::visit_item_extern_crate(self, node);
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        if let Some(first) = node.segments.first() {
            self.check_ref(&first.ident.to_string(), first.ident.span());
        }
        syn::visit::visit_path(self, node);
    }
}

fn is_rust_file(path: &Path) -> bool {
    path.is_file() && path.extension().is_some_and(|extension| extension == "rs")
}

fn source_message(scope: VendorScope, name: &str) -> String {
    let _ = scope;
    format!("core render code must not reference vendor adapter `{name}`.")
}
