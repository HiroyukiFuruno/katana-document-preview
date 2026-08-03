use super::*;
use crate::document_surface::host::test_support::{grid_cell, raw_input, rgb_hex, run_ui};
use katana_ui_core::render_model::{
    UiGridDataBar, UiGridHorizontalAlignment, UiGridVerticalAlignment,
};

#[test]
fn text_bounds_fonts_and_horizontal_alignment_are_typed() {
    let rect = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(100.0, 40.0));
    assert!(text_rect(rect, 0.0).is_some());
    assert!(text_rect(rect, 200.0).is_none());
    assert_eq!(egui::FontFamily::Monospace, font_id(0, "Consolas").family);
    assert_eq!(egui::FontFamily::Proportional, font_id(14, "Aptos").family);
    assert_eq!(
        egui::Align::RIGHT,
        horizontal_alignment(UiGridHorizontalAlignment::Right)
    );
    assert_eq!(
        egui::Align::Center,
        horizontal_alignment(UiGridHorizontalAlignment::Center)
    );
    assert_eq!(
        egui::Align::LEFT,
        horizontal_alignment(UiGridHorizontalAlignment::Left)
    );
}

#[test]
fn text_position_covers_horizontal_and_vertical_alignment() {
    let rect = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(100.0, 40.0));
    let mut cell = grid_cell(0, 0, "value");
    cell.appearance.horizontal_alignment = UiGridHorizontalAlignment::Right;
    cell.appearance.vertical_alignment = UiGridVerticalAlignment::Top;
    assert_eq!(
        egui::pos2(80.0, 10.0),
        text_position(rect, egui::vec2(30.0, 10.0), &cell)
    );
    cell.appearance.horizontal_alignment = UiGridHorizontalAlignment::Center;
    cell.appearance.vertical_alignment = UiGridVerticalAlignment::Bottom;
    assert_eq!(
        egui::pos2(45.0, 40.0),
        text_position(rect, egui::vec2(30.0, 10.0), &cell)
    );
    cell.appearance.horizontal_alignment = UiGridHorizontalAlignment::General;
    cell.appearance.vertical_alignment = UiGridVerticalAlignment::Center;
    assert_eq!(
        egui::pos2(10.0, 25.0),
        text_position(rect, egui::vec2(30.0, 10.0), &cell)
    );
}

#[test]
fn text_paint_runs_styled_plain_hidden_and_clipped_cells_headlessly() {
    let context = egui::Context::default();
    run_ui(&context, raw_input(Vec::new()), |ui| {
        let painter = ui.painter().clone();
        let rect = egui::Rect::from_min_size(ui.min_rect().min, egui::vec2(120.0, 40.0));
        paint_styled(ui, &painter, rect);
        paint_plain(ui, &painter, rect);
        paint_ignored(ui, &painter, rect);
    });
}

fn paint_styled(ui: &egui::Ui, painter: &egui::Painter, rect: egui::Rect) {
    let mut cell = grid_cell(0, 0, "styled");
    cell.appearance.font_family = "Consolas".to_owned();
    cell.appearance.text_color = Some(rgb_hex(0x11, 0x22, 0x33));
    cell.appearance.bold = true;
    cell.appearance.italic = true;
    cell.appearance.underline = true;
    cell.appearance.strike = true;
    cell.appearance.wrap_text = true;
    cell.appearance.horizontal_alignment = UiGridHorizontalAlignment::Justify;
    paint_cell_text(painter, rect, &cell, ui, 0.0);
    assert_eq!(
        egui::Color32::from_rgb(0x11, 0x22, 0x33),
        text_color(&cell, ui)
    );
}

fn paint_plain(ui: &egui::Ui, painter: &egui::Painter, rect: egui::Rect) {
    let mut cell = grid_cell(0, 0, "plain");
    cell.appearance.font_family = "Aptos".to_owned();
    cell.appearance.horizontal_alignment = UiGridHorizontalAlignment::Distributed;
    let color = text_color(&cell, ui);
    let galley = layout_cell_text(ui, &cell, 80.0, color);
    paint_galley(painter, rect.min, galley, color, false);
    let format = text_format(&cell, color);
    assert_eq!(egui::Stroke::default(), format.underline);
    assert_eq!(egui::Stroke::default(), format.strikethrough);
}

fn paint_ignored(ui: &egui::Ui, painter: &egui::Painter, rect: egui::Rect) {
    paint_cell_text(painter, rect, &grid_cell(0, 0, ""), ui, 0.0);
    let mut hidden = grid_cell(0, 0, "hidden");
    hidden.appearance.data_bar = Some(UiGridDataBar {
        show_value: false,
        ..Default::default()
    });
    paint_cell_text(painter, rect, &hidden, ui, 0.0);
    paint_cell_text(painter, rect, &grid_cell(0, 0, "plain"), ui, 500.0);
}
