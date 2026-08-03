use super::grid_conditional::show_cell_value;
use super::grid_paint::parse_color;
use katana_ui_core::render_model::{
    UiGridCell, UiGridHorizontalAlignment, UiGridVerticalAlignment,
};

const HORIZONTAL_PADDING: f32 = 5.0;
const VERTICAL_PADDING: f32 = 3.0;

pub(super) fn paint_cell_text(
    painter: &egui::Painter,
    rect: egui::Rect,
    cell: &UiGridCell,
    ui: &egui::Ui,
    indicator_width: f32,
) {
    if cell.text.is_empty() || !show_cell_value(cell) {
        return;
    }
    let Some(text_rect) = text_rect(rect, indicator_width) else {
        return;
    };
    let color = text_color(cell, ui);
    let galley = layout_cell_text(ui, cell, text_rect.width(), color);
    let position = text_position(text_rect, galley.size(), cell);
    paint_galley(painter, position, galley, color, cell.appearance.bold);
}

fn text_rect(rect: egui::Rect, indicator_width: f32) -> Option<egui::Rect> {
    let text_rect = egui::Rect::from_min_max(
        egui::pos2(
            rect.left() + HORIZONTAL_PADDING + indicator_width,
            rect.top() + VERTICAL_PADDING,
        ),
        egui::pos2(
            rect.right() - HORIZONTAL_PADDING,
            rect.bottom() - VERTICAL_PADDING,
        ),
    );
    (text_rect.width() > 0.0 && text_rect.height() > 0.0).then_some(text_rect)
}

fn text_color(cell: &UiGridCell, ui: &egui::Ui) -> egui::Color32 {
    cell.appearance
        .text_color
        .as_deref()
        .and_then(parse_color)
        .unwrap_or_else(|| ui.visuals().text_color())
}

fn paint_galley(
    painter: &egui::Painter,
    position: egui::Pos2,
    galley: std::sync::Arc<egui::Galley>,
    color: egui::Color32,
    bold: bool,
) {
    painter.galley(position, galley.clone(), color);
    if bold {
        painter.galley(position + egui::vec2(0.55, 0.0), galley, color);
    }
}

fn layout_cell_text(
    ui: &egui::Ui,
    cell: &UiGridCell,
    max_width: f32,
    color: egui::Color32,
) -> std::sync::Arc<egui::Galley> {
    let appearance = &cell.appearance;
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = if appearance.wrap_text {
        max_width
    } else {
        f32::INFINITY
    };
    job.halign = horizontal_alignment(appearance.horizontal_alignment);
    job.justify = matches!(
        appearance.horizontal_alignment,
        UiGridHorizontalAlignment::Justify | UiGridHorizontalAlignment::Distributed
    );
    job.append(&cell.text, 0.0, text_format(cell, color));
    ui.fonts_mut(|fonts| fonts.layout_job(job))
}

fn text_format(cell: &UiGridCell, color: egui::Color32) -> egui::TextFormat {
    let appearance = &cell.appearance;
    let decoration = egui::Stroke::new(1.0, color);
    egui::TextFormat {
        font_id: font_id(appearance.font_size_px, &appearance.font_family),
        color,
        italics: appearance.italic,
        underline: if appearance.underline {
            decoration
        } else {
            Default::default()
        },
        strikethrough: if appearance.strike {
            decoration
        } else {
            Default::default()
        },
        ..Default::default()
    }
}

fn font_id(size: u16, family: &str) -> egui::FontId {
    let size = if size == 0 { 13.0 } else { f32::from(size) };
    let normalized_family = family.to_ascii_lowercase();
    let family = if ["mono", "courier", "consolas"]
        .iter()
        .any(|name| normalized_family.contains(name))
    {
        egui::FontFamily::Monospace
    } else {
        egui::FontFamily::Proportional
    };
    egui::FontId::new(size, family)
}

fn horizontal_alignment(alignment: UiGridHorizontalAlignment) -> egui::Align {
    match alignment {
        UiGridHorizontalAlignment::Right => egui::Align::RIGHT,
        UiGridHorizontalAlignment::Center => egui::Align::Center,
        _ => egui::Align::LEFT,
    }
}

fn text_position(rect: egui::Rect, size: egui::Vec2, cell: &UiGridCell) -> egui::Pos2 {
    let x = match cell.appearance.horizontal_alignment {
        UiGridHorizontalAlignment::Right => rect.right() - size.x,
        UiGridHorizontalAlignment::Center => rect.center().x - size.x * 0.5,
        _ => rect.left(),
    };
    let y = match cell.appearance.vertical_alignment {
        UiGridVerticalAlignment::Top => rect.top(),
        UiGridVerticalAlignment::Bottom => rect.bottom() - size.y,
        _ => rect.center().y - size.y * 0.5,
    };
    egui::pos2(x, y)
}

#[cfg(test)]
#[path = "grid_cell_text_tests.rs"]
mod tests;
