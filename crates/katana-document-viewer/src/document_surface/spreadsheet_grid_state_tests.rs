use super::{nearest_visible_row, restore_selection};
use crate::{SpreadsheetSheetArtifact, SpreadsheetTrackArtifact};
use katana_ui_core::molecule::{GenericGrid, GridCoordinate, GridSelection};

fn sheet(hidden_rows: &[usize]) -> SpreadsheetSheetArtifact {
    let mut sheet = super::super::test_support::sample_sheet();
    sheet.row_count = 4;
    sheet.column_count = 4;
    sheet.row_tracks = (0..4)
        .map(|row| SpreadsheetTrackArtifact {
            size: 20.0,
            hidden: hidden_rows.contains(&row),
        })
        .collect();
    sheet
}

#[test]
fn selection_restore_handles_empty_extended_and_hidden_ranges() {
    let mut grid = GenericGrid::new("grid", 4, 4);
    restore_selection(&mut grid, &sheet(&[]), None);
    assert_eq!(None, grid.selection());

    let forward = sheet(&[1]);
    restore_selection(
        &mut grid,
        &forward,
        Some(GridSelection::new(
            GridCoordinate::new(1, 0),
            GridCoordinate::new(3, 1),
        )),
    );
    assert_eq!(
        Some(GridSelection::new(
            GridCoordinate::new(2, 0),
            GridCoordinate::new(3, 1)
        )),
        grid.selection()
    );

    let backward = sheet(&[2, 3]);
    assert_eq!(1, nearest_visible_row(&backward, 3));
}
