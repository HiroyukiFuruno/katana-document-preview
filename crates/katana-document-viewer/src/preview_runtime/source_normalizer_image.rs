use super::{PreparedPreviewSource, PreviewSourceNormalizer};
use std::path::{Path, PathBuf};

impl PreviewSourceNormalizer {
    pub(super) fn image_source(
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

    fn image_markdown(content: &str, source_name: &str) -> String {
        let trimmed = content.trim();
        if Self::is_markdown_image(trimmed) {
            return trimmed.to_string();
        }
        let image_uri = Self::image_uri(trimmed, source_name);
        let alt_source_name = Self::normalized_text(source_name);
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
        if source_name.starts_with("http://") || source_name.starts_with("https://") {
            return source_name.to_string();
        }
        let normalized = Self::normalized_text(source_name);
        if normalized.starts_with("file://") {
            return normalized;
        }
        if normalized.starts_with("//") {
            return format!("file:{normalized}");
        }
        if normalized.starts_with('/') {
            return format!("file://{normalized}");
        }
        if Self::starts_with_windows_drive(&normalized) {
            return format!("file:///{normalized}");
        }
        format!("file://{normalized}")
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

    pub(super) fn is_image_path(path: &Path) -> bool {
        Self::extension(path).is_some_and(|extension| {
            super::IMAGE_EXTENSIONS
                .iter()
                .any(|item| *item == extension)
        })
    }

    pub(super) fn extension(path: &Path) -> Option<String> {
        let path_text = path.to_string_lossy();
        let normalized_text = Self::normalized_text(&path_text);
        let normalized = Self::strip_query_fragment(&normalized_text);
        Path::new(normalized)
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
    }

    fn strip_query_fragment(value: &str) -> &str {
        value.split(['?', '#']).next().unwrap_or(value)
    }

    fn normalized_text(value: &str) -> String {
        let normalized = value.replace('\\', "/");
        if let Some(unc) = normalized.strip_prefix("//?/UNC/") {
            return format!("//{unc}");
        }
        normalized
            .strip_prefix("//?/")
            .unwrap_or(&normalized)
            .to_string()
    }
}
