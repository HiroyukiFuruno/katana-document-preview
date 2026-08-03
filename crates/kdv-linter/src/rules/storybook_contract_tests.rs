use super::StorybookContractRule;
use crate::diagnostics::KdvLintError;
use crate::rules::test_helpers::FixtureWorkspace;

#[test]
fn storybook_contract_flags_manual_hit_test_and_action_synthesis() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    write_storybook_violation_fixtures(&fixture)?;

    let violations = StorybookContractRule::check(&fixture.workspace()?)?;

    assert!(has_rule(&violations, "no_manual_tree_hit_test"));
    assert!(has_rule(&violations, "no_manual_media_hit_test"));
    assert!(has_rule(&violations, "no_storybook_action_synthesis"));
    assert!(has_rule(&violations, "no_manual_settings_action"));
    assert!(has_rule(&violations, "no_style_class_action_contract"));
    assert!(has_rule(&violations, "no_kuc_analytic_hit_target"));
    assert!(has_rule(
        &violations,
        "no_manual_interactive_preset_override"
    ));
    assert!(has_rule(&violations, "no_manual_window_presentation"));
    Ok(())
}

fn write_storybook_violation_fixtures(fixture: &FixtureWorkspace) -> Result<(), KdvLintError> {
    fixture.write_rust_file(
        "tools/kdv-storybook/src/sidebar_hit_walk.rs",
        SIDEBAR_HIT_SOURCE,
    )?;
    fixture.write_rust_file(
        "tools/kdv-storybook/src/mouse_media_hit.rs",
        MEDIA_HIT_SOURCE,
    )?;
    fixture.write_rust_file(
        "tools/kdv-storybook/src/settings_action.rs",
        SETTINGS_ACTION_SOURCE,
    )?;
    fixture.write_rust_file(
        "tools/kdv-storybook/src/mouse_media_target.rs",
        MEDIA_TARGET_SOURCE,
    )?;
    fixture.write_rust_file(
        "tools/kdv-storybook/src/window_loop.rs",
        MANUAL_WINDOW_PRESENTATION_SOURCE,
    )?;
    fixture.write_rust_file(
        "crates/katana-document-viewer/src/document_surface/node_factory_link.rs",
        INTERACTIVE_PRESET_OVERRIDE_SOURCE,
    )
}

const SIDEBAR_HIT_SOURCE: &str = r#"
use katana_ui_core::interaction::UiAction;

const TEXT_HEIGHT: f32 = 24.0;

fn selected() {
    let _ = UiAction::SetValue { target: Default::default(), value: String::new() };
    FileTree::hit_target_for_item_with_state(&[], state, "id", 0, 240);
    list.hit_target_for_field("dark", 240);
    list.hit_target_for_section("display", 240);
}
"#;

const MEDIA_HIT_SOURCE: &str = r#"
const BUTTON_WIDTH: f32 = 96.0;
const BUTTON_HEIGHT: f32 = 20.0;
fn parse_state_id() {}
const IMAGE_ACTION_PREFIX: &str = "viewer.image.";
const DIAGRAM_ACTION_PREFIX: &str = "viewer.diagram.";
const CODE_ACTION_PREFIX: &str = "viewer.code.";
"#;

const SETTINGS_ACTION_SOURCE: &str = r#"
fn update() {
    let _ = "settings-field:";
    let _ = "kdv-task-state:";
    self.dark = !self.dark;
    let _ = mode_from_label("slideshow");
    apply_interaction_field();
}
"#;

const MEDIA_TARGET_SOURCE: &str = r#"
use katana_ui_core::render_model::{UiMediaControlAction, UiMediaControlTarget};

fn update(action: UiMediaControlAction) {
    let _ = UiMediaControlTarget::Diagram;
    let _ = action.media_control_action();
}
"#;

const MANUAL_WINDOW_PRESENTATION_SOURCE: &str = r#"
use katana_ui_core_storybook::{Canvas, present_frame};

fn should_present_physical_frame_directly(frame: &Canvas) -> bool {
    frame.scale_factor() > 1.0
}

fn presented_frame(frame: &Canvas) -> Canvas {
    present_frame(frame, 100, 100, 0)
}
"#;

const INTERACTIVE_PRESET_OVERRIDE_SOURCE: &str = r#"
use katana_ui_core::render_model::UiCursor;

fn link(node: UiNode) -> UiNode {
    node.cursor(UiCursor::Pointer)
}
"#;

#[test]
fn storybook_contract_ignores_window_presentation_terms_in_tests() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    fixture.write_rust_file(
        "tools/kdv-storybook/src/window_loop_tests.rs",
        MANUAL_WINDOW_PRESENTATION_SOURCE,
    )?;

    let violations = StorybookContractRule::check(&fixture.workspace()?)?;

    assert!(!has_rule(&violations, "no_manual_window_presentation"));
    Ok(())
}

#[test]
fn storybook_contract_flags_diagram_style_class_contract() -> Result<(), KdvLintError> {
    let fixture = FixtureWorkspace::new().with_default_manifests()?;
    fixture.write_rust_file(
        "crates/katana-document-viewer/src/document_surface/node_factory_media_diagram_controls.rs",
        r#"
fn control(node: UiNode) -> UiNode {
    node
        .style_class("kdv-diagram-button")
        .style_class("kdv-diagram-toolbar")
        .style_class("kdv-image-control")
}
"#,
    )?;

    let violations = StorybookContractRule::check(&fixture.workspace()?)?;

    let media_style_violations = violations
        .iter()
        .filter(|violation| violation.rule == "no_style_class_action_contract")
        .count();
    assert_eq!(3, media_style_violations);
    Ok(())
}

fn has_rule(violations: &[crate::diagnostics::Violation], rule: &str) -> bool {
    violations.iter().any(|violation| violation.rule == rule)
}
