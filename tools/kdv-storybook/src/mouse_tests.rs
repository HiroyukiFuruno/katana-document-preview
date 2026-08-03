use super::mouse_host_action::StorybookHostActionHits;
use super::mouse_test_support::{
    WINDOW_HEIGHT, WINDOW_WIDTH, direct_image_controls_scene, pointer_for_link,
    pointer_for_media_action, pointer_for_task, sample_basic_scene, sample_diagram_controls_scene,
};
use super::{StorybookMouse, StorybookMouseButton, StorybookPointer};
use crate::catalog::StorybookFixture;
use crate::layout::{StorybookPreviewArea, preview_content_height, preview_content_width};
use crate::preview::PreviewBuilder;
use crate::preview_build_request::{PreviewBuildAssetMode, PreviewBuildRequest};
use katana_document_viewer::{
    DiagramControlCommand, DocumentViewerCommand, ImageControlAction, SlideshowCommand,
    ViewerCommand, ViewerInteractionConfig, ViewerSlideshowControlAction, ViewerTaskState,
};
use katana_document_viewer::{ViewerMode, ViewerSearchState, ViewerViewport};
use katana_ui_core::render_model::{UiTaskMarker, UiTextSpanAction};
use katana_ui_core_storybook::UiTreeHostActionHit;
use std::path::PathBuf;

#[test]
fn document_command_uses_neutral_storybook_label() {
    assert_eq!(
        "document",
        StorybookMouse::command_label(&ViewerCommand::Document(DocumentViewerCommand::Previous))
    );
}

#[test]
fn mouse_left_click_on_list_markdown_link_returns_real_link_command()
-> Result<(), Box<dyn std::error::Error>> {
    let scene = sample_basic_scene()?;
    let hit = pointer_for_link(&scene, "Normal link", "https://github.com")?;

    let command = StorybookMouse::command_for_click(
        &scene,
        hit.scroll_y,
        hit.pointer,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    )
    .ok_or_else(|| std::io::Error::other("missing link command"))?;

    let ViewerCommand::Link(link) = command else {
        return Err(std::io::Error::other("expected link command").into());
    };
    assert_eq!("https://github.com", link.uri);
    assert_ne!("storybook-interaction-node", link.target.node_id.0);
    assert!(link.target.source.raw.text.contains("Normal link"));
    Ok(())
}

#[test]
fn mouse_left_click_on_footnote_reference_returns_internal_scroll_command()
-> Result<(), Box<dyn std::error::Error>> {
    let scene = sample_basic_scene()?;
    let hit = pointer_for_link(&scene, "[1]", "#fn-1")?;

    let command = StorybookMouse::command_for_click(
        &scene,
        hit.scroll_y,
        hit.pointer,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    )
    .ok_or_else(|| std::io::Error::other("missing footnote command"))?;

    let ViewerCommand::ScrollToHeading(scroll) = command else {
        return Err(std::io::Error::other("expected internal scroll command").into());
    };
    assert!(
        scroll
            .target
            .source
            .raw
            .text
            .contains("First footnote content")
    );
    Ok(())
}

#[test]
fn mouse_left_click_on_footnote_backlink_returns_internal_scroll_command()
-> Result<(), Box<dyn std::error::Error>> {
    let scene = sample_basic_scene()?;
    let hit = pointer_for_link(&scene, "↩", "#fnref-1")?;

    let command = StorybookMouse::command_for_click(
        &scene,
        hit.scroll_y,
        hit.pointer,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    )
    .ok_or_else(|| std::io::Error::other("missing footnote backlink command"))?;

    let ViewerCommand::ScrollToHeading(scroll) = command else {
        return Err(std::io::Error::other("expected internal scroll command").into());
    };
    assert!(
        scroll.target.source.raw.text.contains("[^1]"),
        "backlink must scroll to the footnote reference source: {:?}",
        scroll.target.source.raw.text
    );
    Ok(())
}

#[test]
fn mouse_right_click_does_not_open_link() -> Result<(), Box<dyn std::error::Error>> {
    let scene = sample_basic_scene()?;
    let mut hit = pointer_for_link(&scene, "Normal link", "https://github.com")?;
    hit.pointer.button = StorybookMouseButton::Right;

    let command = StorybookMouse::command_for_click(
        &scene,
        hit.scroll_y,
        hit.pointer,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    );

    assert!(command.is_none());
    Ok(())
}

