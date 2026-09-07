use super::*;
use crate::preview_runtime::types::PreviewError;

#[test]
fn image_source_becomes_single_markdown_image() -> Result<(), PreviewError> {
    let prepared = normalize(&source("", "/tmp/sample.png"))?;

    assert_eq!("![sample.png](file:///tmp/sample.png)", prepared.content);
    assert_eq!(crate::SourceKind::Image, prepared.source_kind);
    assert_eq!(crate::DocumentKind::Image, prepared.document_kind);
    Ok(())
}

#[test]
fn image_source_preserves_katana_reference_image_buffer() -> Result<(), PreviewError> {
    let prepared = normalize(&source("![](file:///tmp/sample.png)", "/tmp/sample.png"))?;

    assert_eq!("![](file:///tmp/sample.png)", prepared.content);
    assert_eq!(crate::SourceKind::Image, prepared.source_kind);
    assert_eq!(crate::DocumentKind::Image, prepared.document_kind);
    Ok(())
}

#[test]
fn raw_svg_image_source_uses_document_path_as_image_uri() -> Result<(), PreviewError> {
    let prepared = normalize(&source(
        r#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#,
        "/tmp/sample.svg",
    ))?;

    assert_eq!("![sample.svg](file:///tmp/sample.svg)", prepared.content);
    assert!(!prepared.content.contains("<svg"));
    assert_eq!(crate::SourceKind::Image, prepared.source_kind);
    Ok(())
}

#[test]
fn drawio_source_becomes_drawio_fence() -> Result<(), PreviewError> {
    let prepared = normalize(&source("<mxfile />", "sample.drawio"))?;

    assert_eq!("```drawio\n<mxfile />\n```", prepared.content);
    assert_eq!(crate::SourceKind::Diagram, prepared.source_kind);
    Ok(())
}

#[test]
fn source_name_defaults_to_preview_md_when_document_id_missing() -> Result<(), PreviewError> {
    let prepared = PreviewSourceNormalizer::normalize(&MarkdownSource {
        content: "# title".to_string(),
        document_id: None,
    })?;

    assert_eq!("preview.md", prepared.source_path.to_string_lossy());
    assert_eq!(crate::SourceKind::Markdown, prepared.source_kind);
    assert_eq!(crate::DocumentKind::Markdown, prepared.document_kind);
    Ok(())
}

#[test]
fn image_markdown_preserves_image_markdown_reference() -> Result<(), PreviewError> {
    let prepared = normalize(&source("![alt](http://example.com/image.png)", "image.png"))?;

    assert_eq!("![alt](http://example.com/image.png)", prepared.content);
    Ok(())
}

#[test]
fn image_markdown_uses_file_uri_for_plain_path_when_reference_missing() -> Result<(), PreviewError>
{
    let prepared = normalize(&source("plain content", "image.png"))?;

    assert_eq!("![image.png](file://image.png)", prepared.content);
    Ok(())
}

#[test]
fn html_source_with_query_fragment_or_hash_uses_html_path_extension() -> Result<(), PreviewError> {
    let prepared = normalize(&source(
        r#"<main><h1>Title</h1></main>"#,
        "sample.html?x=1#fragment",
    ))?;

    assert_eq!(crate::SourceKind::Html, prepared.source_kind);
    assert_eq!(crate::DocumentKind::Html, prepared.document_kind);
    assert!(prepared.content.contains("Title"));
    assert!(!prepared.content.contains("<main>"));
    Ok(())
}

#[test]
fn image_source_preserves_http_image_uri_as_is() -> Result<(), PreviewError> {
    let prepared = normalize(&source("raw", "https://example.com/assets/logo.png"))?;

    assert_eq!(crate::SourceKind::Image, prepared.source_kind);
    assert_eq!(
        "![logo.png](https://example.com/assets/logo.png)",
        prepared.content
    );
    Ok(())
}

#[test]
fn mermaid_source_becomes_mermaid_fence() -> Result<(), PreviewError> {
    let prepared = normalize(&source("graph TD\nA --> B", "sample.mmd"))?;

    assert_eq!("```mermaid\ngraph TD\nA --> B\n```", prepared.content);
    assert_eq!(crate::SourceKind::Diagram, prepared.source_kind);
    assert_eq!(crate::DocumentKind::Diagram, prepared.document_kind);
    Ok(())
}

#[test]
fn mermaid_extension_source_becomes_mermaid_fence() -> Result<(), PreviewError> {
    let prepared = normalize(&source("sequenceDiagram\nA->>B: ok", "sample.mermaid"))?;

    assert_eq!(
        "```mermaid\nsequenceDiagram\nA->>B: ok\n```",
        prepared.content
    );
    assert_eq!(crate::SourceKind::Diagram, prepared.source_kind);
    Ok(())
}

#[test]
fn markdown_source_normalizes_crlf_before_markdown_runtime_parse() -> Result<(), PreviewError> {
    let prepared = normalize(&source(
        "| Feature | Status |\r\n| --- | --- |\r\n| PreviewPane | ok |\r\n",
        "sample.md",
    ))?;

    assert!(!prepared.content.contains('\r'));
    assert_eq!(
        "| Feature | Status |\n| --- | --- |\n| PreviewPane | ok |\n",
        prepared.content
    );
    assert_eq!(crate::SourceKind::Markdown, prepared.source_kind);
    Ok(())
}

#[test]
fn windows_image_source_becomes_valid_file_uri() -> Result<(), PreviewError> {
    let prepared = normalize(&source("", r"C:\tmp\sample.png"))?;

    assert_eq!("![sample.png](file:///C:/tmp/sample.png)", prepared.content);
    Ok(())
}

#[test]
fn windows_verbatim_image_source_keeps_extension_and_valid_file_uri() -> Result<(), PreviewError> {
    let prepared = normalize(&source("", r"\\?\D:\repo\assets\sample.png"))?;

    assert_eq!(crate::SourceKind::Image, prepared.source_kind);
    assert_eq!(crate::DocumentKind::Image, prepared.document_kind);
    assert_eq!(
        "![sample.png](file:///D:/repo/assets/sample.png)",
        prepared.content
    );
    Ok(())
}

#[test]
fn windows_verbatim_unc_image_source_keeps_extension_and_valid_file_uri() -> Result<(), PreviewError>
{
    let prepared = normalize(&source("", r"\\?\UNC\server\share\assets\sample.png"))?;

    assert_eq!(crate::SourceKind::Image, prepared.source_kind);
    assert_eq!(
        "![sample.png](file://server/share/assets/sample.png)",
        prepared.content
    );
    Ok(())
}

#[test]
fn html_source_keeps_html_document_kind() -> Result<(), PreviewError> {
    let prepared = normalize(&source(
        r#"<main><h1>Title</h1><p align="center">Body</p></main>"#,
        "sample.html",
    ))?;

    assert!(prepared.content.contains("Title"));
    assert!(prepared.content.contains("Body"));
    assert!(!prepared.content.contains("<main>"));
    assert_eq!(crate::SourceKind::Html, prepared.source_kind);
    assert_eq!(crate::DocumentKind::Html, prepared.document_kind);
    Ok(())
}

fn normalize(source: &MarkdownSource) -> Result<PreparedPreviewSource, PreviewError> {
    PreviewSourceNormalizer::normalize(source)
}

fn source(content: &str, document_id: &str) -> MarkdownSource {
    MarkdownSource {
        content: content.to_string(),
        document_id: Some(document_id.to_string()),
    }
}
