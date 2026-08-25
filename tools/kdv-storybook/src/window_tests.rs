use super::window_loop::{FrameLoopChanges, should_update_document_hover_for_loop};
use super::{
    MAX_LOADED_ASSET_SCENES, StorybookDocumentInteractionSurfaceCache, StorybookFrameCache,
    StorybookLoadedAssetScene, StorybookWindow,
};
use crate::args::StorybookArgs;
use crate::canvas::Canvas;
use crate::catalog::{FixtureCatalog, StorybookFixture};
use crate::frame::{FrameRenderRequest, StorybookFrameRenderer};
use crate::frame_pixel_guard::StorybookFramePixelGuard;
use crate::layout::{
    SIDEBAR_WIDTH, StorybookPreviewArea, preview_content_height, preview_content_width,
    sidebar_content_height, sidebar_content_width,
};
use crate::mouse::{StorybookHostActionHits, StorybookMouseButton, StorybookPointer};
use crate::preview::PreviewBuilder;
use crate::settings_action::StorybookSettingsField;
use crate::sidebar::StorybookSidebar;
use crate::sidebar_hit::SidebarHitResult;
use crate::sidebar_test_support::{
    StorybookFileTreeItemPointRequest, StorybookSettingsFieldHitRequest,
    StorybookSettingsSectionHitRequest,
};
use crate::smoke_assertions::StorybookSmokeAssertions;
use crate::window_asset_job::{StorybookAssetJob, StorybookAssetJobKey, StorybookAssetJobKeyInput};
use katana_document_viewer::{
    DiagramRenderEngine, DiagramRenderRequest, RenderedDiagram, ViewerMode,
    ViewerSlideshowControlAction, ViewerViewport,
};
use katana_ui_core::molecule::{FileTreeAction, SettingsListAction};
use katana_ui_core::render_model::{UiCursor, UiNode, UiNodeId};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[test]
fn frame_size_update_detects_window_resize_after_first_frame() {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        FixtureCatalog {
            fixtures: Vec::new(),
        },
        PreviewBuilder::default(),
    );

    assert!(!storybook.update_frame_size(1280, 900));
    assert!(!storybook.update_frame_size(1280, 900));
    assert!(storybook.update_frame_size(1440, 900));
    assert!(storybook.update_frame_size(1440, 960));
}

#[test]
fn frame_size_update_discards_cached_canvases_on_resize() {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_basic.md"),
        PreviewBuilder::default(),
    );
    storybook.update_frame_size(1280, 900);
    storybook.frame_cache = Some(StorybookFrameCache::new(Canvas::new_scaled(
        1280, 900, 2.0, 0,
    )));
    let _ = storybook.render_canvas(1280, 900);

    assert!(storybook.frame_cache.is_some());
    assert!(!storybook.sidebar_frame_cache.is_empty());
    assert!(storybook.update_frame_size(1440, 900));
    assert!(storybook.frame_cache.is_none());
    assert!(storybook.sidebar_frame_cache.is_empty());
}

#[test]
fn frame_loop_redraws_only_for_visible_state_changes() {
    assert!(!FrameLoopChanges::idle().needs_redraw());
    assert!(FrameLoopChanges::scene_changed().needs_redraw());
    assert!(FrameLoopChanges::input_changed().needs_redraw());
    assert!(FrameLoopChanges::scroll_changed().needs_redraw());
    assert!(FrameLoopChanges::hover_changed().needs_redraw());
    assert!(FrameLoopChanges::asset_changed().needs_redraw());
}

#[test]
fn frame_loop_defers_hover_recalculation_while_preview_scrolls() {
    assert!(!should_update_document_hover_for_loop(false, true, true));
    assert!(!should_update_document_hover_for_loop(true, false, true));
    assert!(should_update_document_hover_for_loop(false, true, false));
    assert!(should_update_document_hover_for_loop(true, false, false));
}

#[test]
fn scene_refresh_is_not_needed_for_redraw_only_input() -> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_basic.md"),
        PreviewBuilder::default(),
    );
    storybook.update_scene(1000, 900)?;

    assert!(!storybook.scene_refresh_needed(false));
    assert!(storybook.scene_refresh_needed(true));
    Ok(())
}

#[test]
fn storybook_window_selection_copy_includes_viewer_body_text()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_basic.md"),
        PreviewBuilder::default(),
    );
    let width = 1000;
    let height = 900;
    storybook.update_frame_size(width, height);
    storybook.update_scene(width, height)?;
    storybook.set_text_selection_for_tests((0, 0), (width, height));

    let payload = storybook
        .selected_text_payload_for_tests(width, height)
        .ok_or("selection payload missing")?;

    assert!(
        payload.contains("KatanA Rendering")
            || payload.contains("Basic Markdown")
            || payload.contains("core Markdown rendering"),
        "selection payload must include viewer body text: {payload}"
    );
    Ok(())
}

#[test]
fn storybook_window_selection_copy_includes_loaded_artifact_text()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_markdown_fixture(
        "artifact-selection.md",
        "```mermaid\ngraph TD\n  A[Start] --> B[End]\n```\n",
    )?;
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        FixtureCatalog {
            fixtures: vec![fixture],
        },
        PreviewBuilder::with_diagram_engine(Arc::new(TextDiagramEngine)),
    );
    let width = 1000;
    let height = 900;
    storybook.update_frame_size(width, height);
    storybook.update_scene_loaded(width, height)?;
    storybook.set_text_selection_for_tests((0, 0), (width, height));

    let payload = storybook
        .selected_text_payload_for_tests(width, height)
        .ok_or("selection payload missing")?;

    assert!(
        payload.contains("Artifact Needle"),
        "selection payload must include loaded artifact text extraction: {payload}"
    );
    Ok(())
}

#[test]
fn storybook_window_selection_copy_includes_settings_text() -> Result<(), Box<dyn std::error::Error>>
{
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_basic.md"),
        PreviewBuilder::default(),
    );
    let width = 1000;
    let height = 900;
    storybook.update_frame_size(width, height);
    storybook.update_scene(width, height)?;
    storybook.set_text_selection_for_tests((0, 0), (width, height));

    let payload = storybook
        .selected_text_payload_for_tests(width, height)
        .ok_or("selection payload missing")?;

    assert!(
        payload.contains("Hover highlight") && payload.contains("Diagram controls"),
        "selection payload must include settings text: {payload}"
    );
    Ok(())
}

#[test]
fn storybook_window_selection_copy_includes_file_tree_text()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_basic.md"),
        PreviewBuilder::default(),
    );
    let width = 1000;
    let height = 900;
    storybook.update_frame_size(width, height);
    storybook.update_scene(width, height)?;
    storybook.set_text_selection_for_tests((0, 0), (width, height));

    let payload = storybook
        .selected_text_payload_for_tests(width, height)
        .ok_or("selection payload missing")?;

    assert!(
        payload.contains("katana/sample_basic.md"),
        "selection payload must include file tree text: {payload}"
    );
    Ok(())
}

#[test]
fn storybook_window_selection_copy_includes_header_and_status_text()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_basic.md"),
        PreviewBuilder::default(),
    );
    let width = 1000;
    let height = 900;
    storybook.update_frame_size(width, height);
    storybook.update_scene(width, height)?;
    storybook.set_text_selection_for_tests((0, 0), (width, height));

    let payload = storybook
        .selected_text_payload_for_tests(width, height)
        .ok_or("selection payload missing")?;

    assert!(
        payload.contains("katana/sample_basic.md") && payload.contains("command=none"),
        "selection payload must include header and status text: {payload}"
    );
    Ok(())
}

#[test]
fn storybook_window_full_selection_copy_covers_visible_shell_text()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_basic.md"),
        PreviewBuilder::default(),
    );
    let width = 1000;
    let height = 900;
    storybook.update_frame_size(width, height);
    storybook.update_scene(width, height)?;
    storybook.set_text_selection_for_tests((0, 0), (width, height));

    let payload = storybook
        .selected_text_payload_for_tests(width, height)
        .ok_or("selection payload missing")?;
    let required = [
        "Files",
        "katana",
        "markdown",
        "sample_basic.md",
        "KDV settings",
        "Display",
        "Dark",
        "Theme",
        "Mode",
        "Preview font",
        "Viewport",
        "Interaction",
        "Hover highlight",
        "Selection",
        "Image controls",
        "Diagram controls",
        "Code controls",
        "State",
        "Slide",
        "Scene font",
        "Nodes",
        "Loaded assets",
        "Failed assets",
        "katana/sample_basic.md",
        "dark document",
        "KatanA Rendering",
        "Heading Levels",
        "Text Decoration",
        "command=none",
    ];

    let missing = required
        .iter()
        .filter(|value| !payload.contains(**value))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "selection payload missed visible text labels {missing:?}: {payload}"
    );
    Ok(())
}

