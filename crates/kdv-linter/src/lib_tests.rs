use super::*;
use crate::rules::test_helpers::FixtureWorkspace;

#[test]
fn lint_workspace_returns_rule_violations_for_valid_root() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    fixture.write_rust_file(
        "crates/katana-document-viewer/src/document_surface/leaked_entry.rs",
        "pub fn leaked_entry() {}",
    )?;
    let violations = KdvLinter::lint_workspace(&fixture.root)?;

    assert!(
        violations
            .iter()
            .any(|violation| violation.rule == "public-free-function")
    );
    Ok(())
}

#[test]
fn lint_workspace_returns_error_for_missing_root() -> Result<(), KdvLintError> {
    let missing = std::env::temp_dir().join("kdv-linter-missing-root-fixture");
    let result = KdvLinter::lint_workspace(&missing);
    assert!(result.is_err());
    Ok(())
}
