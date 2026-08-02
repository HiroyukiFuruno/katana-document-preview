use super::{
    SpreadsheetCellArtifact, SpreadsheetCoordinate, SpreadsheetSheetArtifact,
    SpreadsheetViewerLimits, spreadsheet_engine_cell::SpreadsheetCellMaterializer,
    spreadsheet_engine_sheet::SpreadsheetSheetBuilder,
};
use ironcalc::base::Model;
use std::collections::HashSet;
use thiserror::Error;

const LANGUAGE: &str = "en";
const LOCALE: &str = "en";
const TIMEZONE: &str = "UTC";

#[derive(Debug, Error)]
pub(super) enum SpreadsheetEngineError {
    #[error("XLSX import failed: {0}")]
    Import(String),
    #[error("spreadsheet model failed: {0}")]
    Model(String),
    #[error("spreadsheet resource limit `{kind}` exceeded: {actual} > {limit}")]
    ResourceLimit {
        kind: &'static str,
        actual: usize,
        limit: usize,
    },
    #[error("invalid merged-cell range `{0}`")]
    InvalidMergedCell(String),
    #[error("sheet index {requested} is outside the {sheet_count}-sheet workbook")]
    SheetOutsideDocument {
        requested: usize,
        sheet_count: usize,
    },
    #[error("cell ({row}, {column}) is outside sheet {sheet_index}")]
    CellOutsideSheet {
        sheet_index: usize,
        row: usize,
        column: usize,
    },
    #[error("cell ({row}, {column}) was requested more than once")]
    DuplicateCell { row: usize, column: usize },
}

pub(super) struct SpreadsheetEngineSession {
    model: Model<'static>,
    sheets: Vec<SpreadsheetSheetArtifact>,
    limits: SpreadsheetViewerLimits,
}

impl SpreadsheetEngineSession {
    pub(super) fn open(
        bytes: &[u8],
        name: &str,
        limits: SpreadsheetViewerLimits,
    ) -> Result<Self, SpreadsheetEngineError> {
        let workbook = ironcalc::import::load_from_xlsx_bytes(bytes, name, LOCALE, TIMEZONE)
            .map_err(|error| SpreadsheetEngineError::Import(error.to_string()))?;
        let mut model =
            Model::from_workbook(workbook, LANGUAGE).map_err(SpreadsheetEngineError::Model)?;
        model.evaluate();
        let sheets =
            SpreadsheetSheetBuilder::build(&model, limits.max_sheets, limits.max_logical_cells)?;
        Ok(Self {
            model,
            sheets,
            limits,
        })
    }

    pub(super) fn sheets(&self) -> &[SpreadsheetSheetArtifact] {
        &self.sheets
    }

    pub(super) fn materialize(
        &self,
        sheet_index: usize,
        coordinates: &[SpreadsheetCoordinate],
    ) -> Result<Vec<SpreadsheetCellArtifact>, SpreadsheetEngineError> {
        self.validate_request(sheet_index, coordinates)?;
        coordinates
            .iter()
            .copied()
            .map(|coordinate| {
                SpreadsheetCellMaterializer::materialize(&self.model, sheet_index, coordinate)
            })
            .collect()
    }

    fn validate_request(
        &self,
        sheet_index: usize,
        coordinates: &[SpreadsheetCoordinate],
    ) -> Result<(), SpreadsheetEngineError> {
        SpreadsheetEngineSupport::check_limit(
            "materialized_cell_count",
            coordinates.len(),
            self.limits.max_materialized_cells,
        )?;
        let sheet = self.sheet(sheet_index)?;
        let mut seen = HashSet::with_capacity(coordinates.len());
        for coordinate in coordinates {
            if coordinate.row >= sheet.row_count || coordinate.column >= sheet.column_count {
                return Err(Self::outside_cell(sheet_index, *coordinate));
            }
            if !seen.insert(*coordinate) {
                return Err(SpreadsheetEngineError::DuplicateCell {
                    row: coordinate.row,
                    column: coordinate.column,
                });
            }
        }
        Ok(())
    }

    fn sheet(&self, requested: usize) -> Result<&SpreadsheetSheetArtifact, SpreadsheetEngineError> {
        self.sheets
            .get(requested)
            .ok_or(SpreadsheetEngineError::SheetOutsideDocument {
                requested,
                sheet_count: self.sheets.len(),
            })
    }

    fn outside_cell(
        sheet_index: usize,
        coordinate: SpreadsheetCoordinate,
    ) -> SpreadsheetEngineError {
        SpreadsheetEngineError::CellOutsideSheet {
            sheet_index,
            row: coordinate.row,
            column: coordinate.column,
        }
    }
}

pub(crate) struct SpreadsheetEngineSupport;

impl SpreadsheetEngineSupport {
    pub(crate) fn check_limit(
        kind: &'static str,
        actual: usize,
        limit: usize,
    ) -> Result<(), SpreadsheetEngineError> {
        if actual <= limit {
            return Ok(());
        }
        Err(SpreadsheetEngineError::ResourceLimit {
            kind,
            actual,
            limit,
        })
    }

    pub(crate) fn track_size(size: f64) -> f32 {
        if size.is_finite() && size > 0.0 {
            size.min(f64::from(f32::MAX)) as f32
        } else {
            0.0
        }
    }

    pub(crate) fn positive_count(value: i32) -> Result<usize, SpreadsheetEngineError> {
        usize::try_from(value.max(1)).map_err(Self::model_error)
    }

    pub(crate) fn non_negative(value: i32) -> Result<usize, SpreadsheetEngineError> {
        usize::try_from(value.max(0)).map_err(Self::model_error)
    }

    pub(crate) fn zero_based(value: i32) -> Result<usize, SpreadsheetEngineError> {
        usize::try_from(value.saturating_sub(1)).map_err(Self::model_error)
    }

    pub(crate) fn span(start: i32, end: i32) -> Result<usize, SpreadsheetEngineError> {
        usize::try_from(end.saturating_sub(start).saturating_add(1)).map_err(Self::model_error)
    }

    pub(crate) fn engine_index(index: usize) -> Result<i32, String> {
        match i32::try_from(index.saturating_add(1)) {
            Ok(index) => Ok(index),
            Err(error) => Err(format!("cell index conversion failed: {error}")),
        }
    }

    pub(crate) fn model_error(error: std::num::TryFromIntError) -> SpreadsheetEngineError {
        SpreadsheetEngineError::Model(error.to_string())
    }

    pub(crate) fn engine_error(error: String) -> SpreadsheetEngineError {
        SpreadsheetEngineError::Model(error)
    }
}

#[cfg(test)]
#[path = "spreadsheet_engine_tests.rs"]
mod tests;
