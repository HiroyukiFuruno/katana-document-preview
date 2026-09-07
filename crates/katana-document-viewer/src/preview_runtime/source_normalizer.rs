use crate::MarkdownSource;
use crate::preview_runtime::direct_html_preview_renderer::DirectHtmlPreviewRenderer;
use crate::preview_runtime::types::PreviewError;
use std::path::{Path, PathBuf};

#[path = "source_normalizer_image.rs"]
mod image;

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg"];
const HTML_EXTENSIONS: &[&str] = &["html", "htm"];
const MERMAID_EXTENSIONS: &[&str] = &["mmd", "mermaid"];
const PLANTUML_EXTENSIONS: &[&str] = &["puml", "plantuml"];

pub(super) struct PreparedPreviewSource {
    pub(super) content: String,
    pub(super) source_path: PathBuf,
    pub(super) source_kind: crate::SourceKind,
    pub(super) document_kind: crate::DocumentKind,
}

pub(super) struct PreviewSourceNormalizer;

impl PreviewSourceNormalizer {
    pub(super) fn normalize(
        source: &MarkdownSource,
    ) -> Result<PreparedPreviewSource, PreviewError> {
        let source_name = Self::source_name(source);
        let source_path = PathBuf::from(&source_name);
        let content = Self::normalize_newlines(&source.content);
        if Self::is_image_path(&source_path) {
            return Self::image_source(&content, source_name, source_path);
        }
        if Self::is_drawio_path(&source_path) {
            return Ok(Self::drawio_source(&content, source_path));
        }
        if Self::is_mermaid_path(&source_path) {
            return Ok(Self::diagram_source(&content, source_path, "mermaid"));
        }
        if Self::is_plantuml_path(&source_path) {
            return Ok(Self::diagram_source(&content, source_path, "plantuml"));
        }
        if Self::is_html_path(&source_path) {
            return Self::html_source(&content, source_path);
        }
        Ok(Self::markdown_source(content, source_path))
    }

    fn drawio_source(content: &str, source_path: PathBuf) -> PreparedPreviewSource {
        Self::diagram_source(content, source_path, "drawio")
    }

    fn diagram_source(content: &str, source_path: PathBuf, fence: &str) -> PreparedPreviewSource {
        PreparedPreviewSource {
            content: Self::diagram_markdown(content, fence),
            source_path,
            source_kind: crate::SourceKind::Diagram,
            document_kind: crate::DocumentKind::Diagram,
        }
    }

    fn html_source(
        content: &str,
        source_path: PathBuf,
    ) -> Result<PreparedPreviewSource, PreviewError> {
        Ok(PreparedPreviewSource {
            content: DirectHtmlPreviewRenderer::render_html(content)?,
            source_path,
            source_kind: crate::SourceKind::Html,
            document_kind: crate::DocumentKind::Html,
        })
    }

    fn markdown_source(content: String, source_path: PathBuf) -> PreparedPreviewSource {
        PreparedPreviewSource {
            content,
            source_path,
            source_kind: crate::SourceKind::Markdown,
            document_kind: crate::DocumentKind::Markdown,
        }
    }

    fn source_name(source: &MarkdownSource) -> String {
        match &source.document_id {
            Some(document_id) => document_id.clone(),
            None => "preview.md".to_string(),
        }
    }

    fn diagram_markdown(content: &str, fence: &str) -> String {
        let body = content.trim();
        format!("```{fence}\n{body}\n```")
    }

    fn normalize_newlines(content: &str) -> String {
        content.replace("\r\n", "\n").replace('\r', "\n")
    }

    fn starts_with_windows_drive(value: &str) -> bool {
        let bytes = value.as_bytes();
        bytes.len() >= 3 && bytes[1] == b':' && bytes[2] == b'/' && bytes[0].is_ascii_alphabetic()
    }

    fn is_drawio_path(path: &Path) -> bool {
        Self::extension(path)
            .is_some_and(|extension| extension == "drawio" || extension == "drowio")
    }

    fn is_mermaid_path(path: &Path) -> bool {
        Self::extension(path)
            .is_some_and(|extension| MERMAID_EXTENSIONS.iter().any(|item| *item == extension))
    }

    fn is_plantuml_path(path: &Path) -> bool {
        Self::extension(path)
            .is_some_and(|extension| PLANTUML_EXTENSIONS.iter().any(|item| *item == extension))
    }

    fn is_html_path(path: &Path) -> bool {
        Self::extension(path)
            .is_some_and(|extension| HTML_EXTENSIONS.iter().any(|item| *item == extension))
    }
}

#[cfg(test)]
#[path = "source_normalizer_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "source_normalizer_image_tests.rs"]
mod image_tests;
