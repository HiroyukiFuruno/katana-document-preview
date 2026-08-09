use super::{grid_action, navigation_intent};
use crate::{DocumentGridCommand, DocumentGridNavigation};
use katana_ui_core::molecule::{GenericGrid, GridAction, GridCoordinate, GridNavigationIntent};

#[test]
fn every_grid_command_and_navigation_variant_maps_to_kuc_internally() {
    let grid = GenericGrid::new("grid", 4, 4);
    assert_select_mapping(&grid);
    assert_navigation_mappings();
}

fn assert_select_mapping(grid: &GenericGrid) {
    assert!(matches!(
        grid_action(
            grid,
            DocumentGridCommand::Select {
                row: 2,
                column: 3,
                extend: true,
            }
        ),
        Some(GridAction::Select {
            coordinate: GridCoordinate { row: 2, column: 3 },
            extend: true,
        })
    ));
}

fn assert_navigation_mappings() {
    let mappings = [
        (DocumentGridNavigation::Left, GridNavigationIntent::Left),
        (DocumentGridNavigation::Right, GridNavigationIntent::Right),
        (DocumentGridNavigation::Up, GridNavigationIntent::Up),
        (DocumentGridNavigation::Down, GridNavigationIntent::Down),
        (DocumentGridNavigation::Home, GridNavigationIntent::Home),
        (DocumentGridNavigation::End, GridNavigationIntent::End),
        (DocumentGridNavigation::PageUp, GridNavigationIntent::PageUp),
        (
            DocumentGridNavigation::PageDown,
            GridNavigationIntent::PageDown,
        ),
    ];
    for (source, expected) in mappings {
        assert_eq!(expected, navigation_intent(source));
    }
}
