use crate::{
    SpreadsheetBorderSideArtifact, SpreadsheetCellArtifact, SpreadsheetCellBorderArtifact,
    SpreadsheetCellStyleArtifact, SpreadsheetConditionalFormattingArtifact, SpreadsheetCoordinate,
    SpreadsheetHorizontalAlignment, SpreadsheetMergedCellArtifact, SpreadsheetTrackArtifact,
    SpreadsheetVerticalAlignment,
};
use katana_ui_core::molecule::{
    GridCellAppearance, GridCellContent, GridCellSpan, GridCoordinate, GridHorizontalAlignment,
    GridTrackSizeProvider, GridVerticalAlignment,
};
use katana_ui_core::render_model::{UiGridBorderLineStyle, UiGridBorderSide, UiGridCellBorders};

#[path = "spreadsheet_grid_mapping_values.rs"]
mod values;

pub(super) fn spreadsheet_coordinate(coordinate: GridCoordinate) -> SpreadsheetCoordinate {
    SpreadsheetCoordinate::new(coordinate.row, coordinate.column)
}

fn grid_coordinate(coordinate: SpreadsheetCoordinate) -> GridCoordinate {
    GridCoordinate::new(coordinate.row, coordinate.column)
}

pub(super) fn cell_span(span: SpreadsheetMergedCellArtifact) -> GridCellSpan {
    GridCellSpan::new(
        grid_coordinate(span.anchor),
        span.row_span,
        span.column_span,
    )
}

pub(super) fn track_provider(
    tracks: &[SpreadsheetTrackArtifact],
    fallback_size: u32,
) -> GridTrackSizeProvider {
    let sizes = tracks.iter().map(|track| track_size(track.size)).collect();
    let hidden_indices = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| track.hidden.then_some(index))
        .collect();
    GridTrackSizeProvider::VariableWithHidden {
        sizes,
        fallback_size,
        hidden_indices,
    }
}

pub(super) fn row_track_provider(
    tracks: &[SpreadsheetTrackArtifact],
    fallback_size: u32,
    filtered_out_rows: &[usize],
) -> GridTrackSizeProvider {
    let sizes = tracks.iter().map(|track| track_size(track.size)).collect();
    let hidden_indices = tracks
        .iter()
        .enumerate()
        .filter_map(|(index, track)| {
            (track.hidden || filtered_out_rows.binary_search(&index).is_ok()).then_some(index)
        })
        .collect();
    GridTrackSizeProvider::VariableWithHidden {
        sizes,
        fallback_size,
        hidden_indices,
    }
}

pub(super) fn track_size(value: f32) -> u32 {
    if !value.is_finite() || value <= 0.0 {
        return 1;
    }
    value.round().clamp(1.0, u32::MAX as f32) as u32
}

pub(super) fn cell_content(cell: SpreadsheetCellArtifact) -> GridCellContent {
    GridCellContent::new(grid_coordinate(cell.coordinate), cell.display_text)
        .appearance(cell_appearance(cell.style, cell.conditional_formatting))
}

fn cell_appearance(
    style: SpreadsheetCellStyleArtifact,
    conditional: SpreadsheetConditionalFormattingArtifact,
) -> GridCellAppearance {
    GridCellAppearance {
        font_family: style.font_name,
        font_size_px: font_size(style.font_size),
        text_color: style.font_color,
        fill_color: style.fill_color,
        bold: style.bold,
        italic: style.italic,
        underline: style.underline,
        strike: style.strike,
        horizontal_alignment: horizontal_alignment(style.horizontal_alignment),
        vertical_alignment: vertical_alignment(style.vertical_alignment),
        wrap_text: style.wrap_text,
        data_bar: conditional.data_bar.map(values::data_bar),
        icon: conditional.icon.map(values::icon),
        rating: conditional.rating.map(values::rating),
        borders: cell_borders(style.borders),
    }
}

fn cell_borders(value: SpreadsheetCellBorderArtifact) -> UiGridCellBorders {
    UiGridCellBorders {
        left: border_side(value.left),
        right: border_side(value.right),
        top: border_side(value.top),
        bottom: border_side(value.bottom),
    }
}

fn border_side(value: Option<SpreadsheetBorderSideArtifact>) -> UiGridBorderSide {
    value.map_or_else(UiGridBorderSide::default, |side| UiGridBorderSide {
        line_style: border_line_style(&side.style),
        color: side.color,
    })
}

fn border_line_style(value: &str) -> UiGridBorderLineStyle {
    match value.trim().to_ascii_lowercase().as_str() {
        "" | "none" => UiGridBorderLineStyle::None,
        "hair" => UiGridBorderLineStyle::Hair,
        "thin" => UiGridBorderLineStyle::Thin,
        "medium" => UiGridBorderLineStyle::Medium,
        "thick" => UiGridBorderLineStyle::Thick,
        "double" => UiGridBorderLineStyle::Double,
        "dotted" => UiGridBorderLineStyle::Dotted,
        "dashed" => UiGridBorderLineStyle::Dashed,
        "dashdot" | "dash-dot" => UiGridBorderLineStyle::DashDot,
        "dashdotdot" | "dash-dot-dot" => UiGridBorderLineStyle::DashDotDot,
        "mediumdashed" | "medium-dashed" => UiGridBorderLineStyle::MediumDashed,
        "mediumdashdot" | "medium-dash-dot" => UiGridBorderLineStyle::MediumDashDot,
        "mediumdashdotdot" | "medium-dash-dot-dot" => UiGridBorderLineStyle::MediumDashDotDot,
        "slantdashdot" | "slant-dash-dot" => UiGridBorderLineStyle::SlantDashDot,
        "solid" => UiGridBorderLineStyle::Solid,
        _ => UiGridBorderLineStyle::Solid,
    }
}

pub(super) fn font_size(value: f32) -> u16 {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }
    value.round().clamp(1.0, f32::from(u16::MAX)) as u16
}

const fn horizontal_alignment(value: SpreadsheetHorizontalAlignment) -> GridHorizontalAlignment {
    match value {
        SpreadsheetHorizontalAlignment::General => GridHorizontalAlignment::General,
        SpreadsheetHorizontalAlignment::Left => GridHorizontalAlignment::Left,
        SpreadsheetHorizontalAlignment::Center
        | SpreadsheetHorizontalAlignment::CenterContinuous => GridHorizontalAlignment::Center,
        SpreadsheetHorizontalAlignment::Right => GridHorizontalAlignment::Right,
        SpreadsheetHorizontalAlignment::Fill => GridHorizontalAlignment::Fill,
        SpreadsheetHorizontalAlignment::Justify => GridHorizontalAlignment::Justify,
        SpreadsheetHorizontalAlignment::Distributed => GridHorizontalAlignment::Distributed,
    }
}

const fn vertical_alignment(value: SpreadsheetVerticalAlignment) -> GridVerticalAlignment {
    match value {
        SpreadsheetVerticalAlignment::Bottom => GridVerticalAlignment::Bottom,
        SpreadsheetVerticalAlignment::Center => GridVerticalAlignment::Center,
        SpreadsheetVerticalAlignment::Top => GridVerticalAlignment::Top,
        SpreadsheetVerticalAlignment::Justify => GridVerticalAlignment::Justify,
        SpreadsheetVerticalAlignment::Distributed => GridVerticalAlignment::Distributed,
    }
}
