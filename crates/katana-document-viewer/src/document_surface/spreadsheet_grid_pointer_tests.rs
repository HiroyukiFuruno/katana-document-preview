use super::{SpreadsheetGridSurface, test_support::sample_sheet};
use crate::{
    DocumentGridCommand, DocumentGridCoordinate, DocumentGridEvent, DocumentSurfaceError,
    DocumentViewport,
};
use katana_ui_core::molecule::GridCoordinate;

type TestResult = Result<(), Box<dyn std::error::Error>>;
type VisibleCellHit = (i32, i32, GridCoordinate);

#[test]
fn pointer_selection_is_delegated_to_kuc_hit_testing() -> TestResult {
    let mut surface =
        SpreadsheetGridSurface::new(&sample_sheet(), DocumentViewport::new(320, 120))?;
    let Some((x, y, expected)) = visible_cell_hit(&surface)? else {
        return Err("KUC did not materialize a grid cell".into());
    };

    assert_eq!(
        DocumentGridEvent::SelectionChanged,
        surface.apply_command(DocumentGridCommand::SelectAt {
            x,
            y,
            extend: false,
        })
    );
    assert_eq!(Some(expected), surface.grid.active_coordinate());
    assert_missed_pointer_is_neutral(&mut surface);
    Ok(())
}

fn visible_cell_hit(
    surface: &SpreadsheetGridSurface,
) -> Result<Option<VisibleCellHit>, DocumentSurfaceError> {
    let frame = surface.frame()?;
    let Some(grid) = frame.grid() else {
        return Ok(None);
    };
    let Some(cell) = grid.cells.iter().find(|cell| {
        cell.clipped_bounds.width > 0
            && cell.clipped_bounds.height > 0
            && cell.coordinate != DocumentGridCoordinate { row: 0, column: 0 }
    }) else {
        return Ok(None);
    };
    Ok(Some((
        cell.clipped_bounds.x,
        cell.clipped_bounds.y,
        GridCoordinate::new(cell.coordinate.row, cell.coordinate.column),
    )))
}

fn assert_missed_pointer_is_neutral(surface: &mut SpreadsheetGridSurface) {
    assert_eq!(
        DocumentGridEvent::None,
        surface.apply_command(DocumentGridCommand::SelectAt {
            x: i32::MAX,
            y: i32::MAX,
            extend: false,
        })
    );
}
