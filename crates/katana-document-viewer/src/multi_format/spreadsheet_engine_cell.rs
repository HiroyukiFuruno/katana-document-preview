use super::{
    SpreadsheetCellArtifact, SpreadsheetCellStyleArtifact, SpreadsheetCellValue,
    SpreadsheetConditionalFormattingArtifact, SpreadsheetCoordinate, SpreadsheetDataBarArtifact,
    SpreadsheetHorizontalAlignment, SpreadsheetIconArtifact, SpreadsheetRatingArtifact,
    SpreadsheetVerticalAlignment,
    spreadsheet_engine::{SpreadsheetEngineError, SpreadsheetEngineSupport},
    spreadsheet_engine_cell_border::{borders, color},
};
use ironcalc::base::Model;
use ironcalc::base::cell::CellValue;
use ironcalc::base::cf_types::{CfDataBar, CfIcon, CfRating, ExtendedStyle};
use ironcalc::base::types::{Alignment, HorizontalAlignment, Style, Theme, VerticalAlignment};

pub(crate) struct SpreadsheetCellMaterializer;

type SpreadsheetCellData = (CellValue, String, Option<String>, Style, ExtendedStyle);

impl SpreadsheetCellMaterializer {
    pub(crate) fn materialize(
        model: &Model<'_>,
        sheet_index: usize,
        coordinate: SpreadsheetCoordinate,
    ) -> Result<SpreadsheetCellArtifact, SpreadsheetEngineError> {
        let (sheet, row, column) = Self::engine_coordinate(sheet_index, coordinate)?;
        let (value, display_text, formula, base, extended) =
            Self::cell_data(model, sheet, row, column)?;
        Ok(SpreadsheetCellArtifact {
            coordinate,
            display_text,
            value: Self::cell_value(value),
            formula,
            style: Self::cell_style(&extended.style, &model.workbook.theme),
            conditional_formatting: Self::conditional_formatting(
                &base,
                extended,
                &model.workbook.theme,
            ),
        })
    }

    fn engine_coordinate(
        sheet_index: usize,
        coordinate: SpreadsheetCoordinate,
    ) -> Result<(u32, i32, i32), SpreadsheetEngineError> {
        let sheet = u32::try_from(sheet_index).map_err(SpreadsheetEngineSupport::model_error)?;
        let row = SpreadsheetEngineSupport::engine_index(coordinate.row)
            .map_err(SpreadsheetEngineError::Model)?;
        let column = SpreadsheetEngineSupport::engine_index(coordinate.column)
            .map_err(SpreadsheetEngineError::Model)?;
        Ok((sheet, row, column))
    }

    fn cell_data(
        model: &Model<'_>,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<SpreadsheetCellData, SpreadsheetEngineError> {
        let value = model
            .get_cell_value_by_index(sheet, row, column)
            .map_err(SpreadsheetEngineSupport::engine_error)?;
        let display = model
            .get_formatted_cell_value(sheet, row, column)
            .map_err(SpreadsheetEngineSupport::engine_error)?;
        let formula = model
            .get_cell_formula(sheet, row, column)
            .map_err(SpreadsheetEngineSupport::engine_error)?;
        let style = model
            .get_style_for_cell(sheet, row, column)
            .map_err(SpreadsheetEngineSupport::engine_error)?;
        let extended = model
            .get_extended_style_for_cell(sheet, row, column)
            .map_err(SpreadsheetEngineSupport::engine_error)?;
        Ok((value, display, formula, style, extended))
    }

    fn cell_style(style: &Style, theme: &Theme) -> SpreadsheetCellStyleArtifact {
        let alignment = match style.alignment.as_ref() {
            Some(alignment) => alignment.clone(),
            None => Alignment::default(),
        };
        SpreadsheetCellStyleArtifact {
            font_name: style.font.name.clone(),
            font_size: style.font.sz.max(0) as f32,
            font_color: color(&style.font.color, theme),
            fill_color: color(&style.fill.color, theme),
            bold: style.font.b,
            italic: style.font.i,
            underline: style.font.u,
            strike: style.font.strike,
            horizontal_alignment: Self::horizontal_alignment(&alignment),
            vertical_alignment: Self::vertical_alignment(&alignment),
            wrap_text: alignment.wrap_text,
            number_format: style.num_fmt.clone(),
            borders: borders(&style.border, theme),
        }
    }

