use crate::multi_format::{
    SpreadsheetBorderSideArtifact, SpreadsheetCellArtifact, SpreadsheetCellBorderArtifact,
    SpreadsheetCellStyleArtifact, SpreadsheetCellValue, SpreadsheetConditionalFormattingArtifact,
    SpreadsheetDataBarArtifact, SpreadsheetIconArtifact, SpreadsheetRatingArtifact,
};

pub(super) fn cell_bytes(cell: &SpreadsheetCellArtifact) -> usize {
    std::mem::size_of::<SpreadsheetCellArtifact>()
        .saturating_add(string_bytes(&cell.display_text))
        .saturating_add(cell_value_bytes(&cell.value))
        .saturating_add(optional_string_bytes(cell.formula.as_ref()))
        .saturating_add(cell_style_bytes(&cell.style))
        .saturating_add(conditional_formatting_bytes(&cell.conditional_formatting))
}

fn string_bytes(value: &String) -> usize {
    value.capacity()
}

fn optional_string_bytes(value: Option<&String>) -> usize {
    value.map_or(0, string_bytes)
}

fn cell_value_bytes(value: &SpreadsheetCellValue) -> usize {
    match value {
        SpreadsheetCellValue::Text(value) => string_bytes(value),
        SpreadsheetCellValue::Empty
        | SpreadsheetCellValue::Number(_)
        | SpreadsheetCellValue::Boolean(_) => 0,
    }
}

fn cell_style_bytes(style: &SpreadsheetCellStyleArtifact) -> usize {
    string_bytes(&style.font_name)
        .saturating_add(optional_string_bytes(style.font_color.as_ref()))
        .saturating_add(optional_string_bytes(style.fill_color.as_ref()))
        .saturating_add(string_bytes(&style.number_format))
        .saturating_add(cell_border_bytes(&style.borders))
}

fn cell_border_bytes(borders: &SpreadsheetCellBorderArtifact) -> usize {
    optional_border_side_bytes(borders.left.as_ref())
        .saturating_add(optional_border_side_bytes(borders.right.as_ref()))
        .saturating_add(optional_border_side_bytes(borders.top.as_ref()))
        .saturating_add(optional_border_side_bytes(borders.bottom.as_ref()))
}

fn optional_border_side_bytes(side: Option<&SpreadsheetBorderSideArtifact>) -> usize {
    side.map_or(0, border_side_bytes)
}

fn border_side_bytes(side: &SpreadsheetBorderSideArtifact) -> usize {
    string_bytes(&side.style).saturating_add(optional_string_bytes(side.color.as_ref()))
}

fn conditional_formatting_bytes(formatting: &SpreadsheetConditionalFormattingArtifact) -> usize {
    formatting
        .data_bar
        .as_ref()
        .map_or(0, data_bar_bytes)
        .saturating_add(formatting.icon.as_ref().map_or(0, icon_bytes))
        .saturating_add(formatting.rating.as_ref().map_or(0, rating_bytes))
}

fn data_bar_bytes(data_bar: &SpreadsheetDataBarArtifact) -> usize {
    optional_string_bytes(data_bar.positive_color.as_ref())
        .saturating_add(optional_string_bytes(data_bar.negative_color.as_ref()))
}

fn icon_bytes(icon: &SpreadsheetIconArtifact) -> usize {
    string_bytes(&icon.name).saturating_add(optional_string_bytes(icon.color.as_ref()))
}

fn rating_bytes(rating: &SpreadsheetRatingArtifact) -> usize {
    string_bytes(&rating.icon_name).saturating_add(optional_string_bytes(rating.color.as_ref()))
}
