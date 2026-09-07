use super::super::planned_node::PlannedNode;
use super::super::types::ViewerDiagramKind;
use super::{
    DIAGRAM_MAX_WIDTH, MEDIA_VERTICAL_MARGIN, SURFACE_CONTENT_WIDTH, ViewerHeightMode,
    ViewerMediaHeight,
};
use crate::viewer::VIEWER_DIAGRAM_DISPLAY_MAX_WIDTH;
use crate::{Artifact, ArtifactId, ViewerImageSurfaceFactory};

const DIAGRAM_MIN_CONTAINER_HEIGHT: f32 = 145.0;
impl ViewerMediaHeight {
    pub(super) fn diagram_height(
        artifacts: &[Artifact],
        planned: &PlannedNode,
        _kind: ViewerDiagramKind,
        content_width: u32,
        height_mode: ViewerHeightMode,
    ) -> Option<f32> {
        Self::diagram_content_height(
            artifacts,
            planned,
            Self::diagram_max_width(content_width, height_mode),
            height_mode,
        )
        .map(|height| Self::diagram_container_height(height, height_mode))
    }

    fn diagram_container_height(height: f32, height_mode: ViewerHeightMode) -> f32 {
        match height_mode {
            // KatanA の interactive diagram は 145px 最小値以外に行高を足さない。
            // 共通 media margin は export と非 diagram の契約として維持する。
            ViewerHeightMode::InteractivePreview => height.max(DIAGRAM_MIN_CONTAINER_HEIGHT),
            ViewerHeightMode::ExportSurface => height + MEDIA_VERTICAL_MARGIN,
        }
    }

    fn diagram_max_width(content_width: u32, height_mode: ViewerHeightMode) -> u32 {
        match height_mode {
            ViewerHeightMode::InteractivePreview => {
                content_width.min(VIEWER_DIAGRAM_DISPLAY_MAX_WIDTH)
            }
            ViewerHeightMode::ExportSurface => DIAGRAM_MAX_WIDTH,
        }
        .max(1)
    }

    pub(super) fn svg_height(
        artifacts: &[Artifact],
        planned: &PlannedNode,
        max_width: u32,
    ) -> Option<f32> {
        Self::svg_content_height(artifacts, planned, max_width)
            .map(|height| height + MEDIA_VERTICAL_MARGIN)
    }

    pub(super) fn image_height(artifacts: &[Artifact], planned: &PlannedNode) -> Option<f32> {
        let artifact = Self::artifact(artifacts, planned.reference.as_ref()?.artifact_id.clone())?;
        let surface =
            ViewerImageSurfaceFactory::from_artifact(artifact, SURFACE_CONTENT_WIDTH).ok()?;
        Some(
            Self::scaled_height_to_max_width(
                surface.logical_width(),
                surface.logical_height(),
                SURFACE_CONTENT_WIDTH,
            ) + MEDIA_VERTICAL_MARGIN,
        )
    }

    pub(super) fn scaled_height_to_max_width(width: u32, height: u32, max_width: u32) -> f32 {
        if width <= max_width {
            return height as f32;
        }
        (height as f32 * max_width as f32 / width as f32)
            .round()
            .max(1.0)
    }

    fn svg_content_height(
        artifacts: &[Artifact],
        planned: &PlannedNode,
        max_width: u32,
    ) -> Option<f32> {
        let artifact = Self::artifact(artifacts, planned.reference.as_ref()?.artifact_id.clone())?;
        let surface = ViewerImageSurfaceFactory::from_artifact(artifact, max_width).ok()?;
        Some(Self::scaled_height_to_max_width(
            surface.logical_width(),
            surface.logical_height(),
            max_width,
        ))
    }

    fn diagram_content_height(
        artifacts: &[Artifact],
        planned: &PlannedNode,
        max_width: u32,
        height_mode: ViewerHeightMode,
    ) -> Option<f32> {
        let artifact = Self::artifact(artifacts, planned.reference.as_ref()?.artifact_id.clone())?;
        let surface = match height_mode {
            ViewerHeightMode::InteractivePreview => {
                ViewerImageSurfaceFactory::from_diagram_artifact(artifact, max_width).ok()?
            }
            ViewerHeightMode::ExportSurface => {
                ViewerImageSurfaceFactory::from_export_surface_diagram_artifact(artifact, max_width)
                    .ok()?
            }
        };
        Some(surface.display_height)
    }

