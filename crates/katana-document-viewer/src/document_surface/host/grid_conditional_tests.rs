use super::*;
use crate::document_surface::host::test_support::{grid_cell, raw_input, rgb_hex, run_ui};
use katana_ui_core::render_model::{UiGridDataBar, UiGridIcon, UiGridRating};

#[test]
fn conditional_cell_value_visibility_is_fail_closed() {
    let mut cell = grid_cell(0, 0, "value");
    assert!(show_cell_value(&cell));

    cell.appearance.data_bar = Some(UiGridDataBar {
        show_value: false,
        ..Default::default()
    });
    assert!(!show_cell_value(&cell));
    cell.appearance.data_bar = None;
    cell.appearance.icon = Some(UiGridIcon {
        show_value: false,
        ..Default::default()
    });
    assert!(!show_cell_value(&cell));
    cell.appearance.icon = None;
    cell.appearance.rating = Some(UiGridRating {
        show_value: false,
        ..Default::default()
    });
    assert!(!show_cell_value(&cell));
}

#[test]
fn conditional_ratios_symbols_and_bar_geometry_are_typed() {
    assert_eq!(0.0, ratio(0));
    assert_eq!(1.0, ratio(u16::MAX));
    assert_eq!("\u{2191}", icon_symbol("arrow-up"));
    assert_eq!("\u{2193}", icon_symbol("arrow-down"));
    assert_eq!("\u{2190}", icon_symbol("arrow-left"));
    assert_eq!("\u{2192}", icon_symbol("arrow-right"));
    assert_eq!("\u{2605}", icon_symbol("star"));
    assert_eq!("\u{2713}", icon_symbol("check"));
    assert_eq!("\u{00d7}", icon_symbol("cross"));
    assert_eq!("\u{00d7}", icon_symbol("xmark"));
    assert_eq!("\u{2691}", icon_symbol("flag"));
    assert_eq!("\u{25cf}", icon_symbol("unknown"));

    let positive_color = rgb_hex(0, 0xAA, 0);
    let negative_color = rgb_hex(0xAA, 0, 0);
    let positive = data_bar_style(8_000, 5_000, Some(&positive_color), None);
    let negative = data_bar_style(2_000, 5_000, None, Some(&negative_color));
    assert_eq!((0.5, 0.8), (positive.0, positive.1));
    assert_eq!((0.2, 0.5), (negative.0, negative.1));
}

#[test]
fn conditional_paint_runs_positive_negative_icon_and_rating_paths() {
    let context = egui::Context::default();
    run_ui(&context, raw_input(Vec::new()), |ui| {
        let painter = ui.painter().clone();
        let rect = egui::Rect::from_min_size(ui.min_rect().min, egui::vec2(120.0, 32.0));
        paint_empty(ui, &painter, rect);
        paint_positive(ui, &painter, rect);
        paint_negative(ui, &painter, rect);
        assert_fallback_icon(ui);
    });
}

fn paint_empty(ui: &egui::Ui, painter: &egui::Painter, rect: egui::Rect) {
    let cell = grid_cell(0, 0, "empty");
    assert_eq!(
        0.0,
        paint_conditional_formatting(painter, rect, rect, &cell, ui)
    );
    paint_data_bar(painter, rect, rect, &cell);
}

fn paint_positive(ui: &egui::Ui, painter: &egui::Painter, rect: egui::Rect) {
    let mut cell = grid_cell(0, 0, "positive");
    cell.appearance.data_bar = Some(UiGridDataBar {
        positive_color: Some(rgb_hex(0, 0xAA, 0)),
        fill_ratio_basis_points: 8_000,
        axis_ratio_basis_points: 5_000,
        gradient: true,
        show_value: true,
        ..Default::default()
    });
    cell.appearance.icon = Some(UiGridIcon {
        name: "arrow-up".to_owned(),
        color: Some(rgb_hex(0, 0x88, 0)),
        show_value: true,
    });
    assert!(paint_conditional_formatting(painter, rect, rect, &cell, ui) > 0.0);
    assert!(indicator(&cell, ui).is_some());
}

fn paint_negative(ui: &egui::Ui, painter: &egui::Painter, rect: egui::Rect) {
    let mut cell = grid_cell(0, 0, "negative");
    cell.appearance.data_bar = Some(UiGridDataBar {
        fill_ratio_basis_points: 2_000,
        axis_ratio_basis_points: 5_000,
        show_value: true,
        ..Default::default()
    });
    cell.appearance.rating = Some(UiGridRating {
        icon_name: "star".to_owned(),
        count: 12,
        maximum: 5,
        show_value: true,
        ..Default::default()
    });
    assert!(paint_conditional_formatting(painter, rect, rect, &cell, ui) > 0.0);
    assert!(indicator(&cell, ui).is_some_and(|(text, _)| text.chars().count() == 5));
}

fn assert_fallback_icon(ui: &egui::Ui) {
    let mut cell = grid_cell(0, 0, "fallback");
    cell.appearance.icon = Some(UiGridIcon {
        name: "unknown".to_owned(),
        show_value: true,
        ..Default::default()
    });
    assert!(indicator(&cell, ui).is_some());
}
