use super::grid_paint::parse_color;
use katana_ui_core::render_model::UiGridCell;

const INDICATOR_SIZE: f32 = 13.0;
const INDICATOR_GAP: f32 = 3.0;

pub(super) fn paint_conditional_formatting(
    painter: &egui::Painter,
    rect: egui::Rect,
    clip: egui::Rect,
    cell: &UiGridCell,
    ui: &egui::Ui,
) -> f32 {
    paint_data_bar(painter, rect, clip, cell);
    let Some((text, color)) = indicator(cell, ui) else {
        return 0.0;
    };
    let position = egui::pos2(rect.left() + INDICATOR_GAP, rect.center().y);
    painter.text(
        position,
        egui::Align2::LEFT_CENTER,
        &text,
        egui::FontId::proportional(INDICATOR_SIZE),
        color,
    );
    INDICATOR_SIZE * text.chars().count().max(1) as f32 + INDICATOR_GAP
}

pub(super) fn show_cell_value(cell: &UiGridCell) -> bool {
    cell.appearance
        .data_bar
        .as_ref()
        .is_none_or(|bar| bar.show_value)
        && cell
            .appearance
            .icon
            .as_ref()
            .is_none_or(|icon| icon.show_value)
        && cell
            .appearance
            .rating
            .as_ref()
            .is_none_or(|rating| rating.show_value)
}

fn paint_data_bar(painter: &egui::Painter, rect: egui::Rect, clip: egui::Rect, cell: &UiGridCell) {
    let Some(bar) = &cell.appearance.data_bar else {
        return;
    };
    let (start, end, color) = data_bar_style(
        bar.fill_ratio_basis_points,
        bar.axis_ratio_basis_points,
        bar.positive_color.as_deref(),
        bar.negative_color.as_deref(),
    );
    let min = egui::pos2(rect.left() + rect.width() * start, rect.top());
    let size = egui::vec2(rect.width() * (end - start), rect.height());
    let opacity = if bar.gradient { 0.35 } else { 0.5 };
    painter.rect_filled(
        egui::Rect::from_min_size(min, size).intersect(clip),
        0.0,
        color.gamma_multiply(opacity),
    );
}

fn data_bar_style(
    fill_ratio_basis_points: u16,
    axis_ratio_basis_points: u16,
    positive_color: Option<&str>,
    negative_color: Option<&str>,
) -> (f32, f32, egui::Color32) {
    let value = ratio(fill_ratio_basis_points);
    let axis = ratio(axis_ratio_basis_points);
    if value < axis {
        (
            value,
            axis,
            negative_color
                .and_then(parse_color)
                .unwrap_or(egui::Color32::from_rgb(210, 80, 80)),
        )
    } else {
        (
            axis,
            value,
            positive_color
                .and_then(parse_color)
                .unwrap_or(egui::Color32::from_rgb(99, 190, 123)),
        )
    }
}

fn indicator(cell: &UiGridCell, ui: &egui::Ui) -> Option<(String, egui::Color32)> {
    if let Some(icon) = &cell.appearance.icon {
        let color = icon
            .color
            .as_deref()
            .and_then(parse_color)
            .unwrap_or_else(|| ui.visuals().text_color());
        return Some((icon_symbol(&icon.name).to_owned(), color));
    }
    let rating = cell.appearance.rating.as_ref()?;
    let color = rating
        .color
        .as_deref()
        .and_then(parse_color)
        .unwrap_or(egui::Color32::from_rgb(218, 165, 32));
    let count = rating.count.min(rating.maximum).min(10) as usize;
    Some((icon_symbol(&rating.icon_name).repeat(count), color))
}

fn ratio(basis_points: u16) -> f32 {
    f32::from(basis_points.min(10_000)) / 10_000.0
}

fn icon_symbol(name: &str) -> &'static str {
    let name = name.to_ascii_lowercase();
    let symbols = [
        ("up", "\u{2191}"),
        ("down", "\u{2193}"),
        ("left", "\u{2190}"),
        ("right", "\u{2192}"),
        ("star", "\u{2605}"),
        ("check", "\u{2713}"),
        ("cross", "\u{00d7}"),
        ("xmark", "\u{00d7}"),
        ("flag", "\u{2691}"),
    ];
    symbols
        .into_iter()
        .find_map(|(needle, symbol)| name.contains(needle).then_some(symbol))
        .unwrap_or("\u{25cf}")
}

#[cfg(test)]
#[path = "grid_conditional_tests.rs"]
mod tests;
