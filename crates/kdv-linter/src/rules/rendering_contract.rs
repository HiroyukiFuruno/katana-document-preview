use crate::diagnostics::{KdvLintError, Violation};
use crate::span::SpanOps;
use crate::workspace::{SourceFile, WorkspaceModel};
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

pub struct RenderingContractRule;

impl RenderingContractRule {
    pub fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KdvLintError> {
        let mut violations = Vec::new();
        for file in workspace.rust_files() {
            if !RenderingContractScope::contains(file.path()) {
                continue;
            }
            let mut visitor = RenderingContractVisitor::new(file);
            visitor.visit_file(file.syntax());
            violations.extend(visitor.into_violations());
        }
        Ok(violations)
    }
}

struct RenderingContractScope;

impl RenderingContractScope {
    fn contains(path: &Path) -> bool {
        let text = path.to_string_lossy();
        text.contains("crates/katana-document-viewer/src/viewer/")
            || text.contains("crates/katana-document-viewer/src/document_surface/")
    }
}

struct RenderingContractVisitor<'a> {
    file: &'a SourceFile,
    violations: Vec<Violation>,
}

impl<'a> RenderingContractVisitor<'a> {
    fn new(file: &'a SourceFile) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn into_violations(self) -> Vec<Violation> {
        self.violations
    }

    fn check_string_literal(&mut self, literal: &syn::LitStr) {
        let value = literal.value();
        if Self::is_color_literal(&value) {
            self.push(
                literal.span(),
                "rendering-color-literal",
                "rendering code must use theme tokens instead of hard-coded color literals.",
            );
        }
        if Self::is_os_font_path(&value) {
            self.push(literal.span(), "rendering-font-path", "rendering code must receive fonts from the caller contract instead of OS font paths.");
        }
    }

    fn check_path(&mut self, path: &syn::Path) {
        let Some(last) = path.segments.last() else {
            return;
        };
        if matches!(
            last.ident.to_string().as_str(),
            "katana_light" | "katana_dark"
        ) {
            self.push(
                path.span(),
                "rendering-preset-reference",
                "rendering code must not call theme presets directly.",
            );
        }
    }

    fn is_color_literal(value: &str) -> bool {
        let Some(hex) = value.strip_prefix('#') else {
            return value.starts_with("rgb(") || value.starts_with("rgba(");
        };
        matches!(hex.len(), 3 | 4 | 6 | 8)
            && hex.chars().all(|character| character.is_ascii_hexdigit())
    }

    fn is_os_font_path(value: &str) -> bool {
        [
            "/System/Library/Fonts",
            "/Library/Fonts",
            "/usr/share/fonts",
            "C:\\Windows\\Fonts",
        ]
        .iter()
        .any(|fragment| value.contains(fragment))
    }

    fn push(&mut self, span: proc_macro2::Span, rule: &'static str, message: &'static str) {
        let location = SpanOps::start(span);
        self.violations.push(Violation::new(
            PathBuf::from(self.file.path()),
            location.line,
            location.column,
            rule,
            message,
        ));
    }
}

impl<'ast> Visit<'ast> for RenderingContractVisitor<'_> {
    fn visit_lit_str(&mut self, node: &'ast syn::LitStr) {
        self.check_string_literal(node);
        syn::visit::visit_lit_str(self, node);
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        self.check_path(node);
        syn::visit::visit_path(self, node);
    }
}

#[cfg(test)]
#[path = "rendering_contract_tests.rs"]
mod tests;
