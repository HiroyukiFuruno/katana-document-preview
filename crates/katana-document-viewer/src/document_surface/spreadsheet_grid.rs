#[path = "spreadsheet_grid_mapping.rs"]
mod mapping;

use super::{
    DocumentGridCommand, DocumentGridNavigation, DocumentSurfaceError, DocumentSurfaceFrame,
    DocumentViewport,
};
use crate::{SpreadsheetCellArtifact, SpreadsheetCoordinate, SpreadsheetSheetArtifact};
use katana_ui_core::molecule::{
    GenericGrid, GridAction, GridCoordinate, GridEvent, GridNavigationIntent, GridViewport,
};
use katana_ui_core::render_model::UiGridValidationError;
use mapping::{cell_content, cell_span, spreadsheet_coordinate, track_provider};

const DEFAULT_ROW_SIZE: u32 = 20;
const DEFAULT_COLUMN_SIZE: u32 = 80;

impl From<UiGridValidationError> for DocumentSurfaceError {
    fn from(value: UiGridValidationError) -> Self {
        Self::InvalidGrid {
            detail: format!("{value:?}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpreadsheetGridSurface {
    sheet_index: usize,
    grid: GenericGrid,
}

impl SpreadsheetGridSurface {
    pub fn new(
        sheet: &SpreadsheetSheetArtifact,
        viewport: DocumentViewport,
    ) -> Result<Self, DocumentSurfaceError> {
        let mut grid = GenericGrid::new(&sheet.name, sheet.row_count, sheet.column_count)
            .row_tracks(track_provider(&sheet.row_tracks, DEFAULT_ROW_SIZE))
            .column_tracks(track_provider(&sheet.column_tracks, DEFAULT_COLUMN_SIZE))
            .viewport(GridViewport::new(viewport.width, viewport.height))
            .overscan(1, 1)
            .frozen(sheet.frozen_rows, sheet.frozen_columns)
            .show_grid_lines(sheet.show_grid_lines)
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
    ) -> Result<(), DocumentSurfaceError> {
        self.grid = self
            .grid
            .clone()
            .with_visible_cells(cells.into_iter().map(cell_content).collect())?;
        Ok(())
    }

    pub fn apply_command(&mut self, command: DocumentGridCommand) -> GridEvent {
        self.grid.apply_action(grid_action(command))
    }

    #[must_use]
    pub fn frame(&self) -> DocumentSurfaceFrame {
        DocumentSurfaceFrame::from_node(self.grid.clone().into())
    }
}

fn grid_action(command: DocumentGridCommand) -> GridAction {
    match command {
        DocumentGridCommand::ScrollTo { x, y } => GridAction::ScrollTo { x, y },
        DocumentGridCommand::Select {
            row,
            column,
            extend,
        } => GridAction::Select {
            coordinate: GridCoordinate::new(row, column),
            extend,
        },
        DocumentGridCommand::Navigate { intent, extend } => GridAction::Navigate {
            intent: navigation_intent(intent),
            extend,
        },
    }
}

const fn navigation_intent(intent: DocumentGridNavigation) -> GridNavigationIntent {
    match intent {
        DocumentGridNavigation::Left => GridNavigationIntent::Left,
        DocumentGridNavigation::Right => GridNavigationIntent::Right,
        DocumentGridNavigation::Up => GridNavigationIntent::Up,
        DocumentGridNavigation::Down => GridNavigationIntent::Down,
        DocumentGridNavigation::Home => GridNavigationIntent::Home,
        DocumentGridNavigation::End => GridNavigationIntent::End,
        DocumentGridNavigation::PageUp => GridNavigationIntent::PageUp,
        DocumentGridNavigation::PageDown => GridNavigationIntent::PageDown,
    }
}

#[cfg(test)]
#[path = "spreadsheet_grid_alignment_tests.rs"]
mod alignment_tests;
#[cfg(test)]
#[path = "spreadsheet_grid_command_tests.rs"]
mod command_tests;
#[cfg(test)]
#[path = "spreadsheet_grid_test_support.rs"]
mod test_support;
#[cfg(test)]
#[path = "spreadsheet_grid_tests.rs"]
mod tests;
