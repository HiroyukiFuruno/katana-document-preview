use super::{RenderingContractRule, RenderingContractVisitor};
use crate::diagnostics::KdvLintError;
use crate::rules::test_helpers::FixtureWorkspace;

#[test]
fn rendering_contract_flags_color_font_path_and_preset_refs() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    fixture.write_rust_file(
        "crates/katana-document-viewer/src/document_surface/adapter.rs",
        r##"
pub fn render() {
    let _color = "#ff00aa";
    let _font = "/System/Library/Fonts/SFNS.ttf";
    let _theme = KdvThemeSnapshot::katana_light();
}
"##,
    )?;

    let violations = RenderingContractRule::check(&fixture.workspace()?)?;

    assert!(has_rule(&violations, "rendering-color-literal"));
    assert!(has_rule(&violations, "rendering-font-path"));
    assert!(has_rule(&violations, "rendering-preset-reference"));
    Ok(())
}

#[test]
fn rendering_contract_ignores_theme_preset_module() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    fixture.write_rust_file(
        "crates/katana-document-viewer/src/theme_presets.rs",
        r##"
pub fn render() {
    let _color = "#ff00aa";
    let _theme = KdvThemeSnapshot::katana_light();
}
"##,
    )?;

    let violations = RenderingContractRule::check(&fixture.workspace()?)?;

    assert!(violations.is_empty());
    Ok(())
}

#[test]
fn rendering_contract_ignores_empty_syn_path() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    fixture.write_rust_file(
        "crates/katana-document-viewer/src/document_surface/adapter.rs",
        "",
    )?;
    let workspace = fixture.workspace()?;
    let Some(file) = workspace.rust_files().first() else {
        return Err(Box::new(std::io::Error::other("fixture file missing")));
    };
    let empty_path = syn::Path {
        leading_colon: None,
        segments: syn::punctuated::Punctuated::new(),
    };
    let mut visitor = RenderingContractVisitor::new(file);

    visitor.check_path(&empty_path);

    assert!(visitor.into_violations().is_empty());
    Ok(())
}

fn has_rule(violations: &[crate::diagnostics::Violation], rule: &str) -> bool {
    violations.iter().any(|violation| violation.rule == rule)
}
