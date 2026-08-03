use super::{
    FrameLoopChanges, StorybookFrameCache, StorybookWindow, default_storybook_scale_factor,
    interactive_window_size, render_size_for_window,
};
use crate::args::StorybookArgs;
use crate::canvas::Canvas;
use crate::catalog::{FixtureCatalog, StorybookFixture};
use crate::layout::{preview_content_width, preview_viewport_height};
use crate::media_host_action::StorybookMediaHostAction;
use crate::preview::PreviewBuilder;
use crate::window_host_event::StorybookHostEvent;
use katana_document_viewer::{DiagramViewportState, ViewerMediaControlKind};
use katana_ui_core::render_model::UiHostActionPlan;
use std::path::PathBuf;
use std::time::Duration;

const TEST_WINDOW_WIDTH: usize = 1280;
const TEST_WINDOW_HEIGHT: usize = 900;

#[test]
fn scaled_logical_frame_is_presented_at_window_buffer_size() {
    let frame = StorybookFrameCache::new(Canvas::new_scaled(1440, 920, 2.0, 0x111111));

    let presented = StorybookWindow::presented_frame(&frame, 1440, 920);

    assert_eq!(1440, presented.width());
    assert_eq!(920, presented.height());
    assert_eq!(1440, presented.logical_width());
    assert_eq!(920, presented.logical_height());
}

#[test]
fn interactive_window_default_scale_keeps_sidebar_at_logical_size() {
    let expected = if cfg!(target_os = "macos") { 2.0 } else { 1.0 };
    assert_eq!(expected, default_storybook_scale_factor());
}

#[test]
fn narrow_interactive_window_keeps_render_buffer_matched_to_window() {
    let (width, height) = render_size_for_window(816, 518);

    assert_eq!((816, 518), (width, height));
    assert_eq!(
        284,
        preview_content_width(width),
        "host buffer must not be silently upscaled and then stretched back into a smaller OS window"
    );
    assert_eq!(410, preview_viewport_height(height));
}

#[test]
fn narrow_interactive_window_opens_at_readable_size() {
    let (width, height) = interactive_window_size(816, 518);

    assert_eq!((816, 518), (width, height));
}

#[test]
fn resized_window_uses_fitted_presented_frame() {
    let frame = StorybookFrameCache::new(Canvas::new_scaled(1440, 920, 2.0, 0x111111));

    let presented = StorybookWindow::presented_frame(&frame, 1280, 900);

    assert_eq!(1280, presented.width());
    assert_eq!(900, presented.height());
    assert_eq!(1280, presented.logical_width());
    assert_eq!(900, presented.logical_height());
}

#[test]
fn scaled_logical_frame_buffer_is_presented_through_kuc_at_window_buffer_size() {
    let frame = StorybookFrameCache::new(Canvas::new_scaled(1440, 920, 2.0, 0x111111));

    let presented = StorybookWindow::presented_frame_buffer(&frame, 1440, 920);

    assert_ne!(frame.pixels().as_ptr(), presented.pixels().as_ptr());
    assert_eq!(1440, presented.width());
    assert_eq!(920, presented.height());
    assert_eq!(1440, presented.logical_width());
    assert_eq!(920, presented.logical_height());
}

#[test]
fn resized_window_still_uses_owned_fitted_presented_frame() {
    let frame = StorybookFrameCache::new(Canvas::new_scaled(1440, 920, 2.0, 0x111111));

    let presented = StorybookWindow::presented_frame_buffer(&frame, 1280, 900);

    assert_ne!(frame.pixels().as_ptr(), presented.pixels().as_ptr());
    assert_eq!(1280, presented.width());
    assert_eq!(900, presented.height());
}

#[test]
fn window_loop_delegates_window_presentation_to_kuc() {
    let source = include_str!("window_loop.rs");

    assert!(source.contains("present_frame_for_window"));
    assert!(!source.contains("fn should_present_physical_frame_directly"));
    assert!(!source.contains("fn should_present_cached_frame_directly"));
    assert!(!source.contains("scale_factor() > 1.0"));
    assert!(!source.contains("present_frame(frame.canvas()"));
}

#[test]
fn diagram_screenshot_current_alias_keeps_initial_frame_not_mutated_control_state() {
    let source = include_str!("window_loop.rs").replace("\r\n", "\n");

    assert!(source.contains("let base = self.presented_frame_for_current_window(width, height)?;"));
    assert!(source.contains("let mut last = base.clone();"));
    assert!(
        source.contains("write_canvas_png_with_current_alias(\n            &self.args.screenshot_output,\n            &base,")
    );
    assert!(
        !source.contains("write_canvas_png_with_current_alias(\n            &self.args.screenshot_output,\n            &last,")
    );
}