    fn conditional_formatting(
        base: &Style,
        extended: ExtendedStyle,
        theme: &Theme,
    ) -> SpreadsheetConditionalFormattingArtifact {
        let data_bar = extended
            .data_bar
            .map(|data_bar| Self::data_bar(data_bar, theme));
        let icon = extended.icon.map(|icon| Self::icon(icon, theme));
        let rating = extended.rating.map(|rating| Self::rating(rating, theme));
        SpreadsheetConditionalFormattingArtifact {
            applied: *base != extended.style
                || data_bar.is_some()
                || icon.is_some()
                || rating.is_some(),
            data_bar,
            icon,
            rating,
        }
    }

    fn data_bar(bar: CfDataBar, theme: &Theme) -> SpreadsheetDataBarArtifact {
        SpreadsheetDataBarArtifact {
            positive_color: color(&bar.positive_color, theme),
            negative_color: color(&bar.negative_color, theme),
            value: bar.value,
            axis_position: bar.axis_position,
            gradient: bar.is_gradient,
            show_value: bar.show_value,
        }
    }

    fn icon(icon: CfIcon, theme: &Theme) -> SpreadsheetIconArtifact {
        SpreadsheetIconArtifact {
            name: format!("{:?}", icon.icon),
            color: color(&icon.color, theme),
            show_value: icon.show_value,
        }
    }

    fn rating(rating: CfRating, theme: &Theme) -> SpreadsheetRatingArtifact {
        SpreadsheetRatingArtifact {
            icon_name: format!("{:?}", rating.icon),
            count: rating.count,
            maximum: rating.max,
            color: color(&rating.color, theme),
            show_value: rating.show_value,
        }
    }

    fn horizontal_alignment(alignment: &Alignment) -> SpreadsheetHorizontalAlignment {
        match alignment.horizontal {
            HorizontalAlignment::General => SpreadsheetHorizontalAlignment::General,
            HorizontalAlignment::Left => SpreadsheetHorizontalAlignment::Left,
            HorizontalAlignment::Center => SpreadsheetHorizontalAlignment::Center,
            HorizontalAlignment::CenterContinuous => {
                SpreadsheetHorizontalAlignment::CenterContinuous
            }
            HorizontalAlignment::Right => SpreadsheetHorizontalAlignment::Right,
            HorizontalAlignment::Fill => SpreadsheetHorizontalAlignment::Fill,
            HorizontalAlignment::Justify => SpreadsheetHorizontalAlignment::Justify,
            HorizontalAlignment::Distributed => SpreadsheetHorizontalAlignment::Distributed,
        }
    }

    fn vertical_alignment(alignment: &Alignment) -> SpreadsheetVerticalAlignment {
        match alignment.vertical {
            VerticalAlignment::Bottom => SpreadsheetVerticalAlignment::Bottom,
            VerticalAlignment::Center => SpreadsheetVerticalAlignment::Center,
            VerticalAlignment::Top => SpreadsheetVerticalAlignment::Top,
            VerticalAlignment::Justify => SpreadsheetVerticalAlignment::Justify,
            VerticalAlignment::Distributed => SpreadsheetVerticalAlignment::Distributed,
        }
    }

    fn cell_value(value: CellValue) -> SpreadsheetCellValue {
        match value {
            CellValue::None => SpreadsheetCellValue::Empty,
            CellValue::String(value) => SpreadsheetCellValue::Text(value),
            CellValue::Number(value) => SpreadsheetCellValue::Number(value),
            CellValue::Boolean(value) => SpreadsheetCellValue::Boolean(value),
        }
    }
}

#[cfg(test)]
#[path = "spreadsheet_engine_cell_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "spreadsheet_engine_cell_value_tests.rs"]
mod value_tests;
