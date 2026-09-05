use super::StorybookWindow;
use crate::args::StorybookArgs;
use crate::catalog::{FixtureCatalog, StorybookFixture};
use crate::layout::{
    StorybookPreviewArea, preview_viewport_height as window_preview_viewport_height,
};
use crate::mouse::mouse_test_support::{WINDOW_HEIGHT, WINDOW_WIDTH, pointer_for_media_action};
use crate::preview::PreviewBuilder;
use katana_document_viewer::{DiagramRenderEngine, DiagramRenderRequest, RenderedDiagram};
use std::path::PathBuf;
use std::sync::Arc;

const FRAME_WIDTH: usize = 1280;
const FRAME_HEIGHT: usize = 720;
const PREVIEW_HEIGHT: f32 = 600.0;

#[test]
fn preview_scroll_keeps_current_scene_and_asset_job_scope() -> Result<(), Box<dyn std::error::Error>>
{
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_diagrams.md"),
        PreviewBuilder::with_diagram_engine(Arc::new(FastDiagramEngine)),
    );
    storybook.update_frame_size(1000, 900);
    storybook.update_scene(1000, 900)?;
    let first_stats = storybook.preview.builder_cache_stats()?;
    let first_scene = storybook.scene.clone().ok_or("scene missing")?;
    let first_scope = first_scene.asset_request_key.clone();
    let first_job_key = storybook
        .asset_job
        .as_ref()
        .ok_or("asset job missing")?
        .key()
        .clone();

    assert!(storybook.apply_preview_scroll(-24.0, 900));
    let _canvas = storybook.render_canvas(1000, 900);
    let scrolled_stats = storybook.preview.builder_cache_stats()?;
    let scrolled_scene = storybook.scene.as_ref().ok_or("scrolled scene missing")?;
    let scrolled_job_key = storybook
        .asset_job
        .as_ref()
        .ok_or("asset job missing after scroll")?
        .key();

    assert_eq!(
        first_stats.lazy_scene_misses,
        scrolled_stats.lazy_scene_misses
    );
    assert_eq!(first_scope, scrolled_scene.asset_request_key);
    assert_eq!(&first_job_key, scrolled_job_key);
    Ok(())
}

#[test]
fn loaded_diagram_wheel_scroll_uses_presented_band_redraw_without_full_fallback()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_diagrams.md"),
        PreviewBuilder::with_diagram_engine(Arc::new(FastDiagramEngine)),
    );
    storybook.update_scene_for_tests(FRAME_WIDTH, FRAME_HEIGHT)?;
    storybook.wait_loaded_asset_scene_for_tests(FRAME_WIDTH, FRAME_HEIGHT)?;
    let max_scroll = storybook
        .scene_for_tests()
        .map(|scene| (scene.content_height - PREVIEW_HEIGHT).max(0.0))
        .unwrap_or(0.0);
    storybook.scroll_y_for_tests(max_scroll.min(1_800.0));
    storybook.render_cached_scroll_canvas_scaled_for_tests(FRAME_WIDTH, FRAME_HEIGHT, 2.0)?;

    let phases = storybook.render_wheel_scroll_cached_frame_phase_times_for_tests(
        -1.0,
        FRAME_WIDTH,
        FRAME_HEIGHT,
        2.0,
    )?;

    assert!(
        !phases.full_preview_redraw_fallback,
        "loaded diagram wheel scroll must update the presented scroll band without falling back to full preview redraw: {phases:?}"
    );
    Ok(())
}

