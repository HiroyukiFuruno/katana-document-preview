use katana_document_viewer::{KdvThemeSnapshot, KrrMathRenderEngine};

fn main() {
    let _ = KrrMathRenderEngine::render_display_svg("", &KdvThemeSnapshot::katana_light());
}
