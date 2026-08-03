use super::ViewerMediaActionPrefixRule;
use crate::diagnostics::{KdvLintError, Violation};
use crate::rules::test_helpers::FixtureWorkspace;

#[test]
fn viewer_media_prefix_rule_flags_document_surface_duplicate_prefix() -> Result<(), KdvLintError> {
    let violations = violations_for_document_surface_prefix()?;

    assert!(violations.iter().any(|violation| {
        violation.rule == "no_duplicate_viewer_media_action_prefix"
            && violation
                .file
                .to_string_lossy()
                .contains("document_surface")
    }));
    Ok(())
}

#[test]
fn viewer_media_prefix_rule_allows_kdv_core_owner() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    fixture.write_rust_file(
        "crates/katana-document-viewer/src/viewer/media_action.rs",
        r#"
const IMAGE_ACTION_PREFIX: &str = "viewer.image.";
"#,
    )?;

    let violations = ViewerMediaActionPrefixRule::check(&fixture.workspace()?)?;

    assert!(violations.is_empty());
    Ok(())
}

#[test]
fn viewer_media_prefix_rule_allows_kdv_core_spec_owner() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    fixture.write_rust_file(
        "crates/katana-document-viewer/src/viewer/media_control_spec_tests.rs",
        r#"
fn expected() {
    let _ = "viewer.diagram.zoom-in";
}
"#,
    )?;

    let violations = ViewerMediaActionPrefixRule::check(&fixture.workspace()?)?;

    assert!(violations.is_empty());
    Ok(())
}

fn violations_for_document_surface_prefix() -> Result<Vec<Violation>, KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    fixture.write_rust_file(
        "crates/katana-document-viewer/src/document_surface/node_factory_media_controls.rs",
        r#"
const IMAGE_HOST_ACTION_PREFIX: &str = "viewer.image.";
const DIAGRAM_HOST_ACTION_PREFIX: &str = "viewer.diagram.";
"#,
    )?;
    ViewerMediaActionPrefixRule::check(&fixture.workspace()?)
}
