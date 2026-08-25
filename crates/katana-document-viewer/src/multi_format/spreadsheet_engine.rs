use super::{
    SpreadsheetCellArtifact, SpreadsheetCoordinate, SpreadsheetSheetArtifact,
    SpreadsheetViewerLimits, spreadsheet_engine_cell::SpreadsheetCellMaterializer,
    spreadsheet_engine_sheet::SpreadsheetSheetBuilder,
    spreadsheet_streaming::StreamingSpreadsheetSession,
};
use ironcalc::base::Model;
use std::collections::HashSet;
use thiserror::Error;

pub(crate) use super::spreadsheet_engine_support::SpreadsheetEngineSupport;

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
    backend: SpreadsheetEngineBackend,
    sheets: Vec<SpreadsheetSheetArtifact>,
    limits: SpreadsheetViewerLimits,
}

enum SpreadsheetEngineBackend {
    Model(Box<Model<'static>>),
    Streaming(StreamingSpreadsheetSession),
}

impl SpreadsheetEngineSession {
    pub(super) fn open(
        bytes: Vec<u8>,
        name: &str,
        limits: SpreadsheetViewerLimits,
    ) -> Result<Self, SpreadsheetEngineError> {
        if StreamingSpreadsheetSession::is_required(&bytes)? {
            let streaming = StreamingSpreadsheetSession::open(bytes, limits)?;
            let sheets = streaming.sheets().to_vec();
            return Ok(Self {
                backend: SpreadsheetEngineBackend::Streaming(streaming),
                sheets,
                limits,
            });
        }
        let workbook = ironcalc::import::load_from_xlsx_bytes(&bytes, name, LOCALE, TIMEZONE)
            .map_err(|error| SpreadsheetEngineError::Import(error.to_string()))?;
        let mut model =
            Model::from_workbook(workbook, LANGUAGE).map_err(SpreadsheetEngineError::Model)?;
        model.evaluate();
        let sheets =
            SpreadsheetSheetBuilder::build(&model, limits.max_sheets, limits.max_logical_cells)?;
        Ok(Self {
            backend: SpreadsheetEngineBackend::Model(Box::new(model)),
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
        match &self.backend {
            SpreadsheetEngineBackend::Model(model) => coordinates
                .iter()
                .copied()
                .map(|coordinate| {
                    SpreadsheetCellMaterializer::materialize(model, sheet_index, coordinate)
                })
                .collect(),
            SpreadsheetEngineBackend::Streaming(streaming) => {
                streaming.materialize(sheet_index, coordinates)
            }
        }
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

#[cfg(test)]
#[path = "spreadsheet_engine_tests.rs"]
mod tests;
