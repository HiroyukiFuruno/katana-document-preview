use super::{
    SpreadsheetCellArtifact, SpreadsheetCoordinate, SpreadsheetSheetArtifact,
    SpreadsheetViewerLimits, spreadsheet_engine_cell::SpreadsheetCellMaterializer,
    spreadsheet_engine_sheet::SpreadsheetSheetBuilder,
    spreadsheet_filter_engine::SpreadsheetActiveFilters,
    spreadsheet_filter_xml::SpreadsheetFilterCatalog,
    spreadsheet_streaming::StreamingSpreadsheetSession,
};
use ironcalc::base::Model;
use std::collections::HashSet;

pub(crate) use super::spreadsheet_engine_support::SpreadsheetEngineSupport;
#[path = "spreadsheet_engine_error.rs"]
mod error;
pub(super) use error::SpreadsheetEngineError;

const LANGUAGE: &str = "en";
const LOCALE: &str = "en";
const TIMEZONE: &str = "UTC";

pub(super) struct SpreadsheetEngineSession {
    backend: SpreadsheetEngineBackend,
    sheets: Vec<SpreadsheetSheetArtifact>,
    limits: SpreadsheetViewerLimits,
    active_filters: SpreadsheetActiveFilters,
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
        let filters = SpreadsheetFilterCatalog::read(&bytes, limits.max_sheets)?;
        if StreamingSpreadsheetSession::is_required(&bytes)? {
            return Self::open_streaming(bytes, limits, filters);
        }
        Self::open_model(bytes, name, limits, filters)
    }

    fn open_model(
        bytes: Vec<u8>,
        name: &str,
        limits: SpreadsheetViewerLimits,
        filters: Vec<Option<super::SpreadsheetAutoFilterArtifact>>,
    ) -> Result<Self, SpreadsheetEngineError> {
        let workbook = match ironcalc::import::load_from_xlsx_bytes(&bytes, name, LOCALE, TIMEZONE)
        {
            Ok(workbook) => workbook,
            Err(error) => return Err(SpreadsheetEngineError::Import(error.to_string())),
        };
        let mut model =
            Model::from_workbook(workbook, LANGUAGE).map_err(SpreadsheetEngineError::Model)?;
        model.evaluate();
        let mut sheets =
            SpreadsheetSheetBuilder::build(&model, limits.max_sheets, limits.max_logical_cells)?;
        SpreadsheetFilterCatalog::attach(&mut sheets, filters);
        let mut session = Self {
            backend: SpreadsheetEngineBackend::Model(Box::new(model)),
            active_filters: Vec::new(),
            sheets,
            limits,
        };
        session.initialize_persisted_filters()?;
        Ok(session)
    }

    fn open_streaming(
        bytes: Vec<u8>,
        limits: SpreadsheetViewerLimits,
        filters: Vec<Option<super::SpreadsheetAutoFilterArtifact>>,
    ) -> Result<Self, SpreadsheetEngineError> {
        let streaming = StreamingSpreadsheetSession::open(bytes, limits)?;
        let mut sheets = streaming.sheets().to_vec();
        SpreadsheetFilterCatalog::attach(&mut sheets, filters);
        let mut session = Self {
            backend: SpreadsheetEngineBackend::Streaming(streaming),
            active_filters: Vec::new(),
            sheets,
            limits,
        };
        session.initialize_persisted_filters()?;
        Ok(session)
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

    pub(super) fn sheet(
        &self,
        requested: usize,
    ) -> Result<&SpreadsheetSheetArtifact, SpreadsheetEngineError> {
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

#[path = "spreadsheet_engine_filter.rs"]
mod filter;

#[cfg(test)]
#[path = "spreadsheet_engine_filter_error_tests.rs"]
mod filter_error_tests;
#[cfg(test)]
#[path = "spreadsheet_engine_filter_persisted_tests.rs"]
mod filter_persisted_tests;
#[cfg(test)]
#[path = "spreadsheet_engine_filter_tests.rs"]
mod filter_tests;
#[cfg(test)]
#[path = "spreadsheet_engine_tests.rs"]
mod tests;