#[test]
fn table_presented_scroll_matches_full_redraw_at_retina_scale()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = sample_window();
    storybook.update_scene_for_tests(FRAME_WIDTH, FRAME_HEIGHT)?;
    let start_scroll = table_start_scroll(&storybook, FRAME_HEIGHT)?;
    let target_scroll = start_scroll + 48.0;
    storybook.scroll_y_for_tests(start_scroll);
    storybook.render_cached_scroll_canvas_scaled_for_tests(FRAME_WIDTH, FRAME_HEIGHT, 2.0)?;

    storybook.render_wheel_scroll_presented_frame_for_tests(
        -1.0,
        FRAME_WIDTH,
        FRAME_HEIGHT,
        2.0,
    )?;
    assert_eq!(target_scroll, storybook.scroll_y_value_for_tests());
    let cached_source = storybook.cached_source_frame_for_tests()?;
    let cached = storybook.present_cached_frame_for_tests(FRAME_WIDTH, FRAME_HEIGHT)?;

    let mut reference = sample_window();
    reference.update_scene_for_tests(FRAME_WIDTH, FRAME_HEIGHT)?;
    reference.scroll_y_for_tests(target_scroll);
    reference.render_cached_scroll_canvas_scaled_for_tests(FRAME_WIDTH, FRAME_HEIGHT, 2.0)?;
    let full_source = reference.cached_source_frame_for_tests()?;
    let full = reference.present_cached_frame_for_tests(FRAME_WIDTH, FRAME_HEIGHT)?;

    assert_eq!(
        DiffStats::default(),
        source_diff_stats(&cached_source, &full_source),
        "table source cache after cached presented-scroll must match full redraw"
    );
    assert_eq!(
        DiffStats::default(),
        preview_diff_stats(&cached, &full),
        "table viewport cached presented-scroll must match a full redraw at the same scroll offset"
    );
    Ok(())
}

#[test]
fn sequence_diagram_presented_scroll_matches_full_redraw_at_retina_scale()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = sample_window_with_fast_diagram_engine();
    storybook.update_scene_for_tests(FRAME_WIDTH, FRAME_HEIGHT)?;
    storybook.wait_loaded_asset_scene_for_tests(FRAME_WIDTH, FRAME_HEIGHT)?;
    let start_scroll = sequence_diagram_start_scroll(&storybook, FRAME_HEIGHT)?;
    let target_scroll = start_scroll + 48.0;
    storybook.scroll_y_for_tests(start_scroll);
    storybook.render_cached_scroll_canvas_scaled_for_tests(FRAME_WIDTH, FRAME_HEIGHT, 2.0)?;

    storybook.render_wheel_scroll_presented_frame_for_tests(
        -1.0,
        FRAME_WIDTH,
        FRAME_HEIGHT,
        2.0,
    )?;
    assert_eq!(target_scroll, storybook.scroll_y_value_for_tests());
    let cached_source = storybook.cached_source_frame_for_tests()?;
    let cached = storybook.present_cached_frame_for_tests(FRAME_WIDTH, FRAME_HEIGHT)?;

    let mut reference = sample_window_with_fast_diagram_engine();
    reference.update_scene_for_tests(FRAME_WIDTH, FRAME_HEIGHT)?;
    reference.wait_loaded_asset_scene_for_tests(FRAME_WIDTH, FRAME_HEIGHT)?;
    reference.scroll_y_for_tests(target_scroll);
    reference.render_cached_scroll_canvas_scaled_for_tests(FRAME_WIDTH, FRAME_HEIGHT, 2.0)?;
    let full_source = reference.cached_source_frame_for_tests()?;
    let full = reference.present_cached_frame_for_tests(FRAME_WIDTH, FRAME_HEIGHT)?;

    assert_eq!(
        DiffStats::default(),
        source_diff_stats(&cached_source, &full_source),
        "Sequence Diagram source cache after cached presented-scroll must match full redraw"
    );
    assert_eq!(
        DiffStats::default(),
        preview_diff_stats(&cached, &full),
        "Sequence Diagram viewport cached presented-scroll must not leave duplicated heading pixels"
    );
    Ok(())
}

