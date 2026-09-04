use super::super::{SpreadsheetEngineBackend, SpreadsheetEngineSession};
use crate::multi_format::SpreadsheetViewerLimits;
use std::io::Write;
use zip::write::SimpleFileOptions;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn streaming_filter_scans_large_sheet_once_per_operation_and_preserves_filter_semantics()
-> TestResult {
    let limits = SpreadsheetViewerLimits {
        max_sheets: 256,
        max_logical_cells: 25_000_000,
        max_materialized_cells: 2,
    };
    let mut engine =
        SpreadsheetEngineSession::open(large_streaming_filter_workbook()?, "filter.xlsx", limits)?;
    reset_streaming_filter_grid_scan_count(&engine)?;

    let (candidates, truncated) = engine.filter_candidates(0, 0, 16)?;
    assert_eq!(vec!["East", "North", "South", "West"], candidates);
    assert!(!truncated);
    assert_eq!(1, streaming_filter_grid_scan_count(&engine)?);

    reset_streaming_filter_grid_scan_count(&engine)?;
    let applied = engine.apply_filter(0, 0, vec!["North".to_owned()])?;
    assert_eq!(vec![2, 3, 4, 5], applied.filtered_out_rows);
    assert_eq!(3, applied.visible_row_count);
    assert_eq!(1, streaming_filter_grid_scan_count(&engine)?);

    let cleared = engine.clear_filter(0, None)?;
    assert!(cleared.applied_columns.is_empty());
    assert!(cleared.filtered_out_rows.is_empty());
    assert_eq!(7, cleared.visible_row_count);
    Ok(())
}

fn large_streaming_filter_workbook() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let cursor = std::io::Cursor::new(Vec::new());
    let mut writer = zip::ZipWriter::new(cursor);
    for (name, xml) in [
        (
            "xl/workbook.xml",
            r#"<workbook><sheets><sheet name="One" id="r1"/></sheets></workbook>"#,
        ),
        (
            "xl/_rels/workbook.xml.rels",
            r#"<Relationships><Relationship Id="r1" Target="worksheets/sheet1.xml"/></Relationships>"#,
        ),
    ] {
        writer.start_file(name, SimpleFileOptions::default())?;
        writer.write_all(xml.as_bytes())?;
    }
    writer.start_file(
        "xl/worksheets/sheet1.xml",
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
    )?;
    writer.write_all(
        br#"<worksheet><dimension ref="A1:B7"/><autoFilter ref="A1:B7"/><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>Region</t></is></c><c r="B1" t="inlineStr"><is><t>Score</t></is></c></row><row r="2"><c r="A2" t="inlineStr"><is><t>North</t></is></c><c r="B2"><v>98</v></c></row><row r="3"><c r="A3" t="inlineStr"><is><t>East</t></is></c><c r="B3"><v>87</v></c></row><row r="4"><c r="A4" t="inlineStr"><is><t>South</t></is></c><c r="B4"><v>76</v></c></row><row r="5"><c r="A5" t="inlineStr"><is><t>West</t></is></c><c r="B5"><v>65</v></c></row><row r="6"><c r="A6" t="inlineStr"><is><t>East</t></is></c><c r="B6"><v>54</v></c></row><row r="7"><c r="A7" t="inlineStr"><is><t>North</t></is></c><c r="B7"><v>43</v></c></row></sheetData></worksheet>"#,
    )?;
    let padding = [b' '; 1024 * 1024];
    for _ in 0..129 {
        writer.write_all(&padding)?;
    }
    Ok(writer.finish()?.into_inner())
}

fn reset_streaming_filter_grid_scan_count(engine: &SpreadsheetEngineSession) -> TestResult {
    match &engine.backend {
        SpreadsheetEngineBackend::Streaming(streaming) => {
            streaming.reset_filter_grid_scan_count();
            Ok(())
        }
        SpreadsheetEngineBackend::Model(_) => Err(std::io::Error::other(
            "large filter fixture must select the streaming backend",
        )
        .into()),
    }
}

fn streaming_filter_grid_scan_count(
    engine: &SpreadsheetEngineSession,
) -> Result<usize, Box<dyn std::error::Error>> {
    match &engine.backend {
        SpreadsheetEngineBackend::Streaming(streaming) => Ok(streaming.filter_grid_scan_count()),
        SpreadsheetEngineBackend::Model(_) => Err(std::io::Error::other(
            "large filter fixture must select the streaming backend",
        )
        .into()),
    }
}
