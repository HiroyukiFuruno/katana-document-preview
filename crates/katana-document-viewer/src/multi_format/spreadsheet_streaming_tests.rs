use super::{StreamingSpreadsheetSession, worksheet_requires_streaming};
use crate::multi_format::spreadsheet_engine::{SpreadsheetEngineError, SpreadsheetEngineSession};
use crate::multi_format::{SpreadsheetCellValue, SpreadsheetCoordinate, SpreadsheetViewerLimits};

#[path = "spreadsheet_streaming_cell_tests.rs"]
mod cell_tests;
#[path = "spreadsheet_streaming_test_support.rs"]
mod support;
#[path = "spreadsheet_streaming_xml_tests.rs"]
mod xml_tests;

use support::{
    corrupt_shared_strings, large_shared_strings_workbook, large_workbook, two_sheet_workbook,
    workbook, workbook_with_worksheet,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn session_materializes_requested_cell_types() -> TestResult {
    let session =
        StreamingSpreadsheetSession::open(workbook()?, SpreadsheetViewerLimits::strict())?;
    assert_eq!(session.sheets()[0].name, "Large data");
    assert_eq!(
        (
            session.sheets()[0].row_count,
            session.sheets()[0].column_count
        ),
        (2, 3)
    );
    let cells = session.materialize(
        0,
        &[
            SpreadsheetCoordinate::new(0, 0),
            SpreadsheetCoordinate::new(0, 1),
            SpreadsheetCoordinate::new(0, 2),
            SpreadsheetCoordinate::new(1, 2),
        ],
    )?;
    assert_eq!(
        cells[0].value,
        SpreadsheetCellValue::Text("Header".to_owned())
    );
    assert_eq!(cells[1].value, SpreadsheetCellValue::Number(42.5));
    assert_eq!(cells[2].value, SpreadsheetCellValue::Boolean(true));
    assert_eq!(cells[3].value, SpreadsheetCellValue::Empty);
    Ok(())
}

#[test]
fn only_large_worksheet_xml_entries_select_streaming() {
    let large = 128 * 1024 * 1024 + 1;
    assert!(worksheet_requires_streaming(
        "xl/worksheets/sheet1.xml",
        large
    ));
    assert!(!worksheet_requires_streaming("word/document.xml", large));
    assert!(!worksheet_requires_streaming(
        "xl/worksheets/sheet1.xml",
        large - 1
    ));
}

#[test]
fn session_rejects_invalid_archives_and_sheet_indices() -> TestResult {
    assert!(matches!(
        StreamingSpreadsheetSession::is_required(b"not a zip"),
        Err(SpreadsheetEngineError::Import(_))
    ));
    let session =
        StreamingSpreadsheetSession::open(workbook()?, SpreadsheetViewerLimits::strict())?;
    assert!(matches!(
        session.materialize(1, &[]),
        Err(SpreadsheetEngineError::SheetOutsideDocument {
            requested: 1,
            sheet_count: 1
        })
    ));
    Ok(())
}

#[test]
fn large_compressed_worksheet_uses_streaming_backend() -> TestResult {
    let bytes = large_workbook()?;
    assert!(StreamingSpreadsheetSession::is_required(&bytes)?);
    let session =
        SpreadsheetEngineSession::open(bytes, "large.xlsx", SpreadsheetViewerLimits::strict())?;
    let cells = session.materialize(0, &[SpreadsheetCoordinate::new(0, 0)])?;
    assert_eq!(cells[0].display_text, "large");
    Ok(())
}

#[test]
fn session_enforces_sheet_and_cell_limits() -> TestResult {
    let bytes = two_sheet_workbook()?;
    let session =
        StreamingSpreadsheetSession::open(bytes.clone(), SpreadsheetViewerLimits::strict())?;
    assert_eq!(session.sheets().len(), 2);
    assert_limit(&bytes, 1, 100, "sheet_count");
    assert_limit(&bytes, 2, 6, "logical_cell_count");
    assert_limit(&bytes, 2, 5, "logical_cell_count");
    Ok(())
}

fn assert_limit(bytes: &[u8], max_sheets: usize, max_cells: usize, kind: &'static str) {
    assert!(matches!(
        StreamingSpreadsheetSession::open(
            bytes.to_vec(),
            SpreadsheetViewerLimits {
                max_sheets,
                max_logical_cells: max_cells,
                max_materialized_cells: 10,
            }
        ),
        Err(SpreadsheetEngineError::ResourceLimit { kind: actual, .. }) if actual == kind
    ));
}

#[test]
fn session_rejects_shared_string_errors() -> TestResult {
    assert!(matches!(
        StreamingSpreadsheetSession::open(
            large_shared_strings_workbook()?,
            SpreadsheetViewerLimits::strict()
        ),
        Err(SpreadsheetEngineError::ResourceLimit {
            kind: "spreadsheet_metadata_bytes",
            ..
        })
    ));
    for bytes in corrupt_shared_strings()? {
        assert!(matches!(
            StreamingSpreadsheetSession::open(bytes, SpreadsheetViewerLimits::strict()),
            Err(SpreadsheetEngineError::Import(_))
        ));
    }
    Ok(())
}

#[test]
fn session_rejects_malformed_worksheet_metadata() -> TestResult {
    for worksheet in [
        r#"<worksheet><sheetFormatPr defaultRowHeight="bad"/><sheetData/></worksheet>"#,
        r#"<worksheet><pane xSplit="bad"/><sheetData/></worksheet>"#,
        "<worksheet></sheetData>",
    ] {
        assert!(
            StreamingSpreadsheetSession::open(
                workbook_with_worksheet(worksheet)?,
                SpreadsheetViewerLimits::strict()
            )
            .is_err()
        );
    }
    Ok(())
}