#[test]
fn sequence_diagram_scroll_uses_presented_band_redraw_without_full_fallback()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = sample_window_with_fast_diagram_engine();
    storybook.update_scene_for_tests(FRAME_WIDTH, FRAME_HEIGHT)?;
    storybook.wait_loaded_asset_scene_for_tests(FRAME_WIDTH, FRAME_HEIGHT)?;
    let start_scroll = sequence_diagram_start_scroll(&storybook, FRAME_HEIGHT)?;
    storybook.scroll_y_for_tests(start_scroll);
    storybook.render_cached_scroll_canvas_scaled_for_tests(FRAME_WIDTH, FRAME_HEIGHT, 2.0)?;

    let phases = storybook.render_wheel_scroll_cached_frame_phase_times_for_tests(
        -1.0,
        FRAME_WIDTH,
        FRAME_HEIGHT,
        2.0,
    )?;

    assert!(
        !phases.full_preview_redraw_fallback,
        "Sequence Diagram scroll must preserve presented-band redraw: {phases:?}"
    );
    Ok(())
}

#[test]
fn table_hover_after_presented_scroll_matches_full_redraw() -> Result<(), Box<dyn std::error::Error>>
{
    let mut storybook = sample_window();
    storybook.update_scene_for_tests(FRAME_WIDTH, FRAME_HEIGHT)?;
    let table_y = table_target_y(&storybook)?;
    let start_scroll = table_start_scroll(&storybook, FRAME_HEIGHT)?;
    storybook.scroll_y_for_tests(start_scroll);
    storybook.render_cached_scroll_canvas_scaled_for_tests(FRAME_WIDTH, FRAME_HEIGHT, 2.0)?;
    storybook.render_wheel_scroll_presented_frame_for_tests(
        -1.0,
        FRAME_WIDTH,
        FRAME_HEIGHT,
        2.0,
    )?;

    let target_scroll = start_scroll + 48.0;
    let hover = table_hover_point(table_y, target_scroll);
    assert!(
        storybook.update_document_hover_for_canvas_point(
            hover.0,
            hover.1,
            FRAME_WIDTH,
            FRAME_HEIGHT
        ),
        "hovering the table after presented-scroll must resolve through KUC hits"
    );
    storybook.render_cached_scroll_presented_frame_for_tests(FRAME_WIDTH, FRAME_HEIGHT, 2.0)?;
    let cached_hover = storybook.present_cached_frame_for_tests(FRAME_WIDTH, FRAME_HEIGHT)?;

    let mut reference = sample_window();
    reference.update_scene_for_tests(FRAME_WIDTH, FRAME_HEIGHT)?;
    reference.scroll_y_for_tests(target_scroll);
    assert!(
        reference.update_document_hover_for_canvas_point(
            hover.0,
            hover.1,
            FRAME_WIDTH,
            FRAME_HEIGHT
        ),
        "hovering the table on a full redraw reference must resolve through KUC hits"
    );
    reference.render_cached_scroll_canvas_scaled_for_tests(FRAME_WIDTH, FRAME_HEIGHT, 2.0)?;
    let full_hover = reference.present_cached_frame_for_tests(FRAME_WIDTH, FRAME_HEIGHT)?;

    assert_eq!(
        DiffStats::default(),
        preview_diff_stats(&cached_hover, &full_hover),
        "table hover after cached presented-scroll must not change layout compared with full redraw"
    );
    Ok(())
}

