use super::VendorBoundaryRule;
use crate::diagnostics::KdvLintError;
use crate::rules::test_helpers::FixtureWorkspace;

#[test]
fn vendor_boundary_rejects_core_source_vendor_refs() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    fixture.write_rust_file(
        "crates/katana-document-viewer/src/vendor_leak.rs",
        r#"
use egui::Color32;

pub struct Core;

impl Core {
    pub fn render(&self) {
        let _ = katana_ui_core_egui::EguiCompatAdapter;
        let _color = Color32::BLACK;
    }
}
"#,
    )?;

    let violations = VendorBoundaryRule::check(&fixture.workspace()?)?;

    assert!(has_rule(&violations, "vendor-boundary-source"));
    Ok(())
}

#[test]
fn vendor_boundary_rejects_core_manifest_vendor_refs() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    fixture.write_manifest(
        "crates/katana-document-viewer/Cargo.toml",
        r#"
[package]
name = "katana-document-viewer"
version = "0.1.0"
edition = "2021"

[dependencies]
egui = "0.34"
katana-ui-core-egui = { path = "../katana-ui-core-egui" }
"#,
    )?;

    let violations = VendorBoundaryRule::check(&fixture.workspace()?)?;

    assert!(has_rule(&violations, "vendor-boundary-manifest"));
    Ok(())
}

#[test]
fn vendor_boundary_rejects_core_source_cross_vendor_refs() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    fixture.write_rust_file(
        "crates/katana-document-viewer/src/document_surface/cross_vendor_leak.rs",
        r#"
use floem::View;

pub struct Adapter;

impl Adapter {
    pub fn paint(&self) {
        let _ = katana_ui_core_floem::FloemAdapter;
    }
}
"#,
    )?;

    let violations = VendorBoundaryRule::check(&fixture.workspace()?)?;

    assert!(has_rule(&violations, "vendor-boundary-source"));
    Ok(())
}

fn has_rule(violations: &[crate::diagnostics::Violation], rule: &str) -> bool {
    violations.iter().any(|violation| violation.rule == rule)
}
