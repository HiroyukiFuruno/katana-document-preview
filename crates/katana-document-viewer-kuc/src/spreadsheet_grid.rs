#[path = "spreadsheet_grid_mapping.rs"]
mod mapping;

use katana_document_viewer::{
    SpreadsheetCellArtifact, SpreadsheetCoordinate, SpreadsheetSheetArtifact,
};
use katana_ui_core::molecule::{GenericGrid, GridAction, GridCoordinate, GridEvent, GridViewport};
use katana_ui_core::render_model::{UiGridValidationError, UiNode};
use mapping::{cell_content, cell_span, spreadsheet_coordinate, track_provider};
use thiserror::Error;

const DEFAULT_ROW_SIZE: u32 = 20;
const DEFAULT_COLUMN_SIZE: u32 = 80;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KucSpreadsheetGridError {
    #[error("KDV spreadsheet artifact cannot be represented by the KUC grid: {0:?}")]
    InvalidGrid(UiGridValidationError),
}

impl From<UiGridValidationError> for KucSpreadsheetGridError {
    fn from(value: UiGridValidationError) -> Self {
        Self::InvalidGrid(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KucSpreadsheetGridAdapter {
    sheet_index: usize,
    grid: GenericGrid,
}

impl KucSpreadsheetGridAdapter {
    pub fn new(
        sheet: &SpreadsheetSheetArtifact,
        viewport: GridViewport,
    ) -> Result<Self, KucSpreadsheetGridError> {
        let mut grid = GenericGrid::new(&sheet.name, sheet.row_count, sheet.column_count)
            .row_tracks(track_provider(&sheet.row_tracks, DEFAULT_ROW_SIZE))
            .column_tracks(track_provider(&sheet.column_tracks, DEFAULT_COLUMN_SIZE))
            .viewport(viewport)
            .overscan(1, 1)
            .frozen(sheet.frozen_rows, sheet.frozen_columns)
            .with_cell_spans(sheet.merged_cells.iter().copied().map(cell_span).collect())?;
        if sheet.row_count > 0 && sheet.column_count > 0 {
            grid = grid.active_cell(GridCoordinate::new(0, 0));
        }
        Ok(Self {
            sheet_index: sheet.index,
            grid,
        })
    }

    #[must_use]
    pub const fn sheet_index(&self) -> usize {
        self.sheet_index
    }

    #[must_use]
    pub const fn grid(&self) -> &GenericGrid {
        &self.grid
    }

    #[must_use]
    pub fn materialization_request(&self) -> Vec<SpreadsheetCoordinate> {
        self.grid
            .visible_coordinates()
            .into_iter()
            .map(spreadsheet_coordinate)
            .collect()
    }

    pub fn supply_cells(
        &mut self,
        cells: Vec<SpreadsheetCellArtifact>,
    ) -> Result<(), KucSpreadsheetGridError> {
        self.grid = self
            .grid
            .clone()
            .with_visible_cells(cells.into_iter().map(cell_content).collect())?;
        Ok(())
    }

    pub fn apply_action(&mut self, action: GridAction) -> GridEvent {
        self.grid.apply_action(action)
    }

    #[must_use]
    pub fn node(&self) -> UiNode {
        self.grid.clone().into()
    }
}

#[cfg(test)]
#[path = "spreadsheet_grid_alignment_tests.rs"]
mod alignment_tests;
#[cfg(test)]
#[path = "spreadsheet_grid_test_support.rs"]
mod test_support;
#[cfg(test)]
#[path = "spreadsheet_grid_tests.rs"]
mod tests;
