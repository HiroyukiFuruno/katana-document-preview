use crate::preview_runtime::direct_html_normalizer::DirectHtmlNormalizer;
use crate::preview_runtime::types::PreviewError;
use katana_render_runtime::{HtmlRenderInput, HtmlRenderer};

pub struct DirectHtmlPreviewRenderer;

impl DirectHtmlPreviewRenderer {
    pub fn render_html(content: &str) -> Result<String, PreviewError> {
        let rendered = HtmlRenderer
            .render(&HtmlRenderInput {
                source: content.to_string(),
            })
            .map_err(|error| PreviewError::Render(error.to_string()))?;
        Ok(DirectHtmlNormalizer::normalize(&rendered.content))
    }
}