#[test]
fn storybook_window_viewer_text_runs_are_individually_selectable_and_copyable()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_basic.md"),
        PreviewBuilder::default(),
    );
    let width = 1000;
    let height = 900;
    storybook.update_frame_size(width, height);
    storybook.update_scene(width, height)?;
    let canvas = storybook.render_canvas(width, height);
    let area = StorybookPreviewArea::for_window(width, height, storybook.scroll_y);
    let required = ["KatanA Rendering", "Heading Levels", "Text Decoration"];
    let missing = required
        .iter()
        .filter(|needle| {
            !canvas
                .text_runs()
                .iter()
                .filter(|run| run.x() >= area.x && run.y() >= area.y)
                .filter(|run| run.text().contains(**needle))
                .any(|run| {
                    canvas
                        .copy_text_in_selection(
                            Some((run.x(), run.y())),
                            Some((run.right(), run.bottom())),
                        )
                        .is_some_and(|payload| payload.contains(run.text()))
                })
        })
        .copied()
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "viewer text runs must be selectable and copyable by their own rendered bounds: {missing:?}"
    );
    Ok(())
}

#[test]
fn storybook_window_visible_text_runs_are_individually_selectable_and_copyable()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_basic.md"),
        PreviewBuilder::default(),
    );
    let width = 1000;
    let height = 900;
    storybook.update_frame_size(width, height);
    storybook.update_scene(width, height)?;
    let canvas = storybook.render_canvas(width, height);
    let missing = canvas
        .text_runs()
        .iter()
        .filter(|run| run.x() < width && run.y() < height)
        .filter(|run| {
            canvas
                .copy_text_in_selection(Some((run.x(), run.y())), Some((run.right(), run.bottom())))
                .is_none_or(|payload| !payload.contains(run.text()))
        })
        .map(|run| run.text().to_string())
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "every visible text run must be selectable and copyable by its rendered bounds: {missing:?}"
    );
    Ok(())
}

#[test]
fn storybook_window_selection_highlight_changes_viewer_pixels()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_basic.md"),
        PreviewBuilder::default(),
    );
    let width = 1000;
    let height = 900;
    storybook.update_frame_size(width, height);
    storybook.update_scene(width, height)?;
    let base = storybook.render_canvas(width, height);
    storybook.set_text_selection_for_tests((0, 0), (width, height));

    let selected = storybook.render_canvas(width, height);

    assert!(pixel_diff_count(&base, &selected) > 0);
    Ok(())
}

#[test]
fn storybook_window_selection_setting_disables_selection_copy()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_basic.md"),
        PreviewBuilder::default(),
    );
    let width = 1000;
    let height = 900;
    storybook.update_frame_size(width, height);
    storybook.update_scene(width, height)?;
    storybook.interaction.selection_enabled = false;
    storybook.set_text_selection_for_tests((0, 0), (width, height));

    assert_eq!(
        None,
        storybook.selected_text_payload_for_tests(width, height)
    );
    Ok(())
}

#[test]
fn frame_loop_keeps_fast_polling_while_asset_is_pending() {
    let idle_delay = FrameLoopChanges::idle().delay(false);
    let pending_delay = FrameLoopChanges::idle().delay(true);
    let active_delay = FrameLoopChanges::scene_changed().delay(false);

    assert!(idle_delay > active_delay);
    assert_eq!(active_delay, pending_delay);
}

#[test]
fn escape_closes_window_only_outside_slideshow_mode() {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        FixtureCatalog {
            fixtures: Vec::new(),
        },
        PreviewBuilder::default(),
    );

    assert!(!storybook.escape_keeps_window_open(true));

    storybook.mode = ViewerMode::Slideshow;

    assert!(storybook.escape_keeps_window_open(true));
}

#[test]
fn scene_rebuild_recomputes_slideshow_pages_for_new_viewport()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    );
    storybook.mode = ViewerMode::Slideshow;

    storybook.update_scene(1000, 420)?;
    let compact_pages = slideshow_max_page(&storybook)?;

    storybook.update_scene(1000, 1000)?;
    let tall_pages = slideshow_max_page(&storybook)?;

    assert!(compact_pages > tall_pages);
    Ok(())
}

#[test]
fn sidebar_mode_setting_rebuilds_scene_as_slideshow() -> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    );

    assert!(storybook.apply_settings_field(StorybookSettingsField::Mode, 1000, 900)?);
    storybook.update_scene(1000, 900)?;

    assert_eq!(ViewerMode::Slideshow, storybook.mode);
    assert_eq!(ViewerMode::Slideshow, storybook_scene_mode(&storybook)?);
    Ok(())
}

#[test]
fn sidebar_mode_canvas_click_rebuilds_scene_as_slideshow() -> Result<(), Box<dyn std::error::Error>>
{
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    );
    let (x, y) = settings_field_click_point(&storybook, StorybookSettingsField::Mode, 900)?;
    let before = storybook.render_canvas(1000, 900);

    assert!(storybook.apply_canvas_click(
        StorybookPointer::new(x, y, StorybookMouseButton::Left),
        1000,
        900,
    )?);
    storybook.update_scene(1000, 900)?;
    let after = storybook.render_canvas(1000, 900);

    assert_eq!(ViewerMode::Slideshow, storybook.mode);
    assert_eq!(ViewerMode::Slideshow, storybook_scene_mode(&storybook)?);
    assert!(
        pixel_diff_count(&before, &after) > 64,
        "mode setting click must repaint the visible viewer frame as slideshow"
    );
    Ok(())
}

#[test]
fn slideshow_page_scroll_updates_scene_state_without_scene_rebuild()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    );
    storybook.mode = ViewerMode::Slideshow;
    storybook.update_scene(1000, 900)?;
    let original_scene_key = storybook
        .scene
        .as_ref()
        .ok_or("scene missing")?
        .asset_request_key
        .clone();
    let max_page = storybook
        .scene
        .as_ref()
        .ok_or("scene missing")?
        .slideshow_max_page;
    assert!(max_page > 0);

    let viewport_height = preview_content_height(900) as f32;
    storybook.scroll_y = viewport_height;
    storybook.apply_slideshow_page_scroll(viewport_height);

    let scene = storybook.scene.as_ref().ok_or("scene missing")?;
    assert_eq!(1, scene.slideshow_current_page);
    assert_eq!(original_scene_key, scene.asset_request_key);
    assert!(!storybook.scene_refresh_needed(false));
    Ok(())
}

#[test]
fn slideshow_next_button_click_advances_page_without_scene_rebuild()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    );
    storybook.mode = ViewerMode::Slideshow;
    storybook.update_frame_size(1000, 900);
    storybook.update_scene(1000, 900)?;
    let original_scene_key = storybook
        .scene
        .as_ref()
        .ok_or("scene missing")?
        .asset_request_key
        .clone();
    assert!(slideshow_max_page(&storybook)? > 0);

    let pointer = slideshow_action_pointer(
        storybook.scene.as_ref().ok_or("scene missing")?,
        ViewerSlideshowControlAction::NextPage,
        0.0,
    )?;
    let before = storybook.render_canvas(1000, 900);

    assert!(storybook.apply_canvas_click(pointer, 1000, 900)?);
    let after = storybook.render_canvas(1000, 900);
    let scene = storybook.scene.as_ref().ok_or("scene missing")?;
    assert_eq!(1, scene.slideshow_current_page);
    assert_eq!(original_scene_key, scene.asset_request_key);
    assert!(!storybook.scene_refresh_needed(false));
    assert!(
        pixel_diff_count(&before, &after) > 64,
        "slideshow next click must repaint the visible viewer frame"
    );
    Ok(())
}

#[test]
fn slideshow_previous_button_click_works_after_page_scroll()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = slideshow_storybook_at_first_page(1000, 900)?;
    let viewport_height = preview_content_height(900) as f32;
    storybook.scroll_y = viewport_height;
    storybook.apply_slideshow_page_scroll(viewport_height);
    assert_eq!(
        1,
        storybook
            .scene
            .as_ref()
            .ok_or("scene missing")?
            .slideshow_current_page
    );

    let pointer = slideshow_action_pointer(
        storybook.scene.as_ref().ok_or("scene missing")?,
        ViewerSlideshowControlAction::PreviousPage,
        storybook.scroll_y,
    )?;
    let before = storybook.render_canvas(1000, 900);

    assert!(storybook.apply_canvas_click(pointer, 1000, 900)?);
    let after = storybook.render_canvas(1000, 900);
    assert_eq!(
        0,
        storybook
            .scene
            .as_ref()
            .ok_or("scene missing")?
            .slideshow_current_page
    );
    assert!(
        pixel_diff_count(&before, &after) > 64,
        "slideshow previous click must repaint the visible viewer frame"
    );
    Ok(())
}

