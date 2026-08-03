use super::grid_cell_text::paint_cell_text;
use super::grid_conditional::paint_conditional_formatting;
use katana_ui_core::render_model::{UiGridCell, UiGridProps, UiRect};

pub(super) fn paint_grid(ui: &egui::Ui, viewport: egui::Rect, props: &UiGridProps) {
    let painter = ui.painter().with_clip_rect(viewport);
    painter.rect_filled(viewport, 0.0, ui.visuals().extreme_bg_color);
    for cell in &props.cells {
        paint_grid_cell(ui, &painter, viewport, cell, props.show_grid_lines);
    }
}

fn paint_grid_cell(
    ui: &egui::Ui,
    painter: &egui::Painter,
    viewport: egui::Rect,
    cell: &UiGridCell,
    show_grid_lines: bool,
) {
    let rect = translated(viewport.min, cell.bounds);
    let clip = translated(viewport.min, cell.clipped_bounds).intersect(viewport);
    if clip.is_negative() || clip.width() <= 0.0 || clip.height() <= 0.0 {
        return;
    }
    let painter = painter.with_clip_rect(clip);
    let fill = cell
        .appearance
        .fill_color
        .as_deref()
        .and_then(parse_color)
        .unwrap_or_else(|| ui.visuals().faint_bg_color);
    painter.rect_filled(rect, 0.0, fill);
    let indicator_width = paint_conditional_formatting(&painter, rect, clip, cell, ui);
    paint_selection(ui, &painter, rect, cell);
    paint_grid_line(ui, &painter, rect, show_grid_lines);
    paint_cell_text(&painter, rect, cell, ui, indicator_width);
}

fn paint_selection(ui: &egui::Ui, painter: &egui::Painter, rect: egui::Rect, cell: &UiGridCell) {
    if cell.selected || cell.active {
        let opacity = if cell.active { 0.45 } else { 0.25 };
        let color = ui.visuals().selection.bg_fill.gamma_multiply(opacity);
        painter.rect_filled(rect, 0.0, color);
    }
}

fn paint_grid_line(
    ui: &egui::Ui,
    painter: &egui::Painter,
    rect: egui::Rect,
    show_grid_lines: bool,
) {
    if !show_grid_lines {
        return;
    }
    let color = ui.visuals().widgets.noninteractive.bg_stroke.color;
    painter.rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.0, color),
        egui::StrokeKind::Inside,
    );
}

pub(super) fn translated(origin: egui::Pos2, rect: UiRect) -> egui::Rect {
    egui::Rect::from_min_size(
        origin + egui::vec2(rect.x as f32, rect.y as f32),
        egui::vec2(rect.width as f32, rect.height as f32),
    )
}

pub(super) fn parse_color(value: &str) -> Option<egui::Color32> {
    egui::Color32::from_hex(value).ok()
}

#[cfg(test)]
#[path = "grid_paint_tests.rs"]
mod tests;