#[test]
fn table_target_height_does_not_overlap_following_viewer_node()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = sample_window();
    storybook.update_scene_for_tests(FRAME_WIDTH, FRAME_HEIGHT)?;
    let scene = storybook.scene_for_tests().ok_or("scene missing")?;
    let table = table_target(scene)?;
    let table_bottom = table.rect.y + table.rect.height;
    let next = scene
        .targets
        .iter()
        .filter(|target| target.rect.y > table.rect.y + 1.0)
        .min_by(|left, right| {
            left.rect
                .y
                .partial_cmp(&right.rect.y)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .ok_or("following table target missing")?;

    assert!(
        next.rect.y + 0.5 >= table_bottom,
        "table planned rect must reserve enough height for wrapped cells: table={:?} next={:?}",
        table.rect,
        next.rect
    );
    Ok(())
}

#[test]
fn zoomed_loaded_diagram_wheel_scroll_uses_presented_band_redraw_without_full_fallback()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample_diagrams.md"),
        PreviewBuilder::with_diagram_engine(Arc::new(FastDiagramEngine)),
    );
    storybook.update_scene_for_tests(WINDOW_WIDTH, WINDOW_HEIGHT)?;
    storybook.wait_loaded_asset_scene_for_tests(WINDOW_WIDTH, WINDOW_HEIGHT)?;
    let scene = storybook
        .scene_for_tests()
        .cloned()
        .ok_or("loaded scene missing")?;
    let zoom_hit = pointer_for_media_action(&scene, "zoom-in")?;
    storybook.scroll_y_for_tests(zoom_hit.scroll_y);

    assert!(storybook.apply_canvas_click(zoom_hit.pointer, WINDOW_WIDTH, WINDOW_HEIGHT,)?);
    let zoomed = storybook
        .diagram_viewports
        .values()
        .any(|state| state.zoom > 1.0);
    assert!(zoomed, "zoom-in control must update diagram viewport state");
    storybook.update_scene_for_refresh(WINDOW_WIDTH, WINDOW_HEIGHT)?;

    let max_scroll = storybook
        .scene_for_tests()
        .map(|scene| (scene.content_height - preview_viewport_height()).max(0.0))
        .unwrap_or(0.0);
    storybook.scroll_y_for_tests(max_scroll.min(1_800.0));
    storybook.render_cached_scroll_canvas_scaled_for_tests(WINDOW_WIDTH, WINDOW_HEIGHT, 2.0)?;

    let phases = storybook.render_wheel_scroll_cached_frame_phase_times_for_tests(
        -1.0,
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        2.0,
    )?;

    assert!(
        !phases.full_preview_redraw_fallback,
        "zoomed loaded diagram wheel scroll must keep the presented band path instead of full preview redraw: {phases:?}"
    );
    Ok(())
}

fn sample_window() -> StorybookWindow {
    StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::default(),
    )
}

fn sample_window_with_fast_diagram_engine() -> StorybookWindow {
    StorybookWindow::new(
        StorybookArgs::default(),
        catalog_with("katana/sample.md"),
        PreviewBuilder::with_diagram_engine(Arc::new(FastDiagramEngine)),
    )
}

fn sequence_diagram_start_scroll(
    storybook: &StorybookWindow,
    height: usize,
) -> Result<f32, Box<dyn std::error::Error>> {
    let heading_y = sequence_diagram_heading_y(storybook)?;
    let scene = storybook.scene_for_tests().ok_or("scene missing")?;
    let max_scroll =
        (scene.content_height - window_preview_viewport_height(height) as f32).max(0.0);
    Ok((heading_y - 120.0).clamp(0.0, max_scroll))
}

fn sequence_diagram_heading_y(
    storybook: &StorybookWindow,
) -> Result<f32, Box<dyn std::error::Error>> {
    Ok(
        sequence_diagram_heading_target(storybook.scene_for_tests().ok_or("scene missing")?)?
            .rect
            .y,
    )
}

fn sequence_diagram_heading_target(
    scene: &crate::preview::PreviewScene,
) -> Result<&katana_document_viewer::ViewerTarget, Box<dyn std::error::Error>> {
    scene
        .targets
        .iter()
        .find(|target| target.source.raw.text.contains("10.2 Sequence Diagram"))
        .ok_or_else(|| "Sequence Diagram heading target missing".into())
}

fn table_start_scroll(
    storybook: &StorybookWindow,
    height: usize,
) -> Result<f32, Box<dyn std::error::Error>> {
    let table_y = table_target_y(storybook)?;
    let scene = storybook.scene_for_tests().ok_or("scene missing")?;
    let max_scroll =
        (scene.content_height - window_preview_viewport_height(height) as f32).max(0.0);
    Ok((table_y - 120.0).clamp(0.0, max_scroll))
}