#[test]
fn slideshow_close_button_click_works_after_page_scroll() -> Result<(), Box<dyn std::error::Error>>
{
    let mut storybook = slideshow_storybook_at_first_page(1000, 900)?;
    let viewport_height = preview_content_height(900) as f32;
    storybook.scroll_y = viewport_height;
    storybook.apply_slideshow_page_scroll(viewport_height);
    let pointer = slideshow_action_pointer(
        storybook.scene.as_ref().ok_or("scene missing")?,
        ViewerSlideshowControlAction::Close,
        storybook.scroll_y,
    )?;
    let before = storybook.render_canvas(1000, 900);

    assert!(storybook.apply_canvas_click(pointer, 1000, 900)?);
    storybook.update_scene(1000, 900)?;
    let after = storybook.render_canvas(1000, 900);
    assert_eq!(ViewerMode::Document, storybook.mode);
    assert_eq!(0.0, storybook.scroll_y);
    assert!(
        pixel_diff_count(&before, &after) > 64,
        "slideshow close click must repaint the visible viewer frame as document"
    );
    Ok(())
}

#[test]
fn sidebar_theme_canvas_click_rebuilds_scene_as_light() -> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    );
    storybook.dark = true;
    storybook.update_scene(1000, 900)?;
    let (x, y) = settings_field_click_point(&storybook, StorybookSettingsField::Theme, 900)?;

    assert!(storybook.apply_canvas_click(
        StorybookPointer::new(x, y, StorybookMouseButton::Left),
        1000,
        900,
    )?);
    storybook.update_scene(1000, 900)?;

    assert!(!storybook.dark);
    assert_eq!("katana-light", storybook_scene_theme_id(&storybook)?);
    Ok(())
}

#[test]
fn live_dark_toggle_point_clicks_dark_field_not_display_section()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    );
    storybook.dark = true;
    storybook.update_scene(1280, 900)?;
    let (x, y) = storybook.live_dark_toggle_point_for_acceptance(1280, 900)?;

    assert!(storybook.apply_canvas_click(
        StorybookPointer::new(x, y, StorybookMouseButton::Left),
        1280,
        900,
    )?);

    assert!(!storybook.dark);
    assert!(!storybook.settings_state.is_collapsed("display"));
    storybook.update_scene(1280, 900)?;
    assert_eq!("katana-light", storybook_scene_theme_id(&storybook)?);
    Ok(())
}

#[test]
fn sidebar_settings_section_click_toggles_section_state() -> Result<(), Box<dyn std::error::Error>>
{
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    );

    assert!(storybook.apply_sidebar_hit(
        SidebarHitResult::SettingsAction(SettingsListAction::ToggleSection {
            section_id: "display".to_string(),
        }),
        1000,
        900,
    )?);

    assert!(storybook.settings_state.is_collapsed("display"));
    assert_eq!("toggle-settings-section", storybook.last_command_label);
    Ok(())
}

#[test]
fn sidebar_settings_section_canvas_click_toggles_section_state()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    );
    let (x, y) = settings_section_click_point(&storybook, "display", 900)?;

    assert!(storybook.apply_canvas_click(
        StorybookPointer::new(x, y, StorybookMouseButton::Left),
        1000,
        900,
    )?);

    assert!(storybook.settings_state.is_collapsed("display"));
    assert_eq!("toggle-settings-section", storybook.last_command_label);
    Ok(())
}

#[test]
fn sidebar_settings_section_hover_marks_kuc_section_node() -> Result<(), Box<dyn std::error::Error>>
{
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    );
    let (x, y) = settings_section_click_point(&storybook, "display", 900)?;

    assert!(storybook.update_sidebar_settings_hover_for_canvas_point(x, y, 1000, 900));

    assert_eq!(
        Some(UiNodeId::new("settings-section:display")),
        storybook.settings_state.hovered_node_id()
    );
    Ok(())
}

#[test]
fn hover_setting_does_not_invalidate_preview_scene() -> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    );
    storybook.update_scene(1000, 900)?;
    storybook.hovered_node_id = Some("node-1".to_string());

    assert!(storybook.apply_settings_field(StorybookSettingsField::Hover, 1000, 900)?);

    assert!(storybook.scene.is_some());
    assert!(storybook.hovered_node_id.is_none());
    assert!(!storybook.scene_refresh_needed(false));
    Ok(())
}

#[test]
fn sidebar_tree_scroll_does_not_move_preview_scroll() {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with_many_katana_labels(),
        PreviewBuilder::default(),
    );
    storybook.scroll_y = 120.0;

    assert!(storybook.apply_sidebar_scroll(-1.0, 24.0, 900));

    assert_eq!(120.0, storybook.scroll_y);
    assert!(storybook.sidebar_scroll.tree_y > 0);
    assert_eq!(0, storybook.sidebar_scroll.settings_y);
}

#[test]
fn sidebar_scroll_rejects_unrendered_sidebar_content_y_inset() {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with_many_katana_labels(),
        PreviewBuilder::default(),
    );
    storybook.scroll_y = 120.0;

    assert!(!storybook.apply_sidebar_scroll(-1.0, 2.0, 900));
    assert!(!storybook.apply_sidebar_scroll(-1.0, 898.0, 900));

    assert_eq!(120.0, storybook.scroll_y);
    assert_eq!(0, storybook.sidebar_scroll.tree_y);
    assert_eq!(0, storybook.sidebar_scroll.settings_y);
}

#[test]
fn preview_scroll_updates_document_scroll_for_tall_content()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    );
    storybook.update_scene(1000, 640)?;
    let content_height = storybook
        .scene
        .as_ref()
        .ok_or("scene missing")?
        .content_height;
    assert!(content_height > 640.0);

    assert!(storybook.apply_preview_scroll(-1.0, 640));

    assert!(storybook.scroll_y > 0.0);
    Ok(())
}

#[test]
fn preview_scroll_accumulates_subpixel_trackpad_delta_without_redraw()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    );
    storybook.update_scene(1000, 640)?;
    storybook.scroll_y = 120.0;

    assert!(!storybook.apply_preview_scroll(-0.01, 640));
    assert_eq!(120.0, storybook.scroll_y);

    assert!(!storybook.apply_preview_scroll(-0.01, 640));
    assert_eq!(120.0, storybook.scroll_y);

    assert!(storybook.apply_preview_scroll(-0.01, 640));
    assert_eq!(121.0, storybook.scroll_y);
    Ok(())
}

#[test]
fn scene_resize_clamps_preview_scroll_to_new_viewport_bounds()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    );
    storybook.update_scene(1000, 640)?;
    let initial_height = storybook
        .scene
        .as_ref()
        .ok_or("scene missing")?
        .content_height;

    storybook.scroll_y = initial_height + 1000.0;
    storybook.update_scene(1000, 1200)?;
    assert_preview_scroll_within_scene(&storybook, 1200)?;

    storybook.scroll_y = initial_height + 1000.0;
    storybook.update_scene_loaded(1000, 1200)?;
    assert_preview_scroll_within_scene(&storybook, 1200)?;
    Ok(())
}

#[test]
fn preview_scroll_reuses_sidebar_frame_cache() -> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    );
    storybook.update_scene(1000, 900)?;

    let _ = storybook.render_canvas(1000, 900);
    let first_misses = storybook.sidebar_frame_cache_misses;

    storybook.scroll_y = 96.0;
    let _ = storybook.render_canvas(1000, 900);

    assert_eq!(first_misses, storybook.sidebar_frame_cache_misses);
    Ok(())
}

#[test]
fn scaled_preview_scroll_reuses_sidebar_frame_cache() -> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    );
    storybook.update_scene(1000, 900)?;

    let _ = storybook.render_canvas_scaled(1000, 900, 2.0);
    let first_misses = storybook.sidebar_frame_cache_misses;

    storybook.scroll_y = 96.0;
    let _ = storybook.render_canvas_scaled(1000, 900, 2.0);

    assert_eq!(first_misses, storybook.sidebar_frame_cache_misses);
    Ok(())
}

#[test]
fn sidebar_scroll_refreshes_sidebar_frame_cache() -> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with_many_katana_labels(),
        PreviewBuilder::default(),
    );

    let _ = storybook.render_canvas(1000, 900);
    let first_misses = storybook.sidebar_frame_cache_misses;

    assert!(storybook.apply_sidebar_scroll(-1.0, 24.0, 900));
    let _ = storybook.render_canvas(1000, 900);

    assert_eq!(first_misses + 1, storybook.sidebar_frame_cache_misses);
    Ok(())
}

