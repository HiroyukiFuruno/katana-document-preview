use crate::MarkdownSource;
use crate::preview_runtime::direct_html_normalizer::DirectHtmlNormalizer;
use crate::preview_runtime::source_path_normalizer::PreviewSourcePathNormalizer;
use std::path::{Path, PathBuf};

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
    pub(super) fn normalize(source: &MarkdownSource) -> PreparedPreviewSource {
        let source_name = Self::source_name(source);
        let source_path = PathBuf::from(&source_name);
        let content = Self::normalize_newlines(&source.content);
        if Self::is_image_path(&source_path) {
            return Self::image_source(&content, source_name, source_path);
        }
        if Self::is_drawio_path(&source_path) {
            return Self::drawio_source(&content, source_path);
        }
        if Self::is_mermaid_path(&source_path) {
            return Self::diagram_source(&content, source_path, "mermaid");
        }
        if Self::is_plantuml_path(&source_path) {
            return Self::diagram_source(&content, source_path, "plantuml");
        }
        if Self::is_html_path(&source_path) {
            return Self::html_source(&content, source_path);
        }
        Self::markdown_source(content, source_path)
    }

    fn image_source(
        content: &str,
        source_name: String,
        source_path: PathBuf,
    ) -> PreparedPreviewSource {
        PreparedPreviewSource {
            content: Self::image_markdown(content, &source_name),
            source_path,
            source_kind: crate::SourceKind::Image,
            document_kind: crate::DocumentKind::Image,
        }
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

    fn html_source(content: &str, source_path: PathBuf) -> PreparedPreviewSource {
        PreparedPreviewSource {
            content: DirectHtmlNormalizer::normalize(content),
            source_path,
            source_kind: crate::SourceKind::Html,
            document_kind: crate::DocumentKind::Html,
        }
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

    fn image_markdown(content: &str, source_name: &str) -> String {
        let trimmed = content.trim();
        if Self::is_markdown_image(trimmed) {
            return trimmed.to_string();
        }
        let image_uri = Self::image_uri(trimmed, source_name);
        let alt_source_name = PreviewSourcePathNormalizer::normalized_text(source_name);
        let alt = Path::new(&alt_source_name)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("image");
        format!("![{alt}]({image_uri})")
    }

    fn image_uri(trimmed: &str, source_name: &str) -> String {
        if trimmed.is_empty() {
            return Self::file_uri(source_name);
        }
        if Self::is_image_reference(trimmed) {
            return trimmed.to_string();
        }
        Self::file_uri(source_name)
    }

    fn file_uri(source_name: &str) -> String {
        PreviewSourcePathNormalizer::file_uri(source_name)
    }

    fn is_image_reference(value: &str) -> bool {
        value.starts_with("file://")
            || value.starts_with("http://")
            || value.starts_with("https://")
            || Self::is_image_path(Path::new(value))
    }

    fn is_markdown_image(content: &str) -> bool {
        content.starts_with("![") && content.contains("](") && content.ends_with(')')
    }

    fn diagram_markdown(content: &str, fence: &str) -> String {
        let body = content.trim();
        format!("```{fence}\n{body}\n```")
    }

    fn normalize_newlines(content: &str) -> String {
        content.replace("\r\n", "\n").replace('\r', "\n")
    }

    fn is_image_path(path: &Path) -> bool {
        Self::extension(path)
            .is_some_and(|extension| IMAGE_EXTENSIONS.iter().any(|item| *item == extension))
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

    fn extension(path: &Path) -> Option<String> {
        PreviewSourcePathNormalizer::extension(path)
    }
}

#[cfg(test)]
#[path = "source_normalizer_tests.rs"]
mod tests;