#[test]
fn mouse_click_uses_external_scroll_for_scroll_independent_scene()
-> Result<(), Box<dyn std::error::Error>> {
    let scene = build_scroll_independent_scene("katana/sample_basic.md")?;
    assert_eq!(0, scene.tree.root().props().scroll_area.offset_y);
    let (hit, scroll_y) = scrolled_visible_link_hit(&scene)?;
    assert!(scroll_y > 0.0);
    let pointer = pointer_for_visible_hit(&hit, scroll_y);

    let command =
        StorybookMouse::command_for_click(&scene, scroll_y, pointer, WINDOW_WIDTH, WINDOW_HEIGHT)
            .ok_or_else(|| std::io::Error::other("missing link command with external scroll"))?;

    let ViewerCommand::Link(link) = command else {
        return Err(std::io::Error::other("expected link command with external scroll").into());
    };
    assert!(!link.uri.trim().is_empty());
    Ok(())
}

#[test]
fn mouse_left_click_on_task_checkbox_toggles_task() -> Result<(), Box<dyn std::error::Error>> {
    let scene = sample_basic_scene()?;
    let hit = pointer_for_task(&scene, "[ ]")?;

    let command = StorybookMouse::command_for_click(
        &scene,
        hit.scroll_y,
        hit.pointer,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    )
    .ok_or_else(|| std::io::Error::other("missing task command"))?;

    let ViewerCommand::Task(task) = command else {
        return Err(std::io::Error::other("expected task command").into());
    };
    assert_eq!(ViewerTaskState::Done, task.state);
    let task_target = task
        .task_target
        .as_ref()
        .ok_or_else(|| std::io::Error::other("task command missing typed target"))?;
    assert_eq!(task.target.node_id, task_target.node_id);
    assert!(task_target.state_id.starts_with("ui-task-state:"));
    assert_ne!("storybook-interaction-node", task.target.node_id.0);
    assert_ne!(task_target.state_id, task.target.artifact_id.0);
    assert!(task.target.source.raw.text.contains("[ ]"));
    Ok(())
}

#[test]
fn mouse_left_click_on_task_row_body_toggles_task() -> Result<(), Box<dyn std::error::Error>> {
    let scene = sample_basic_scene()?;
    let hit = task_row_body_hit(&scene, "[ ]")?;
    let scroll_y = (hit.rect.y as f32 - 120.0).max(0.0);
    let (_, center_y) = hit.center_point();
    let area = StorybookPreviewArea::for_window(WINDOW_WIDTH, WINDOW_HEIGHT, scroll_y);
    let (x, y) = area
        .canvas_point_for_document_point(hit.rect.x as f32 + hit.rect.width as f32 - 8.0, center_y);
    let pointer = StorybookPointer::new(x, y, StorybookMouseButton::Left);

    let command =
        StorybookMouse::command_for_click(&scene, scroll_y, pointer, WINDOW_WIDTH, WINDOW_HEIGHT)
            .ok_or_else(|| std::io::Error::other("missing task row command"))?;

    let ViewerCommand::Task(task) = command else {
        return Err(std::io::Error::other("expected task command from row body").into());
    };
    assert_eq!(ViewerTaskState::Done, task.state);
    assert!(task.target.source.raw.text.contains("[ ]"));
    Ok(())
}

#[test]
fn mouse_task_context_menu_selection_sets_task_state() -> Result<(), Box<dyn std::error::Error>> {
    let scene = sample_basic_scene()?;
    let mut hit = pointer_for_task(&scene, "[ ]")?;
    hit.pointer.button = StorybookMouseButton::Right;
    let menu = super::task_context_menu::StorybookTaskContextMenu::open(
        &scene,
        hit.scroll_y,
        hit.pointer,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    )
    .ok_or_else(|| std::io::Error::other("missing task context menu"))?;
    let pointer = menu
        .test_pointer_for_marker("[-]")
        .ok_or_else(|| std::io::Error::other("missing blocked item"))?;
    let command = menu
        .command_for_pointer(pointer)
        .ok_or_else(|| std::io::Error::other("missing context command"))?;

    let ViewerCommand::Task(task) = command else {
        return Err(std::io::Error::other("expected task command").into());
    };
    assert_eq!(ViewerTaskState::Blocked, task.state);
    assert_ne!("storybook-interaction-node", task.target.node_id.0);
    Ok(())
}

#[test]
fn mouse_left_click_on_image_control_returns_image_command()
-> Result<(), Box<dyn std::error::Error>> {
    let scene = direct_image_controls_scene()?;
    let hit = pointer_for_media_action(&scene, "open")?;

    let command = StorybookMouse::command_for_click(
        &scene,
        hit.scroll_y,
        hit.pointer,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    )
    .ok_or_else(|| std::io::Error::other("missing image command"))?;

    let ViewerCommand::Image(image) = command else {
        return Err(std::io::Error::other("expected image command").into());
    };
    assert_eq!(ImageControlAction::Open, image.action);
    assert_ne!("storybook-interaction-node", image.target.node_id.0);
    Ok(())
}

