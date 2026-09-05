use super::{PreparedPreviewSource, PreviewSourceNormalizer};
use crate::preview_runtime::types::PreviewError;
use std::path::{Path, PathBuf};

#[path = "source_normalizer_image_reference.rs"]
mod reference;

#[path = "source_normalizer_image_markup.rs"]
mod markup;

impl PreviewSourceNormalizer {
    pub(super) fn image_source(
        content: &str,
        source_name: String,
        source_path: PathBuf,
    ) -> Result<PreparedPreviewSource, PreviewError> {
        Ok(PreparedPreviewSource {
            content: Self::image_markdown(content, &source_name, &source_path)?,
            source_path,
            source_kind: crate::SourceKind::Image,
            document_kind: crate::DocumentKind::Image,
        })
    }

    fn file_uri(source_name: &str) -> String {
        if source_name.starts_with("http://") || source_name.starts_with("https://") {
            return source_name.to_string();
        }
        let normalized = Self::normalized_text(source_name);
        let (raw, preserve_uri_suffix) = Self::local_file_uri_path(normalized);
        Self::encode_file_uri(&raw, preserve_uri_suffix)
    }

    fn local_file_uri_path(normalized: String) -> (String, bool) {
        if let Some(raw) = normalized.strip_prefix("file://") {
            return (raw.to_string(), true);
        }
        if let Some(raw) = normalized.strip_prefix("//") {
            return (raw.to_string(), false);
        }
        if normalized.starts_with('/') {
            return (normalized, false);
        }
        if Self::starts_with_windows_drive(&normalized) {
            return (format!("/{normalized}"), false);
        }
        (normalized, false)
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
        let Some(index) = value.find(['?', '#']) else {
            return value;
        };
        let prefix = &value[..index];
        if Path::new(prefix).extension().is_some() {
            prefix
        } else {
            value
        }
    }

    fn escape_markdown_alt(value: &str) -> String {
        value
            .replace('\\', "\\\\")
            .replace('[', "\\[")
            .replace(']', "\\]")
    }

    fn encode_file_uri(raw: &str, preserve_uri_suffix: bool) -> String {
        let (raw, suffix) = if preserve_uri_suffix {
            match raw.find(['?', '#']) {
                Some(index) => (&raw[..index], &raw[index..]),
                None => (raw, ""),
            }
        } else {
            (raw, "")
        };
        let (authority, path) = if raw.starts_with('/') || !raw.contains('/') {
            ("", raw)
        } else {
            let split = raw.find('/').unwrap_or(raw.len());
            raw.split_at(split)
        };
        format!("file://{authority}{}{suffix}", Self::encode_uri_path(path))
    }

    fn encode_uri_path(value: &str) -> String {
        let mut encoded = String::with_capacity(value.len());
        let mut remaining = value.as_bytes();
        while !remaining.is_empty() {
            let consumed = Self::encode_uri_fragment(&mut encoded, remaining);
            remaining = &remaining[consumed..];
        }
        encoded
    }

    fn encode_uri_fragment(encoded: &mut String, bytes: &[u8]) -> usize {
        if Self::has_percent_encoding(bytes) {
            encoded.push('%');
            encoded.push(bytes[1] as char);
            encoded.push(bytes[2] as char);
            return 3;
        }
        let byte = bytes[0];
        if Self::is_uri_path_byte(byte) {
            encoded.push(byte as char);
        } else {
            Self::push_percent_encoded(encoded, byte);
        }
        1
    }

    fn has_percent_encoding(bytes: &[u8]) -> bool {
        bytes.len() >= 3
            && bytes[0] == b'%'
            && bytes[1].is_ascii_hexdigit()
            && bytes[2].is_ascii_hexdigit()
    }

    fn is_uri_path_byte(byte: u8) -> bool {
        byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/' | b':')
    }

    fn push_percent_encoded(encoded: &mut String, byte: u8) {
        const HEX: &[u8; 16] = b"0123456789ABCDEF";

        encoded.push('%');
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
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
