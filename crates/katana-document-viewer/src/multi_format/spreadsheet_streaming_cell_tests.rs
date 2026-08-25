use super::support::xml_cursor;
use crate::multi_format::spreadsheet_streaming_cell_reader::StreamingCellReader;
use crate::multi_format::spreadsheet_streaming_cell_types::{Capture, CellAccumulator};
use crate::multi_format::{SpreadsheetCellValue, SpreadsheetCoordinate};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn coordinates() -> Vec<SpreadsheetCoordinate> {
    (0..6)
        .map(|column| SpreadsheetCoordinate::new(0, column))
        .collect()
}

#[test]
fn cell_reader_covers_all_cell_types() -> TestResult {
    let coordinates = coordinates();
    let cells = StreamingCellReader::read(
        xml_cursor(br#"<worksheet><sheetData><row><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>99</v></c><c r="C1" t="str"><f>TEXT()</f><v>formula text</v></c><c r="D1" t="b"><v>true</v></c><c r="E1"><v>not-number</v></c><c r="F1" t="inlineStr"><is><t>inline</t></is></c><c r="Z1"><v>ignored</v></c><c/></row></sheetData></worksheet>"#),
        &coordinates,
        &["shared".to_owned()],
    )?;
    assert_eq!(cells[0].display_text, "shared");
    assert_eq!(cells[1].value, SpreadsheetCellValue::Empty);
    assert_eq!(cells[2].formula.as_deref(), Some("TEXT()"));
    assert_eq!(cells[3].display_text, "TRUE");
    assert_eq!(cells[4].display_text, "not-number");
    assert_eq!(cells[5].display_text, "inline");
    Ok(())
}

#[test]
fn cell_reader_rejects_xml_and_attribute_errors() {
    let coordinates = coordinates();
    for invalid in [
        b"<worksheet></row>".as_slice(),
        br#"<row r="bad"></row>"#.as_slice(),
        br#"<row r="&invalid;"><c r="A1"><v>1</v></c></row>"#.as_slice(),
        b"<c r=\"A1\"><v>\xff</v></c>".as_slice(),
        br#"<c r="&invalid;"><v/></c>"#.as_slice(),
    ] {
        assert!(StreamingCellReader::read(xml_cursor(invalid), &coordinates, &[]).is_err());
    }
}

#[test]
fn cell_reader_ignores_empty_unrequested_and_unknown_attributes() -> TestResult {
    let coordinates = coordinates();
    assert!(StreamingCellReader::read(xml_cursor(b"anything"), &[], &[])?.is_empty());
    let ignored = StreamingCellReader::read(
        xml_cursor(
            br#"<row r="1"><c unknown="value"><v>1</v></c><c r="invalid"><v>2</v></c></row>"#,
        ),
        &coordinates,
        &[],
    )?;
    assert!(
        ignored
            .iter()
            .all(|cell| cell.value == SpreadsheetCellValue::Empty)
    );
    let implicit_row = StreamingCellReader::read(
        xml_cursor(br#"<row unknown="value"><c r="A1"><v>1</v></c></row>"#),
        &coordinates,
        &[],
    )?;
    assert!(implicit_row.iter().any(|cell| cell.display_text == "1"));
    Ok(())
}

#[test]
fn cell_accumulator_covers_capture_and_display_variants() {
    let coordinate = SpreadsheetCoordinate::new(2, 3);
    let mut cell = CellAccumulator::new(0, coordinate, String::new());
    cell.append(Capture::None, "ignored");
    cell.append(Capture::Value, "12.5");
    assert_eq!(cell.result_index(), 0);
    assert_eq!(cell.finish(&[]).display_text, "12.5");
}