#[test]
fn mouse_left_click_on_diagram_control_returns_diagram_command()
-> Result<(), Box<dyn std::error::Error>> {
    let scene = sample_diagram_controls_scene()?;
    let hit = pointer_for_media_action(&scene, "fullscreen")?;

    let command = StorybookMouse::command_for_click(
        &scene,
        hit.scroll_y,
        hit.pointer,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    )
    .ok_or_else(|| std::io::Error::other("missing diagram command"))?;

    let ViewerCommand::Diagram(DiagramControlCommand::FullscreenOpen(target)) = command else {
        return Err(std::io::Error::other("expected diagram fullscreen command").into());
    };
    assert_ne!("storybook-interaction-node", target.node_id.0);
    Ok(())
}

#[test]
fn mouse_left_click_on_every_diagram_control_returns_command()
-> Result<(), Box<dyn std::error::Error>> {
    let scene = sample_diagram_controls_scene()?;

    for action in diagram_control_actions() {
        let hit = pointer_for_media_action(&scene, action)?;
        let command = StorybookMouse::command_for_click(
            &scene,
            hit.scroll_y,
            hit.pointer,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        )
        .ok_or_else(|| std::io::Error::other(format!("missing diagram command: {action}")))?;
        assert_diagram_control_command(action, command)?;
    }
    Ok(())
}

#[test]
fn mouse_left_click_on_slideshow_next_control_returns_slideshow_command()
-> Result<(), Box<dyn std::error::Error>> {
    let scene =
        build_scroll_independent_scene_with_mode("katana/sample.md", ViewerMode::Slideshow)?;
    let action_id = ViewerSlideshowControlAction::NextPage.host_action_id();
    let hit = StorybookHostActionHits::hits(&scene, WINDOW_WIDTH)
        .into_iter()
        .find(|hit| hit.action.action_id == action_id)
        .ok_or_else(|| std::io::Error::other("missing slideshow next host action"))?;
    let pointer = pointer_for_host_action_hit(&hit, 0.0);

    let command =
        StorybookMouse::command_for_click(&scene, 0.0, pointer, WINDOW_WIDTH, WINDOW_HEIGHT)
            .ok_or_else(|| std::io::Error::other("missing slideshow command"))?;

    assert_eq!(
        ViewerCommand::Slideshow(SlideshowCommand::NextPage),
        command
    );
    Ok(())
}

fn diagram_control_actions() -> [&'static str; 9] {
    [
        "fullscreen",
        "pan-up",
        "pan-down",
        "pan-left",
        "pan-right",
        "zoom-in",
        "zoom-out",
        "reset-view",
        "trackpad-help",
    ]
}

fn task_row_body_hit(
    scene: &crate::preview::PreviewScene,
    marker: &str,
) -> Result<UiTreeHostActionHit, Box<dyn std::error::Error>> {
    let expected_marker = UiTaskMarker::from_marker(marker)
        .ok_or_else(|| std::io::Error::other("unsupported task marker"))?;
    StorybookHostActionHits::hits(scene, WINDOW_WIDTH)
        .into_iter()
        .find(|hit| {
            hit.rect.width >= 80
                && hit
                    .action
                    .task_control_action_from_root(scene.tree.root())
                    .is_some_and(|action| action.current_marker == expected_marker)
        })
        .ok_or_else(|| std::io::Error::other("missing task row body hit").into())
}

fn assert_diagram_control_command(
    action: &str,
    command: ViewerCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    let ViewerCommand::Diagram(command) = command else {
        return Err(format!("expected diagram command: {action}").into());
    };
    match (action, &command) {
        ("fullscreen", DiagramControlCommand::FullscreenOpen(_)) => {}
        ("pan-up", DiagramControlCommand::Pan(command)) => {
            assert_eq!(
                katana_document_viewer::DiagramPanSource::ButtonUp,
                command.source
            );
        }
        ("pan-down", DiagramControlCommand::Pan(command)) => {
            assert_eq!(
                katana_document_viewer::DiagramPanSource::ButtonDown,
                command.source
            );
        }
        ("pan-left", DiagramControlCommand::Pan(command)) => {
            assert_eq!(
                katana_document_viewer::DiagramPanSource::ButtonLeft,
                command.source
            );
        }
        ("pan-right", DiagramControlCommand::Pan(command)) => {
            assert_eq!(
                katana_document_viewer::DiagramPanSource::ButtonRight,
                command.source
            );
        }
        ("zoom-in", DiagramControlCommand::Zoom(command)) => {
            assert_eq!(
                katana_document_viewer::DiagramZoomSource::ButtonIn,
                command.source
            );
        }
        ("zoom-out", DiagramControlCommand::Zoom(command)) => {
            assert_eq!(
                katana_document_viewer::DiagramZoomSource::ButtonOut,
                command.source
            );
        }
        ("reset-view", DiagramControlCommand::Reset(_)) => {}
        ("trackpad-help", DiagramControlCommand::TrackpadHelp(_)) => {}
        _ => return Err(format!("unexpected diagram command for {action}: {command:?}").into()),
    }
    assert_ne!(
        "storybook-interaction-node",
        diagram_command_target_id(&command)
    );
    Ok(())
}

