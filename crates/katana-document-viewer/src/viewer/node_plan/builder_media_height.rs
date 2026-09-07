use super::metrics::ViewerNodeMetrics;
use super::planned_node::PlannedNode;
use super::types::{ViewerHtmlRole, ViewerNodeKind};
use crate::Artifact;
use crate::forge::BuildGraph;
use crate::preview_surface::{
    KDV_INTERACTIVE_PREVIEW_SURFACE_HORIZONTAL_PADDING_PX, KDV_VIEWER_SURFACE_PADDING_PX,
};
use crate::viewer::settings_update::ViewerTypographyConfig;
use katana_markdown_model::KmmNode;

const SURFACE_CONTENT_WIDTH: u32 = 1168;
const DIAGRAM_MAX_WIDTH: u32 = 860;
const MATH_MAX_WIDTH: u32 = 760;
const MEDIA_VERTICAL_MARGIN: f32 = 34.0;
const DIAGRAM_FALLBACK_HEIGHT: f32 = 156.0;
const MATH_FALLBACK_HEIGHT: f32 = 74.0;

#[derive(Clone, Copy)]
struct MediaHeightContext<'a> {
    graph: Option<&'a BuildGraph>,
    artifacts: &'a [Artifact],
    node: &'a KmmNode,
    planned: &'a PlannedNode,
    typography: ViewerTypographyConfig,
    content_width: u32,
    height_mode: ViewerHeightMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ViewerHeightMode {
    InteractivePreview,
    ExportSurface,
}

pub(super) struct ViewerMediaHeight;

impl ViewerMediaHeight {
    pub(super) fn block_height(
        graph: Option<&BuildGraph>,
        artifacts: &[Artifact],
        node: &KmmNode,
        planned: &PlannedNode,
        typography: ViewerTypographyConfig,
        viewport_width: f32,
        height_mode: ViewerHeightMode,
    ) -> f32 {
        let content_width = Self::content_width(viewport_width, height_mode);
        let context = MediaHeightContext {
            graph,
            artifacts,
            node,
            planned,
            typography,
            content_width,
            height_mode,
        };
        if let Some(height) = Self::precomputed_height(context) {
            return height;
        }
        Self::kind_height(context)
    }

    fn precomputed_height(context: MediaHeightContext<'_>) -> Option<f32> {
        if let Some(height) =
            data_image::HtmlDataImageHeight::height(context.planned, context.content_width)
        {
            return Some(height);
        }
        if context.height_mode == ViewerHeightMode::ExportSurface
            && let Some(height) = Self::preferred_surface_height(
                context.graph,
                context.node,
                context.planned,
                context.typography,
            )
        {
            return Some(height);
        }
        Self::text_height(context.planned, context.typography, context.content_width)
    }

    fn kind_height(context: MediaHeightContext<'_>) -> f32 {
        match &context.planned.kind {
            ViewerNodeKind::Diagram { kind } => Self::diagram_or_fallback_height(
                context.artifacts,
                context.planned,
                *kind,
                DIAGRAM_FALLBACK_HEIGHT,
                Self::diagram_layout_width(context),
                context.height_mode,
            ),
            ViewerNodeKind::Math => {
                Self::svg_height(context.artifacts, context.planned, MATH_MAX_WIDTH)
                    .unwrap_or(MATH_FALLBACK_HEIGHT)
            }
            ViewerNodeKind::Image => {
                Self::image_or_text_height(context.artifacts, context.planned, context.typography)
            }
            ViewerNodeKind::Html {
                role: ViewerHtmlRole::Accordion,
            } => Self::accordion_or_default_height(context.planned, context.typography),
            kind => Self::fallback_node_height(
                context.graph,
                context.node,
                context.planned,
                context.typography,
                context.content_width,
                context.height_mode,
                kind,
            ),
        }
    }

    fn diagram_or_fallback_height(
        artifacts: &[Artifact],
        planned: &PlannedNode,
        kind: super::types::ViewerDiagramKind,
        fallback_height: f32,
        content_width: u32,
        height_mode: ViewerHeightMode,
    ) -> f32 {
        Self::diagram_height(artifacts, planned, kind, content_width, height_mode)
            .unwrap_or(fallback_height)
    }

    fn image_or_text_height(
        artifacts: &[Artifact],
        planned: &PlannedNode,
        typography: ViewerTypographyConfig,
    ) -> f32 {
        Self::image_height(artifacts, planned).unwrap_or_else(|| {
            ViewerNodeMetrics::block_height(&planned.kind, &planned.text, typography)
        })
    }

    fn accordion_or_default_height(
        planned: &PlannedNode,
        typography: ViewerTypographyConfig,
    ) -> f32 {
        Self::accordion_height(planned, typography)
            .unwrap_or_else(|| ViewerNodeMetrics::body_line_height(typography) * 2.0)
    }

    fn content_width(viewport_width: f32, height_mode: ViewerHeightMode) -> u32 {
        let width = viewport_width.round().max(1.0) as u32;
        let horizontal_padding = match height_mode {
            ViewerHeightMode::InteractivePreview => {
                u32::from(KDV_INTERACTIVE_PREVIEW_SURFACE_HORIZONTAL_PADDING_PX) * 2
            }
            ViewerHeightMode::ExportSurface => u32::from(KDV_VIEWER_SURFACE_PADDING_PX) * 2,
        };
        width.saturating_sub(horizontal_padding).max(1)
    }

