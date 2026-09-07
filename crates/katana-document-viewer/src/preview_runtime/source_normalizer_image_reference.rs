use super::PreviewSourceNormalizer;
use crate::preview_runtime::types::PreviewError;
use std::io;
use std::path::{Path, PathBuf};

impl PreviewSourceNormalizer {
    pub(super) fn image_reference_uri(
        reference: &str,
        source_path: &Path,
    ) -> Result<String, PreviewError> {
        if !Self::is_relative_image_reference(reference) {
            return Ok(Self::file_uri(reference));
        }
        let path = Self::image_reference_path(reference, source_path);
        let absolute = Self::absolute_image_reference_path(path, std::env::current_dir())?;
        Ok(Self::file_uri(&absolute.to_string_lossy()))
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

    fn absolute_image_reference_path(
        path: PathBuf,
        current_directory: io::Result<PathBuf>,
    ) -> Result<PathBuf, PreviewError> {
        if path.is_absolute() {
            return Ok(path);
        }
        let directory = current_directory.map_err(|error| {
            PreviewError::Render(format!(
                "relative image reference cannot be resolved: {error}"
            ))
        })?;
        Ok(directory.join(path))
    }
}

#[cfg(test)]
mod tests {
    use super::PreviewSourceNormalizer;
    use crate::preview_runtime::types::PreviewError;
    use std::io;
    use std::path::PathBuf;

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

    #[test]
    fn relative_image_reference_reports_current_directory_failure() {
        let result = PreviewSourceNormalizer::absolute_image_reference_path(
            PathBuf::from("assets/photo.png"),
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "working directory unavailable",
            )),
        );

        assert!(matches!(
            result,
            Err(PreviewError::Render(message))
                if message.contains("relative image reference cannot be resolved")
        ));
    }

    #[test]
    fn relative_image_reference_joins_current_directory_when_source_path_is_relative() {
        let result = PreviewSourceNormalizer::absolute_image_reference_path(
            PathBuf::from("assets/photo.png"),
            Ok(PathBuf::from("/workspace")),
        );

        assert_eq!(
            Some(&PathBuf::from("/workspace/assets/photo.png")),
            result.as_ref().ok()
        );
    }
}
