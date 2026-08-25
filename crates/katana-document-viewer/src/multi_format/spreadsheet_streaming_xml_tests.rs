use super::super::{StreamingSpreadsheetSession, worksheet_path};
use super::support::{corrupt_deflated_workbook, workbook, workbook_without_worksheet, xml_cursor};
use crate::multi_format::SpreadsheetViewerLimits;
use crate::multi_format::spreadsheet_engine::SpreadsheetEngineError;
use crate::multi_format::spreadsheet_streaming_sheet_metadata::WorksheetMetadata;
use crate::multi_format::spreadsheet_streaming_xml::{
    WorkbookSheet, parse_relationships, parse_workbook_sheets, read_zip_entry,
};
use crate::multi_format::spreadsheet_streaming_xml_values::{
    attribute, attribute_f32, attribute_usize, decode_text, parse_shared_strings,
    required_attribute,
};
use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::HashMap;
use std::io::Cursor;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type EmptyEvent = (
    Reader<Cursor<Vec<u8>>>,
    quick_xml::events::BytesStart<'static>,
    Vec<u8>,
);

#[test]
fn metadata_parsers_cover_optional_and_invalid_values() -> TestResult {
    let strings = parse_shared_strings(
        br#"<sst><t>ignored</t><si><t>one &amp; </t><r><t>two</t></r></si><si></si></sst>"#,
    )?;
    assert_eq!(strings, ["one  two", ""]);
    for invalid in [
        b"<sst><si><t></si>".as_slice(),
        b"<sst><si><t>\xff</t></si></sst>".as_slice(),
        b"<?xml version=\"1.0\" encoding=\"windows-1252\"?><sst><si><t>\x80</t></si></sst>"
            .as_slice(),
    ] {
        assert!(parse_shared_strings(invalid).is_err());
    }
    assert!(decode_text(&[0xff]).is_err());
    assert!(decode_text(b"&invalid;").is_err());
    assert_workbook_and_relationship_parsers()?;
    Ok(())
}

