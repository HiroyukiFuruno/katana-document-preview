use super::frame_surface_dump::{SurfaceDump, SurfaceDumpImage};
use super::frame_surface_similarity::SurfaceParityScorer;
use crate::canvas::{Canvas, SurfaceArea};
use crate::catalog::StorybookFixture;
use crate::palette::StorybookPalette;
use crate::preview::PreviewBuilder;
use katana_document_viewer::{ViewerInteractionConfig, ViewerTypographyConfig, ViewerViewport};
use katana_ui_core::render_model::{UiImageSurfaceRenderPlan, UiNode};
use katana_ui_core_storybook::UiTreeStorybookHost;
use std::path::{Path, PathBuf};

const VISUAL_SCORE_THRESHOLD: u8 = 95;
const SURFACE_WIDTH: usize = 1280;
const CROP_HEIGHT: usize = 2400;
const PREVIEW_FONT_SIZE: u16 = 14;
const COMPACT_BADGE_VERTICAL_MARGIN: usize = 15;
const KATANA_REFERENCE_CROP_PHYSICAL_WIDTH: usize = 2374;
const KATANA_REFERENCE_CROP_PHYSICAL_HEIGHT: usize = 4450;
const KATANA_REFERENCE_DEVICE_SCALE: f32 = 2.0;
const LEGACY_SCORE_RENDER_SCALE: f32 =
    KATANA_REFERENCE_CROP_PHYSICAL_WIDTH as f32 / SURFACE_WIDTH as f32;
const KATANA_PREVIEW_CONTENT_LOGICAL_WIDTH: usize = 1187;
const KATANA_PREVIEW_CONTENT_LOGICAL_HEIGHT: usize = 2225;
const KATANA_PREVIEW_CROP_REFERENCE: &str = "assets/reference/katana/preview_crops/sample-top.png";
const KATANA_SAMPLE_DIAGRAMS_CROP_REFERENCE: &str =
    "assets/reference/katana/preview_crops/sample-diagrams-top.png";

#[test]
fn storybook_score_visual_uses_katana_preview_crop_reference()
-> Result<(), Box<dyn std::error::Error>> {
    assert_preview_crop_score(
        KATANA_PREVIEW_CROP_REFERENCE,
        "katana/sample.md",
        "katana/sample.md-preview-crop",
        false,
    )
}

#[test]
fn storybook_score_visual_uses_katana_sample_diagrams_crop_reference()
-> Result<(), Box<dyn std::error::Error>> {
    assert_preview_crop_score(
        KATANA_SAMPLE_DIAGRAMS_CROP_REFERENCE,
        "katana/sample_diagrams.md",
        "katana/sample_diagrams.md-preview-crop",
        true,
    )
}

#[test]
fn storybook_preview_crop_score_uses_scaled_canvas_pixels() -> Result<(), Box<dyn std::error::Error>>
{
    let scaled = PreviewCrop::render_storybook_top_score_crop_info("katana/sample.md", false)?;

    assert_eq!(SURFACE_WIDTH, scaled.crop.width);
    assert_eq!(CROP_HEIGHT, scaled.crop.height);
    assert_eq!(SURFACE_WIDTH, scaled.rendered_logical_width);
    assert_eq!(CROP_HEIGHT, scaled.rendered_logical_height);
    assert_eq!(
        KATANA_REFERENCE_CROP_PHYSICAL_WIDTH,
        scaled.crop_physical_width
    );
    assert!(
        scaled
            .crop_physical_height
            .abs_diff(KATANA_REFERENCE_CROP_PHYSICAL_HEIGHT)
            <= 1,
        "storybook score crop physical height must match the KatanA crop within rounding: expected={} actual={}",
        KATANA_REFERENCE_CROP_PHYSICAL_HEIGHT,
        scaled.crop_physical_height
    );
    assert_eq!(
        KATANA_REFERENCE_CROP_PHYSICAL_WIDTH,
        scaled.rendered_physical_width
    );
    assert!(
        scaled
            .rendered_physical_height
            .abs_diff(KATANA_REFERENCE_CROP_PHYSICAL_HEIGHT)
            <= 1,
        "storybook score rendered physical height must match the KatanA crop within rounding: expected={} actual={}",
        KATANA_REFERENCE_CROP_PHYSICAL_HEIGHT,
        scaled.rendered_physical_height
    );
    Ok(())
}

#[test]
fn storybook_sample_diagrams_score_uses_katana_preview_content_rect()
-> Result<(), Box<dyn std::error::Error>> {
    let scaled =
        PreviewCrop::render_storybook_top_score_crop_info("katana/sample_diagrams.md", true)?;

    assert_eq!(
        KATANA_PREVIEW_CONTENT_LOGICAL_WIDTH,
        scaled.rendered_logical_width
    );
    assert_eq!(
        KATANA_PREVIEW_CONTENT_LOGICAL_HEIGHT,
        scaled.rendered_logical_height
    );
    assert_eq!(
        KATANA_REFERENCE_CROP_PHYSICAL_WIDTH,
        scaled.rendered_physical_width
    );
    assert_eq!(
        KATANA_REFERENCE_CROP_PHYSICAL_HEIGHT,
        scaled.rendered_physical_height
    );
    Ok(())
}

#[test]
fn storybook_preview_crop_score_excludes_storybook_overlay_controls()
-> Result<(), Box<dyn std::error::Error>> {
    let diagrams = PreviewCrop::render_score_scene("katana/sample_diagrams.md", true)?;
    assert_no_overlay_controls(diagrams.tree.root(), "katana/sample_diagrams.md");

    let code_blocks = PreviewCrop::render_score_scene("katana/sample_basic.md", false)?;
    assert_no_overlay_controls(code_blocks.tree.root(), "katana/sample_basic.md");
    Ok(())
}

#[test]
fn storybook_preview_crop_score_excludes_host_scrollbar() -> Result<(), Box<dyn std::error::Error>>
{
    let rendered =
        PreviewCrop::render_storybook_top_score_crop_info("katana/sample_diagrams.md", true)?;

    assert_eq!(0, right_edge_scrollbar_pixels(&rendered.crop));
    Ok(())
}

