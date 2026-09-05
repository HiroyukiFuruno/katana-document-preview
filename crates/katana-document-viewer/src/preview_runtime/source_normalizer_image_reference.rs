use super::PreviewSourceNormalizer;
use std::path::{Path, PathBuf};

impl PreviewSourceNormalizer {
    pub(super) fn image_reference_uri(reference: &str, source_path: &Path) -> Option<String> {
        if !Self::is_relative_image_reference(reference) {
            return Some(Self::file_uri(reference));
        }
        let path = Self::image_reference_path(reference, source_path);
        let absolute = Self::absolute_image_reference_path(path)?;
        Some(Self::file_uri(&absolute.to_string_lossy()))
    }

    fn is_relative_image_reference(reference: &str) -> bool {
        let normalized = Self::normalized_text(reference);
        !normalized.starts_with("file://")
            && !normalized.starts_with("http://")
            && !normalized.starts_with("https://")
            && !normalized.starts_with("//")
            && !normalized.starts_with('/')
            && !Self::starts_with_windows_drive(&normalized)
    }

    fn image_reference_path(reference: &str, source_path: &Path) -> PathBuf {
        source_path
            .parent()
            .unwrap_or(source_path)
            .join(Self::normalized_text(reference))
    }

    fn absolute_image_reference_path(path: PathBuf) -> Option<PathBuf> {
        if path.is_absolute() {
            return Some(path);
        }
        match std::env::current_dir() {
            Ok(directory) => Some(directory.join(path)),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PreviewSourceNormalizer;

    #[test]
    fn relative_image_reference_excludes_uris_and_absolute_paths() {
        assert!(PreviewSourceNormalizer::is_relative_image_reference(
            "assets/photo.png"
        ));
        for reference in [
            "file:///tmp/photo.png",
            "http://example.com/photo.png",
            "https://example.com/photo.png",
            "//server/share/photo.png",
            "/tmp/photo.png",
            r"C:\\tmp\\photo.png",
        ] {
            assert!(
                !PreviewSourceNormalizer::is_relative_image_reference(reference),
                "{reference}"
            );
        }
    }
}
