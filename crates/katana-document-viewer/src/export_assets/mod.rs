use crate::SourceUri;
use std::path::{Path, PathBuf};

pub(crate) struct ExportAssetResolver;

impl ExportAssetResolver {
    pub(crate) fn resolve_src(source_uri: &SourceUri, src: &str) -> String {
        if Self::is_non_file_reference(src) {
            return src.to_string();
        }
        if let Some(url) = Self::resolve_file_url(source_uri, src) {
            return url;
        }
        let Some(path) = Self::resolve_file_path(source_uri, src) else {
            return src.to_string();
        };
        Self::file_url(&path)
    }

    pub(crate) fn resolve_file_path(source_uri: &SourceUri, src: &str) -> Option<PathBuf> {
        if let Some(path) = Self::file_uri_path(src) {
            return Some(path);
        }
        if Self::is_non_file_reference(src) {
            return None;
        }
        let path = Path::new(src);
        if path.is_absolute() {
            return Some(path.to_path_buf());
        }
        let base_dir = Self::source_base_dir(source_uri)?;
        Some(base_dir.join(path))
    }

    pub(crate) fn rewrite_html_image_sources(fragment: &str, source_uri: &SourceUri) -> String {
        let mut output = String::with_capacity(fragment.len());
        let mut rest = fragment;
        while let Some(src_start) = rest.find("src=\"") {
            let value_start = src_start + "src=\"".len();
            let Some(value_end) = rest[value_start..].find('"') else {
                break;
            };
            let src = &rest[value_start..value_start + value_end];
            output.push_str(&rest[..value_start]);
            output.push_str(&Self::resolve_src(source_uri, src));
            rest = &rest[value_start + value_end..];
        }
        output.push_str(rest);
        output
    }

    fn source_base_dir(source_uri: &SourceUri) -> Option<PathBuf> {
        if source_uri.0.is_empty() {
            return None;
        }
        let path = source_uri
            .0
            .strip_prefix("file://")
            .unwrap_or(&source_uri.0);
        let path = Path::new(path);
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().ok()?.join(path)
        };
        absolute.parent().map(Path::to_path_buf)
    }

    fn is_non_file_reference(src: &str) -> bool {
        src.is_empty()
            || src.starts_with('#')
            || src.starts_with("data:")
            || src.starts_with("http://")
            || src.starts_with("https://")
            || src.starts_with("file://")
    }

    fn file_uri_path(src: &str) -> Option<PathBuf> {
        let path = src
            .strip_prefix("file://")?
            .split(['?', '#'])
            .next()
            .unwrap_or(src);
        #[cfg(windows)]
        let path = path
            .strip_prefix('/')
            .filter(|value| Self::starts_with_windows_drive(value))
            .unwrap_or(path);
        let path = Path::new(path);
        if path.is_absolute() {
            return Some(path.to_path_buf());
        }
        Some(std::env::current_dir().ok()?.join(path))
    }

    fn resolve_file_url(source_uri: &SourceUri, src: &str) -> Option<String> {
        let path = Path::new(src);
        if path.is_absolute() {
            return Some(Self::file_url(path));
        }
        let source_path = source_uri.0.strip_prefix("file://")?;
        let source_path = source_path.replace('\\', "/");
        let (base_dir, _) = source_path.rsplit_once('/')?;
        let src = src.replace('\\', "/");
        let src = src.trim_start_matches("./");
        if base_dir.is_empty() {
            return Some(format!("file:///{src}"));
        }
        Some(format!("file://{}/{}", base_dir.trim_end_matches('/'), src))
    }

    fn file_url(path: &Path) -> String {
        let path = path.to_string_lossy().replace('\\', "/");
        if path.starts_with('/') {
            return format!("file://{path}");
        }
        format!("file:///{path}")
    }

    #[cfg(windows)]
    fn starts_with_windows_drive(value: &str) -> bool {
        let bytes = value.as_bytes();
        bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && matches!(bytes[2], b'/' | b'\\')
    }
}

#[cfg(test)]
#[path = "../export_assets_tests.rs"]
mod tests;
