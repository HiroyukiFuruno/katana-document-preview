use super::{
    OfficeWorkerError, SpreadsheetCoordinate, SpreadsheetDocumentArtifact, SpreadsheetSheetArtifact,
};
use std::collections::HashSet;

pub(super) struct SpreadsheetMaterializationValidator;

impl SpreadsheetMaterializationValidator {
    pub(super) fn validate(
        artifact: &SpreadsheetDocumentArtifact,
        limit: usize,
        sheet_index: usize,
        coordinates: &[SpreadsheetCoordinate],
    ) -> Result<(), OfficeWorkerError> {
        validate_count(coordinates.len(), limit)?;
        let sheet = artifact.sheets.get(sheet_index).ok_or_else(|| {
            spreadsheet_failure(format!("sheet index {sheet_index} is outside the document"))
        })?;
        validate_coordinates(sheet_index, sheet, coordinates)
    }
}

fn validate_count(actual: usize, limit: usize) -> Result<(), OfficeWorkerError> {
    if actual > limit {
        return Err(spreadsheet_failure(format!(
            "spreadsheet resource limit `materialized_cell_count` exceeded: {actual} > {limit}"
        )));
    }
    Ok(())
}

fn validate_coordinates(
    sheet_index: usize,
    sheet: &SpreadsheetSheetArtifact,
    coordinates: &[SpreadsheetCoordinate],
) -> Result<(), OfficeWorkerError> {
    let mut seen = HashSet::with_capacity(coordinates.len());
    for coordinate in coordinates {
        if coordinate.row >= sheet.row_count || coordinate.column >= sheet.column_count {
            return Err(spreadsheet_failure(format!(
                "cell ({}, {}) is outside sheet {sheet_index}",
                coordinate.row, coordinate.column
            )));
        }
        if !seen.insert(*coordinate) {
            return Err(spreadsheet_failure(format!(
                "cell ({}, {}) was requested more than once",
                coordinate.row, coordinate.column
            )));
        }
    }
    Ok(())
}

fn spreadsheet_failure(message: String) -> OfficeWorkerError {
    OfficeWorkerError::EngineFailure {
        stage: "spreadsheet".to_owned(),
        message,
    }
}
