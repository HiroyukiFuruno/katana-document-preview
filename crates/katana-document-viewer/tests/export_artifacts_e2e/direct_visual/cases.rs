use super::fixture_files::{drawio_source, mermaid_source, write_jpeg, write_png, write_svg};
use std::error::Error;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct DirectVisualCase {
    pub label: &'static str,
    pub document_id: String,
    pub content: String,
    html_marker: &'static str,
}

impl DirectVisualCase {
    fn image(label: &'static str, path: PathBuf) -> Self {
        let uri = file_uri(&path);
        Self {
            label,
            document_id: path.to_string_lossy().to_string(),
            content: uri,
            html_marker: "<img ",
        }
    }

    fn mermaid(label: &'static str, document_id: &str) -> Self {
        Self::diagram(
            label,
            document_id,
            mermaid_source(),
            r#"data-kdv-diagram="mermaid""#,
        )
    }

    fn drawio(label: &'static str, document_id: &str) -> Self {
        Self::diagram(
            label,
            document_id,
            drawio_source(),
            r#"data-kdv-diagram="drawio""#,
        )
    }

    fn diagram(
        label: &'static str,
        document_id: &str,
        content: &str,
        html_marker: &'static str,
    ) -> Self {
        Self {
            label,
            document_id: document_id.to_string(),
            content: content.to_string(),
            html_marker,
        }
    }

    pub fn assert_html_media(&self, html: &str) {
        assert!(
            html.contains(self.html_marker),
            "{} HTML missing media marker: {}",
            self.label,
            self.html_marker
        );
        assert!(
            !html.contains("data-kdv-render-error="),
            "{} HTML contains render error",
            self.label
        );
    }
}

fn file_uri(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    let prefix = if normalized.starts_with('/') { "" } else { "/" };
    format!("file://{prefix}{normalized}")
}

#[test]
fn file_uri_formats_windows_path_as_a_file_url() {
    assert_eq!(
        "file:///C:/Users/runner/AppData/Local/Temp/direct-source.png",
        file_uri(Path::new(
            r"C:\Users\runner\AppData\Local\Temp\direct-source.png"
        ))
    );
}

pub struct DirectVisualCases {
    pub items: Vec<DirectVisualCase>,
}

impl DirectVisualCases {
    pub fn create(output_dir: &Path) -> Result<Self, Box<dyn Error>> {
        let png = output_dir.join("direct-source.png");
        let jpg = output_dir.join("direct-source.jpg");
        let svg = output_dir.join("direct-source.svg");
        write_png(&png)?;
        write_jpeg(&jpg)?;
        write_svg(&svg)?;

        Ok(Self {
            items: vec![
                DirectVisualCase::image("png", png),
                DirectVisualCase::image("jpg", jpg),
                DirectVisualCase::image("svg", svg),
                DirectVisualCase::mermaid("mmd", "direct-source.mmd"),
                DirectVisualCase::mermaid("mermaid", "direct-source.mermaid"),
                DirectVisualCase::drawio("drawio", "direct-source.drawio"),
                DirectVisualCase::drawio("drowio", "direct-source.drowio"),
            ],
        })
    }
}