#[test]
fn storybook_sample_top_local_text_metrics_match_katana_reference()
-> Result<(), Box<dyn std::error::Error>> {
    let reference = PreviewCrop::load(KATANA_PREVIEW_CROP_REFERENCE)?;
    let candidate = PreviewCrop::render_storybook_top_score_crop("katana/sample.md", false)?;
    let reference_metrics = LocalTextMetrics::collect(&reference)?;
    let candidate_metrics = LocalTextMetrics::collect(&candidate)?;

    assert!(
        within_percent(
            candidate_metrics.title_height,
            reference_metrics.title_height,
            90,
            115
        ),
        "storybook title text is crushed relative to KatanA reference: reference={reference_metrics:?} candidate={candidate_metrics:?}"
    );
    assert!(
        within_percent(
            candidate_metrics.body_first_height,
            reference_metrics.body_first_height,
            90,
            115
        ),
        "storybook first body line is crushed relative to KatanA reference: reference={reference_metrics:?} candidate={candidate_metrics:?}"
    );
    assert!(
        within_percent(
            candidate_metrics.body_second_height,
            reference_metrics.body_second_height,
            90,
            115
        ),
        "storybook second body line is crushed relative to KatanA reference: reference={reference_metrics:?} candidate={candidate_metrics:?}"
    );
    assert!(
        within_percent(
            candidate_metrics.link_blue_count,
            reference_metrics.link_blue_count,
            80,
            140
        ),
        "storybook language link lost too many blue text/underline pixels relative to KatanA reference: reference={reference_metrics:?} candidate={candidate_metrics:?}"
    );
    assert!(
        within_percent(
            candidate_metrics.link_blue_width,
            reference_metrics.link_blue_width,
            90,
            110
        ),
        "storybook language link underline width diverges from KatanA reference: reference={reference_metrics:?} candidate={candidate_metrics:?}"
    );
    Ok(())
}

#[test]
fn storybook_sample_rule_and_following_heading_bands_match_katana_reference()
-> Result<(), Box<dyn std::error::Error>> {
    let reference = PreviewCrop::load(KATANA_PREVIEW_CROP_REFERENCE)?;
    let candidate = PreviewCrop::render_storybook_top_score_crop("katana/sample.md", false)?;
    let reference_metrics = LocalRuleHeadingMetrics::collect(&reference)?;
    let candidate_metrics = LocalRuleHeadingMetrics::collect(&candidate)?;

    assert!(
        candidate_metrics
            .rule
            .min_y
            .abs_diff(reference_metrics.rule.min_y)
            <= 2
            && candidate_metrics
                .rule
                .max_y
                .abs_diff(reference_metrics.rule.max_y)
                <= 2
            && within_percent(
                candidate_metrics.rule.width(),
                reference_metrics.rule.width(),
                99,
                101
            ),
        "storybook horizontal rule band diverges from KatanA reference: reference={reference_metrics:?} candidate={candidate_metrics:?}"
    );
    assert!(
        candidate_metrics
            .heading
            .min_y
            .abs_diff(reference_metrics.heading.min_y)
            <= 2
            && candidate_metrics
                .heading
                .max_y
                .abs_diff(reference_metrics.heading.max_y)
                <= 2,
        "storybook heading after horizontal rule diverges from KatanA reference: reference={reference_metrics:?} candidate={candidate_metrics:?}"
    );
    Ok(())
}

#[test]
fn storybook_sample_badge_rows_match_katana_reference_bands()
-> Result<(), Box<dyn std::error::Error>> {
    let reference = PreviewCrop::load(KATANA_PREVIEW_CROP_REFERENCE)?;
    let candidate = PreviewCrop::render_storybook_top_score_crop("katana/sample.md", false)?;
    let reference_metrics = LocalBadgeMetrics::collect(&reference)?;
    let candidate_metrics = LocalBadgeMetrics::collect(&candidate)?;

    for (index, (reference_row, candidate_row)) in reference_metrics
        .rows
        .iter()
        .zip(candidate_metrics.rows.iter())
        .take(2)
        .enumerate()
    {
        let min_y_tolerance = if index == 0 { 1 } else { 2 };
        assert!(
            candidate_row.min_y.abs_diff(reference_row.min_y) <= min_y_tolerance
                && candidate_row.max_y.abs_diff(reference_row.max_y) <= 1
                && within_percent(candidate_row.width(), reference_row.width(), 96, 104)
                && within_percent(candidate_row.height(), reference_row.height(), 94, 106),
            "storybook badge row {index} diverges from KatanA reference: reference={reference_metrics:?} candidate={candidate_metrics:?}"
        );
    }
    Ok(())
}

