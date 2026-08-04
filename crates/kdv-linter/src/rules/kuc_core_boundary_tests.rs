use super::KucCoreBoundaryRule;
use crate::diagnostics::KdvLintError;
use crate::rules::test_helpers::FixtureWorkspace;
use crate::workspace::PortablePath;

#[test]
fn kuc_core_boundary_flags_viewer_media_semantic_leak() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    fixture.write_rust_file(
        "crates/katana-ui-core/src/render_model/host_action_media.rs",
        r#"
pub enum UiMediaControlTarget {
    Diagram,
}

pub const PREFIX: &str = "viewer.diagram.zoom-in";
"#,
    )?;

    let violations = KucCoreBoundaryRule::check(&fixture.workspace()?)?;

    assert!(violations.iter().any(|violation| {
        violation.rule == "kuc-core-boundary"
            && PortablePath::new(&violation.file).contains("crates/katana-ui-core/src")
    }));
    Ok(())
}

#[test]
fn kuc_core_boundary_flags_viewer_like_action_examples() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    fixture.write_rust_file(
        "crates/katana-ui-core/src/render_model/host_action_plan.rs",
        r#"
pub const UI_DIAGRAM_ZOOM_IN: &str = "ui.diagram.zoom-in";

pub fn test_action() -> &'static str {
    "diagram.zoom"
}
"#,
    )?;

    let violations = KucCoreBoundaryRule::check(&fixture.workspace()?)?;

    assert!(violations.iter().any(|violation| {
        violation.rule == "kuc-core-boundary"
            && PortablePath::new(&violation.file).contains("crates/katana-ui-core/src")
    }));
    Ok(())
}

#[test]
fn kuc_core_boundary_allows_kdv_core_owner() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    fixture.write_rust_file(
        "crates/katana-document-viewer/src/viewer/media_action.rs",
        r#"
pub struct ViewerMediaControlAction;
"#,
    )?;

    let violations = KucCoreBoundaryRule::check(&fixture.workspace()?)?;

    assert!(violations.is_empty());
    Ok(())
}

#[test]
fn kuc_core_boundary_flags_file_tree_that_builds_separate_row_ui() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    fixture.write_rust_file(
        "crates/katana-ui-core/src/molecule/structured/file_tree.rs",
        r#"
use crate::layout::Row;

pub struct FileTree;

impl FileTree {
    pub fn render() {
        let _ = Row::new();
    }
}
"#,
    )?;

    let violations = KucCoreBoundaryRule::check(&fixture.workspace()?)?;

    assert!(violations.iter().any(|violation| {
        violation.rule == "kuc-file-tree-facade"
            && PortablePath::new(&violation.file).contains("molecule/structured/file_tree.rs")
    }));
    Ok(())
}

#[test]
fn kuc_core_boundary_flags_file_tree_facade_even_when_file_moved() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    fixture.write_rust_file(
        "crates/katana-ui-core/src/molecule/structured/file_tree/facade.rs",
        r#"
use crate::layout::Row;

pub struct FileTree;

impl FileTree {
    pub fn render() {
        let _ = Row::new();
    }
}
"#,
    )?;

    let violations = KucCoreBoundaryRule::check(&fixture.workspace()?)?;

    assert!(violations.iter().any(|violation| {
        violation.rule == "kuc-file-tree-facade"
            && PortablePath::new(&violation.file)
                .contains("molecule/structured/file_tree/facade.rs")
    }));
    Ok(())
}

#[test]
fn kuc_core_boundary_allows_file_tree_tree_view_facade() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    fixture.write_rust_file(
        "crates/katana-ui-core/src/molecule/structured/file_tree.rs",
        r#"
use super::{TreeView, TreeViewAction, TreeViewHitTestInput};

pub fn render_tree() {
    let _ = TreeView::new("Files");
}

pub fn map_action(action: TreeViewAction) {
    let _ = TreeViewHitTestInput { pointer_y: 0, scroll_offset_y: 0 };
    match action {
        TreeViewAction::SelectNode { node_id: _ } => {}
        TreeViewAction::ToggleNode { node_id: _ } => {}
        _ => {}
    }
}
"#,
    )?;

    let violations = KucCoreBoundaryRule::check(&fixture.workspace()?)?;

    assert!(violations.is_empty());
    Ok(())
}
