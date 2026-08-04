use std::path::Path;

pub(super) struct PreviewSourcePathNormalizer;

impl PreviewSourcePathNormalizer {
    pub(super) fn normalized_text(value: &str) -> String {
        let normalized = value.replace('\\', "/");
        if let Some(unc) = normalized.strip_prefix("//?/UNC/") {
            return format!("//{unc}");
        }
        normalized
            .strip_prefix("//?/")
            .unwrap_or(&normalized)
            .to_string()
    }

    pub(super) fn file_uri(value: &str) -> String {
        if value.starts_with("http://") || value.starts_with("https://") {
            return value.to_string();
        }
        let normalized = Self::normalized_text(value);
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

    pub(super) fn extension(path: &Path) -> Option<String> {
        let path_text = Self::normalized_text(&path.to_string_lossy());
        let normalized = Self::strip_query_fragment(&path_text);
        Path::new(normalized)
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
    }

    fn starts_with_windows_drive(value: &str) -> bool {
        let bytes = value.as_bytes();
        bytes.len() >= 3 && bytes[1] == b':' && bytes[2] == b'/' && bytes[0].is_ascii_alphabetic()
    }

    fn strip_query_fragment(value: &str) -> &str {
        value.split(['?', '#']).next().unwrap_or(value)
    }
}
