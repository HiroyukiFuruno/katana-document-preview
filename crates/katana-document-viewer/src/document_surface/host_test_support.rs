use super::{DocumentSurfaceFrame, DocumentSurfaceHost, DocumentSurfaceHostOutput};
use crate::{DocumentSurfaceError, PdfRenderedPage, ViewerImageSurface};
use katana_ui_core::render_model::{
    UiGridCell, UiGridCoordinate, UiGridProps, UiGridViewport, UiNode, UiNodeKind, UiRect,
};

pub(super) fn raw_input(events: Vec<egui::Event>) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(240.0, 180.0),
        )),
        events,
        ..Default::default()
    }
}

pub(super) fn run_surface(
    context: &egui::Context,
    host: &mut DocumentSurfaceHost,
    frame: &DocumentSurfaceFrame,
    input: egui::RawInput,
) -> DocumentSurfaceHostOutput {
    let mut output = DocumentSurfaceHostOutput::default();
    let _ = context.run_ui(input, |ui| {
        output = host.show(ui, frame, 7);
    });
    output
}

pub(super) fn run_ui(
    context: &egui::Context,
    input: egui::RawInput,
    mut render: impl FnMut(&mut egui::Ui),
) {
    let _ = context.run_ui(input, |ui| render(ui));
}

pub(super) fn page_frame(
    display_width: f32,
    display_height: f32,
) -> Result<DocumentSurfaceFrame, DocumentSurfaceError> {
    let rendered = PdfRenderedPage {
        page_index: 0,
        scale: 1.0,
        surface: ViewerImageSurface {
            fingerprint: format!("page-{display_width}-{display_height}"),
            width: 2,
            height: 2,
            display_width,
            display_height,
            content_scale: 100,
            rgba: vec![255; 16],
        },
    };
    DocumentSurfaceFrame::from_rendered_page("Page", rendered)
}

pub(super) fn grid_frame(cells: Vec<UiGridCell>, show_grid_lines: bool) -> DocumentSurfaceFrame {
    let active_cell = cells.first().map(|cell| cell.coordinate);
    let props = UiGridProps {
        row_count: 20,
        column_count: 20,
        total_width: 400,
        total_height: 400,
        viewport: UiGridViewport {
            width: 220,
            height: 160,
            scroll_x: 20,
            scroll_y: 20,
        },
        active_cell,
        show_grid_lines,
        cells,
        ..Default::default()
    };
    DocumentSurfaceFrame::from_node(UiNode::new(UiNodeKind::Grid, "Grid").grid(props))
}

pub(super) fn grid_cell(row: usize, column: usize, text: &str) -> UiGridCell {
    UiGridCell {
        coordinate: UiGridCoordinate { row, column },
        bounds: UiRect::new((column * 80) as i32, (row * 32) as i32, 80, 32),
        clipped_bounds: UiRect::new((column * 80) as i32, (row * 32) as i32, 80, 32),
        text: text.to_owned(),
        accessibility_row_index: row + 1,
        accessibility_column_index: column + 1,
        ..Default::default()
    }
}

pub(super) fn pointer_button(pos: egui::Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::default(),
    }
}

pub(super) fn key_event(key: egui::Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::default(),
    }
}

pub(super) fn rgb_hex(red: u8, green: u8, blue: u8) -> String {
    format!("#{red:02X}{green:02X}{blue:02X}")
}
