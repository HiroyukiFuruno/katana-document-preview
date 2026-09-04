use super::*;
use crate::multi_format::{SpreadsheetCellArtifact, SpreadsheetCoordinate};
use quick_xml::{Reader, events::Event};
use std::io::Cursor;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn xml_cursor(bytes: &[u8]) -> Cursor<Vec<u8>> {
    Cursor::new(bytes.to_vec())
}

#[test]
fn filter_reader_parses_requested_cells_and_ignores_unrequested_xml() -> TestResult {
    let xml = br#"<worksheet><sheetData><row><c r="A1" t="inlineStr"><is><t>kept</t></is></c><c r="C1"><v>ignored</v></c><c><v>missing reference</v></c></row><row r="3"><c r="A3"><f>1+1</f><v>2</v></c></row></sheetData></worksheet>"#;
    let mut visits = Vec::new();
    let mut input = xml_cursor(xml);
    let mut collect_visits = |rows: std::ops::Range<usize>, cells: Vec<SpreadsheetCellArtifact>| {
        visits.push((rows, cells));
        Ok(())
    };
    StreamingFilterGridReader::read(&mut input, &[0], 0..4, 2, &[], &mut collect_visits)?;
    assert_eq!(2, visits.len());
    assert_eq!("kept", visits[0].1[0].display_text);
    assert_eq!("2", visits[1].1[0].display_text);
    assert_eq!(Some("1+1"), visits[1].1[0].formula.as_deref());
    Ok(())
}

#[test]
fn filter_reader_rejects_mismatched_xml_tags() -> TestResult {
    let mut visits = 0;
    let mut input = xml_cursor(b"<worksheet></sheetData>");
    let mut count_visits = |_: std::ops::Range<usize>, _: Vec<SpreadsheetCellArtifact>| {
        visits += 1;
        Ok(())
    };
    let result = StreamingFilterGridReader::read(&mut input, &[0], 0..1, 1, &[], &mut count_visits);
    assert!(result.is_err());
    assert_eq!(0, visits);
    Ok(())
}

#[test]
fn filter_reader_rejects_invalid_row_attributes() -> TestResult {
    let mut visits = 0;
    let mut input = xml_cursor(b"<worksheet><row r=\"not-a-number\"></row></worksheet>");
    let mut count_visits = |_: std::ops::Range<usize>, _: Vec<SpreadsheetCellArtifact>| {
        visits += 1;
        Ok(())
    };
    let result = StreamingFilterGridReader::read(&mut input, &[0], 0..1, 1, &[], &mut count_visits);
    assert!(result.is_err());
    assert_eq!(0, visits);
    Ok(())
}

#[test]
fn filter_reader_rejects_non_utf8_cell_text() -> TestResult {
    let mut visits = 0;
    let mut input = xml_cursor(b"<worksheet><row><c r=\"A1\"><v>\xff</v></c></row></worksheet>");
    let mut count_visits = |_: std::ops::Range<usize>, _: Vec<SpreadsheetCellArtifact>| {
        visits += 1;
        Ok(())
    };
    let result = StreamingFilterGridReader::read(&mut input, &[0], 0..1, 1, &[], &mut count_visits);
    assert!(result.is_err());
    assert_eq!(0, visits);
    Ok(())
}

#[test]
fn filter_reader_emits_empty_chunks_at_eof() -> TestResult {
    let mut chunks = Vec::new();
    let mut input = xml_cursor(b"<worksheet/>");
    let mut collect_chunks = |rows: std::ops::Range<usize>, cells: Vec<SpreadsheetCellArtifact>| {
        chunks.push((rows, cells));
        Ok(())
    };
    StreamingFilterGridReader::read(&mut input, &[0], 0..2, 1, &[], &mut collect_chunks)?;
    assert_eq!(
        vec![0..1, 1..2],
        chunks
            .iter()
            .map(|(rows, _)| rows.clone())
            .collect::<Vec<_>>()
    );
    assert!(chunks.into_iter().all(|(_, cells)| {
        cells
            .into_iter()
            .all(|cell| cell.value == crate::multi_format::SpreadsheetCellValue::Empty)
    }));
    Ok(())
}

#[test]
fn filter_reader_parses_cell_attribute_edges() -> TestResult {
    let mut reader = Reader::from_reader(xml_cursor(br#"<c r="A1" t="inlineStr" ignored="x"/>"#));
    let mut buffer = Vec::new();
    let Event::Empty(event) = reader.read_event_into(&mut buffer)? else {
        return Err("expected empty cell".into());
    };
    let (coordinate, cell_type) = cell_attributes(&event)?;
    assert_eq!(coordinate, Some(SpreadsheetCoordinate::new(0, 0)));
    assert_eq!(cell_type, "inlineStr");
    assert_eq!(filter_coordinate("not-a-cell"), None);
    assert_eq!(filter_coordinate("A0"), None);
    Ok(())
}

#[test]
fn filter_reader_converts_one_based_row_attributes() -> TestResult {
    let mut reader = Reader::from_reader(xml_cursor(br#"<row r="1"/>"#));
    let mut buffer = Vec::new();
    let Event::Empty(row) = reader.read_event_into(&mut buffer)? else {
        return Err("expected empty row".into());
    };
    assert_eq!(filter_row(&row)?, Some(0));
    Ok(())
}

#[test]
fn filter_reader_parses_row_attribute_edges() -> TestResult {
    let mut row_reader = Reader::from_reader(xml_cursor(br#"<row r="0"/>"#));
    let mut row_buffer = Vec::new();
    let Event::Empty(row) = row_reader.read_event_into(&mut row_buffer)? else {
        return Err("expected empty row".into());
    };
    assert_eq!(filter_row(&row)?, None);

    let mut no_row_reader = Reader::from_reader(xml_cursor(br#"<row ignored="x"/>"#));
    let mut no_row_buffer = Vec::new();
    let Event::Empty(no_row) = no_row_reader.read_event_into(&mut no_row_buffer)? else {
        return Err("expected empty row".into());
    };
    assert_eq!(filter_row(&no_row)?, None);
    Ok(())
}