#[test]
fn diagram_screenshot_fullscreen_smoke_uses_window_viewport_height()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = storybook("katana/sample_diagrams.md");
    storybook
        .update_full_height_diagram_scene_for_window_smoke(TEST_WINDOW_WIDTH, TEST_WINDOW_HEIGHT)?;
    let node_id =
        first_media_action_node_id(&storybook, ViewerMediaControlKind::Diagram, "fullscreen")?;
    storybook.diagram_viewports.insert(
        node_id,
        DiagramViewportState {
            fullscreen_open: true,
            ..DiagramViewportState::default()
        },
    );

    storybook
        .update_full_height_diagram_scene_for_window_smoke(TEST_WINDOW_WIDTH, TEST_WINDOW_HEIGHT)?;
    let frame = storybook.render_canvas(TEST_WINDOW_WIDTH, TEST_WINDOW_HEIGHT);
    let visible = non_background_pixels(&frame);

    assert!(
        visible > 800,
        "fullscreen screenshot smoke must keep the active diagram and controls inside the window viewport: visible={visible}"
    );
    Ok(())
}

#[test]
fn diagram_fullscreen_host_event_is_overlay_only_and_does_not_native_fullscreen() {
    let mut storybook = storybook("katana/sample_diagrams.md");
    storybook
        .host_events
        .push(StorybookHostEvent::DiagramFullscreen {
            node_id: "diagram-1".to_string(),
            open: true,
        });

    assert!(storybook.drain_diagram_fullscreen_events());
    assert!(
        storybook.host_events.is_empty(),
        "diagram fullscreen host event is a typed overlay notification and must be consumed without OS window side effects"
    );

    storybook
        .host_events
        .push(StorybookHostEvent::DiagramFullscreen {
            node_id: "diagram-1".to_string(),
            open: false,
        });

    assert!(storybook.drain_diagram_fullscreen_events());
    assert!(storybook.host_events.is_empty());
}

#[test]
fn interaction_loop_delay_accounts_for_frame_elapsed_time() {
    let delay =
        FrameLoopChanges::scroll_changed().delay_after_frame(false, Duration::from_millis(5));

    assert_eq!(Duration::from_millis(3), delay);
}

#[test]
fn scene_loop_delay_accounts_for_frame_elapsed_time() {
    let delay =
        FrameLoopChanges::scene_changed().delay_after_frame(false, Duration::from_millis(11));

    assert_eq!(Duration::from_millis(5), delay);
}

#[test]
fn interaction_loop_does_not_sleep_after_over_budget_frame() {
    let delay =
        FrameLoopChanges::scroll_changed().delay_after_frame(false, Duration::from_millis(9));

    assert_eq!(Duration::ZERO, delay);
}

#[test]
fn scroll_loop_defers_asset_update() {
    assert!(FrameLoopChanges::scroll_changed().should_defer_asset_update());
    assert!(FrameLoopChanges::input_changed().should_defer_asset_update());
    assert!(!FrameLoopChanges::hover_changed().should_defer_asset_update());
    assert!(!FrameLoopChanges::scene_changed().should_defer_asset_update());
}

#[test]
fn scroll_loop_pauses_loading_animation() {
    assert!(FrameLoopChanges::scroll_changed().should_pause_loading_animation());
    assert!(FrameLoopChanges::input_changed().should_pause_loading_animation());
    assert!(!FrameLoopChanges::asset_changed().should_pause_loading_animation());
    assert!(!FrameLoopChanges::idle().should_pause_loading_animation());
}

#[test]
fn preview_scroll_with_pointer_motion_keeps_delta_redraw_path() {
    assert!(
        FrameLoopChanges::scroll_and_input_changed().can_redraw_preview_only(false),
        "trackpad scroll normally arrives with pointer/input state and must not force full preview redraw"
    );
}

#[test]
fn pure_input_change_still_requires_full_redraw() {
    assert!(!FrameLoopChanges::input_changed().can_redraw_preview_only(false));
}

fn storybook(label: &str) -> StorybookWindow {
    StorybookWindow::new(
        StorybookArgs::default(),
        FixtureCatalog {
            fixtures: vec![StorybookFixture {
                label: label.to_string(),
                path: fixture_path(&format!("assets/fixtures/{label}")),
            }],
        },
        PreviewBuilder::default(),
    )
}

fn first_media_action_node_id(
    storybook: &StorybookWindow,
    kind: ViewerMediaControlKind,
    command: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let scene = storybook.scene.as_ref().ok_or("scene missing")?;
    UiHostActionPlan::collect_from_tree(&scene.tree)
        .into_iter()
        .filter_map(|plan| {
            let action =
                StorybookMediaHostAction::from_host_action_plan(&plan)?.into_viewer_action();
            (action.kind == kind && action.command == command).then_some(action.node_id)
        })
        .find(|node_id| scene.target_for_node_id(node_id).is_some())
        .ok_or_else(|| std::io::Error::other("media action node id missing").into())
}

fn non_background_pixels(canvas: &Canvas) -> usize {
    let background = canvas.pixels().first().copied().unwrap_or_default();
    canvas
        .pixels()
        .iter()
        .filter(|pixel| **pixel != background)
        .count()
}

fn fixture_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../{path}"))
}
