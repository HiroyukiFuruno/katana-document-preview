use super::{
    DocumentGridSurfaceFrame, DocumentPageSurfaceFrame, DocumentSurfaceContent,
    DocumentSurfaceFrame, DocumentSurfaceKind, PdfOutlineItem,
};
use crate::{DocumentGridViewport, SpreadsheetCellBorderArtifact, SpreadsheetCoordinate};
use std::collections::HashMap;

#[test]
fn grid_frame_reports_grid_kind_at_runtime() {
    let frame = DocumentSurfaceFrame {
        content: DocumentSurfaceContent::Grid(DocumentGridSurfaceFrame {
            row_count: 0,
            column_count: 0,
            total_width: 0,
            total_height: 0,
            viewport: DocumentGridViewport::default(),
            active_cell: None,
            show_grid_lines: true,
            cells: Vec::new(),
        }),
        navigation: super::DocumentNavigationMetadata::default(),
    }
    .with_navigation_metadata(
        vec!["Sheet 1".to_owned()],
        vec![PdfOutlineItem {
            title: "Section".to_owned(),
            level: 0,
            page_index: Some(0),
        }],
    );

    assert_eq!(
        DocumentSurfaceKind::Grid,
        std::hint::black_box(&frame).kind()
    );
    assert_eq!(["Sheet 1"], frame.item_labels());
    assert_eq!("Section", frame.outline_items()[0].title);
}

#[test]
fn grid_borders_leave_page_surface_unchanged() {
    let frame = DocumentSurfaceFrame {
        content: DocumentSurfaceContent::Page(DocumentPageSurfaceFrame {
            fingerprint: "page".to_owned(),
            width: 1,
            height: 1,
            display_width_milli: 1,
            display_height_milli: 1,
            content_scale: 1,
            accessibility_label: "page".to_owned(),
            rgba: vec![0, 0, 0, 0],
        }),
        navigation: super::DocumentNavigationMetadata::default(),
    };
    let borders = HashMap::from([(
        SpreadsheetCoordinate::new(0, 0),
        SpreadsheetCellBorderArtifact::default(),
    )]);

    assert_eq!(frame.clone(), frame.with_grid_cell_borders(&borders));
}
