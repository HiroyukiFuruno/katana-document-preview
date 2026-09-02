use katana_document_viewer::{KdvThemeSnapshot, KrrMathRenderEngine};

#[test]
fn public_consumer_links_kdv_and_krr_through_one_v8_runtime() {
    let result = KrrMathRenderEngine::render_display_svg("", &KdvThemeSnapshot::katana_light());

    assert!(matches!(result, Err(message) if !message.is_empty()));
}
