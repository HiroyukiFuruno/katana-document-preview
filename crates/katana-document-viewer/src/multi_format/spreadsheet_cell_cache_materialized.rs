use super::{OfficeWorkerError, SpreadsheetCellArtifact, SpreadsheetCoordinate};
use std::collections::{HashMap, HashSet};

pub(super) struct SpreadsheetMaterializedResponse {
    cells: HashMap<SpreadsheetCoordinate, SpreadsheetCellArtifact>,
}

impl SpreadsheetMaterializedResponse {
    pub(super) fn from_cells(
        coordinates: &[SpreadsheetCoordinate],
        materialized: Vec<SpreadsheetCellArtifact>,
    ) -> Result<Self, OfficeWorkerError> {
        let requested = coordinates.iter().copied().collect::<HashSet<_>>();
        let mut cells = HashMap::with_capacity(materialized.len());
        for cell in materialized {
            let coordinate = cell.coordinate;
            if !requested.contains(&coordinate) {
                return Err(materialization_error("unrequested", coordinate));
            }
            if cells.insert(coordinate, cell).is_some() {
                return Err(materialization_error("duplicate", coordinate));
            }
        }
        Ok(Self { cells })
    }

    pub(super) fn cell(
        &self,
        coordinate: SpreadsheetCoordinate,
    ) -> Option<&SpreadsheetCellArtifact> {
        self.cells.get(&coordinate)
    }

    pub(super) fn resolve(
        &self,
        coordinates: &[SpreadsheetCoordinate],
        cached: &HashMap<SpreadsheetCoordinate, SpreadsheetCellArtifact>,
    ) -> Result<Vec<SpreadsheetCellArtifact>, OfficeWorkerError> {
        let mut resolved = Vec::with_capacity(coordinates.len());
        for coordinate in coordinates {
            if let Some(cell) = self.cells.get(coordinate) {
                resolved.push(cell.clone());
                continue;
            }
            let Some(cell) = cached.get(coordinate) else {
                return Err(missing_materialization_error(*coordinate));
            };
            resolved.push(cell.clone());
        }
        Ok(resolved)
    }
}

fn materialization_error(reason: &str, coordinate: SpreadsheetCoordinate) -> OfficeWorkerError {
    OfficeWorkerError::protocol(format!(
        "spreadsheet materialization returned {reason} cell ({}, {})",
        coordinate.row, coordinate.column
    ))
}

fn missing_materialization_error(coordinate: SpreadsheetCoordinate) -> OfficeWorkerError {
    OfficeWorkerError::protocol(format!(
        "spreadsheet cell ({}, {}) was not materialized",
        coordinate.row, coordinate.column
    ))
}
