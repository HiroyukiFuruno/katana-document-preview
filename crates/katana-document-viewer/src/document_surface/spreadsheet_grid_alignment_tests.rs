use super::{
    SpreadsheetGridSurface,
    test_support::{sample_cell, sample_sheet},
};
use crate::{
    DocumentSurfaceError, DocumentViewport, SpreadsheetCoordinate, SpreadsheetHorizontalAlignment,
    SpreadsheetVerticalAlignment,
};
use katana_ui_core::molecule::{GridHorizontalAlignment, GridVerticalAlignment};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn every_alignment_variant_maps_without_host_format_semantics() -> TestResult {
    for (source, expected) in horizontal_cases() {
        let mut cell = sample_cell(SpreadsheetCoordinate::new(0, 0));
        cell.style.horizontal_alignment = source;
        let mut surface = surface()?;
        surface.supply_cells(vec![cell])?;
        assert_eq!(
            expected,
            surface.frame().node().props().grid.cells[0]
                .appearance
                .horizontal_alignment
        );
    }
    assert_vertical_alignments()
}

fn assert_vertical_alignments() -> TestResult {
    for (source, expected) in vertical_cases() {
        let mut cell = sample_cell(SpreadsheetCoordinate::new(0, 0));
        cell.style.vertical_alignment = source;
        let mut surface = surface()?;
        surface.supply_cells(vec![cell])?;
        assert_eq!(
            expected,
            surface.frame().node().props().grid.cells[0]
                .appearance
                .vertical_alignment
        );
    }
    Ok(())
}

fn surface() -> Result<SpreadsheetGridSurface, DocumentSurfaceError> {
    SpreadsheetGridSurface::new(&sample_sheet(), DocumentViewport::new(100, 40))
}

fn horizontal_cases() -> [(SpreadsheetHorizontalAlignment, GridHorizontalAlignment); 8] {
    use GridHorizontalAlignment as Target;
    use SpreadsheetHorizontalAlignment as Source;
    [
        (Source::General, Target::General),
        (Source::Left, Target::Left),
        (Source::Center, Target::Center),
        (Source::CenterContinuous, Target::Center),
        (Source::Right, Target::Right),
        (Source::Fill, Target::Fill),
        (Source::Justify, Target::Justify),
        (Source::Distributed, Target::Distributed),
    ]
}

fn vertical_cases() -> [(SpreadsheetVerticalAlignment, GridVerticalAlignment); 5] {
    use GridVerticalAlignment as Target;
    use SpreadsheetVerticalAlignment as Source;
    [
        (Source::Bottom, Target::Bottom),
        (Source::Center, Target::Center),
        (Source::Top, Target::Top),
        (Source::Justify, Target::Justify),
        (Source::Distributed, Target::Distributed),
    ]
}
