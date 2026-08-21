use super::*;
use crate::{DiagramRenderEngine, DiagramRenderRequest, KdvThemeSnapshot, RenderedDiagram};
use crate::{MarkdownPreview, PreviewAssetLoader, PreviewConfig, ViewerViewport};

#[test]
fn render_engine_returns_kdv_surface_without_vendor_ui() -> Result<(), Box<dyn std::error::Error>> {
    let source = MarkdownSource {
        content: [
            "# Surface",
            "",
            "<p align=\"center\">Centered fallback</p>",
            "",
            "```rust",
            "fn main() {}",
            "```",
        ]
        .join("\n"),
        document_id: Some("surface.md".to_string()),
    };

    let output = PreviewRenderEngine.render(&source, &PreviewConfig::default())?;
    let Some(surface) = output.surface else {
        return Err(Box::new(std::io::Error::other("surface image missing")));
    };

    assert!(surface.width > 0);
    assert!(surface.height > 0);
    assert_eq!(
        surface.rgba.len(),
        surface.width as usize * surface.height as usize * 4
    );
    Ok(())
}

#[test]
fn render_engine_keeps_diagram_rendering_lazy_for_initial_preview() {
    let source = [
        include_str!("engine.rs"),
        include_str!("engine_surface_exporter.rs"),
    ]
    .join("\n");

    assert!(!source.contains("KrrDiagramRenderEngine"));
    assert!(!source.contains("DiagramRenderingBackend"));
    assert!(!source.contains("ExportRequest"));
    assert!(!source.contains("ExportFormat"));
    assert!(!source.contains("decode_png"));
    assert!(source.contains("BuildGraph::from_request"));
}

#[test]
fn render_engine_handles_diagram_fixture_without_external_renderers()
-> Result<(), Box<dyn std::error::Error>> {
    let source = MarkdownSource {
        content: include_str!("../../../../assets/fixtures/katana/sample_diagrams.md").to_string(),
        document_id: Some("sample_diagrams.md".to_string()),
    };

    let output = PreviewRenderEngine.render(&source, &PreviewConfig::default())?;

    assert!(output.surface.is_some());
    assert!(output.diagnostics.warnings.is_empty());
    Ok(())
}

#[test]
fn render_engine_surface_uses_loaded_diagram_assets() -> Result<(), Box<dyn std::error::Error>> {
    let source = MarkdownSource {
        content: "```mermaid\ngraph TD\n  A --> B\n```".to_string(),
        document_id: Some("diagram.md".to_string()),
    };
    let config = PreviewConfig {
        viewport: ViewerViewport {
            width: 1280.0,
            height: 720.0,
        },
        ..PreviewConfig::default()
    };
    let engine = PreviewRenderEngine;
    let mut fallback = engine.render_viewer_output(&source, &config)?;
    engine.attach_surface(&mut fallback, &config);
    let fallback_red_pixels = red_pixel_count(&fallback)?;
    let output = engine.render_viewer_output(&source, &config)?;
    let (mut loaded, report) = PreviewAssetLoader::new(TallDiagramEngine)
        .load_requested(&output, &KdvThemeSnapshot::katana_light())?;

    engine.attach_surface(&mut loaded, &config);
    let loaded_red_pixels = red_pixel_count(&loaded)?;

    assert_eq!(1, report.loaded_artifact_count);
    assert!(
        loaded_red_pixels > fallback_red_pixels + 20_000,
        "fallback_red_pixels={fallback_red_pixels} loaded_red_pixels={loaded_red_pixels}"
    );
    Ok(())
}

#[test]
fn render_engine_preserves_table_inline_code_spans() -> Result<(), Box<dyn std::error::Error>> {
    let source = MarkdownSource {
        content: [
            "| Component | Role |",
            "| --- | --- |",
            "| `PreviewPane` | Section management |",
            "| `show_content` | UI rendering |",
        ]
        .join("\n"),
        document_id: Some("table-inline-code.md".to_string()),
    };

    let output = PreviewRenderEngine.render(&source, &PreviewConfig::default())?;
    let plan = crate::ViewerNodePlanner::create(&output.input, 0.0);
    let table = plan
        .nodes
        .iter()
        .find(|node| matches!(node.kind, crate::ViewerNodeKind::Table))
        .ok_or_else(|| std::io::Error::other("table node missing"))?;

    assert!(table.text.contains("PreviewPane"));
    assert!(table.text.contains("show_content"));
    assert!(!table.text.contains("`PreviewPane`"));
    assert!(!table.text.contains("`show_content`"));
    assert!(has_inline_code_span(table, "PreviewPane"));
    assert!(has_inline_code_span(table, "show_content"));
    Ok(())
}

fn red_pixel_count(output: &PreviewOutput) -> Result<usize, Box<dyn std::error::Error>> {
    let surface = output
        .surface
        .as_ref()
        .ok_or_else(|| std::io::Error::other("surface missing"))?;
    Ok(surface
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel| pixel[0] > 180 && pixel[1] < 80 && pixel[2] < 80 && pixel[3] > 200)
        .count())
}

struct TallDiagramEngine;

impl DiagramRenderEngine for TallDiagramEngine {
    fn render(&self, request: DiagramRenderRequest<'_>) -> Result<RenderedDiagram, String> {
        Ok(RenderedDiagram {
            node_id: request.node_id.to_string(),
            kind: "mermaid".to_string(),
            svg: r##"<svg xmlns="http://www.w3.org/2000/svg" width="160" height="360"><rect width="160" height="360" fill="#ff0000"/></svg>"##.to_string(),
        })
    }
}

fn has_inline_code_span(node: &crate::ViewerNode, expected: &str) -> bool {
    node.spans
        .iter()
        .any(|span| span.text == expected && span.style.inline_code)
}