fn assert_workbook_and_relationship_parsers() -> TestResult {
    let sheets =
        parse_workbook_sheets(br#"<workbook><ignored/><sheet name="One" id="r1"/></workbook>"#)?;
    assert_eq!(sheets[0].name, "One");
    assert!(parse_workbook_sheets(br#"<sheet name="One"/>"#).is_err());
    assert!(parse_workbook_sheets(b"<workbook></sheet>").is_err());
    let relationships = parse_relationships(
        br#"<Relationships><Relationship Id="r1" Target="/xl/worksheets/sheet1.xml"/><Relationship Id="missing"/><ignored/></Relationships>"#,
    )?;
    assert_eq!(relationships["r1"], "/xl/worksheets/sheet1.xml");
    assert!(parse_relationships(b"<Relationships></Relationship>").is_err());
    Ok(())
}

#[test]
fn attribute_helpers_cover_values_and_errors() -> TestResult {
    let (reader, event, buffer) = empty_event(br#"<node present="7" float="1.5" bad="nope"/>"#)?;
    assert_eq!(
        attribute(&reader, &event, b"present")?.as_deref(),
        Some("7")
    );
    assert_eq!(attribute(&reader, &event, b"missing")?, None);
    assert_eq!(attribute_f32(&reader, &event, b"float")?, Some(1.5));
    assert_eq!(attribute_f32(&reader, &event, b"missing")?, None);
    assert!(attribute_f32(&reader, &event, b"bad").is_err());
    assert_eq!(attribute_usize(&reader, &event, b"present")?, Some(7));
    assert_eq!(attribute_usize(&reader, &event, b"missing")?, None);
    assert!(attribute_usize(&reader, &event, b"bad").is_err());
    assert!(required_attribute(&reader, &event, b"missing").is_err());
    drop((event, buffer));
    assert_attribute_error(br#"<node ="value"/>"#)?;
    assert_attribute_error(br#"<node name="&invalid;"/>"#)?;
    Ok(())
}

fn empty_event(bytes: &[u8]) -> TestResult<EmptyEvent> {
    let mut reader = Reader::from_reader(xml_cursor(bytes));
    let mut buffer = Vec::new();
    let Event::Empty(event) = reader.read_event_into(&mut buffer)? else {
        return Err("expected empty XML event".into());
    };
    Ok((reader, event.into_owned(), buffer))
}

fn assert_attribute_error(bytes: &[u8]) -> TestResult {
    let (reader, event, buffer) = empty_event(bytes)?;
    assert!(attribute(&reader, &event, b"name").is_err());
    drop(buffer);
    Ok(())
}

#[test]
fn worksheet_metadata_covers_defaults_overrides_and_errors() -> TestResult {
    let metadata = WorksheetMetadata::read(xml_cursor(
        br#"<worksheet><dimension/><sheetView showGridLines="false"/><sheetFormatPr defaultRowHeight="18" defaultColWidth="10"/><pane ySplit="2" xSplit="3"/><ignored/><sheetData/></worksheet>"#,
    ))?;
    assert_eq!((metadata.row_count, metadata.column_count), (1, 1));
    assert_eq!((metadata.frozen_rows, metadata.frozen_columns), (2, 3));
    assert!(!metadata.show_grid_lines);
    assert_eq!((metadata.row_height, metadata.column_width), (24.0, 70.0));
    assert_worksheet_metadata_errors()?;
    Ok(())
}

fn assert_worksheet_metadata_errors() -> TestResult {
    let eof = WorksheetMetadata::read(xml_cursor(b"<worksheet></worksheet>"))?;
    assert_eq!((eof.row_count, eof.column_count), (1, 1));
    for invalid in [
        b"<worksheet></sheetData>".as_slice(),
        br#"<dimension ref="invalid"/>"#.as_slice(),
        br#"<sheetFormatPr defaultRowHeight="bad"/>"#.as_slice(),
        br#"<pane xSplit="bad"/>"#.as_slice(),
    ] {
        assert!(WorksheetMetadata::read(xml_cursor(invalid)).is_err());
    }
    assert!(
        !WorksheetMetadata::read(xml_cursor(br#"<sheetView showGridLines="0"/>"#))?.show_grid_lines
    );
    assert!(WorksheetMetadata::read(xml_cursor(br#"<sheetView/>"#))?.show_grid_lines);
    assert!(
        WorksheetMetadata::read(xml_cursor(br#"<sheetView showGridLines="true"/>"#))?
            .show_grid_lines
    );
    Ok(())
}

#[test]
fn worksheet_paths_and_zip_errors_are_typed() -> TestResult {
    let sheet = WorkbookSheet {
        name: "One".into(),
        relationship_id: "r1".into(),
    };
    for target in ["./worksheets/sheet1.xml", "/xl/worksheets/sheet1.xml"] {
        let targets = HashMap::from([("r1".into(), target.into())]);
        assert_eq!(
            worksheet_path(&sheet, &targets)?,
            "xl/worksheets/sheet1.xml"
        );
    }
    assert!(worksheet_path(&sheet, &HashMap::new()).is_err());
    assert_zip_entry_errors()?;
    Ok(())
}

fn assert_zip_entry_errors() -> TestResult {
    let bytes = workbook()?;
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes.as_slice()))?;
    assert!(matches!(
        read_zip_entry(&mut archive, "xl/workbook.xml", 1),
        Err(SpreadsheetEngineError::ResourceLimit { .. })
    ));
    assert!(read_zip_entry(&mut archive, "missing", 100).is_err());
    let corrupt = corrupt_deflated_workbook()?;
    let mut corrupt_archive = zip::ZipArchive::new(Cursor::new(corrupt.as_slice()))?;
    assert!(read_zip_entry(&mut corrupt_archive, "xl/workbook.xml", 2_000_000).is_err());
    assert!(
        StreamingSpreadsheetSession::open(
            workbook_without_worksheet()?,
            SpreadsheetViewerLimits::strict()
        )
        .is_err()
    );
    Ok(())
}