    fn diagram_layout_width(context: MediaHeightContext<'_>) -> u32 {
        match context.height_mode {
            ViewerHeightMode::InteractivePreview => context.content_width.max(1),
            ViewerHeightMode::ExportSurface => context.content_width.max(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ViewerHeightMode, ViewerMediaHeight, ViewerNodeMetrics};
    use crate::artifact::{ArtifactBytes, ArtifactDiagnostics, ArtifactFactory, ArtifactFormat};
    use crate::viewer::asset::ViewerAssetReference;
    use crate::viewer::node_plan::planned_node::PlannedNode;
    use crate::viewer::node_plan::types::{ViewerDiagramKind, ViewerNodeKind};
    use crate::{ArtifactId, DocumentId, SourceRevision, ViewerTypographyConfig};
    use katana_markdown_model::{
        ByteRange, CodeBlockRole, DiagramKind, KmmNode, KmmNodeId, KmmNodeKind, LineColumn,
        LineColumnRange, RawSnippet, SourceSpan,
    };

    #[test]
    fn interactive_content_width_matches_katana_preview_symmetric_margin_width() {
        assert_eq!(
            1256,
            ViewerMediaHeight::content_width(1280.0, ViewerHeightMode::InteractivePreview)
        );
    }

    #[test]
    fn accordion_height_falls_back_for_non_accordion_plan() {
        let planned = PlannedNode {
            node_id: KmmNodeId("node-paragraph".to_string()),
            kind: ViewerNodeKind::Paragraph,
            source: source("paragraph"),
            text: "paragraph".to_string(),
            spans: Vec::new(),
            reference: None,
        };
        let typography = ViewerTypographyConfig {
            preview_font_size: 14,
        };

        assert_eq!(
            ViewerNodeMetrics::body_line_height(typography) * 2.0,
            ViewerMediaHeight::accordion_or_default_height(&planned, typography)
        );
    }

    #[test]
    fn export_content_width_keeps_page_padding() {
        assert_eq!(
            1168,
            ViewerMediaHeight::content_width(1280.0, ViewerHeightMode::ExportSurface)
        );
    }

    #[test]
    fn interactive_diagram_height_uses_viewer_row_width_without_upscaling()
    -> Result<(), Box<dyn std::error::Error>> {
        let artifact_id = ArtifactId("doc:gantt:Svg".to_string());
        let artifact = ArtifactFactory::image_asset_with_id(
            artifact_id.clone(),
            ArtifactFormat::Svg,
            DocumentId("doc".to_string()),
            SourceRevision("rev".to_string()),
            ArtifactBytes {
                bytes: r#"<svg xmlns="http://www.w3.org/2000/svg" width="1520" height="244"><rect width="1520" height="244"/></svg>"#
                    .as_bytes()
                    .to_vec(),
            },
            "test",
            ArtifactDiagnostics {
                entries: Vec::new(),
            },
        );
        let node = diagram_kmm_node();
        let planned = diagram_planned_node(artifact_id);
        let height = ViewerMediaHeight::block_height(
            None,
            &[artifact],
            &node,
            &planned,
            ViewerTypographyConfig {
                preview_font_size: 14,
            },
            1280.0,
            ViewerHeightMode::InteractivePreview,
        );

        let expected = 244.0
            * ViewerMediaHeight::content_width(1280.0, ViewerHeightMode::InteractivePreview) as f32
            / 1520.0;

        assert!(
            (expected - height).abs() <= 0.01,
            "expected {expected}, actual {height}"
        );
        Ok(())
    }

    fn diagram_planned_node(artifact_id: ArtifactId) -> PlannedNode {
        PlannedNode {
            node_id: KmmNodeId("node-diagram".to_string()),
            kind: ViewerNodeKind::Diagram {
                kind: ViewerDiagramKind::Mermaid,
            },
            source: source("```mermaid\ngantt\n```"),
            text: String::new(),
            spans: Vec::new(),
            reference: Some(ViewerAssetReference {
                node_id: KmmNodeId("node-diagram".to_string()),
                artifact_id,
                uri: crate::ArtifactUri("kdv://asset/doc:gantt:Svg".to_string()),
                format: ArtifactFormat::Svg,
            }),
        }
    }

    fn diagram_kmm_node() -> KmmNode {
        KmmNode {
            id: KmmNodeId("node-diagram".to_string()),
            kind: KmmNodeKind::CodeBlock(CodeBlockRole::Diagram {
                kind: DiagramKind::Mermaid,
            }),
            source: source("```mermaid\ngantt\n```"),
            children: Vec::new(),
        }
    }

    fn source(text: &str) -> SourceSpan {
        SourceSpan {
            byte_range: ByteRange {
                start: 0,
                end: text.len(),
            },
            line_column_range: LineColumnRange {
                start: LineColumn { line: 1, column: 1 },
                end: LineColumn {
                    line: 1,
                    column: text.len() + 1,
                },
            },
            raw: RawSnippet {
                text: text.to_string(),
            },
        }
    }
}

#[path = "builder_media_asset_height.rs"]
mod asset_height;

#[path = "builder_media_data_image.rs"]
mod data_image;

#[path = "builder_media_surface_height.rs"]
mod surface_height;

#[path = "builder_media_text_height.rs"]
mod text_height;

#[path = "builder_span_line_counter.rs"]
mod span_line_counter;
