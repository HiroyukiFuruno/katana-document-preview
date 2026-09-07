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
use mapping::{
    cell_content, cell_span, row_track_provider, spreadsheet_coordinate, track_provider,
};

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
        let (frozen_rows, frozen_columns) = adjusted_frozen_panes(sheet);
        let filtered_out_rows = sheet
            .auto_filter
            .as_ref()
            .map_or(&[][..], |filter| filter.filtered_out_rows.as_slice());
        let mut grid = GenericGrid::new(&sheet.name, sheet.row_count, sheet.column_count)
            .row_tracks(row_track_provider(
                &sheet.row_tracks,
                DEFAULT_ROW_SIZE,
                filtered_out_rows,
            ))
            .column_tracks(track_provider(&sheet.column_tracks, DEFAULT_COLUMN_SIZE))
            .viewport(GridViewport::new(viewport.width, viewport.height))
            .overscan(1, 1)
            .frozen(frozen_rows, frozen_columns)
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

    pub fn apply_command(&mut self, command: DocumentGridCommand) -> super::DocumentGridEvent {
        let Some(action) = grid_action(&self.grid, command) else {
            return super::DocumentGridEvent::None;
        };
        document_grid_event(self.grid.apply_action(action))
    }

    pub fn frame(&self) -> Result<DocumentSurfaceFrame, DocumentSurfaceError> {
        DocumentSurfaceFrame::from_node(self.grid.clone().into())
    }
}

fn adjusted_frozen_panes(sheet: &SpreadsheetSheetArtifact) -> (usize, usize) {
    let mut rows = sheet.frozen_rows.min(sheet.row_count);
    let mut columns = sheet.frozen_columns.min(sheet.column_count);
    loop {
        let previous = (rows, columns);
        for merged in &sheet.merged_cells {
            let row_end = merged.anchor.row.saturating_add(merged.row_span);
            if merged.anchor.row < rows && rows < row_end {
                rows = row_end.min(sheet.row_count);
            }
            let column_end = merged.anchor.column.saturating_add(merged.column_span);
            if merged.anchor.column < columns && columns < column_end {
                columns = column_end.min(sheet.column_count);
            }
        }
        if previous == (rows, columns) {
            return (rows, columns);
        }
    }
}

const fn document_grid_event(event: GridEvent) -> super::DocumentGridEvent {
    match event {
        GridEvent::None => super::DocumentGridEvent::None,
        GridEvent::SelectionChanged(_) => super::DocumentGridEvent::SelectionChanged,
        GridEvent::Scrolled(_) => super::DocumentGridEvent::Scrolled,
    }
}

fn grid_action(grid: &GenericGrid, command: DocumentGridCommand) -> Option<GridAction> {
    Some(match command {
        DocumentGridCommand::SelectAt { x, y, extend } => GridAction::Select {
            coordinate: grid.hit_test(x, y)?.coordinate,
            extend,
        },
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
    })
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
#[path = "spreadsheet_grid_appearance_tests.rs"]
mod appearance_tests;
#[cfg(test)]
#[path = "spreadsheet_grid_command_tests.rs"]
mod command_tests;
#[cfg(test)]
#[path = "spreadsheet_grid_filter_tests.rs"]
mod filter_tests;
#[cfg(test)]
#[path = "spreadsheet_grid_mapping_tests.rs"]
mod mapping_tests;
#[cfg(test)]
#[path = "spreadsheet_grid_pointer_tests.rs"]
mod pointer_tests;
#[path = "spreadsheet_grid_state.rs"]
mod state;
#[cfg(test)]
#[path = "spreadsheet_grid_test_support.rs"]
mod test_support;
#[cfg(test)]
#[path = "spreadsheet_grid_tests.rs"]
mod tests;
