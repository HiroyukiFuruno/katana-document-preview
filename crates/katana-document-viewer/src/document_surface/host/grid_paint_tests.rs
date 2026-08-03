use super::*;
use crate::document_surface::host::test_support::{grid_cell, raw_input, rgb_hex, run_ui};
use katana_ui_core::render_model::{UiGridProps, UiRect};

#[test]
fn grid_paint_covers_clipping_fill_selection_and_line_modes() {
    let context = egui::Context::default();
    run_ui(&context, raw_input(Vec::new()), |ui| {
        let viewport = egui::Rect::from_min_size(ui.min_rect().min, egui::vec2(220.0, 140.0));
        let mut active = grid_cell(0, 0, "active");
        active.appearance.fill_color = Some(rgb_hex(0x22, 0x33, 0x44));
        active.active = true;
        let mut selected = grid_cell(0, 1, "selected");
        selected.selected = true;
        let mut clipped = grid_cell(1, 0, "clipped");
        clipped.clipped_bounds = UiRect::new(0, 32, 0, 0);
        let props = UiGridProps {
            cells: vec![active, selected, clipped],
            show_grid_lines: true,
            ..Default::default()
        };

        paint_grid(ui, viewport, &props);
        let without_lines = UiGridProps {
            show_grid_lines: false,
            ..props
        };
        paint_grid(ui, viewport, &without_lines);
    });
}

#[test]
fn grid_paint_geometry_and_color_parsing_are_typed() {
    let rect = translated(egui::pos2(10.0, 20.0), UiRect::new(-2, 3, 40, 50));
    let color = rgb_hex(1, 2, 3);
    assert_eq!(egui::pos2(8.0, 23.0), rect.min);
    assert_eq!(egui::vec2(40.0, 50.0), rect.size());
    assert_eq!(Some(egui::Color32::from_rgb(1, 2, 3)), parse_color(&color));
    assert_eq!(None, parse_color("not-a-color"));
}