    fn artifact(artifacts: &[Artifact], artifact_id: ArtifactId) -> Option<&Artifact> {
        artifacts
            .iter()
            .find(|artifact| artifact.manifest.id == artifact_id)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::types::{ViewerDiagramKind, ViewerNodeKind};
    use super::{ViewerHeightMode, ViewerMediaHeight};
    use crate::artifact::{ArtifactFormat, ArtifactId, ArtifactUri};
    use crate::viewer::asset::ViewerAssetReference;
    use crate::{ArtifactBytes, ArtifactDiagnostics, ArtifactFactory, DocumentId, SourceRevision};
    use katana_markdown_model::{
        ByteRange, KmmNodeId, LineColumn, LineColumnRange, RawSnippet, SourceSpan,
    };

    #[test]
    fn svg_layout_height_scales_intrinsic_surface_to_display_max_width() {
        assert_eq!(
            20.0,
            ViewerMediaHeight::scaled_height_to_max_width(200, 80, 50)
        );
    }

    #[test]
    fn diagram_height_uses_katana_minimum_container_height()
    -> Result<(), Box<dyn std::error::Error>> {
        let artifact_id = ArtifactId("doc:diagram:Svg".to_string());
        let artifact = ArtifactFactory::image_asset_with_id(
            artifact_id.clone(),
            ArtifactFormat::Svg,
            DocumentId("doc".to_string()),
            SourceRevision("rev".to_string()),
            ArtifactBytes {
                bytes: r#"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20"/></svg>"#
                    .as_bytes()
                    .to_vec(),
            },
            "test",
            ArtifactDiagnostics {
                entries: Vec::new(),
            },
        );
        let planned = super::super::super::planned_node::PlannedNode {
            node_id: KmmNodeId("node-diagram".to_string()),
            kind: ViewerNodeKind::Diagram {
                kind: ViewerDiagramKind::Mermaid,
            },
            source: source("```mermaid\nA-->B\n```"),
            text: String::new(),
            spans: Vec::new(),
            reference: Some(ViewerAssetReference {
                node_id: KmmNodeId("node-diagram".to_string()),
                artifact_id,
                uri: ArtifactUri("kdv://asset/doc:diagram:Svg".to_string()),
                format: ArtifactFormat::Svg,
            }),
        };

        assert_eq!(
            145.0,
            ViewerMediaHeight::diagram_height(
                &[artifact],
                &planned,
                ViewerDiagramKind::Mermaid,
                1252,
                ViewerHeightMode::InteractivePreview,
            )
            .ok_or("diagram height")?
        );
        Ok(())
    }

    #[test]
    fn interactive_diagram_height_uses_viewer_width_for_wide_diagram()
    -> Result<(), Box<dyn std::error::Error>> {
        let artifact_id = ArtifactId("doc:wide-diagram:Svg".to_string());
        let artifact = ArtifactFactory::image_asset_with_id(
            artifact_id.clone(),
            ArtifactFormat::Svg,
            DocumentId("doc".to_string()),
            SourceRevision("rev".to_string()),
            ArtifactBytes {
                bytes: r#"<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="600"><rect width="1200" height="600"/></svg>"#
                    .as_bytes()
                    .to_vec(),
            },
            "test",
            ArtifactDiagnostics {
                entries: Vec::new(),
            },
        );
        let planned = diagram_planned_node(artifact_id);

        assert_eq!(
            600.0,
            ViewerMediaHeight::diagram_height(
                &[artifact],
                &planned,
                ViewerDiagramKind::Mermaid,
                1252,
                ViewerHeightMode::InteractivePreview,
            )
            .ok_or("diagram height")?
        );
        Ok(())
    }

    #[test]
    fn interactive_diagram_height_uses_katana_reference_width_cap_for_large_viewports()
    -> Result<(), Box<dyn std::error::Error>> {
        let artifact_id = ArtifactId("doc:large-diagram:Svg".to_string());
        let artifact = ArtifactFactory::image_asset_with_id(
            artifact_id.clone(),
            ArtifactFormat::Svg,
            DocumentId("doc".to_string()),
            SourceRevision("rev".to_string()),
            ArtifactBytes {
                bytes: r#"<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="1200"><rect width="1600" height="1200"/></svg>"#
                    .as_bytes()
                    .to_vec(),
            },
            "test",
            ArtifactDiagnostics {
                entries: Vec::new(),
            },
        );
        let planned = diagram_planned_node(artifact_id);

        assert_near(
            948.0,
            ViewerMediaHeight::diagram_height(
                &[artifact],
                &planned,
                ViewerDiagramKind::Mermaid,
                3000,
                ViewerHeightMode::InteractivePreview,
            )
            .ok_or("diagram height")?,
        );
        Ok(())
    }

    #[test]
    fn export_surface_diagram_height_keeps_export_width() -> Result<(), Box<dyn std::error::Error>>
    {
        let artifact_id = ArtifactId("doc:wide-diagram:Svg".to_string());
        let artifact = ArtifactFactory::image_asset_with_id(
            artifact_id.clone(),
            ArtifactFormat::Svg,
            DocumentId("doc".to_string()),
            SourceRevision("rev".to_string()),
            ArtifactBytes {
                bytes: r#"<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="600"><rect width="1200" height="600"/></svg>"#
                    .as_bytes()
                    .to_vec(),
            },
            "test",
            ArtifactDiagnostics {
                entries: Vec::new(),
            },
        );
        let planned = diagram_planned_node(artifact_id);

        assert_eq!(
            464.0,
            ViewerMediaHeight::diagram_height(
                &[artifact],
                &planned,
                ViewerDiagramKind::Mermaid,
                1252,
                ViewerHeightMode::ExportSurface,
            )
            .ok_or("diagram height")?
        );
        Ok(())
    }

    #[test]
    fn export_surface_diagram_height_does_not_apply_interactive_minimum_container()
    -> Result<(), Box<dyn std::error::Error>> {
        let artifact_id = ArtifactId("doc:short-diagram:Svg".to_string());
        let artifact = ArtifactFactory::image_asset_with_id(
            artifact_id.clone(),
            ArtifactFormat::Svg,
            DocumentId("doc".to_string()),
            SourceRevision("rev".to_string()),
            ArtifactBytes {
                bytes: r#"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20"/></svg>"#
                    .as_bytes()
                    .to_vec(),
            },
            "test",
            ArtifactDiagnostics {
                entries: Vec::new(),
            },
        );
        let planned = diagram_planned_node(artifact_id);

        assert_eq!(
            54.0,
            ViewerMediaHeight::diagram_height(
                &[artifact],
                &planned,
                ViewerDiagramKind::Mermaid,
                1252,
                ViewerHeightMode::ExportSurface,
            )
            .ok_or("diagram height")?
        );
        Ok(())
    }

    #[test]
    fn image_and_svg_heights_use_materialized_surface_dimensions()
    -> Result<(), Box<dyn std::error::Error>> {
        let artifact_id = ArtifactId("doc:image:Svg".to_string());
        let artifact = materialized_svg_artifact(artifact_id.clone());
        let planned = diagram_planned_node(artifact_id);

        assert_eq!(
            Some(74.0),
            ViewerMediaHeight::image_height(std::slice::from_ref(&artifact), &planned),
            "non-diagram images retain the shared media margin"
        );
        assert_eq!(
            Some(54.0),
            ViewerMediaHeight::svg_height(&[artifact], &planned, 40),
            "math and generic SVG surfaces retain the shared media margin"
        );
        Ok(())
    }

    fn materialized_svg_artifact(artifact_id: ArtifactId) -> crate::Artifact {
        ArtifactFactory::image_asset_with_id(
            artifact_id,
            ArtifactFormat::Svg,
            DocumentId("doc".to_string()),
            SourceRevision("rev".to_string()),
            ArtifactBytes {
                bytes: r#"<svg xmlns="http://www.w3.org/2000/svg" width="80" height="40"><rect width="80" height="40"/></svg>"#
                    .as_bytes()
                    .to_vec(),
            },
            "test",
            ArtifactDiagnostics {
                entries: Vec::new(),
            },
        )
    }

    fn diagram_planned_node(
        artifact_id: ArtifactId,
    ) -> super::super::super::planned_node::PlannedNode {
        super::super::super::planned_node::PlannedNode {
            node_id: KmmNodeId("node-diagram".to_string()),
            kind: ViewerNodeKind::Diagram {
                kind: ViewerDiagramKind::Mermaid,
            },
            source: source("```mermaid\nA-->B\n```"),
            text: String::new(),
            spans: Vec::new(),
            reference: Some(ViewerAssetReference {
                node_id: KmmNodeId("node-diagram".to_string()),
                artifact_id,
                uri: ArtifactUri("kdv://asset/doc:diagram:Svg".to_string()),
                format: ArtifactFormat::Svg,
            }),
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

    fn assert_near(expected: f32, actual: f32) {
        assert!(
            (expected - actual).abs() <= 0.01,
            "expected {expected}, actual {actual}"
        );
    }
}
