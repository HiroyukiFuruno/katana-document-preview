use crate::{
    SpreadsheetCellArtifact, SpreadsheetCellStyleArtifact,
    SpreadsheetConditionalFormattingArtifact, SpreadsheetCoordinate, SpreadsheetDataBarArtifact,
    SpreadsheetHorizontalAlignment, SpreadsheetIconArtifact, SpreadsheetMergedCellArtifact,
    SpreadsheetRatingArtifact, SpreadsheetTrackArtifact, SpreadsheetVerticalAlignment,
};
use katana_ui_core::molecule::{
    GridCellAppearance, GridCellContent, GridCellSpan, GridCoordinate, GridDataBar,
    GridHorizontalAlignment, GridIcon, GridRating, GridTrackSizeProvider, GridVerticalAlignment,
};

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
        data_bar: conditional.data_bar.map(data_bar),
        icon: conditional.icon.map(icon),
        rating: conditional.rating.map(rating),
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

fn data_bar(value: SpreadsheetDataBarArtifact) -> GridDataBar {
    GridDataBar {
        positive_color: value.positive_color,
        negative_color: value.negative_color,
        fill_ratio_basis_points: ratio_basis_points(value.value),
        axis_ratio_basis_points: ratio_basis_points(value.axis_position),
        gradient: value.gradient,
        show_value: value.show_value,
    }
}

fn icon(value: SpreadsheetIconArtifact) -> GridIcon {
    GridIcon {
        name: value.name,
        color: value.color,
        show_value: value.show_value,
    }
}

fn rating(value: SpreadsheetRatingArtifact) -> GridRating {
    GridRating {
        icon_name: value.icon_name,
        count: value.count,
        maximum: value.maximum,
        color: value.color,
        show_value: value.show_value,
    }
}

pub(super) fn ratio_basis_points(value: f64) -> u16 {
    if !value.is_finite() {
        return 0;
    }
    (value.clamp(0.0, 1.0) * 10_000.0).round() as u16
}
