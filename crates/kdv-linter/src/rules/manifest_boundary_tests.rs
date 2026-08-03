use super::super::architecture::VIEWER_CRATE;
use super::*;
use crate::rules::test_helpers::FixtureWorkspace;
use std::path::Path;
use toml::Value;

#[test]
fn manifest_boundary_check_rejects_ui_dependency() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    let viewer_manifest = r#"
[package]
name = "katana-document-viewer"
version = "0.1.0"
edition = "2021"

[dependencies]
egui = "0.29"
"#;
    fixture.write_manifest(&format!("{VIEWER_CRATE}/Cargo.toml"), viewer_manifest)?;

    let violations = ManifestBoundaryRule::check(&fixture.root)?;

    assert!(
        violations
            .iter()
            .any(|violation| violation.message.contains("UI crate `egui`"))
    );
    Ok(())
}

#[test]
fn manifest_boundary_check_accepts_feature_scoped_optional_host_dependency()
-> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    let viewer_manifest = r#"
[package]
name = "katana-document-viewer"
version = "0.1.0"
edition = "2021"

[features]
egui = ["dep:egui"]

[dependencies]
egui = { version = "0.35", optional = true }
"#;
    fixture.write_manifest(&format!("{VIEWER_CRATE}/Cargo.toml"), viewer_manifest)?;

    let violations = ManifestBoundaryRule::check(&fixture.root)?;

    assert!(
        !violations
            .iter()
            .any(|violation| violation.rule == "preview-boundary")
    );
    Ok(())
}

#[test]
fn manifest_boundary_check_rejects_removed_adapter_dependency() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    let viewer_manifest = r#"
[package]
name = "katana-document-viewer"
version = "0.1.0"
edition = "2021"

[dependencies]
katana-document-preview-egui = { path = "../katana-document-preview-egui" }
"#;
    fixture.write_manifest(&format!("{VIEWER_CRATE}/Cargo.toml"), viewer_manifest)?;
    let violations = ManifestBoundaryRule::check(&fixture.root)?;

    assert!(
        violations
            .iter()
            .any(|violation| violation.rule == "preview-boundary")
    );
    Ok(())
}

#[test]
fn manifest_boundary_check_neutral_manifest_collects_package_dependency() -> Result<(), KdvLintError>
{
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    let viewer_manifest = r#"
[package]
name = "katana-document-viewer"
version = "0.1.0"
edition = "2021"

[dependencies]
alias-ui = { package = "egui", version = "0.29" }
"#;
    fixture.write_manifest("crates/katana-document-viewer/Cargo.toml", viewer_manifest)?;

    let violations = ManifestBoundaryRule::check(&fixture.root)?;
    let has_preview_boundary = violations
        .iter()
        .any(|violation| violation.rule == "preview-boundary");

    assert!(has_preview_boundary);
    Ok(())
}

#[test]
fn manifest_boundary_check_accepts_non_ui_dependency() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    let viewer_manifest = r#"
[package]
name = "katana-document-viewer"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1"
"#;
    fixture.write_manifest("crates/katana-document-viewer/Cargo.toml", viewer_manifest)?;
    let violations = ManifestBoundaryRule::check(&fixture.root)?;
    let has_preview_boundary = violations
        .iter()
        .any(|violation| violation.rule == "preview-boundary");

    assert!(!has_preview_boundary);
    Ok(())
}

#[test]
fn manifest_reader_collects_dependencies_from_all_tables_and_package_names()
-> Result<(), KdvLintError> {
    let fixture = fixture_with_dependency_tables()?;
    let dependencies = manifest_dependencies(&fixture.root)?;

    assert!(dependencies.iter().any(|name| name == "egui"));
    assert!(dependencies.iter().any(|name| name == "floem"));
    assert!(dependencies.iter().any(|name| name == "builder"));
    Ok(())
}

fn fixture_with_dependency_tables() -> Result<FixtureWorkspace, KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    let manifest = r#"
[package]
name = "katana-document-viewer"
version = "0.1.0"
edition = "2021"

[dependencies]
egui = "0.29"

[dev-dependencies]
helper = { package = "floem", version = "0.1" }

[build-dependencies]
builder = "1.0"
"#;
    fixture.write_manifest("crates/katana-document-viewer/Cargo.toml", manifest)?;
    Ok(fixture)
}

fn manifest_dependencies(root: &Path) -> Result<Vec<String>, KdvLintError> {
    let data = manifest_reader_fixture(root)?;
    Ok(ManifestReader::dependency_names(&data))
}

fn manifest_reader_fixture(root: &Path) -> Result<Value, KdvLintError> {
    ManifestReader::read(&root.join("crates/katana-document-viewer/Cargo.toml"))
}

#[test]
fn manifest_reader_reports_toml_parse_error() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    let invalid_manifest = r#"
[package]
name = "katana-document-viewer"
version = "0.1.0"
edition = "2021"

[dependencies]
egui = ["
"#;
    fixture.write_manifest("crates/katana-document-viewer/Cargo.toml", invalid_manifest)?;

    assert!(ManifestBoundaryRule::check(&fixture.root).is_err());
    Ok(())
}
