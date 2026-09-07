use super::PreviewSourceNormalizer;
use crate::preview_runtime::types::PreviewError;
use std::path::Path;

impl PreviewSourceNormalizer {
    pub(super) fn image_markdown(
        content: &str,
        source_name: &str,
        source_path: &Path,
    ) -> Result<String, PreviewError> {
        let trimmed = content.trim();
        if Self::is_markdown_image(trimmed) {
            return Ok(trimmed.to_string());
        }
        let image_uri = Self::image_uri(trimmed, source_name, source_path)?;
        let alt_source_name = Self::normalized_text(source_name);
        let alt = Path::new(&alt_source_name)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("image");
        Ok(format!(
            "![{}]({image_uri})",
            Self::escape_markdown_alt(alt)
        ))
    }

    fn image_uri(
        trimmed: &str,
        source_name: &str,
        source_path: &Path,
    ) -> Result<String, PreviewError> {
        if trimmed.is_empty() {
            return Ok(Self::file_uri(source_name));
        }
        if Self::is_image_reference(trimmed) {
            return Self::image_reference_uri(trimmed, source_path);
        }
        Ok(Self::file_uri(source_name))
    }
}
