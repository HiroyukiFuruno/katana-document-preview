use crate::diagnostics::{KdvLintError, Violation};
use crate::span::SpanOps;
use crate::syntax::AttributeOps;
use crate::workspace::WorkspaceModel;
use std::path::PathBuf;
use syn::visit::Visit;

pub struct PublicFreeFunctionRule;

impl PublicFreeFunctionRule {
    pub fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KdvLintError> {
        let mut violations = Vec::new();
        for file in workspace.rust_files() {
            let mut visitor = PublicFreeFunctionVisitor::new(file.path().to_path_buf());
            visitor.visit_file(file.syntax());
            violations.extend(visitor.into_violations());
        }
        Ok(violations)
    }
}

struct PublicFreeFunctionVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
    in_test_context: bool,
}

impl PublicFreeFunctionVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            in_test_context: false,
        }
    }

    fn into_violations(self) -> Vec<Violation> {
        self.violations
    }

    fn is_public(vis: &syn::Visibility) -> bool {
        matches!(vis, syn::Visibility::Public(_))
            || matches!(vis, syn::Visibility::Restricted(scope) if scope.path.is_ident("crate"))
    }

    fn push_violation(&mut self, node: &syn::ItemFn) {
        let location = SpanOps::start(node.sig.ident.span());
        self.violations.push(Violation::new(
            self.file.clone(),
            location.line,
            location.column,
            "public-free-function",
            format!(
                "public free function `{}` must move behind a struct impl.",
                node.sig.ident
            ),
        ));
    }
}

impl<'ast> Visit<'ast> for PublicFreeFunctionVisitor {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let previous = self.in_test_context;
        self.in_test_context |= AttributeOps::has_cfg_test_attr(&node.attrs);
        syn::visit::visit_item_mod(self, node);
        self.in_test_context = previous;
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if self.in_test_context || node.sig.ident == "main" {
            return;
        }
        if AttributeOps::has_cfg_test_attr(&node.attrs) {
            return;
        }
        if Self::is_public(&node.vis) {
            self.push_violation(node);
        }
        syn::visit::visit_item_fn(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::FixtureWorkspace;
    use super::*;

    #[test]
    fn public_free_function_rule_excludes_private_and_crate_fns() -> Result<(), KdvLintError> {
        let fixture = FixtureWorkspace::new().with_default_manifests()?;
        let source = r#"
fn private() {}
pub(crate) fn inside_crate() {}
pub fn public() {}
"#;
        fixture.write_rust_file("crates/katana-document-viewer/src/public_free.rs", source)?;

        let workspace = fixture.workspace()?;
        let violations = PublicFreeFunctionRule::check(&workspace)?;
        let found = violations
            .iter()
            .any(|violation| violation.message.contains("public"));

        assert!(found);
        Ok(())
    }

    #[test]
    fn public_free_function_rule_skips_test_and_main_contexts() -> Result<(), KdvLintError> {
        let fixture = FixtureWorkspace::new().with_default_manifests()?;
        let source = r#"
pub fn visible() {}

fn main() {}

#[cfg(test)]
mod tests {
    pub fn test_fn() {}
}
"#;
        fixture.write_rust_file("crates/katana-document-viewer/src/main_and_test.rs", source)?;
        let workspace = fixture.workspace()?;
        let violations = PublicFreeFunctionRule::check(&workspace)?;
        let count = violations
            .iter()
            .filter(|violation| violation.message.contains("visible"))
            .count();

        assert_eq!(count, 1);
        Ok(())
    }

    #[test]
    fn public_free_function_rule_counts_document_surface_file() -> Result<(), KdvLintError> {
        let fixture = FixtureWorkspace::new().with_default_manifests()?;
        let source = r#"
pub fn ui_entry() {}
"#;
        fixture.write_rust_file(
            "crates/katana-document-viewer/src/document_surface/public_entry.rs",
            source,
        )?;
        let workspace = fixture.workspace()?;
        let violations = PublicFreeFunctionRule::check(&workspace)?;

        assert!(
            violations
                .iter()
                .any(|violation| violation.message.contains("ui_entry"))
        );
        Ok(())
    }

    #[test]
    fn public_free_function_rule_flags_crate_visibility() -> Result<(), KdvLintError> {
        let fixture = FixtureWorkspace::new().with_default_manifests()?;
        let source = r#"
pub(crate) fn crate_visible() {}
"#;
        fixture.write_rust_file("crates/katana-document-viewer/src/visible.rs", source)?;
        let workspace = fixture.workspace()?;
        let violations = PublicFreeFunctionRule::check(&workspace)?;

        assert!(
            violations
                .iter()
                .any(|violation| violation.message.contains("`crate_visible`"))
        );
        Ok(())
    }

    #[test]
    fn public_free_function_rule_skips_cfg_test_functions() -> Result<(), KdvLintError> {
        let fixture = FixtureWorkspace::new().with_default_manifests()?;
        let source = r#"
#[cfg(test)]
pub fn cfg_function() {}
"#;
        fixture.write_rust_file("crates/katana-document-viewer/src/test_cfg.rs", source)?;
        let workspace = fixture.workspace()?;
        let violations = PublicFreeFunctionRule::check(&workspace)?;

        assert!(violations.is_empty());
        Ok(())
    }
}