#[test]
fn sidebar_scroll_invalidates_presented_frame_before_window_loop_redraw()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with_many_katana_labels(),
        PreviewBuilder::default(),
    );
    storybook.frame_cache = Some(storybook.render_frame_cache_scaled(1000, 900, 2.0));
    assert!(storybook.frame_cache_matches_scaled(1000, 900, 2.0));
    let before = storybook.present_cached_frame_for_tests(1000, 900)?;

    assert!(storybook.apply_sidebar_scroll(-1.0, 24.0, 900));

    assert!(
        storybook.frame_cache.is_none(),
        "sidebar scroll must invalidate the presented frame so the window loop cannot redraw only the preview"
    );
    storybook.frame_cache = Some(storybook.render_frame_cache_scaled(1000, 900, 2.0));
    let after = storybook.present_cached_frame_for_tests(1000, 900)?;
    let changed_sidebar_pixels = (0..900)
        .flat_map(|y| (0..SIDEBAR_WIDTH).map(move |x| y * 1000 + x))
        .filter(|index| before.pixels()[*index] != after.pixels()[*index])
        .count();
    assert!(
        changed_sidebar_pixels > 1_000,
        "sidebar scroll must visibly redraw the presented FileTree: changed={changed_sidebar_pixels}"
    );
    Ok(())
}

#[test]
fn round_trip_fixture_switch_reuses_previous_sidebar_frame_cache()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with_real_labels(&["katana/sample.md", "katana/sample_basic.md"]),
        PreviewBuilder::default(),
    );
    storybook.update_scene(1000, 900)?;
    let _ = storybook.render_canvas(1000, 900);
    let first_misses = storybook.sidebar_frame_cache_misses;

    storybook.selected_index = 1;
    storybook.reset_fixture_state();
    storybook.update_scene(1000, 900)?;
    let _ = storybook.render_canvas(1000, 900);
    let second_misses = storybook.sidebar_frame_cache_misses;

    storybook.selected_index = 0;
    storybook.reset_fixture_state();
    storybook.update_scene(1000, 900)?;
    let _ = storybook.render_canvas(1000, 900);

    assert_eq!(first_misses + 1, second_misses);
    assert_eq!(second_misses, storybook.sidebar_frame_cache_misses);
    Ok(())
}

#[test]
fn preview_scroll_does_not_start_new_asset_job_for_completed_loaded_scene()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_diagrams.md"),
        PreviewBuilder::default(),
    );
    storybook.update_frame_size(1000, 900);
    storybook.update_scene(1000, 900)?;
    let key = asset_key_for_scroll(&storybook, 0.0);
    let scene = storybook.scene.clone().ok_or("scene missing")?;
    storybook.loaded_asset_job_keys.push(key.clone());
    storybook
        .loaded_asset_scenes
        .push(StorybookLoadedAssetScene {
            key,
            scope_key: scene.asset_request_key.clone(),
            scene,
        });
    storybook.asset_job = None;

    assert!(storybook.apply_preview_scroll(-1.0, 900));
    storybook.start_asset_job_for_current_viewport(1000, 900);

    assert!(storybook.asset_job.is_none());
    Ok(())
}

#[test]
fn preview_scroll_keeps_pending_asset_job_identity() -> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/drawio/basic/03-basic-flow.drawio"),
        PreviewBuilder::default(),
    );
    storybook.update_frame_size(1000, 900);
    storybook.update_scene(1000, 900)?;
    let pending_key = storybook
        .asset_job
        .as_ref()
        .ok_or("asset job missing")?
        .key()
        .clone();

    storybook.scroll_y = 1200.0;
    storybook.start_asset_job_for_current_viewport(1000, 900);

    let scrolled_key = storybook
        .asset_job
        .as_ref()
        .ok_or("asset job missing after scroll")?
        .key()
        .clone();

    assert_eq!(pending_key, scrolled_key);
    assert_eq!(
        asset_key_for_fixture_scroll(
            &storybook,
            "katana/drawio/basic/03-basic-flow.drawio",
            1200.0
        ),
        scrolled_key
    );
    Ok(())
}

#[test]
fn interaction_setting_toggles_keep_pending_asset_job() -> Result<(), Box<dyn std::error::Error>> {
    for case in [
        InteractionSettingCase::new(StorybookSettingsField::Hover, false),
        InteractionSettingCase::new(StorybookSettingsField::Selection, true),
        InteractionSettingCase::new(StorybookSettingsField::ImageControls, true),
        InteractionSettingCase::new(StorybookSettingsField::DiagramControls, true),
        InteractionSettingCase::new(StorybookSettingsField::CodeControls, true),
    ] {
        let mut storybook = StorybookWindow::new(
            StorybookArgs::default(),
            catalog_with("katana/sample_diagrams.md"),
            PreviewBuilder::default(),
        );
        storybook.update_frame_size(1000, 900);
        storybook.update_scene(1000, 900)?;
        let pending_key = storybook
            .asset_job
            .as_ref()
            .ok_or("asset job missing")?
            .key()
            .clone();

        assert!(storybook.apply_settings_field(case.field, 1000, 900)?);

        let preserved_key = storybook
            .asset_job
            .as_ref()
            .ok_or("asset job must stay pending after interaction setting toggle")?
            .key()
            .clone();
        assert_eq!(pending_key, preserved_key, "{:?}", case.field);
        if case.rebuilds_scene {
            assert!(
                storybook.scene.is_none(),
                "interaction setting still rebuilds scene so KUC receives visible control state: {:?}",
                case.field
            );
        }
    }
    Ok(())
}

#[test]
fn click_only_text_selection_does_not_poison_scroll_redraw_path()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_basic.md"),
        PreviewBuilder::default(),
    );
    storybook.update_frame_size(1000, 900);
    storybook.update_scene(1000, 900)?;

    assert!(storybook.apply_text_selection_drag_for_smoke((640.0, 240.0), (640.0, 240.0)));

    assert_eq!(
        (None, None),
        storybook.text_selection_for_tests(),
        "plain clicks must not leave an empty selection that disables cached scroll redraw"
    );
    Ok(())
}

struct InteractionSettingCase {
    field: StorybookSettingsField,
    rebuilds_scene: bool,
}

impl InteractionSettingCase {
    const fn new(field: StorybookSettingsField, rebuilds_scene: bool) -> Self {
        Self {
            field,
            rebuilds_scene,
        }
    }
}

