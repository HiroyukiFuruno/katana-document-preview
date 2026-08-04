use super::{MarkdownSource, PreviewConfig, PreviewError, PreviewOutputFactory};
use crate::{ArtifactFormat, DocumentKind, ViewerNodeKind, ViewerNodePlanner};
use katana_markdown_model::{KmmNode, KmmNodeKind};

const CONTENT_HEIGHT: f32 = 120.0;

#[test]
fn direct_png_source_becomes_image_document_node() -> Result<(), PreviewError> {
    let output = direct_output("file:///tmp/kdv-icon.png", "/tmp/kdv-icon.png")?;
    let node = first_document_node(&output)?;

    assert_eq!(DocumentKind::Image, output.input.snapshot.kind);
    assert!(matches!(&node.kind, KmmNodeKind::Paragraph));
    assert!(node_contains_image_child(node));
    assert!(node.source.raw.text.contains("file:///tmp/kdv-icon.png"));
    let plan = ViewerNodePlanner::create(&output.input, 0.0);
    assert_eq!(ViewerNodeKind::Image, plan.nodes[0].kind);
    Ok(())
}

#[test]
fn source_normalizer_treats_file_uri_with_query_fragment_as_direct_image()
-> Result<(), PreviewError> {
    for (document_id, expected) in [
        ("/tmp/kdv-icon.png?cache=1", ArtifactFormat::Png),
        ("/tmp/kdv-icon.png#preview", ArtifactFormat::Png),
        ("/tmp/photo.jpg?cache=1#viewer", ArtifactFormat::Jpeg),
        ("/tmp/photo.jpeg?cache=1#viewer", ArtifactFormat::Jpeg),
        ("/tmp/kdv-icon.gif?cache=1#viewer", ArtifactFormat::Gif),
        ("/tmp/kdv-icon.webp?cache=1#viewer", ArtifactFormat::Webp),
        ("/tmp/kdv-icon.bmp?cache=1#viewer", ArtifactFormat::Bmp),
        ("/tmp/diagram.svg?cache=1#viewer", ArtifactFormat::Svg),
        ("/tmp/KDV-ICON.PNG?cache=1#viewer", ArtifactFormat::Png),
    ] {
        let output = direct_output(&format!("file://{document_id}"), document_id)?;
        let plan = ViewerNodePlanner::create(&output.input, 0.0);

        assert_eq!(DocumentKind::Image, output.input.snapshot.kind);
        assert_eq!(ViewerNodeKind::Image, plan.nodes[0].kind);
        assert_eq!(expected, plan.asset_requests[0].format);
    }
    Ok(())
}

#[test]
fn direct_image_empty_content_preserves_file_uri_document_id() -> Result<(), PreviewError> {
    let output = direct_output("", "file:///tmp/kdv-icon.png")?;
    let node = first_document_node(&output)?;
    let plan = ViewerNodePlanner::create(&output.input, 0.0);

    assert_eq!(DocumentKind::Image, output.input.snapshot.kind);
    assert_eq!(ViewerNodeKind::Image, plan.nodes[0].kind);
    assert!(node.source.raw.text.contains("file:///tmp/kdv-icon.png"));
    assert!(!node.source.raw.text.contains("file://file://"));
    Ok(())
}

#[test]
fn windows_verbatim_document_path_still_plans_direct_image_asset() -> Result<(), PreviewError> {
    let output = direct_output(
        "file:///D:/repo/assets/kdv-icon.png",
        r"\\?\D:\repo\assets\kdv-icon.png",
    )?;
    let plan = ViewerNodePlanner::create(&output.input, 0.0);

    assert_eq!(DocumentKind::Image, output.input.snapshot.kind);
    assert_eq!(ViewerNodeKind::Image, plan.nodes[0].kind);
    assert_eq!(ArtifactFormat::Png, plan.asset_requests[0].format);
    Ok(())
}

#[test]
fn direct_png_source_accepts_katana_reference_image_buffer() -> Result<(), PreviewError> {
    let output = direct_output("![](file:///tmp/kdv-icon.png)", "/tmp/kdv-icon.png")?;
    let node = first_document_node(&output)?;
    let plan = ViewerNodePlanner::create(&output.input, 0.0);

    assert_eq!(DocumentKind::Image, output.input.snapshot.kind);
    assert!(matches!(&node.kind, KmmNodeKind::Paragraph));
    assert!(node_contains_image_child(node));
    assert_eq!(ViewerNodeKind::Image, plan.nodes[0].kind);
    assert_eq!(ArtifactFormat::Png, plan.asset_requests[0].format);
    Ok(())
}

#[test]
fn direct_jpeg_source_keeps_image_document_and_jpeg_asset_request() -> Result<(), PreviewError> {
    let output = direct_output("file:///tmp/photo.jpg", "/tmp/photo.jpg")?;
    let plan = ViewerNodePlanner::create(&output.input, 0.0);

    assert_eq!(DocumentKind::Image, output.input.snapshot.kind);
    assert_eq!(ViewerNodeKind::Image, plan.nodes[0].kind);
    assert_eq!(ArtifactFormat::Jpeg, plan.asset_requests[0].format);
    Ok(())
}

#[test]
fn direct_svg_source_keeps_image_document_and_svg_asset_request() -> Result<(), PreviewError> {
    for content in [
        "file:///tmp/diagram.svg",
        r#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#,
    ] {
        let output = direct_output(content, "/tmp/diagram.svg")?;
        let plan = ViewerNodePlanner::create(&output.input, 0.0);

        assert_eq!(DocumentKind::Image, output.input.snapshot.kind);
        assert_eq!(ViewerNodeKind::Image, plan.nodes[0].kind);
        assert_eq!(ArtifactFormat::Svg, plan.asset_requests[0].format);
        assert!(
            plan.nodes[0]
                .source
                .raw
                .text
                .contains("file:///tmp/diagram.svg")
        );
        assert!(!plan.nodes[0].source.raw.text.contains("<svg"));
    }
    Ok(())
}

#[test]
fn direct_katana_image_extensions_keep_image_document_and_asset_format() -> Result<(), PreviewError>
{
    for (path, expected) in [
        ("/tmp/animation.gif", ArtifactFormat::Gif),
        ("/tmp/photo.webp", ArtifactFormat::Webp),
        ("/tmp/screenshot.bmp", ArtifactFormat::Bmp),
    ] {
        let output = direct_output(&format!("file://{path}"), path)?;
        let plan = ViewerNodePlanner::create(&output.input, 0.0);

        assert_eq!(DocumentKind::Image, output.input.snapshot.kind);
        assert_eq!(ViewerNodeKind::Image, plan.nodes[0].kind);
        assert_eq!(expected, plan.asset_requests[0].format);
    }
    Ok(())
}

fn direct_output(content: &str, document_id: &str) -> Result<super::PreviewOutput, PreviewError> {
    PreviewOutputFactory::from_source(
        &MarkdownSource {
            content: content.to_string(),
            document_id: Some(document_id.to_string()),
        },
        &PreviewConfig::default(),
        CONTENT_HEIGHT,
    )
}

fn first_document_node(output: &super::PreviewOutput) -> Result<&KmmNode, PreviewError> {
    output
        .input
        .snapshot
        .document
        .nodes
        .first()
        .ok_or_else(|| PreviewError::Render("document node missing".to_string()))
}

fn node_contains_image_child(node: &KmmNode) -> bool {
    node.children
        .iter()
        .any(|child| matches!(&child.kind, KmmNodeKind::Image(_)))
}