#[test]
fn storybook_sample_badge_targets_align_with_katana_reference_bands()
-> Result<(), Box<dyn std::error::Error>> {
    let reference = PreviewCrop::load(KATANA_PREVIEW_CROP_REFERENCE)?;
    let reference_metrics = LocalBadgeMetrics::collect(&reference)?;
    let scene = PreviewCrop::render_score_scene("katana/sample.md", false)?;
    let badge_targets = sample_badge_targets(&scene);

    assert!(
        badge_targets.len() >= 2,
        "storybook sample must expose badge rows as KUC semantic targets: {badge_targets:?}"
    );
    let mut mismatches = Vec::new();
    for (index, (reference_row, target)) in reference_metrics
        .rows
        .iter()
        .zip(badge_targets.iter())
        .take(2)
        .enumerate()
    {
        let expected_top = reference_row
            .min_y
            .saturating_sub(COMPACT_BADGE_VERTICAL_MARGIN);
        let actual_top = target.rect.y.round().max(0.0) as usize;
        if actual_top.abs_diff(expected_top) > 2 {
            mismatches.push(format!(
                "index={index} expected_top={expected_top} actual_top={actual_top} reference_row={reference_row:?} target={target:?}"
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "storybook badge target y-positions diverge from KatanA reference: {}",
        mismatches.join("; ")
    );
    Ok(())
}

#[test]
fn storybook_sample_diagrams_dark_top_text_metrics_match_katana_reference()
-> Result<(), Box<dyn std::error::Error>> {
    let reference = PreviewCrop::load(KATANA_SAMPLE_DIAGRAMS_CROP_REFERENCE)?;
    let candidate =
        PreviewCrop::render_storybook_top_score_crop("katana/sample_diagrams.md", true)?;
    let reference_metrics = LocalTextMetrics::collect_dark(&reference)?;
    let candidate_metrics = LocalTextMetrics::collect_dark(&candidate)?;

    assert!(
        within_percent(
            candidate_metrics.title_height,
            reference_metrics.title_height,
            90,
            115
        ) && within_percent(
            candidate_metrics.title_width,
            reference_metrics.title_width,
            96,
            104
        ),
        "storybook dark sample diagrams title is crushed or too narrow relative to KatanA reference: reference={reference_metrics:?} candidate={candidate_metrics:?}"
    );
    assert!(
        candidate_metrics
            .title_to_body_gap()
            .abs_diff(reference_metrics.title_to_body_gap())
            <= 2,
        "storybook dark sample diagrams title/body spacing diverges from KatanA reference: reference={reference_metrics:?} candidate={candidate_metrics:?}"
    );
    assert!(
        within_percent(
            candidate_metrics.body_first_height,
            reference_metrics.body_first_height,
            90,
            115
        ) && within_percent(
            candidate_metrics.body_first_width,
            reference_metrics.body_first_width,
            96,
            104
        ),
        "storybook dark sample diagrams intro body is crushed or too narrow relative to KatanA reference: reference={reference_metrics:?} candidate={candidate_metrics:?}"
    );
    assert!(
        within_percent(
            candidate_metrics.link_blue_width,
            reference_metrics.link_blue_width,
            95,
            110
        ),
        "storybook dark sample diagrams language link underline width diverges from KatanA reference: reference={reference_metrics:?} candidate={candidate_metrics:?}"
    );
    Ok(())
}

#[test]
fn storybook_sample_diagrams_heading_preserves_emoji_span_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let scene = PreviewCrop::render_score_scene("katana/sample_diagrams.md", true)?;
    let heading = find_text_node(scene.tree.root(), "KatanA Rendering")
        .ok_or("sample diagrams heading text node must be present")?;

    assert!(
        heading
            .props()
            .text
            .spans
            .iter()
            .any(|span| span.text == "🧪" && span.style.emoji),
        "sample diagrams heading must preserve the leading emoji as an OS emoji span: label={:?} spans={:?}",
        heading.props().label,
        heading.props().text.spans
    );
    Ok(())
}

#[test]
fn storybook_sample_diagrams_local_svg_metrics_match_katana_reference()
-> Result<(), Box<dyn std::error::Error>> {
    let reference = PreviewCrop::load(KATANA_SAMPLE_DIAGRAMS_CROP_REFERENCE)?;
    let candidate =
        PreviewCrop::render_storybook_top_score_crop("katana/sample_diagrams.md", true)?;
    let reference_metrics = LocalDiagramMetrics::collect(&reference)?;
    let candidate_metrics = LocalDiagramMetrics::collect(&candidate)?;

    assert!(
        within_percent(
            candidate_metrics.foreground_pixels,
            reference_metrics.foreground_pixels,
            85,
            115
        ),
        "storybook diagram foreground density diverges from KatanA reference: reference={reference_metrics:?} candidate={candidate_metrics:?}"
    );
    assert!(
        within_percent(
            candidate_metrics.edge_pixels,
            reference_metrics.edge_pixels,
            85,
            125
        ),
        "storybook diagram edge detail is too low relative to KatanA reference: reference={reference_metrics:?} candidate={candidate_metrics:?}"
    );
    assert!(
        within_percent(
            candidate_metrics.mid_tone_pixels,
            reference_metrics.mid_tone_pixels,
            85,
            125
        ),
        "storybook diagram anti-aliased mid-tone detail is too low relative to KatanA reference: reference={reference_metrics:?} candidate={candidate_metrics:?}"
    );
    assert!(
        within_percent(
            candidate_metrics.bbox_width,
            reference_metrics.bbox_width,
            95,
            105
        ),
        "storybook diagram width diverges from KatanA reference: reference={reference_metrics:?} candidate={candidate_metrics:?}"
    );
    assert!(
        within_percent(
            candidate_metrics.bbox_height,
            reference_metrics.bbox_height,
            95,
            105
        ),
        "storybook diagram height diverges from KatanA reference: reference={reference_metrics:?} candidate={candidate_metrics:?}"
    );
    Ok(())
}

#[test]
fn storybook_sample_diagrams_first_flowchart_bbox_matches_katana_reference()
-> Result<(), Box<dyn std::error::Error>> {
    let reference = PreviewCrop::load(KATANA_SAMPLE_DIAGRAMS_CROP_REFERENCE)?;
    let candidate =
        PreviewCrop::render_storybook_top_score_crop("katana/sample_diagrams.md", true)?;
    let reference_metrics = LocalDiagramBbox::collect(&reference, 300..800, 250)?;
    let candidate_metrics = LocalDiagramBbox::collect(&candidate, 300..800, 250)?;

    assert!(
        candidate_metrics.min_x.abs_diff(reference_metrics.min_x) <= 2
            && candidate_metrics
                .center_x_twice()
                .abs_diff(reference_metrics.center_x_twice())
                <= 4
            && candidate_metrics.min_y.abs_diff(reference_metrics.min_y) <= 6
            && within_percent(candidate_metrics.width, reference_metrics.width, 96, 104)
            && within_percent(candidate_metrics.height, reference_metrics.height, 96, 104),
        "storybook first flowchart bbox diverges from KatanA reference: reference={reference_metrics:?} candidate={candidate_metrics:?}"
    );
    Ok(())
}

#[test]
fn storybook_sample_diagrams_major_diagram_bboxes_match_katana_reference()
-> Result<(), Box<dyn std::error::Error>> {
    let reference = PreviewCrop::load(KATANA_SAMPLE_DIAGRAMS_CROP_REFERENCE)?;
    let candidate =
        PreviewCrop::render_storybook_top_score_crop("katana/sample_diagrams.md", true)?;
    let windows = [
        DiagramBboxWindow::new("flowchart", 300, 800, 250),
        DiagramBboxWindow::new("sequence", 850, 1_225, 250),
        DiagramBboxWindow::new("class", 1_300, 1_815, 250),
        DiagramBboxWindow::new("state", 1_900, 2_225, 250),
    ];
    let mut mismatches = Vec::new();
    let mut summaries = Vec::new();

    for window in windows {
        let reference_metrics = LocalDiagramBbox::collect(
            &reference,
            window.start_y..window.end_y,
            window.min_x_limit,
        )?;
        let candidate_metrics = LocalDiagramBbox::collect(
            &candidate,
            window.start_y..window.end_y,
            window.min_x_limit,
        )?;
        summaries.push(format!(
            "{} reference={reference_metrics:?} candidate={candidate_metrics:?}",
            window.label
        ));
        if candidate_metrics.min_x.abs_diff(reference_metrics.min_x) > 2
            || candidate_metrics
                .center_x_twice()
                .abs_diff(reference_metrics.center_x_twice())
                > 4
            || candidate_metrics.min_y.abs_diff(reference_metrics.min_y) > 6
            || !within_percent(candidate_metrics.width, reference_metrics.width, 96, 104)
            || !within_percent(candidate_metrics.height, reference_metrics.height, 96, 104)
        {
            mismatches.push(format!(
                "{} reference={reference_metrics:?} candidate={candidate_metrics:?}",
                window.label
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "storybook major diagram bboxes diverge from KatanA reference: mismatches={}; all={}",
        mismatches.join("; "),
        summaries.join("; ")
    );
    Ok(())
}

#[test]
fn storybook_sample_diagrams_gantt_tail_matches_katana_reference_crop()
-> Result<(), Box<dyn std::error::Error>> {
    let reference = PreviewCrop::load(KATANA_SAMPLE_DIAGRAMS_CROP_REFERENCE)?;
    let candidate =
        PreviewCrop::render_storybook_top_score_crop("katana/sample_diagrams.md", true)?;
    let reference_metrics =
        LocalDiagramBbox::collect_with_min_row_pixels(&reference, 2_100..2_350, 250, 100)?;
    let candidate_metrics =
        LocalDiagramBbox::collect_with_min_row_pixels(&candidate, 2_100..2_350, 250, 100)?;

    assert!(
        candidate_metrics.max_y + 6 >= reference_metrics.max_y
            && within_percent(candidate_metrics.height, reference_metrics.height, 96, 104),
        "storybook Gantt body tail is clipped relative to KatanA reference: reference={reference_metrics:?} candidate={candidate_metrics:?}"
    );
    Ok(())
}

#[test]
fn storybook_sample_diagrams_gantt_tail_bottom_block_matches_katana_reference()
-> Result<(), Box<dyn std::error::Error>> {
    let reference = PreviewCrop::load(KATANA_SAMPLE_DIAGRAMS_CROP_REFERENCE)?;
    let candidate =
        PreviewCrop::render_storybook_top_score_crop("katana/sample_diagrams.md", true)?;
    let reference_rows = bright_row_counts(&reference, 2_384..2_400, 250);
    let candidate_rows = bright_row_counts(&candidate, 2_384..2_400, 250);
    let mut mismatches = Vec::new();

    for (row, (reference_count, candidate_count)) in
        reference_rows.iter().zip(candidate_rows.iter()).enumerate()
    {
        if *reference_count >= 300 && *candidate_count * 100 < *reference_count * 80 {
            mismatches.push((row, *reference_count, *candidate_count));
        }
    }
    let max_run = max_consecutive_rows(mismatches.iter().map(|(row, _, _)| *row));

    assert!(
        max_run < 3,
        "storybook Gantt bottom tail rows disappear relative to KatanA reference: mismatches={:?}; max_run={max_run}; reference={reference_rows:?}; candidate={candidate_rows:?}; reference_profile={:?}; candidate_profile={:?}",
        mismatches,
        bright_row_profile(&reference, 2_200..2_400, 250),
        bright_row_profile(&candidate, 2_200..2_400, 250),
    );
    Ok(())
}

#[test]
fn storybook_sample_diagrams_score_scene_uses_retina_image_surfaces()
-> Result<(), Box<dyn std::error::Error>> {
    let scene = PreviewCrop::render_score_scene("katana/sample_diagrams.md", true)?;
    let plans = UiImageSurfaceRenderPlan::collect_from_tree(&scene.tree);

    assert!(plans.len() >= 5, "diagram image surfaces missing");
    assert!(
        plans.iter().all(|plan| plan.content_scale >= 100),
        "diagram surfaces must keep explicit content scale: {plans:?}"
    );
    for (index, plan) in plans.iter().enumerate() {
        eprintln!(
            "score-image-surface[{index}] physical={}x{} scale={} logical={}x{} node={}x{}",
            plan.width,
            plan.height,
            plan.content_scale,
            logical_extent(plan.width, plan.content_scale),
            logical_extent(plan.height, plan.content_scale),
            dimension_label(
                &collect_image_surface_nodes(scene.tree.root())[index]
                    .props()
                    .common
                    .width
            ),
            dimension_label(
                &collect_image_surface_nodes(scene.tree.root())[index]
                    .props()
                    .common
                    .height
            )
        );
    }
    for (index, node) in collect_image_surface_nodes(scene.tree.root())
        .iter()
        .take(5)
        .enumerate()
    {
        eprintln!(
            "score-image-foreground[{index}] {:?}",
            image_surface_foreground_bbox(node)
        );
    }
    for (index, target) in scene
        .targets
        .iter()
        .filter(|target| target.artifact_id.0.contains(":Svg"))
        .take(16)
        .enumerate()
    {
        eprintln!(
            "score-image-target[{index}] y={} h={} id={}",
            target.rect.y, target.rect.height, target.node_id.0
        );
    }
    Ok(())
}

struct PreviewCrop {
    width: usize,
    height: usize,
    rgba: Vec<u8>,
}

struct PreviewCropRender {
    crop: PreviewCrop,
    rendered_logical_width: usize,
    rendered_logical_height: usize,
    rendered_physical_width: usize,
    rendered_physical_height: usize,
    crop_physical_width: usize,
    crop_physical_height: usize,
}

#[derive(Debug)]
struct LocalTextMetrics {
    title_width: usize,
    title_height: usize,
    title_min_y: usize,
    body_first_width: usize,
    body_first_height: usize,
    body_first_min_y: usize,
    body_second_height: usize,
    link_blue_count: usize,
    link_blue_width: usize,
}

impl LocalTextMetrics {
    fn title_to_body_gap(&self) -> usize {
        self.body_first_min_y.saturating_sub(self.title_min_y)
    }
}

#[derive(Debug)]
struct LocalBadgeMetrics {
    rows: Vec<ContentBand>,
}

#[derive(Debug)]
struct LocalRuleHeadingMetrics {
    rule: ContentBand,
    heading: ContentBand,
}

fn within_percent(
    candidate: usize,
    reference: usize,
    min_percent: usize,
    max_percent: usize,
) -> bool {
    let lower = reference.saturating_mul(min_percent) / 100;
    let upper = reference.saturating_mul(max_percent).div_ceil(100);
    candidate >= lower && candidate <= upper
}

impl LocalBadgeMetrics {
    fn collect(crop: &PreviewCrop) -> Result<Self, Box<dyn std::error::Error>> {
        let rows = content_bands_by(crop, is_badge_foreground_pixel)
            .into_iter()
            .filter(|band| {
                band.min_y >= 700
                    && band.max_y <= 1_450
                    && band.height() >= 18
                    && (180..=360).contains(&band.width())
                    && band.min_x >= 400
                    && band.max_x <= 860
            })
            .collect::<Vec<_>>();
        if rows.len() < 2 {
            return Err(format!("expected at least two badge rows: {rows:?}").into());
        }
        Ok(Self { rows })
    }
}

impl LocalRuleHeadingMetrics {
    fn collect(crop: &PreviewCrop) -> Result<Self, Box<dyn std::error::Error>> {
        let rule = wide_horizontal_band(crop, 170..220, crop.width * 3 / 4)
            .ok_or("expected top horizontal rule band")?;
        let heading = content_bands(crop)
            .into_iter()
            .find(|band| {
                (230..=300).contains(&band.min_y) && band.width() >= 600 && band.height() >= 12
            })
            .ok_or("expected heading after top horizontal rule")?;
        Ok(Self { rule, heading })
    }
}

fn sample_badge_targets(
    scene: &crate::preview::PreviewScene,
) -> Vec<&katana_document_viewer::ViewerTarget> {
    scene
        .targets
        .iter()
        .filter(|target| target.source.raw.text.contains("img.shields.io/badge"))
        .collect()
}

impl LocalTextMetrics {
    fn collect(crop: &PreviewCrop) -> Result<Self, Box<dyn std::error::Error>> {
        let bands = content_bands(crop);
        Self::from_bands(crop, bands)
    }

    fn collect_dark(crop: &PreviewCrop) -> Result<Self, Box<dyn std::error::Error>> {
        let bands = dark_content_bands(crop)
            .into_iter()
            .filter(|band| band.min_y >= 24)
            .collect();
        Self::from_bands(crop, bands)
    }

    fn from_bands(
        crop: &PreviewCrop,
        bands: Vec<ContentBand>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if bands.len() < 3 {
            return Err(format!("expected title and wrapped body bands: {bands:?}").into());
        }
        let link_blue = link_blue_bounds(crop)
            .ok_or_else(|| format!("expected language link blue pixels: {bands:?}"))?;
        Ok(Self {
            title_width: bands[0].width(),
            title_height: bands[0].height(),
            title_min_y: bands[0].min_y,
            body_first_width: bands[1].width(),
            body_first_height: bands[1].height(),
            body_first_min_y: bands[1].min_y,
            body_second_height: bands[2].height(),
            link_blue_count: link_blue.count,
            link_blue_width: link_blue.width(),
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct ContentBand {
    min_x: usize,
    max_x: usize,
    min_y: usize,
    max_y: usize,
    pixels: usize,
}

impl ContentBand {
    fn new(y: usize) -> Self {
        Self {
            min_x: usize::MAX,
            max_x: 0,
            min_y: y,
            max_y: y,
            pixels: 0,
        }
    }

    fn observe(&mut self, x: usize) {
        self.min_x = self.min_x.min(x);
        self.max_x = self.max_x.max(x);
        self.pixels += 1;
    }

    fn valid(self) -> Option<Self> {
        (self.pixels >= 4 && self.pixels <= 900).then_some(self)
    }

    fn merge(&mut self, other: Self) {
        self.min_x = self.min_x.min(other.min_x);
        self.max_x = self.max_x.max(other.max_x);
        self.min_y = self.min_y.min(other.min_y);
        self.max_y = self.max_y.max(other.max_y);
        self.pixels += other.pixels;
    }

    fn height(self) -> usize {
        self.max_y - self.min_y + 1
    }

    fn width(self) -> usize {
        self.max_x - self.min_x + 1
    }
}

fn wide_horizontal_band(
    crop: &PreviewCrop,
    y_range: std::ops::Range<usize>,
    min_pixels: usize,
) -> Option<ContentBand> {
    let mut current: Option<ContentBand> = None;
    let mut bands = Vec::new();
    for y in y_range {
        let mut row = ContentBand::new(y);
        for x in 0..crop.width {
            if is_non_background_pixel(crop, x, y) {
                row.observe(x);
            }
        }
        if row.pixels < min_pixels {
            if let Some(band) = current.take() {
                bands.push(band);
            }
            continue;
        }
        match &mut current {
            Some(band) if row.min_y <= band.max_y + 1 => band.merge(row),
            Some(_) => {
                if let Some(band) = current.replace(row) {
                    bands.push(band);
                }
            }
            None => current = Some(row),
        }
    }
    if let Some(band) = current {
        bands.push(band);
    }
    bands.into_iter().max_by_key(|band| band.pixels)
}

#[derive(Clone, Copy, Debug)]
struct BlueBounds {
    min_x: usize,
    max_x: usize,
    count: usize,
}

impl BlueBounds {
    fn width(self) -> usize {
        self.max_x - self.min_x + 1
    }
}

#[derive(Debug)]
struct LocalDiagramMetrics {
    foreground_pixels: usize,
    edge_pixels: usize,
    mid_tone_pixels: usize,
    bbox_width: usize,
    bbox_height: usize,
}

#[derive(Debug)]
struct LocalDiagramBbox {
    min_x: usize,
    max_x: usize,
    min_y: usize,
    max_y: usize,
    width: usize,
    height: usize,
}

#[derive(Clone, Copy)]
struct DiagramBboxWindow {
    label: &'static str,
    start_y: usize,
    end_y: usize,
    min_x_limit: usize,
}

impl DiagramBboxWindow {
    const fn new(label: &'static str, start_y: usize, end_y: usize, min_x_limit: usize) -> Self {
        Self {
            label,
            start_y,
            end_y,
            min_x_limit,
        }
    }
}

impl LocalDiagramBbox {
    fn collect(
        crop: &PreviewCrop,
        y_range: std::ops::Range<usize>,
        min_x_limit: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Self::collect_impl(crop, y_range, min_x_limit, 1)
    }

    fn collect_with_min_row_pixels(
        crop: &PreviewCrop,
        y_range: std::ops::Range<usize>,
        min_x_limit: usize,
        min_row_pixels: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Self::collect_impl(crop, y_range, min_x_limit, min_row_pixels)
    }

    fn collect_impl(
        crop: &PreviewCrop,
        y_range: std::ops::Range<usize>,
        min_x_limit: usize,
        min_row_pixels: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut min_x = usize::MAX;
        let mut min_y = usize::MAX;
        let mut max_x = 0usize;
        let mut max_y = 0usize;
        let mut pixels = 0usize;
        for y in y_range {
            let mut row_pixels = 0usize;
            let mut row_min_x = usize::MAX;
            let mut row_max_x = 0usize;
            for x in min_x_limit..crop.width {
                if crop.brightness(x, y) <= 54 {
                    continue;
                }
                row_pixels += 1;
                row_min_x = row_min_x.min(x);
                row_max_x = row_max_x.max(x);
            }
            if row_pixels < min_row_pixels {
                continue;
            }
            pixels += row_pixels;
            min_x = min_x.min(row_min_x);
            min_y = min_y.min(y);
            max_x = max_x.max(row_max_x);
            max_y = max_y.max(y);
        }
        if pixels == 0 {
            return Err("expected diagram pixels in bbox window".into());
        }
        Ok(Self {
            min_x,
            max_x,
            min_y,
            max_y,
            width: max_x - min_x + 1,
            height: max_y - min_y + 1,
        })
    }

    fn center_x_twice(&self) -> usize {
        self.min_x + self.max_x
    }
}

impl LocalDiagramMetrics {
    fn collect(crop: &PreviewCrop) -> Result<Self, Box<dyn std::error::Error>> {
        let scan_top = crop.height / 3;
        let mut foreground_pixels = 0usize;
        let mut edge_pixels = 0usize;
        let mut mid_tone_pixels = 0usize;
        let mut min_x = usize::MAX;
        let mut min_y = usize::MAX;
        let mut max_x = 0usize;
        let mut max_y = 0usize;

        for y in scan_top..crop.height {
            for x in 0..crop.width {
                let brightness = crop.brightness(x, y);
                if brightness <= 70 {
                    continue;
                }
                foreground_pixels += 1;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                if (85..=190).contains(&brightness) {
                    mid_tone_pixels += 1;
                }
                let horizontal_edge =
                    x + 1 < crop.width && brightness.abs_diff(crop.brightness(x + 1, y)) >= 24;
                let vertical_edge =
                    y + 1 < crop.height && brightness.abs_diff(crop.brightness(x, y + 1)) >= 24;
                if horizontal_edge || vertical_edge {
                    edge_pixels += 1;
                }
            }
        }

        if foreground_pixels == 0 {
            return Err("expected diagram foreground pixels in sample diagrams crop".into());
        }
        Ok(Self {
            foreground_pixels,
            edge_pixels,
            mid_tone_pixels,
            bbox_width: max_x - min_x + 1,
            bbox_height: max_y - min_y + 1,
        })
    }
}

impl PreviewCrop {
    fn load(relative_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let image = image::open(workspace_root()?.join(relative_path))?.to_rgba8();
        Ok(Self {
            width: image.width() as usize,
            height: image.height() as usize,
            rgba: image.into_raw(),
        })
    }

    fn render_storybook_top_score_crop(
        path: &str,
        dark: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::render_storybook_top_score_crop_info(path, dark)?.crop)
    }

    fn render_storybook_top_score_crop_info(
        path: &str,
        dark: bool,
    ) -> Result<PreviewCropRender, Box<dyn std::error::Error>> {
        let capture = ScoreCapture::for_fixture(path);
        Self::render_storybook_top_with_canvas_scales(
            path,
            (capture.layout_scale, capture.raster_scale),
            (capture.logical_width, capture.logical_height),
            (SURFACE_WIDTH, CROP_HEIGHT),
            dark,
        )
    }

    fn render_storybook_top_with_canvas_scales(
        path: &str,
        canvas_scales: (f32, f32),
        render_size: (usize, usize),
        crop_size: (usize, usize),
        dark: bool,
    ) -> Result<PreviewCropRender, Box<dyn std::error::Error>> {
        let (layout_scale, raster_scale) = canvas_scales;
        let (render_width, render_height) = render_size;
        let (crop_width, crop_height) = crop_size;
        let scene = Self::render_score_scene_for_viewport(path, render_width, render_height, dark)?;
        let background = StorybookPalette::new(dark).preview_background();
        let mut canvas = Canvas::new_scaled_with_raster_scale(
            render_width,
            render_height,
            layout_scale,
            raster_scale,
            background,
        )
        .with_reference_capture_image_surface_extents();
        UiTreeStorybookHost::new(scene.theme.clone()).render(
            &mut canvas,
            scene.tree.root(),
            SurfaceArea {
                x: 0,
                y: 0,
                width: render_width,
                height: render_height,
                scroll_y: 0.0,
            },
        );
        let physical_width = canvas.width();
        let physical_height = canvas.height();
        let rgba = canvas_to_logical_rgba(&canvas, crop_width, crop_height);
        let crop = Self {
            width: crop_width,
            height: crop_height,
            rgba,
        };
        Ok(PreviewCropRender {
            crop,
            rendered_logical_width: render_width,
            rendered_logical_height: render_height,
            rendered_physical_width: physical_width,
            rendered_physical_height: physical_height,
            crop_physical_width: physical_width,
            crop_physical_height: physical_height,
        })
    }

    fn render_score_scene(
        path: &str,
        dark: bool,
    ) -> Result<crate::preview::PreviewScene, Box<dyn std::error::Error>> {
        Self::render_score_scene_for_viewport(path, SURFACE_WIDTH, CROP_HEIGHT, dark)
    }

    fn render_score_scene_for_viewport(
        path: &str,
        width: usize,
        height: usize,
        dark: bool,
    ) -> Result<crate::preview::PreviewScene, Box<dyn std::error::Error>> {
        let fixture = StorybookFixture {
            label: path.to_string(),
            path: workspace_root()?.join(format!("assets/fixtures/{path}")),
        };
        PreviewBuilder::default().build_with_typography_and_interaction(
            &fixture,
            ViewerViewport {
                width: width as f32,
                height: height as f32,
            },
            dark,
            ViewerTypographyConfig {
                preview_font_size: PREVIEW_FONT_SIZE,
            },
            score_visual_interaction(),
        )
    }

    fn pixel(&self, x: usize, y: usize) -> [u8; 4] {
        let index = (y * self.width + x) * 4;
        [
            self.rgba[index],
            self.rgba[index + 1],
            self.rgba[index + 2],
            self.rgba[index + 3],
        ]
    }

    fn brightness(&self, x: usize, y: usize) -> u8 {
        let [red, green, blue, _alpha] = self.pixel(x, y);
        ((u16::from(red) + u16::from(green) + u16::from(blue)) / 3) as u8
    }
}

#[derive(Clone, Copy)]
struct ScoreCapture {
    logical_width: usize,
    logical_height: usize,
    layout_scale: f32,
    raster_scale: f32,
}

impl ScoreCapture {
    fn for_fixture(path: &str) -> Self {
        if path == "katana/sample_diagrams.md" {
            return Self {
                logical_width: KATANA_PREVIEW_CONTENT_LOGICAL_WIDTH,
                logical_height: KATANA_PREVIEW_CONTENT_LOGICAL_HEIGHT,
                layout_scale: KATANA_REFERENCE_DEVICE_SCALE,
                raster_scale: KATANA_REFERENCE_DEVICE_SCALE,
            };
        }
        Self {
            logical_width: SURFACE_WIDTH,
            logical_height: CROP_HEIGHT,
            layout_scale: LEGACY_SCORE_RENDER_SCALE,
            raster_scale: KATANA_REFERENCE_DEVICE_SCALE,
        }
    }
}

fn score_visual_interaction() -> ViewerInteractionConfig {
    ViewerInteractionConfig {
        hover_highlight_enabled: false,
        selection_enabled: false,
        image_controls_enabled: false,
        diagram_controls_enabled: false,
        code_controls_enabled: false,
    }
}

fn bright_row_counts(
    crop: &PreviewCrop,
    y_range: std::ops::Range<usize>,
    min_x_limit: usize,
) -> Vec<usize> {
    y_range
        .map(|y| {
            (min_x_limit..crop.width)
                .filter(|x| crop.brightness(*x, y) > 54)
                .count()
        })
        .collect()
}

fn max_consecutive_rows(rows: impl IntoIterator<Item = usize>) -> usize {
    let mut previous = None;
    let mut current = 0usize;
    let mut max_run = 0usize;
    for row in rows {
        if previous.is_some_and(|previous| row == previous + 1) {
            current += 1;
        } else {
            current = 1;
        }
        previous = Some(row);
        max_run = max_run.max(current);
    }
    max_run
}

fn bright_row_profile(
    crop: &PreviewCrop,
    y_range: std::ops::Range<usize>,
    min_x_limit: usize,
) -> Vec<(usize, usize)> {
    y_range
        .step_by(8)
        .map(|y| {
            let count = (min_x_limit..crop.width)
                .filter(|x| crop.brightness(*x, y) > 54)
                .count();
            (y, count)
        })
        .collect()
}

fn assert_preview_crop_score(
    reference_path: &str,
    fixture: &str,
    dump_name: &str,
    dark: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let reference = PreviewCrop::load(reference_path)?;
    let candidate = PreviewCrop::render_storybook_top_score_crop(fixture, dark)?;
    let report = SurfaceParityScorer::report_with_dimensions(
        &reference.rgba,
        &candidate.rgba,
        reference.width,
        reference.height,
        candidate.width,
        candidate.height,
    );
    dump_pair(dump_name, &reference, &candidate)?;
    assert!(
        report.score >= VISUAL_SCORE_THRESHOLD,
        "storybook preview crop visual_score is {}/{} for {}; average={} content={} row={} r2c={} c2r={} loss_bands={:?} dimension={} reference={}x{} candidate={}x{}",
        report.score,
        VISUAL_SCORE_THRESHOLD,
        fixture,
        report.average_score,
        report.content_score,
        report.row_score,
        report.reference_to_candidate_row_score,
        report.candidate_to_reference_row_score,
        report.reference_row_loss_bands,
        report.dimension_score,
        reference.width,
        reference.height,
        candidate.width,
        candidate.height
    );
    Ok(())
}

fn dump_pair(name: &str, reference: &PreviewCrop, candidate: &PreviewCrop) -> std::io::Result<()> {
    let Ok(directory) = std::env::var("KDV_STORYBOOK_PREVIEW_CROP_DUMP_DIR") else {
        return Ok(());
    };
    SurfaceDump::write_pair(
        Path::new(&directory),
        name,
        SurfaceDumpImage::new(&reference.rgba, reference.width, reference.height),
        SurfaceDumpImage::new(&candidate.rgba, candidate.width, candidate.height),
    )
}

fn workspace_root() -> Result<PathBuf, std::io::Error> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| std::io::Error::other("storybook crate must be under workspace tools"))
}

fn action_count(node: &UiNode, action: &str) -> usize {
    usize::from(node.props().interaction.value == action)
        + node
            .children()
            .iter()
            .map(|child| action_count(child, action))
            .sum::<usize>()
}

fn assert_no_overlay_controls(node: &UiNode, fixture: &str) {
    assert_eq!(
        0,
        action_count(node, "copy-code"),
        "score crop must exclude code copy controls for {fixture}"
    );
    assert_eq!(
        0,
        action_count(node, "copy-source"),
        "score crop must exclude diagram copy controls for {fixture}"
    );
    assert_eq!(
        0,
        action_count(node, "fullscreen"),
        "score crop must exclude diagram fullscreen controls for {fixture}"
    );
    assert_eq!(
        0,
        action_count(node, "zoom-in"),
        "score crop must exclude diagram zoom controls for {fixture}"
    );
    assert_eq!(
        0,
        action_count(node, "reset-view"),
        "score crop must exclude diagram reset controls for {fixture}"
    );
    let mut host_actions = Vec::new();
    collect_host_actions(node, &mut host_actions);
    assert!(
        host_actions
            .iter()
            .all(|action| !action.starts_with("diagram:") && !action.starts_with("code:")),
        "score crop must not include media overlay host actions for {fixture}: {host_actions:?}"
    );
}

fn collect_host_actions(node: &UiNode, actions: &mut Vec<String>) {
    actions.extend(
        node.props()
            .common
            .host_actions
            .iter()
            .map(|action| action.action_id.clone()),
    );
    for child in node.children() {
        collect_host_actions(child, actions);
    }
}

fn collect_image_surface_nodes(node: &UiNode) -> Vec<&UiNode> {
    let mut nodes = Vec::new();
    collect_image_surface_nodes_into(node, &mut nodes);
    nodes
}

fn find_text_node<'a>(node: &'a UiNode, label_fragment: &str) -> Option<&'a UiNode> {
    if node.props().label.contains(label_fragment)
        && node.kind() == katana_ui_core::render_model::UiNodeKind::Text
    {
        return Some(node);
    }
    node.children()
        .iter()
        .find_map(|child| find_text_node(child, label_fragment))
}

fn collect_image_surface_nodes_into<'a>(node: &'a UiNode, nodes: &mut Vec<&'a UiNode>) {
    if node.kind() == katana_ui_core::render_model::UiNodeKind::ImageSurface {
        nodes.push(node);
    }
    for child in node.children() {
        collect_image_surface_nodes_into(child, nodes);
    }
}

fn dimension_label(value: &katana_ui_core::render_model::UiDimension) -> String {
    match value {
        katana_ui_core::render_model::UiDimension::Px(value) => value.to_string(),
        katana_ui_core::render_model::UiDimension::Auto => "auto".to_string(),
        katana_ui_core::render_model::UiDimension::Percent(value) => format!("{value}%"),
        katana_ui_core::render_model::UiDimension::Fill => "fill".to_string(),
        katana_ui_core::render_model::UiDimension::FitContent => "fit-content".to_string(),
        katana_ui_core::render_model::UiDimension::Token(value) => value.clone(),
    }
}

fn image_surface_foreground_bbox(node: &UiNode) -> Option<(u32, u32, u32, u32, usize)> {
    let image = &node.props().image_surface;
    let mut left = image.width;
    let mut top = image.height;
    let mut right = 0;
    let mut bottom = 0;
    let mut count = 0;
    for y in 0..image.height {
        for x in 0..image.width {
            let index = ((y * image.width + x) * 4) as usize;
            let red = image.rgba[index];
            let green = image.rgba[index + 1];
            let blue = image.rgba[index + 2];
            let alpha = image.rgba[index + 3];
            if alpha > 16 && u16::from(red) + u16::from(green) + u16::from(blue) > 130 {
                left = left.min(x);
                top = top.min(y);
                right = right.max(x);
                bottom = bottom.max(y);
                count += 1;
            }
        }
    }
    (count > 0).then_some((left, top, right, bottom, count))
}

fn canvas_to_logical_rgba(canvas: &Canvas, width: usize, height: usize) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(canvas.width() * canvas.height() * 4);
    for color in canvas.pixels() {
        rgba.push(((color >> 16) & 0xff) as u8);
        rgba.push(((color >> 8) & 0xff) as u8);
        rgba.push((color & 0xff) as u8);
        rgba.push(0xff);
    }
    let Some(image) =
        image::RgbaImage::from_raw(canvas.width() as u32, canvas.height() as u32, rgba)
    else {
        return vec![0xff; width * height * 4];
    };
    image::imageops::resize(
        &image,
        width as u32,
        height as u32,
        image::imageops::FilterType::Triangle,
    )
    .into_raw()
}

fn content_bands(crop: &PreviewCrop) -> Vec<ContentBand> {
    content_bands_by(crop, is_text_like_pixel)
}

fn dark_content_bands(crop: &PreviewCrop) -> Vec<ContentBand> {
    content_bands_by(crop, is_dark_foreground_pixel)
}

fn content_bands_by(
    crop: &PreviewCrop,
    pixel_filter: fn(&PreviewCrop, usize, usize) -> bool,
) -> Vec<ContentBand> {
    let mut bands = Vec::new();
    let mut current: Option<ContentBand> = None;
    for y in 0..crop.height {
        let mut row = ContentBand::new(y);
        for x in 0..crop.width {
            if pixel_filter(crop, x, y) {
                row.observe(x);
            }
        }
        let Some(row) = row.valid() else {
            if let Some(band) = current.take() {
                bands.push(band);
            }
            continue;
        };
        match &mut current {
            Some(band) if row.min_y <= band.max_y + 2 => band.merge(row),
            Some(_) => {
                if let Some(band) = current.replace(row) {
                    bands.push(band);
                }
            }
            None => current = Some(row),
        }
    }
    if let Some(band) = current {
        bands.push(band);
    }
    bands
}

fn link_blue_bounds(crop: &PreviewCrop) -> Option<BlueBounds> {
    let mut bounds: Option<BlueBounds> = None;
    let mut count = 0usize;
    for y in 0..(crop.height / 3) {
        for x in 0..crop.width {
            if !is_link_blue_pixel(crop, x, y) {
                continue;
            }
            count += 1;
            bounds = Some(match bounds {
                Some(current) => BlueBounds {
                    min_x: current.min_x.min(x),
                    max_x: current.max_x.max(x),
                    count,
                },
                None => BlueBounds {
                    min_x: x,
                    max_x: x,
                    count,
                },
            });
        }
    }
    bounds
}

fn is_text_like_pixel(crop: &PreviewCrop, x: usize, y: usize) -> bool {
    let [red, green, blue, _alpha] = crop.pixel(x, y);
    let min_channel = red.min(green).min(blue);
    let max_channel = red.max(green).max(blue);
    min_channel < 235 && !(max_channel - min_channel <= 4 && min_channel > 210)
}

fn is_badge_foreground_pixel(crop: &PreviewCrop, x: usize, y: usize) -> bool {
    let [red, green, blue, _alpha] = crop.pixel(x, y);
    let distance_from_white =
        255usize - red as usize + 255usize - green as usize + 255usize - blue as usize;
    distance_from_white > 24
}

fn is_non_background_pixel(crop: &PreviewCrop, x: usize, y: usize) -> bool {
    let [red, green, blue, _alpha] = crop.pixel(x, y);
    let distance_from_white =
        255usize - red as usize + 255usize - green as usize + 255usize - blue as usize;
    distance_from_white > 18
}

fn is_link_blue_pixel(crop: &PreviewCrop, x: usize, y: usize) -> bool {
    let [red, green, blue, _alpha] = crop.pixel(x, y);
    let red = red as i16;
    let green = green as i16;
    let blue = blue as i16;
    blue > 90 && blue > red + 20 && blue > green + 5
}

fn is_dark_foreground_pixel(crop: &PreviewCrop, x: usize, y: usize) -> bool {
    crop.brightness(x, y) > 54
}

fn right_edge_scrollbar_pixels(crop: &PreviewCrop) -> usize {
    let left = crop.width.saturating_sub(16);
    let mut count = 0;
    for y in 0..crop.height {
        for x in left..crop.width {
            let index = (y * crop.width + x) * 4;
            if crop.rgba[index] == 142 && crop.rgba[index + 1] == 142 && crop.rgba[index + 2] == 142
            {
                count += 1;
            }
        }
    }
    count
}

fn logical_extent(physical_extent: u32, content_scale: u32) -> u32 {
    let scale = u64::from(content_scale.max(1));
    ((u64::from(physical_extent) * 100).div_ceil(scale) as u32).max(1)
}