#[test]
fn asset_job_streams_partial_scene_before_completion() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_markdown_fixture(
        "tmp/streaming-diagrams.md",
        "\
```mermaid
graph TD
  A --> B
```

```mermaid
graph TD
  C --> D
```",
    )?;
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        FixtureCatalog {
            fixtures: vec![fixture],
        },
        PreviewBuilder::default(),
    );
    storybook.update_frame_size(1280, 720);
    storybook.update_scene(1280, 720)?;
    let scene = storybook.scene.as_ref().ok_or("scene missing")?;

    assert!(scene.asset_request_count >= 2);
    assert!(node_role_count(scene.tree.root(), "media-pending") >= 2);

    let deadline = Instant::now() + Duration::from_secs(10);
    let mut saw_partial = false;
    let mut saw_complete = false;
    while Instant::now() < deadline {
        if storybook.apply_asset_job()? {
            let scene = storybook.scene.as_ref().ok_or("scene missing")?;
            if storybook.asset_job.is_some() {
                saw_partial = true;
                assert!(scene.loaded_asset_count + scene.failed_asset_count >= 1);
                assert!(scene.asset_request_count >= 1);
            } else {
                saw_complete = true;
                assert_eq!(0, scene.asset_request_count);
                break;
            }
        }
        std::thread::yield_now();
    }

    assert!(
        saw_partial,
        "asset job did not stream an intermediate scene"
    );
    assert!(saw_complete, "asset job did not complete before deadline");
    Ok(())
}

#[test]
fn active_asset_job_key_survives_scroll_change() -> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/drawio/basic/03-basic-flow.drawio"),
        PreviewBuilder::default(),
    );
    storybook.update_frame_size(1000, 900);
    storybook.update_scene(1000, 900)?;
    let stale_key = storybook
        .asset_job
        .as_ref()
        .ok_or("asset job missing")?
        .key()
        .clone();

    storybook.scroll_y = 1200.0;
    let current_key = asset_key_for_fixture_scroll(
        &storybook,
        "katana/drawio/basic/03-basic-flow.drawio",
        storybook.scroll_y,
    );
    assert_eq!(stale_key, current_key);

    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        let _changed = storybook.apply_asset_job()?;
        if let Some(job) = storybook.asset_job.as_ref() {
            assert_eq!(&current_key, job.key());
        } else {
            return Ok(());
        }
        std::thread::yield_now();
    }

    Err("asset job did not complete after scroll changed".into())
}

#[test]
fn interaction_setting_toggles_preserve_pending_asset_job() -> Result<(), Box<dyn std::error::Error>>
{
    for field in [
        StorybookSettingsField::Hover,
        StorybookSettingsField::Selection,
        StorybookSettingsField::ImageControls,
        StorybookSettingsField::DiagramControls,
        StorybookSettingsField::CodeControls,
    ] {
        let mut storybook = StorybookWindow::new(
            StorybookArgs::default(),
            catalog_with("katana/sample_diagrams.md"),
            PreviewBuilder::default(),
        );
        storybook.update_frame_size(1000, 900);
        storybook.update_scene(1000, 900)?;
        let before_key = storybook
            .asset_job
            .as_ref()
            .ok_or("asset job missing before settings toggle")?
            .key()
            .clone();

        assert!(storybook.apply_settings_field(field, 1000, 900)?);

        let after_key = storybook
            .asset_job
            .as_ref()
            .ok_or("interaction setting toggle must not drop pending asset job")?
            .key()
            .clone();
        assert_eq!(
            before_key, after_key,
            "interaction setting toggle must keep the same asset job key: {field:?}"
        );
    }
    Ok(())
}

#[test]
fn asset_job_result_updates_scene_and_clears_pending_count()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/drawio/basic/03-basic-flow.drawio"),
        PreviewBuilder::default(),
    );
    storybook.update_frame_size(1000, 900);
    storybook.update_scene(1000, 900)?;
    assert!(storybook.asset_job.is_some());
    assert!(
        storybook
            .scene
            .as_ref()
            .is_some_and(|scene| scene.asset_request_count > 0)
    );

    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if storybook.apply_asset_job()? {
            let scene = storybook.scene.as_ref().ok_or("scene missing")?;
            assert_eq!(0, scene.asset_request_count);
            assert!(scene.loaded_asset_count > 0);
            assert!(scene.image_surface_count > 0);
            let stats = storybook.preview.builder_cache_stats()?;
            assert!(stats.parsed_hits >= 1);
            assert_eq!(1, stats.parsed_misses);
            return Ok(());
        }
        std::thread::yield_now();
    }

    Err("asset job did not complete before deadline".into())
}

#[test]
fn asset_job_result_invalidates_scene_dependent_render_caches()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/drawio/basic/03-basic-flow.drawio"),
        PreviewBuilder::default(),
    );
    storybook.update_frame_size(1000, 900);
    storybook.update_scene(1000, 900)?;
    let pending_scene = storybook.scene.as_ref().ok_or("scene missing")?.clone();

    let _ = storybook.render_canvas(1000, 900);
    storybook.frame_cache = Some(StorybookFrameCache::new(Canvas::new_scaled(
        1000, 900, 2.0, 0,
    )));
    storybook.document_interaction_surface_cache =
        Some(StorybookDocumentInteractionSurfaceCache::new(
            1000,
            900,
            storybook.scroll_y,
            &pending_scene,
            Arc::new(Vec::new()),
            Vec::new(),
        ));
    storybook.update_sidebar_hover(Some((24.0, 120.0)), 1000, 900);

    assert!(storybook.frame_cache.is_some());
    assert!(storybook.document_interaction_surface_cache.is_some());
    assert!(storybook.sidebar_interaction_cache.is_some());
    assert!(storybook.sidebar_interaction_surface_cache.is_some());
    assert!(!storybook.sidebar_frame_cache.is_empty());

    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if storybook.apply_asset_job()? {
            assert!(
                storybook.frame_cache.is_none(),
                "loaded asset scene must not keep the pending frame cache"
            );
            assert!(
                storybook.document_interaction_surface_cache.is_none(),
                "loaded asset scene must not keep pending document hit surfaces"
            );
            assert!(
                storybook.sidebar_interaction_cache.is_none(),
                "loaded asset scene must not keep pending sidebar hit cache"
            );
            assert!(
                storybook.sidebar_interaction_surface_cache.is_none(),
                "loaded asset scene must not keep pending sidebar hit surface"
            );
            assert!(
                storybook.sidebar_frame_cache.is_empty(),
                "loaded asset scene must redraw sidebar status from the latest scene"
            );
            return Ok(());
        }
        std::thread::yield_now();
    }

    Err("asset job did not complete before deadline".into())
}

#[test]
fn asset_job_timeout_closes_pending_scene_without_window_error()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/drawio/basic/03-basic-flow.drawio"),
        PreviewBuilder::default(),
    );
    storybook.update_frame_size(1000, 900);
    storybook.update_scene(1000, 900)?;
    let pending_scene = storybook.scene.clone().ok_or("pending scene missing")?;
    let pending_count = pending_scene.asset_request_count;
    let key = asset_key_for_fixture_scroll(
        &storybook,
        "katana/drawio/basic/03-basic-flow.drawio",
        storybook.scroll_y,
    );
    let (_sender, receiver) = std::sync::mpsc::channel();
    storybook.asset_job = Some(StorybookAssetJob::from_receiver_for_test(
        key.clone(),
        pending_scene.asset_request_key,
        receiver,
        Instant::now() - Duration::from_millis(20),
        Duration::from_millis(1),
    ));

    assert!(storybook.apply_asset_job()?);

    let scene = storybook.scene.as_ref().ok_or("scene missing")?;
    assert!(storybook.asset_job.is_none());
    assert_eq!(0, scene.asset_request_count);
    assert!(scene.failed_asset_count >= pending_count.max(1));
    assert!(
        storybook
            .loaded_asset_scenes
            .iter()
            .any(|loaded| loaded.key == key)
    );
    Ok(())
}

#[test]
fn direct_image_asset_job_reaches_image_surface() -> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with_real_labels(&["direct/kdv-icon.png"]),
        PreviewBuilder::default(),
    );
    storybook.update_frame_size(1000, 900);
    storybook.update_scene(1000, 900)?;
    assert!(storybook.asset_job.is_some());

    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if storybook.apply_asset_job()? {
            let scene = storybook.scene.as_ref().ok_or("scene missing")?;
            assert_eq!(0, scene.asset_request_count);
            assert_eq!(0, scene.failed_asset_count);
            assert!(scene.loaded_asset_count > 0);
            assert!(scene.image_surface_count > 0);
            return Ok(());
        }
        std::thread::yield_now();
    }

    Err("direct image asset job did not complete before deadline".into())
}

#[test]
fn direct_image_loaded_scene_reaches_image_surface() -> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with_real_labels(&["direct/kdv-icon.png"]),
        PreviewBuilder::default(),
    );
    storybook.update_scene_loaded(1000, 900)?;

    let scene = storybook.scene.as_ref().ok_or("scene missing")?;
    assert_eq!(0, scene.asset_request_count);
    assert_eq!(0, scene.failed_asset_count);
    assert!(scene.loaded_asset_count > 0);
    assert!(scene.image_surface_count > 0);
    Ok(())
}

#[test]
fn curated_loaded_scenes_do_not_break_direct_image_fixture()
-> Result<(), Box<dyn std::error::Error>> {
    let label = "direct/kdv-icon.png";
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with_real_labels(&[label]),
        PreviewBuilder::default(),
    );
    storybook.update_scene_loaded(1280, 900)?;

    let canvas = storybook.render_canvas(1280, 900);
    StorybookFramePixelGuard::assert_fixture_content(label, &canvas, storybook.dark)?;
    let scene = storybook.scene.as_ref().ok_or("scene missing")?;
    StorybookSmokeAssertions::assert_fixture_visible(label, scene)?;
    assert_eq!(0, scene.asset_request_count);
    assert_eq!(0, scene.failed_asset_count);
    assert!(scene.loaded_asset_count > 0);
    assert!(scene.image_surface_count > 0);
    Ok(())
}

#[test]
fn scene_viewport_matches_rendered_preview_content_area() -> Result<(), Box<dyn std::error::Error>>
{
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    );

    storybook.update_scene(1000, 900)?;
    let scene = storybook.scene.as_ref().ok_or("scene missing")?;
    let scroll_area = &scene.tree.root().props().scroll_area;

    assert_eq!(
        preview_content_width(1000) as u32,
        scroll_area.viewport_width
    );
    assert_eq!(
        preview_content_height(900) as u32,
        scroll_area.viewport_height
    );
    Ok(())
}

#[test]
fn sidebar_settings_scroll_does_not_move_preview_scroll() {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with_many_katana_labels(),
        PreviewBuilder::default(),
    );
    storybook.scroll_y = 120.0;

    assert!(storybook.apply_sidebar_scroll(-1.0, 700.0, 900));

    assert_eq!(120.0, storybook.scroll_y);
    assert_eq!(0, storybook.sidebar_scroll.tree_y);
    assert!(storybook.sidebar_scroll.settings_y > 0);
}

#[test]
fn sidebar_directory_toggle_updates_file_tree_state() -> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with_many_katana_labels(),
        PreviewBuilder::default(),
    );

    assert!(!storybook.file_tree_state.is_collapsed("katana/group"));
    assert!(storybook.apply_sidebar_hit(
        SidebarHitResult::FileTree(FileTreeAction::ToggleDirectory {
            directory_id: "katana/group".to_string()
        }),
        1000,
        900,
    )?);
    assert!(storybook.file_tree_state.is_collapsed("katana/group"));
    Ok(())
}

#[test]
fn sidebar_directory_canvas_click_toggles_file_tree_state() -> Result<(), Box<dyn std::error::Error>>
{
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with_many_katana_labels(),
        PreviewBuilder::default(),
    );
    let (x, y) = tree_directory_click_point(&storybook, "katana/group", 900)?;

    assert!(storybook.apply_canvas_click(
        StorybookPointer::new(x, y, StorybookMouseButton::Left),
        1000,
        900,
    )?);

    assert!(storybook.file_tree_state.is_collapsed("katana/group"));
    assert_eq!("toggle-directory", storybook.last_command_label);
    Ok(())
}

#[test]
fn sidebar_file_canvas_click_selects_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with_many_katana_labels(),
        PreviewBuilder::default(),
    );
    let (x, y) = tree_fixture_click_point(&storybook, 3, 900)?;

    assert!(storybook.apply_canvas_click(
        StorybookPointer::new(x, y, StorybookMouseButton::Left),
        1000,
        900,
    )?);

    assert_eq!(3, storybook.selected_index);
    assert_eq!("select-file", storybook.last_command_label);
    Ok(())
}

#[test]
fn sidebar_file_canvas_click_rebuilds_viewer_scene_for_selected_fixture()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with_real_labels(&["katana/sample.md", "katana/sample_basic.md"]),
        PreviewBuilder::default(),
    );
    storybook.update_scene(1000, 900)?;
    let first_document_id = scene_document_id(&storybook)?.to_string();
    assert!(first_document_id.ends_with("assets/fixtures/katana/sample.md"));
    assert!(scene_contains_label(&storybook, "comprehensive sample"));
    assert!(!scene_contains_label(&storybook, "core Markdown rendering"));

    let (x, y) = tree_fixture_click_point(&storybook, 1, 900)?;
    assert!(storybook.apply_canvas_click(
        StorybookPointer::new(x, y, StorybookMouseButton::Left),
        1000,
        900,
    )?);
    storybook.update_scene(1000, 900)?;

    assert_eq!(1, storybook.selected_index);
    assert_eq!("select-file", storybook.last_command_label);
    let second_document_id = scene_document_id(&storybook)?;
    assert!(second_document_id.ends_with("assets/fixtures/katana/sample_basic.md"));
    assert_ne!(first_document_id, second_document_id);
    assert!(scene_contains_label(&storybook, "core Markdown rendering"));
    assert!(!scene_contains_label(&storybook, "comprehensive sample"));
    Ok(())
}

#[test]
#[ignore = "release-only performance gate"]
fn fixture_switch_first_frame_lazy_rebuild_is_inside_600ms_budget()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with_real_labels(&["katana/sample.md", "katana/sample_diagrams.md"]),
        PreviewBuilder::default(),
    );
    storybook.update_scene(1000, 900)?;

    assert!(storybook.apply_sidebar_hit(
        SidebarHitResult::FileTree(FileTreeAction::SelectFile {
            file_id: "katana/sample_diagrams.md".to_string()
        }),
        1000,
        900,
    )?);
    assert_eq!(1, storybook.selected_index);

    let started = Instant::now();
    storybook.update_scene_for_refresh(1000, 900)?;
    let _ = storybook.render_canvas(1000, 900);
    let elapsed = started.elapsed();

    assert!(
        elapsed <= Duration::from_millis(600),
        "fixture switch first frame took {elapsed:?}, budget 600ms"
    );
    Ok(())
}

#[test]
#[ignore = "release-only performance gate"]
fn fixture_switch_pending_first_frame_is_inside_200ms_budget()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with_real_labels(&["katana/sample.md", "katana/sample_diagrams.md"]),
        PreviewBuilder::default(),
    );
    storybook.update_scene(1280, 720)?;
    let _ = storybook.render_canvas(1280, 720);

    assert!(storybook.apply_sidebar_hit(
        SidebarHitResult::FileTree(FileTreeAction::SelectFile {
            file_id: "katana/sample_diagrams.md".to_string()
        }),
        1280,
        720,
    )?);

    let started = Instant::now();
    storybook.update_scene_for_refresh(1280, 720)?;
    let update_elapsed = started.elapsed();
    let render_started = Instant::now();
    let request = frame_render_request(&storybook, 1280, 720);
    let sidebar = StorybookFrameRenderer::render_sidebar(&request);
    let sidebar_elapsed = render_started.elapsed();
    let _ = StorybookFrameRenderer::render_with_sidebar(request, &sidebar);
    let elapsed = started.elapsed();
    let render_elapsed = elapsed.saturating_sub(update_elapsed);
    let preview_elapsed = render_elapsed.saturating_sub(sidebar_elapsed);
    let scene = storybook.scene.as_ref().ok_or("scene missing")?;

    assert!(scene.asset_request_count >= 3);
    assert_eq!(0, scene.loaded_asset_count);
    assert!(node_role_count(scene.tree.root(), "media-pending") >= 3);
    assert!(
        elapsed <= Duration::from_millis(200),
        "fixture switch pending first frame took {elapsed:?}, update {update_elapsed:?}, render {render_elapsed:?}, sidebar {sidebar_elapsed:?}, preview {preview_elapsed:?}, budget 200ms"
    );
    Ok(())
}

#[test]
fn pending_asset_spinner_animates_between_frames() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = temp_markdown_fixture(
        "tmp/spinner-animation.md",
        "\
```mermaid
graph TD
  PendingA --> PendingB
```",
    )?;
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        FixtureCatalog {
            fixtures: vec![fixture],
        },
        PreviewBuilder::default(),
    );
    storybook.update_scene_for_refresh(1280, 720)?;
    let first = storybook.render_canvas(1280, 720);
    assert!(
        storybook.asset_job.is_some(),
        "diagram fixture must start an asset job after the pending frame is rendered"
    );
    assert!(storybook.update_loading_animation(true));
    let second = storybook.render_canvas(1280, 720);

    assert!(
        pixel_diff_count(&first, &second) > 0,
        "pending spinner frame must change while asset job is pending"
    );
    Ok(())
}

#[test]
#[ignore = "release-only performance/interaction gate"]
fn fixture_switch_pending_first_frame_completes_loaded_diagram_assets()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with_real_labels(&[
            "katana/sample.md",
            "katana/drawio/basic/03-basic-flow.drawio",
        ]),
        PreviewBuilder::default(),
    );
    storybook.update_scene(1280, 720)?;
    let _ = storybook.render_canvas(1280, 720);

    assert!(storybook.apply_sidebar_hit(
        SidebarHitResult::FileTree(FileTreeAction::SelectFile {
            file_id: "katana/drawio/basic/03-basic-flow.drawio".to_string()
        }),
        1280,
        720,
    )?);

    storybook.update_scene_for_refresh(1280, 720)?;
    let pending_canvas = storybook.render_canvas(1280, 720);

    let scene = storybook.scene.as_ref().ok_or("scene missing")?;
    assert!(scene.asset_request_count >= 1);
    assert_eq!(0, scene.loaded_asset_count);
    assert!(node_role_count(scene.tree.root(), "media-pending") >= 1);
    let initial_image_surface_count = scene.image_surface_count;
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if storybook.apply_asset_job()? {
            let scene = storybook.scene.as_ref().ok_or("scene missing")?;
            assert_eq!(0, scene.asset_request_count);
            assert_eq!(0, node_role_count(scene.tree.root(), "media-pending"));
            assert!(scene.image_surface_count > initial_image_surface_count);
            let loaded_canvas = storybook.render_canvas(1280, 720);
            assert!(
                pixel_diff_count(&pending_canvas, &loaded_canvas) > 64,
                "loaded diagram frame must differ from pending diagram frame"
            );
            return Ok(());
        }
        std::thread::yield_now();
    }

    Err("diagram asset job did not complete before deadline".into())
}

#[test]
fn sidebar_tree_hover_updates_kuc_file_tree_hover_state() -> Result<(), Box<dyn std::error::Error>>
{
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with_many_katana_labels(),
        PreviewBuilder::default(),
    );
    let (x, y) = tree_fixture_click_point(&storybook, 3, 900)?;

    assert!(storybook.update_sidebar_tree_hover_for_canvas_point(x, y, 1000, 900));
    assert_eq!(
        Some("katana/group/file-3.md"),
        storybook.file_tree_state.hovered_item_id()
    );

    let (settings_x, settings_y) = settings_section_click_point(&storybook, "display", 900)?;
    assert!(
        storybook.update_sidebar_tree_hover_for_canvas_point(settings_x, settings_y, 1000, 900)
    );
    assert_eq!(None, storybook.file_tree_state.hovered_item_id());
    Ok(())
}

#[test]
fn sidebar_tree_hover_reuses_kuc_interaction_for_cursor_lookup()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with_many_katana_labels(),
        PreviewBuilder::default(),
    );
    let (x, y) = tree_fixture_click_point(&storybook, 3, 900)?;

    assert!(storybook.sidebar_interaction_cache.is_none());
    assert!(storybook.update_sidebar_tree_hover_for_canvas_point(x, y, 1000, 900));
    let cached = storybook
        .sidebar_interaction_cache
        .clone()
        .ok_or("sidebar interaction cache missing")?;
    let cursor = storybook.cursor_for_canvas_point(x, y, 1000, 900);

    assert_eq!(UiCursor::Pointer, cursor);
    assert_eq!(Some(cached), storybook.sidebar_interaction_cache);
    Ok(())
}

#[test]
fn sidebar_scroll_clears_kuc_interaction_cache() -> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with_many_katana_labels(),
        PreviewBuilder::default(),
    );
    let (x, y) = tree_fixture_click_point(&storybook, 3, 900)?;

    assert!(storybook.update_sidebar_tree_hover_for_canvas_point(x, y, 1000, 900));
    assert!(storybook.sidebar_interaction_cache.is_some());
    assert!(storybook.apply_sidebar_scroll(-1.0, 24.0, 900));

    assert!(storybook.sidebar_interaction_cache.is_none());
    Ok(())
}

#[test]
fn sidebar_directory_toggle_does_not_invalidate_preview_scene()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    );
    storybook.update_scene(1000, 900)?;
    assert!(storybook.scene.is_some());

    assert!(storybook.apply_sidebar_hit(
        SidebarHitResult::FileTree(FileTreeAction::ToggleDirectory {
            directory_id: "katana/group".to_string()
        }),
        1000,
        900,
    )?);

    assert!(storybook.scene.is_some());
    assert!(!storybook.scene_refresh_needed(false));
    Ok(())
}

#[test]
fn completed_asset_job_key_prevents_same_viewport_reload() -> Result<(), Box<dyn std::error::Error>>
{
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_diagrams.md"),
        PreviewBuilder::default(),
    );
    storybook.update_scene(1000, 900)?;
    assert!(storybook.asset_job.is_some());

    let key = asset_key_for_scroll(&storybook, storybook.scroll_y);
    let scene = storybook.scene.clone().ok_or("scene missing")?;
    storybook.asset_job = None;
    storybook.loaded_asset_job_keys.push(key.clone());
    storybook
        .loaded_asset_scenes
        .push(StorybookLoadedAssetScene {
            key,
            scope_key: scene.asset_request_key.clone(),
            scene,
        });

    storybook.start_asset_job_for_current_viewport(1000, 900);

    assert!(storybook.asset_job.is_none());
    Ok(())
}

#[test]
fn completed_asset_job_cache_survives_preview_scroll() -> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_diagrams.md"),
        PreviewBuilder::default(),
    );
    storybook.update_scene(1000, 900)?;
    let key = asset_key_for_scroll(&storybook, 0.0);
    let scene = storybook.scene.clone().ok_or("scene missing")?;
    let initial_scope = scene.asset_request_key.clone();
    storybook.asset_job = None;
    storybook.loaded_asset_job_keys.push(key.clone());
    storybook
        .loaded_asset_scenes
        .push(StorybookLoadedAssetScene {
            key,
            scope_key: initial_scope.clone(),
            scene,
        });

    storybook.scroll_y = 0.0;
    storybook.start_asset_job_for_current_viewport(1000, 900);
    assert!(storybook.asset_job.is_none());

    storybook.scroll_y = 1200.0;
    storybook.update_scene(1000, 900)?;
    let scrolled_scope = storybook
        .scene
        .as_ref()
        .ok_or("scrolled scene missing")?
        .asset_request_key
        .clone();
    assert_eq!(initial_scope, scrolled_scope);
    storybook.start_asset_job_for_current_viewport(1000, 900);
    assert!(storybook.asset_job.is_none());
    Ok(())
}

#[test]
fn completed_asset_job_key_restores_cached_loaded_scene() -> Result<(), Box<dyn std::error::Error>>
{
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_diagrams.md"),
        PreviewBuilder::default(),
    );
    storybook.update_scene(1000, 900)?;
    let key = asset_key_for_scroll(&storybook, 0.0);
    let pending_scene = storybook.scene.clone().ok_or("pending scene missing")?;
    let scope_key = pending_scene.asset_request_key.clone();
    let mut loaded_scene = pending_scene.clone();
    loaded_scene.asset_request_count = 0;
    loaded_scene.image_surface_count = 7;
    storybook.loaded_asset_job_keys.push(key.clone());
    storybook
        .loaded_asset_scenes
        .push(StorybookLoadedAssetScene {
            key,
            scope_key,
            scene: loaded_scene.clone(),
        });

    storybook.update_scene(1000, 900)?;

    assert!(storybook.asset_job.is_none());
    assert_eq!(
        loaded_scene.image_surface_count,
        storybook
            .scene
            .as_ref()
            .ok_or("scene missing")?
            .image_surface_count
    );
    Ok(())
}

#[test]
fn loaded_asset_scene_cache_separates_dark_and_light_diagram_theme()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_diagrams.md"),
        PreviewBuilder::default(),
    );
    storybook.dark = true;
    storybook.update_scene_loaded(1000, 900)?;
    let dark_key = asset_key_for_scroll(&storybook, 0.0);
    let dark_scene = storybook.scene.clone().ok_or("dark loaded scene missing")?;
    storybook.remember_loaded_asset_scene(
        dark_key.clone(),
        dark_scene.asset_request_key.clone(),
        dark_scene,
    );

    storybook.dark = false;
    storybook.update_scene(1000, 900)?;

    let light_key = asset_key_for_scroll(&storybook, 0.0);
    assert_ne!(dark_key, light_key);
    assert!(storybook.asset_job.is_some());
    assert_eq!(
        0,
        storybook
            .scene
            .as_ref()
            .ok_or("light scene missing")?
            .image_surface_count
    );
    Ok(())
}

#[test]
fn loaded_scene_after_scroll_does_not_restart_asset_job() -> Result<(), Box<dyn std::error::Error>>
{
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_diagrams.md"),
        PreviewBuilder::default(),
    );
    storybook.update_scene_loaded(1000, 900)?;
    let scene = storybook.scene.as_ref().ok_or("scene missing")?;
    assert!(scene.image_surface_count > 0);

    storybook.scroll_y = 32.0;
    storybook.start_asset_job_for_current_viewport(1000, 900);

    assert!(storybook.asset_job.is_none());
    assert!(
        storybook
            .scene
            .as_ref()
            .is_some_and(|scene| { scene.image_surface_count > 0 })
    );
    Ok(())
}

#[test]
fn loaded_asset_scene_cache_is_bounded() -> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_diagrams.md"),
        PreviewBuilder::default(),
    );
    storybook.update_scene_loaded(1280, 900)?;
    let loaded_scene = storybook.scene.clone().ok_or("loaded scene missing")?;

    for index in 0..MAX_LOADED_ASSET_SCENES + 3 {
        storybook.typography.preview_font_size = 12 + index as u16;
        let key = asset_key_for_scroll(&storybook, 0.0);
        storybook.remember_loaded_asset_scene(
            key,
            loaded_scene.asset_request_key.clone(),
            loaded_scene.clone(),
        );
    }

    assert_eq!(
        MAX_LOADED_ASSET_SCENES,
        storybook.loaded_asset_job_keys.len()
    );
    assert_eq!(MAX_LOADED_ASSET_SCENES, storybook.loaded_asset_scenes.len());
    Ok(())
}

fn asset_key_for_scroll(storybook: &StorybookWindow, _scroll_y: f32) -> StorybookAssetJobKey {
    asset_key_for_fixture_scroll(storybook, "katana/sample_diagrams.md", 0.0)
}

fn asset_key_for_fixture_scroll(
    storybook: &StorybookWindow,
    fixture_label: &str,
    _scroll_y: f32,
) -> StorybookAssetJobKey {
    StorybookAssetJobKey::new(StorybookAssetJobKeyInput {
        fixture_label: fixture_label.to_string(),
        dark: storybook.dark,
        mode: storybook.mode.clone(),
        typography: storybook.typography,
        search: &storybook.search,
        diagram_viewports: &storybook.diagram_viewports,
        image_viewports: &storybook.image_viewports,
        task_state_overrides: &storybook.task_state_overrides,
        accordion_open_overrides: &storybook.accordion_open_overrides,
        viewport: ViewerViewport {
            width: preview_content_width(1000) as f32,
            height: preview_content_height(900) as f32,
        },
    })
}

fn slideshow_max_page(storybook: &StorybookWindow) -> Result<usize, Box<dyn std::error::Error>> {
    let Some(scene) = storybook.scene.as_ref() else {
        return Err("scene missing".into());
    };
    Ok(scene.slideshow_max_page)
}

fn slideshow_storybook_at_first_page(
    width: usize,
    height: usize,
) -> Result<StorybookWindow, Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    );
    storybook.mode = ViewerMode::Slideshow;
    storybook.update_frame_size(width, height);
    storybook.update_scene(width, height)?;
    assert!(slideshow_max_page(&storybook)? > 0);
    Ok(storybook)
}

fn slideshow_action_pointer(
    scene: &crate::preview::PreviewScene,
    action: ViewerSlideshowControlAction,
    scroll_y: f32,
) -> Result<StorybookPointer, Box<dyn std::error::Error>> {
    let action_id = action.host_action_id();
    let area = StorybookPreviewArea::for_window(1000, 900, 0.0);
    let hits = StorybookHostActionHits::viewport_hits_for_preview_width(
        scene,
        area.width,
        area.height,
        scroll_y,
    );
    let hit = hits
        .as_ref()
        .iter()
        .find(|hit| hit.action.action_id == action_id)
        .ok_or_else(|| std::io::Error::other("slideshow action hit missing"))?;
    let (center_x, center_y) = hit.center_point();
    let area = StorybookPreviewArea::for_window(1000, 900, 0.0);
    let (x, y) = area.canvas_point_for_document_point(center_x, center_y);
    Ok(StorybookPointer::new(x, y, StorybookMouseButton::Left))
}

fn assert_preview_scroll_within_scene(
    storybook: &StorybookWindow,
    height: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(scene) = storybook.scene.as_ref() else {
        return Err("scene missing".into());
    };
    let viewport_height = preview_content_height(height) as f32;
    let max_scroll = (scene.content_height - viewport_height).max(0.0);
    assert!(
        storybook.scroll_y <= max_scroll + f32::EPSILON,
        "preview scroll was not clamped: scroll_y={} max_scroll={max_scroll}",
        storybook.scroll_y
    );
    Ok(())
}

fn storybook_scene_mode(
    storybook: &StorybookWindow,
) -> Result<ViewerMode, Box<dyn std::error::Error>> {
    let Some(scene) = storybook.scene.as_ref() else {
        return Err("scene missing".into());
    };
    Ok(scene.mode.clone())
}

fn storybook_scene_theme_id(
    storybook: &StorybookWindow,
) -> Result<String, Box<dyn std::error::Error>> {
    let Some(scene) = storybook.scene.as_ref() else {
        return Err("scene missing".into());
    };
    Ok(scene.theme.id.as_str().to_string())
}

fn settings_field_click_point(
    storybook: &StorybookWindow,
    field: StorybookSettingsField,
    height: usize,
) -> Result<(f32, f32), Box<dyn std::error::Error>> {
    let point = StorybookSidebar::settings_field_canvas_point(StorybookSettingsFieldHitRequest {
        scene: storybook.scene.as_ref(),
        dark: storybook.dark,
        interaction: &storybook.interaction,
        typography: storybook.typography,
        settings_state: &storybook.settings_state,
        width: sidebar_content_width(),
        height: sidebar_content_height(height),
        scroll: storybook.sidebar_scroll,
        field_id: field.id(),
    })
    .ok_or_else(|| format!("settings field hit not found: {field:?}"))?;

    Ok((point.x, point.y))
}

fn settings_section_click_point(
    storybook: &StorybookWindow,
    section_id: &str,
    height: usize,
) -> Result<(f32, f32), Box<dyn std::error::Error>> {
    let point =
        StorybookSidebar::settings_section_canvas_point(StorybookSettingsSectionHitRequest {
            scene: storybook.scene.as_ref(),
            dark: storybook.dark,
            interaction: &storybook.interaction,
            typography: storybook.typography,
            settings_state: &storybook.settings_state,
            width: sidebar_content_width(),
            height: sidebar_content_height(height),
            scroll: storybook.sidebar_scroll,
            section_id,
        })
        .ok_or_else(|| format!("settings section hit not found: {section_id}"))?;

    Ok((point.x, point.y))
}

fn tree_directory_click_point(
    storybook: &StorybookWindow,
    directory_id: &str,
    height: usize,
) -> Result<(f32, f32), Box<dyn std::error::Error>> {
    tree_item_canvas_point(storybook, directory_id, height)
}

fn tree_fixture_click_point(
    storybook: &StorybookWindow,
    fixture_index: usize,
    height: usize,
) -> Result<(f32, f32), Box<dyn std::error::Error>> {
    let expected = &storybook.catalog.fixtures[fixture_index].label;
    tree_item_canvas_point(storybook, expected, height)
}

fn tree_item_canvas_point(
    storybook: &StorybookWindow,
    item_id: &str,
    height: usize,
) -> Result<(f32, f32), Box<dyn std::error::Error>> {
    let point =
        StorybookSidebar::fixture_canvas_point_for_item_id(StorybookFileTreeItemPointRequest {
            fixtures: &storybook.catalog.fixtures,
            selected_index: storybook.selected_index,
            state: &storybook.file_tree_state,
            item_id,
            height: sidebar_content_height(height),
            scroll: storybook.sidebar_scroll,
        })
        .ok_or_else(|| format!("tree item hit not found: {item_id}"))?;
    Ok((point.x, point.y))
}

fn catalog_with(label: &str) -> FixtureCatalog {
    FixtureCatalog {
        fixtures: vec![StorybookFixture {
            label: label.to_string(),
            path: fixture_path(&format!("assets/fixtures/{label}")),
        }],
    }
}

fn catalog_with_real_labels(labels: &[&str]) -> FixtureCatalog {
    FixtureCatalog {
        fixtures: labels
            .iter()
            .map(|label| StorybookFixture {
                label: (*label).to_string(),
                path: fixture_path(&format!("assets/fixtures/{label}")),
            })
            .collect(),
    }
}

fn catalog_with_many_katana_labels() -> FixtureCatalog {
    let fixtures = (0..40)
        .map(|index| StorybookFixture {
            label: format!("katana/group/file-{index}.md"),
            path: PathBuf::from(format!("katana/group/file-{index}.md")),
        })
        .collect();
    FixtureCatalog { fixtures }
}

fn fixture_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../{path}"))
}

fn temp_markdown_fixture(
    label: &str,
    content: &str,
) -> Result<StorybookFixture, Box<dyn std::error::Error>> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path =
        std::env::temp_dir().join(format!("kdv-storybook-{}-{nanos}.md", std::process::id()));
    std::fs::write(&path, content)?;
    Ok(StorybookFixture {
        label: label.to_string(),
        path,
    })
}

fn scene_contains_label(storybook: &StorybookWindow, fragment: &str) -> bool {
    storybook
        .scene
        .as_ref()
        .is_some_and(|scene| node_contains_label(scene.tree.root(), fragment))
}

fn scene_document_id(storybook: &StorybookWindow) -> Result<&str, Box<dyn std::error::Error>> {
    storybook
        .scene
        .as_ref()
        .map(|scene| scene.document_id.as_str())
        .ok_or_else(|| "missing preview scene".into())
}

fn node_contains_label(node: &UiNode, fragment: &str) -> bool {
    node.props().label.contains(fragment)
        || node
            .children()
            .iter()
            .any(|child| node_contains_label(child, fragment))
}

fn node_role_count(node: &UiNode, role: &str) -> usize {
    usize::from(node.props().text.role == role)
        + node
            .children()
            .iter()
            .map(|child| node_role_count(child, role))
            .sum::<usize>()
}

fn pixel_diff_count(left: &crate::canvas::Canvas, right: &crate::canvas::Canvas) -> usize {
    left.pixels()
        .iter()
        .zip(right.pixels().iter())
        .filter(|(left, right)| left != right)
        .count()
}

fn frame_render_request(
    storybook: &StorybookWindow,
    width: usize,
    height: usize,
) -> FrameRenderRequest<'_> {
    FrameRenderRequest {
        width,
        height,
        fixtures: &storybook.catalog.fixtures,
        selected_index: storybook.selected_index,
        scene: storybook.scene.as_ref(),
        scroll_y: storybook.scroll_y,
        sidebar_scroll: storybook.sidebar_scroll,
        file_tree_state: storybook.file_tree_state.clone(),
        settings_state: &storybook.settings_state,
        dark: storybook.dark,
        interaction: &storybook.interaction,
        typography: storybook.typography,
        last_command_label: &storybook.last_command_label,
        task_context_menu: storybook.task_context_menu.as_ref(),
        hovered_node_id: storybook.hovered_node_id.as_deref(),
        hovered_action_node_id: storybook.hovered_action_node_id.as_ref(),
        animation_phase: 0,
    }
}

struct TextDiagramEngine;

impl DiagramRenderEngine for TextDiagramEngine {
    fn render(&self, request: DiagramRenderRequest<'_>) -> Result<RenderedDiagram, String> {
        Ok(RenderedDiagram {
            node_id: request.node_id.to_string(),
            kind: "mermaid".to_string(),
            svg: r#"<svg xmlns="http://www.w3.org/2000/svg"><text>Artifact Needle</text></svg>"#
                .to_string(),
        })
    }
}