fn table_target_y(storybook: &StorybookWindow) -> Result<f32, Box<dyn std::error::Error>> {
    Ok(
        table_target(storybook.scene_for_tests().ok_or("scene missing")?)?
            .rect
            .y,
    )
}

fn table_target(
    scene: &crate::preview::PreviewScene,
) -> Result<&katana_document_viewer::ViewerTarget, Box<dyn std::error::Error>> {
    scene
        .targets
        .iter()
        .find(|target| {
            target.source.raw.text.contains("Feature | Status | Notes")
                || target.source.raw.text.contains("Feature\nStatus\nNotes")
        })
        .ok_or_else(|| "table target missing".into())
}

fn table_hover_point(table_y: f32, scroll_y: f32) -> (f32, f32) {
    let area = StorybookPreviewArea::for_window(FRAME_WIDTH, FRAME_HEIGHT, scroll_y);
    area.canvas_point_for_document_point(120.0, table_y + 32.0)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct DiffStats {
    count: usize,
    min_x: usize,
    min_y: usize,
    max_x: usize,
    max_y: usize,
}

fn source_diff_stats(left: &crate::canvas::Canvas, right: &crate::canvas::Canvas) -> DiffStats {
    let area = StorybookPreviewArea::for_window(FRAME_WIDTH, FRAME_HEIGHT, 0.0);
    let scale = left.scale_factor();
    let x = (area.x as f32 * scale).round() as usize;
    let y = (area.y as f32 * scale).round() as usize;
    let width = (area.width as f32 * scale).round() as usize;
    let height = (area.height as f32 * scale).round() as usize;
    diff_stats_in_rect(left, right, x, y, width, height)
}

fn preview_diff_stats(left: &crate::canvas::Canvas, right: &crate::canvas::Canvas) -> DiffStats {
    let area = StorybookPreviewArea::for_window(FRAME_WIDTH, FRAME_HEIGHT, 0.0);
    diff_stats_in_rect(left, right, area.x, area.y, area.width, area.height)
}

fn diff_stats_in_rect(
    left: &crate::canvas::Canvas,
    right: &crate::canvas::Canvas,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> DiffStats {
    let mut stats = DiffStats {
        min_x: usize::MAX,
        min_y: usize::MAX,
        ..DiffStats::default()
    };
    left.pixels()
        .iter()
        .zip(right.pixels().iter())
        .enumerate()
        .for_each(|(index, (left_pixel, right_pixel))| {
            let pixel_x = index % left.width();
            let pixel_y = index / left.width();
            if pixel_x >= x
                && pixel_x < x + width
                && pixel_y >= y
                && pixel_y < y + height
                && left_pixel != right_pixel
            {
                stats.count += 1;
                stats.min_x = stats.min_x.min(pixel_x);
                stats.min_y = stats.min_y.min(pixel_y);
                stats.max_x = stats.max_x.max(pixel_x);
                stats.max_y = stats.max_y.max(pixel_y);
            }
        });
    if stats.count == 0 {
        return DiffStats::default();
    }
    stats
}

fn catalog_with(path: &str) -> FixtureCatalog {
    FixtureCatalog {
        fixtures: vec![StorybookFixture {
            label: path.to_string(),
            path: fixture_path(path),
        }],
    }
}

fn fixture_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../assets/fixtures/{path}"))
}

struct FastDiagramEngine;

impl DiagramRenderEngine for FastDiagramEngine {
    fn render(&self, request: DiagramRenderRequest<'_>) -> Result<RenderedDiagram, String> {
        Ok(RenderedDiagram {
            node_id: request.node_id.to_string(),
            kind: "mermaid".to_string(),
            svg: format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"240\" height=\"120\" \
                 viewBox=\"0 0 240 120\"><text x=\"16\" y=\"64\">{}</text></svg>",
                request.node_id
            ),
        })
    }
}

fn preview_viewport_height() -> f32 {
    (WINDOW_HEIGHT - crate::layout::HEADER_HEIGHT) as f32
}
