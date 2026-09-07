use crate::preview_runtime::DirectHtmlPreviewRenderer;
use crate::preview_runtime::PreviewError;

#[test]
fn direct_html_preview_renderer_returns_normalized_html() -> Result<(), PreviewError> {
    let normalized = DirectHtmlPreviewRenderer::render_html("<p>Renderer path</p>")?;

    assert!(normalized.contains("Renderer path"));
    Ok(())
}

#[test]
fn direct_html_preview_renderer_maps_external_script_errors() {
    assert!(matches!(
        DirectHtmlPreviewRenderer::render_html(r#"<script src="app.js"></script>"#),
        Err(PreviewError::Render(message))
            if message.contains("external script is not supported: app.js")
    ));
}
