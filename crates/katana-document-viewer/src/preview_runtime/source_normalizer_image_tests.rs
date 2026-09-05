use super::*;
use crate::preview_runtime::types::PreviewError;

#[test]
fn image_source_encodes_local_paths_and_escapes_markdown_alt_delimiters() -> Result<(), PreviewError>
{
    let prepared = normalize(&source("", "/tmp/a [b]#c%.png"))?;

    assert_eq!(
        "![a \\[b\\]#c%.png](file:///tmp/a%20%5Bb%5D%23c%25.png)",
        prepared.content
    );
    assert_eq!(crate::SourceKind::Image, prepared.source_kind);
    Ok(())
}

#[test]
fn image_source_encodes_a_direct_local_image_reference() -> Result<(), PreviewError> {
    let prepared = normalize(&source("/tmp/a [b]#c%.png", "/tmp/document.png"))?;

    assert_eq!(
        "![document.png](file:///tmp/a%20%5Bb%5D%23c%25.png)",
        prepared.content
    );
    Ok(())
}

#[test]
fn image_source_resolves_a_relative_direct_image_reference_from_the_document_directory()
-> Result<(), PreviewError> {
    let prepared = normalize(&source("assets/photo.png", "/tmp/kdv-preview/document.png"))?;

    assert_eq!(
        "![document.png](file:///tmp/kdv-preview/assets/photo.png)",
        prepared.content
    );
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