fn diagram_command_target_id(command: &DiagramControlCommand) -> &str {
    match command {
        DiagramControlCommand::FullscreenOpen(target)
        | DiagramControlCommand::FullscreenClose(target)
        | DiagramControlCommand::Reset(target)
        | DiagramControlCommand::TrackpadHelp(target) => target.node_id.0.as_str(),
        DiagramControlCommand::Pan(command) => command.target.node_id.0.as_str(),
        DiagramControlCommand::Zoom(command) => command.target.node_id.0.as_str(),
    }
}

fn build_scroll_independent_scene(
    path: &str,
) -> Result<crate::preview::PreviewScene, Box<dyn std::error::Error>> {
    build_scroll_independent_scene_with_mode(path, ViewerMode::Document)
}

fn build_scroll_independent_scene_with_mode(
    path: &str,
    mode: ViewerMode,
) -> Result<crate::preview::PreviewScene, Box<dyn std::error::Error>> {
    let fixture = StorybookFixture {
        label: path.to_string(),
        path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../assets/fixtures/{path}")),
    };
    PreviewBuilder::default().build_scene(PreviewBuildRequest {
        fixture: &fixture,
        viewport: ViewerViewport {
            width: preview_content_width(WINDOW_WIDTH) as f32,
            height: preview_content_height(WINDOW_HEIGHT) as f32,
        },
        dark: true,
        theme: None,
        interaction: ViewerInteractionConfig::default(),
        mode,
        typography: Default::default(),
        search: ViewerSearchState::default(),
        diagram_viewports: Default::default(),
        image_viewports: Default::default(),
        task_state_overrides: Default::default(),
        accordion_open_overrides: Default::default(),
        copied_code_node_ids: Default::default(),
        asset_mode: PreviewBuildAssetMode::VisibleAndNearViewport,
        attach_surface: false,
        export_surface: false,
    })
}

fn scrolled_visible_link_hit(
    scene: &crate::preview::PreviewScene,
) -> Result<(UiTreeHostActionHit, f32), Box<dyn std::error::Error>> {
    let viewport_height = preview_content_height(WINDOW_HEIGHT) as f32;
    StorybookHostActionHits::hits(scene, WINDOW_WIDTH)
        .into_iter()
        .filter(|hit| hit.action.text_span_action().is_some())
        .filter_map(|hit| {
            let (_, center_y) = hit.center_point();
            let root_offset = (center_y - viewport_height / 2.0).max(1.0);
            link_hit_is_visible(&hit, root_offset).then_some((hit, root_offset))
        })
        .next()
        .ok_or_else(|| std::io::Error::other("missing visible link host action").into())
}

fn link_hit_is_visible(hit: &UiTreeHostActionHit, root_offset: f32) -> bool {
    let Some(UiTextSpanAction::OpenLink { .. }) = hit.action.text_span_action() else {
        return false;
    };
    let (_, center_y) = hit.center_point();
    let center_y = center_y - root_offset;
    center_y > 0.0 && center_y < preview_content_height(WINDOW_HEIGHT) as f32
}

fn pointer_for_visible_hit(hit: &UiTreeHostActionHit, root_offset: f32) -> StorybookPointer {
    pointer_for_host_action_hit(hit, root_offset)
}

fn pointer_for_host_action_hit(hit: &UiTreeHostActionHit, root_offset: f32) -> StorybookPointer {
    let (center_x, center_y) = hit.center_point();
    let area = StorybookPreviewArea::for_window(WINDOW_WIDTH, WINDOW_HEIGHT, root_offset);
    let (x, y) = area.canvas_point_for_document_point(center_x, center_y);
    StorybookPointer::new(x, y, StorybookMouseButton::Left)
}
