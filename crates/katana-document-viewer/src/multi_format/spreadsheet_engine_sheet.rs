use super::{
    SpreadsheetCoordinate, SpreadsheetMergedCellArtifact, SpreadsheetSheetArtifact,
    SpreadsheetTrackArtifact,
    spreadsheet_engine::{SpreadsheetEngineError, SpreadsheetEngineSupport},
};
use ironcalc::base::Model;
use ironcalc::base::expressions::utils::parse_reference_a1;

pub(crate) struct SpreadsheetSheetBuilder;

impl SpreadsheetSheetBuilder {
    pub(crate) fn build(
        model: &Model<'_>,
        max_sheets: usize,
        max_logical_cells: usize,
    ) -> Result<Vec<SpreadsheetSheetArtifact>, SpreadsheetEngineError> {
        let names = model.workbook.get_worksheet_names();
        SpreadsheetEngineSupport::check_limit("sheet_count", names.len(), max_sheets)?;
        let mut total_cells = 0_usize;
        names
            .into_iter()
            .enumerate()
            .map(|(index, name)| {
                let sheet = Self::build_sheet(model, index, name)?;
                let logical_cells = sheet.row_count.saturating_mul(sheet.column_count);
                total_cells = total_cells.saturating_add(logical_cells);
                SpreadsheetEngineSupport::check_limit(
                    "logical_cell_count",
                    total_cells,
                    max_logical_cells,
                )?;
                Ok(sheet)
            })
            .collect()
    }

    fn build_sheet(
        model: &Model<'_>,
        index: usize,
        name: String,
    ) -> Result<SpreadsheetSheetArtifact, SpreadsheetEngineError> {
        let sheet_index = u32::try_from(index).map_err(SpreadsheetEngineSupport::model_error)?;
        let worksheet = model
            .workbook
            .worksheet(sheet_index)
            .map_err(SpreadsheetEngineError::Model)?;
        let dimension = worksheet.dimension();
        let row_count = SpreadsheetEngineSupport::positive_count(dimension.max_row)?;
        let column_count = SpreadsheetEngineSupport::positive_count(dimension.max_column)?;
        Ok(SpreadsheetSheetArtifact {
            index,
            name,
            row_count,
            column_count,
            row_tracks: Self::row_tracks(model, sheet_index, row_count)?,
            column_tracks: Self::column_tracks(model, sheet_index, column_count)?,
            frozen_rows: SpreadsheetEngineSupport::non_negative(worksheet.frozen_rows)?,
            frozen_columns: SpreadsheetEngineSupport::non_negative(worksheet.frozen_columns)?,
            merged_cells: Self::merged_cells(&worksheet.merge_cells)?,
            show_grid_lines: worksheet.show_grid_lines,
            auto_filter: None,
        })
    }

    fn row_tracks(
        model: &Model<'_>,
        sheet: u32,
        count: usize,
    ) -> Result<Vec<SpreadsheetTrackArtifact>, SpreadsheetEngineError> {
        (0..count)
            .map(|index| {
                let row = SpreadsheetEngineSupport::engine_index(index)?;
                Ok(SpreadsheetTrackArtifact {
                    size: SpreadsheetEngineSupport::track_size(model.get_row_height(sheet, row)?),
                    hidden: model.is_row_hidden(sheet, row)?,
                })
            })
            .collect::<Result<_, String>>()
            .map_err(SpreadsheetEngineError::Model)
    }

    fn column_tracks(
        model: &Model<'_>,
        sheet: u32,
        count: usize,
    ) -> Result<Vec<SpreadsheetTrackArtifact>, SpreadsheetEngineError> {
        (0..count)
            .map(|index| {
                let column = SpreadsheetEngineSupport::engine_index(index)?;
                Ok(SpreadsheetTrackArtifact {
                    size: SpreadsheetEngineSupport::track_size(
                        model.get_column_width(sheet, column)?,
                    ),
                    hidden: model.is_column_hidden(sheet, column)?,
                })
            })
            .collect::<Result<_, String>>()
            .map_err(SpreadsheetEngineError::Model)
    }

    fn merged_cells(
        ranges: &[String],
    ) -> Result<Vec<SpreadsheetMergedCellArtifact>, SpreadsheetEngineError> {
        ranges
            .iter()
            .map(|range| Self::merged_cell(range))
            .collect()
    }

    fn merged_cell(range: &str) -> Result<SpreadsheetMergedCellArtifact, SpreadsheetEngineError> {
        let Some((start, end)) = range.split_once(':') else {
            return Err(SpreadsheetEngineError::InvalidMergedCell(range.to_owned()));
        };
        let start = match parse_reference_a1(start) {
            Some(start) => start,
            None => return Err(SpreadsheetEngineError::InvalidMergedCell(range.to_owned())),
        };
        let end = match parse_reference_a1(end) {
            Some(end) => end,
            None => return Err(SpreadsheetEngineError::InvalidMergedCell(range.to_owned())),
        };
        if end.row < start.row || end.column < start.column {
            return Err(SpreadsheetEngineError::InvalidMergedCell(range.to_owned()));
        }
        Ok(SpreadsheetMergedCellArtifact {
            anchor: SpreadsheetCoordinate::new(
                SpreadsheetEngineSupport::zero_based(start.row)?,
                SpreadsheetEngineSupport::zero_based(start.column)?,
            ),
            row_span: SpreadsheetEngineSupport::span(start.row, end.row)?,
            column_span: SpreadsheetEngineSupport::span(start.column, end.column)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{SpreadsheetEngineError, SpreadsheetSheetBuilder};
    use crate::multi_format::SpreadsheetViewerLimits;
    use crate::multi_format::spreadsheet_engine::SpreadsheetEngineSession;

    #[test]
    fn logical_cell_limit_is_enforced_on_a_real_workbook() {
        let bytes = include_bytes!("../../../../assets/fixtures/multi-format/representative.xlsx");
        let limits = SpreadsheetViewerLimits {
            max_sheets: 16,
            max_logical_cells: 1,
            max_materialized_cells: 16,
        };
        assert!(matches!(
            SpreadsheetEngineSession::open(bytes.to_vec(), "representative.xlsx", limits),
            Err(SpreadsheetEngineError::ResourceLimit {
                kind: "logical_cell_count",
                ..
            })
        ));
    }

    #[test]
    fn malformed_merged_cell_ranges_are_rejected() {
        assert!(matches!(
            SpreadsheetSheetBuilder::merged_cell("A1"),
            Err(SpreadsheetEngineError::InvalidMergedCell(_))
        ));
        assert!(matches!(
            SpreadsheetSheetBuilder::merged_cell("B2:A1"),
            Err(SpreadsheetEngineError::InvalidMergedCell(_))
        ));
        assert!(matches!(
            SpreadsheetSheetBuilder::merged_cell(":A1"),
            Err(SpreadsheetEngineError::InvalidMergedCell(_))
        ));
        assert!(matches!(
            SpreadsheetSheetBuilder::merged_cell("A1:"),
            Err(SpreadsheetEngineError::InvalidMergedCell(_))
        ));
    }
}
