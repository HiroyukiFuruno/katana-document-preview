use super::*;
use crate::document_surface::host::test_support::{page_frame, raw_input, run_surface};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn page_host_loads_reuses_and_replaces_headless_textures() -> TestResult {
    let context = egui::Context::default();
    let mut host = DocumentSurfaceHost::default();
    let small = page_frame(80.0, 60.0)?;

    let first = run_surface(&context, &mut host, &small, raw_input(Vec::new()));
    let fingerprint = host.texture_fingerprint.clone();
    let second = run_surface(&context, &mut host, &small, raw_input(Vec::new()));

    assert_eq!(fingerprint, host.texture_fingerprint);
    assert_eq!(1, first.commands().len());
    assert_eq!(1, second.commands().len());

    let wide = page_frame(320.0, 40.0)?;
    let _ = run_surface(&context, &mut host, &wide, raw_input(Vec::new()));
    assert_ne!(fingerprint, host.texture_fingerprint);
    assert_eq!(
        egui::vec2(320.0, 40.0),
        display_size(&wide.node().props().image_surface)
    );
    Ok(())
}
