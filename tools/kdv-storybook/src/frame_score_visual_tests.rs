use super::frame_surface_dump::{SurfaceDump, SurfaceDumpImage};
use super::frame_surface_similarity::SurfaceParityScorer;
use crate::catalog::StorybookFixture;
use crate::preview::PreviewBuilder;
use katana_document_viewer::{
    BuildProfile, BuildRequest, DiagramRenderingBackend, DocumentSnapshotFactory, DocumentSource,
    ExportFormat, ExportRequest, ForgePipeline, KdvThemeSnapshot, KrrDiagramRenderEngine,
    MarkdownFenceNormalizer, SourceKind, SourceRevision, SourceUri, ViewerTypographyConfig,
    ViewerViewport,
};
use katana_markdown_model::{KatanaMarkdownModel, MarkdownInput};
use katana_ui_core::render_model::UiNode;
use std::path::{Path, PathBuf};

const VISUAL_SCORE_THRESHOLD: u8 = 95;
const SURFACE_WIDTH: usize = 1280;
const PREVIEW_HEIGHT: f32 = 20_000.0;
const KATANA_REFERENCE_PREVIEW_FONT_SIZE: u16 = 24;
const CONTRAST_SAMPLE_HEIGHT: u32 = 900;
const MIN_REFERENCE_LUMA_STDDEV: f64 = 0.08;
const KATANA_EXPORT_PNG_REFERENCE: &str = "assets/reference/katana/export_png/sample.png";

#[test]
fn storybook_score_visual_uses_katana_export_png_reference()
-> Result<(), Box<dyn std::error::Error>> {
    let reference = ReferenceSurface::load(KATANA_EXPORT_PNG_REFERENCE)?;
    reference.assert_readable_foreground();
    let candidate = StorybookSurface::render_native_export("katana/sample.md")?;
    let report = SurfaceParityScorer::report_with_dimensions(
        &reference.rgba,
        &candidate.rgba,
        reference.width as usize,
        reference.height as usize,
        candidate.width as usize,
        candidate.height as usize,
    );
    if let Ok(directory) = std::env::var("KDV_STORYBOOK_SCORE_DUMP_DIR") {
        SurfaceDump::write_pair(
            Path::new(&directory),
            "katana/sample.md",
            SurfaceDumpImage::new(
                &reference.rgba,
                reference.width as usize,
                reference.height as usize,
            ),
            SurfaceDumpImage::new(
                &candidate.rgba,
                candidate.width as usize,
                candidate.height as usize,
            ),
        )?;
    }

    assert!(
        report.score >= VISUAL_SCORE_THRESHOLD,
        "storybook visual_score is {}/{}; average={} content={} dimension={} row={} ref_to_candidate_row={} candidate_to_reference_row={} loss_bands={:?} reference={}x{} candidate={}x{}",
        report.score,
        VISUAL_SCORE_THRESHOLD,
        report.average_score,
        report.content_score,
        report.dimension_score,
        report.row_score,
        report.reference_to_candidate_row_score,
        report.candidate_to_reference_row_score,
        report.reference_row_loss_bands,
        reference.width,
        reference.height,
        candidate.width,
        candidate.height
    );
    Ok(())
}

#[test]
fn storybook_score_export_surface_excludes_overlay_controls()
-> Result<(), Box<dyn std::error::Error>> {
    let diagrams = StorybookSurface::render_scene("katana/sample_diagrams.md")?;
    assert_no_overlay_controls(diagrams.tree.root(), "katana/sample_diagrams.md");

    let code_blocks = StorybookSurface::render_scene("katana/sample_basic.md")?;
    assert_no_overlay_controls(code_blocks.tree.root(), "katana/sample_basic.md");
    Ok(())
}

struct ReferenceSurface {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl ReferenceSurface {
    fn load(relative_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = workspace_root()?.join(relative_path);
        let image = image::open(path)?.to_rgba8();
        Ok(Self {
            width: image.width(),
            height: image.height(),
            rgba: image.into_raw(),
        })
    }

