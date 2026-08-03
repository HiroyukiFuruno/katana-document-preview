use super::*;
use crate::{PdfRenderedPage, ViewerImageSurface};
use katana_ui_core::render_model::{UiGridCell, UiGridCoordinate, UiGridProps};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn active_text_is_available_only_for_a_materialized_active_grid_cell() -> TestResult {
    let coordinate = UiGridCoordinate { row: 1, column: 2 };
    let cell = UiGridCell {
        coordinate,
        text: "active".to_owned(),
        ..Default::default()
    };
    let frame = grid_frame(UiGridProps {
        active_cell: Some(coordinate),
        cells: vec![cell],
        ..Default::default()
    });
    assert_eq!(Some("active"), frame.active_text());

    let no_active = grid_frame(UiGridProps::default());
    assert_eq!(None, no_active.active_text());
    let missing = grid_frame(UiGridProps {
        active_cell: Some(coordinate),
        ..Default::default()
    });
    assert_eq!(None, missing.active_text());

    let page = DocumentSurfaceFrame::from_rendered_page("Page", rendered_page())?;
    assert_eq!(None, page.active_text());
    Ok(())
}

fn grid_frame(props: UiGridProps) -> DocumentSurfaceFrame {
    DocumentSurfaceFrame::from_node(UiNode::new(UiNodeKind::Grid, "Grid").grid(props))
}

fn rendered_page() -> PdfRenderedPage {
    PdfRenderedPage {
        page_index: 0,
        scale: 1.0,
        surface: ViewerImageSurface {
            fingerprint: "page".to_owned(),
            width: 1,
            height: 1,
            display_width: 1.0,
            display_height: 1.0,
            content_scale: 100,
            rgba: vec![0, 0, 0, 255],
        },
    }
}

#[test]
#[should_panic(expected = "unsupported document surface kind")]
fn unsupported_internal_surface_kind_fails_closed() {
    let frame = DocumentSurfaceFrame::from_node(UiNode::new(UiNodeKind::Text, "text"));
    let _ = frame.kind();
}
