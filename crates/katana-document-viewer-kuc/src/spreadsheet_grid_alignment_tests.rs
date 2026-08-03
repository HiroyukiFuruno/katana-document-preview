use super::{
    KucSpreadsheetGridAdapter,
    test_support::{sample_cell, sample_sheet},
};
use katana_document_viewer::{
    SpreadsheetCoordinate, SpreadsheetHorizontalAlignment, SpreadsheetVerticalAlignment,
};
use katana_ui_core::molecule::{GridHorizontalAlignment, GridVerticalAlignment, GridViewport};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn every_alignment_variant_maps_without_format_semantics_in_kuc() -> TestResult {
    for (source, expected) in horizontal_cases() {
        let mut cell = sample_cell(SpreadsheetCoordinate::new(0, 0));
        cell.style.horizontal_alignment = source;
        let mut adapter = adapter()?;
        adapter.supply_cells(vec![cell])?;
        assert_eq!(
            expected,
            adapter.node().props().grid.cells[0]
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
        let mut adapter = adapter()?;
        adapter.supply_cells(vec![cell])?;
        assert_eq!(
            expected,
            adapter.node().props().grid.cells[0]
                .appearance
                .vertical_alignment
        );
    }
    Ok(())
}

fn adapter() -> Result<KucSpreadsheetGridAdapter, super::KucSpreadsheetGridError> {
    KucSpreadsheetGridAdapter::new(&sample_sheet(), GridViewport::new(100, 40))
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