    fn assert_readable_foreground(&self) {
        let sample_height = self.height.min(CONTRAST_SAMPLE_HEIGHT);
        let stats = LumaStats::from_top_rows(&self.rgba, self.width, sample_height);
        assert!(
            stats.stddev >= MIN_REFERENCE_LUMA_STDDEV,
            "KatanA reference screenshot foreground contrast is too low: luma_stddev={:.4} min={MIN_REFERENCE_LUMA_STDDEV}; reference={}x{}",
            stats.stddev,
            self.width,
            self.height
        );
    }
}

struct LumaStats {
    stddev: f64,
}

impl LumaStats {
    fn from_top_rows(rgba: &[u8], width: u32, height: u32) -> Self {
        let pixel_count = width as usize * height as usize;
        let mut values = Vec::with_capacity(pixel_count);
        for pixel in rgba.as_chunks::<4>().0.iter().take(pixel_count) {
            values.push(Self::luma(pixel));
        }
        let mean = values.iter().sum::<f64>() / values.len().max(1) as f64;
        let variance = values
            .iter()
            .map(|value| {
                let delta = value - mean;
                delta * delta
            })
            .sum::<f64>()
            / values.len().max(1) as f64;
        Self {
            stddev: variance.sqrt(),
        }
    }

    fn luma(pixel: &[u8]) -> f64 {
        let red = pixel[0] as f64 / 255.0;
        let green = pixel[1] as f64 / 255.0;
        let blue = pixel[2] as f64 / 255.0;
        red * 0.2126 + green * 0.7152 + blue * 0.0722
    }
}

struct StorybookSurface {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl StorybookSurface {
    fn render_native_export(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = workspace_root()?.join(format!("assets/fixtures/{path}"));
        let source = std::fs::read_to_string(&path)?;
        let content =
            MarkdownFenceNormalizer::normalize(&source.replace("\r\n", "\n").replace('\r', "\n"));
        let document = KatanaMarkdownModel::parse(MarkdownInput::from_content(
            path.display().to_string(),
            content.clone(),
        ))?;
        let snapshot = DocumentSnapshotFactory::from_kmm(
            DocumentSource {
                uri: SourceUri(format!("file://{}", path.display())),
                kind: SourceKind::Markdown,
                revision: SourceRevision("storybook-score".to_string()),
                content,
            },
            document,
        );
        let theme = KdvThemeSnapshot::katana_light();
        let pipeline = ForgePipeline::new(DiagramRenderingBackend::new(KrrDiagramRenderEngine));
        let graph = pipeline.build(&BuildRequest {
            snapshot,
            profile: BuildProfile::markdown_export(),
            theme: theme.clone(),
        })?;
        let output = pipeline.export(&ExportRequest {
            graph,
            format: ExportFormat::Png,
            theme,
        })?;
        let image = image::load_from_memory(&output.artifact.bytes.bytes)?.to_rgba8();
        Ok(Self {
            width: image.width(),
            height: image.height(),
            rgba: image.into_raw(),
        })
    }

    fn render_scene(
        path: &str,
    ) -> Result<crate::preview::PreviewScene, Box<dyn std::error::Error>> {
        let fixture = StorybookFixture {
            label: path.to_string(),
            path: workspace_root()?.join(format!("assets/fixtures/{path}")),
        };
        PreviewBuilder::default().build_surface_with_export_reference_theme(
            &fixture,
            ViewerViewport {
                width: SURFACE_WIDTH as f32,
                height: PREVIEW_HEIGHT,
            },
            ViewerTypographyConfig {
                preview_font_size: KATANA_REFERENCE_PREVIEW_FONT_SIZE,
            },
        )
    }
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
        "export score surface must exclude code copy controls for {fixture}"
    );
    assert_eq!(
        0,
        action_count(node, "copy-source"),
        "export score surface must exclude diagram copy controls for {fixture}"
    );
    assert_eq!(
        0,
        action_count(node, "fullscreen"),
        "export score surface must exclude diagram fullscreen controls for {fixture}"
    );
    assert_eq!(
        0,
        action_count(node, "zoom-in"),
        "export score surface must exclude diagram zoom controls for {fixture}"
    );
    assert_eq!(
        0,
        action_count(node, "reset-view"),
        "export score surface must exclude diagram reset controls for {fixture}"
    );
    let mut host_actions = Vec::new();
    collect_host_actions(node, &mut host_actions);
    assert!(
        host_actions
            .iter()
            .all(|action| !action.starts_with("diagram:") && !action.starts_with("code:")),
        "export score surface must not include media overlay host actions for {fixture}: {host_actions:?}"
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

fn workspace_root() -> Result<PathBuf, std::io::Error> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            std::io::Error::other("storybook crate must be under workspace tools directory")
        })
}
