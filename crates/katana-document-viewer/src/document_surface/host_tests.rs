use super::{DocumentSurfaceHost, test_support::*};
use crate::{DocumentSurfaceCommand, DocumentSurfaceKind, DocumentViewport};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn host_routes_page_and_grid_frames_and_exposes_commands() -> TestResult {
    let context = egui::Context::default();
    let mut host = DocumentSurfaceHost::default();
    let page = page_frame(120.0, 80.0)?;

    let output = run_surface(&context, &mut host, &page, raw_input(Vec::new()));

    assert_eq!(DocumentSurfaceKind::Page, page.kind());
    assert!(matches!(
        output.commands(),
        [DocumentSurfaceCommand::Resize(DocumentViewport { .. })]
    ));
    assert!(host.texture.is_some());
    assert!(format!("{host:?}").contains("texture_fingerprint"));
    assert_eq!(1, output.into_commands().len());

    let grid = grid_frame(vec![grid_cell(0, 0, "A1")], true);
    let output = run_surface(&context, &mut host, &grid, raw_input(Vec::new()));
    assert_eq!(DocumentSurfaceKind::Grid, grid.kind());
    assert!(matches!(
        output.commands().first(),
        Some(DocumentSurfaceCommand::Resize(_))
    ));
    Ok(())
}
