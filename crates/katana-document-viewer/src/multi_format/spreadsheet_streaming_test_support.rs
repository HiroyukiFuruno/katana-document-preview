use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub(super) fn xml_cursor(bytes: &[u8]) -> Cursor<Vec<u8>> {
    Cursor::new(bytes.to_vec())
}

pub(super) fn workbook() -> TestResult<Vec<u8>> {
    zip_entries(&[
        (
            "xl/workbook.xml",
            r#"<workbook xmlns:r="urn:r"><sheets><sheet name="Large data" r:id="rId1"/></sheets></workbook>"#,
        ),
        (
            "xl/_rels/workbook.xml.rels",
            r#"<Relationships><Relationship Id="rId1" Target="worksheets/sheet1.xml"/></Relationships>"#,
        ),
        (
            "xl/worksheets/sheet1.xml",
            r#"<worksheet><dimension ref="A1:C2"/><sheetViews><sheetView showGridLines="1"><pane ySplit="1" xSplit="1"/></sheetView></sheetViews><sheetFormatPr defaultRowHeight="15" defaultColWidth="9"/><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>Header</t></is></c><c r="B1"><v>42.5</v></c><c r="C1" t="b"><v>1</v></c></row><row r="2"><c r="A2" t="inlineStr"><is><t>Second</t></is></c></row></sheetData></worksheet>"#,
        ),
    ])
}

pub(super) fn two_sheet_workbook() -> TestResult<Vec<u8>> {
    zip_entries(&[
        (
            "xl/workbook.xml",
            r#"<workbook><sheets><sheet name="One" id="r1"/><sheet name="Two" id="r2"/></sheets></workbook>"#,
        ),
        (
            "xl/_rels/workbook.xml.rels",
            r#"<Relationships><Relationship Id="r1" Target="worksheets/sheet1.xml"/><Relationship Id="r2" Target="worksheets/sheet2.xml"/></Relationships>"#,
        ),
        ("xl/worksheets/sheet1.xml", worksheet("A1:C1")),
        ("xl/worksheets/sheet2.xml", worksheet("A1:C2")),
        (
            "xl/sharedStrings.xml",
            r#"<sst><si><t>shared</t></si></sst>"#,
        ),
    ])
}

fn worksheet(dimension: &str) -> &'static str {
    match dimension {
        "A1:C1" => r#"<worksheet><dimension ref="A1:C1"/><sheetData/></worksheet>"#,
        _ => r#"<worksheet><dimension ref="A1:C2"/><sheetData/></worksheet>"#,
    }
}

pub(super) fn workbook_with_worksheet(worksheet: &str) -> TestResult<Vec<u8>> {
    zip_entries(&[
        ("xl/workbook.xml", single_workbook()),
        ("xl/_rels/workbook.xml.rels", single_relationship()),
        ("xl/worksheets/sheet1.xml", worksheet),
    ])
}

pub(super) fn workbook_without_worksheet() -> TestResult<Vec<u8>> {
    zip_entries(&[
        ("xl/workbook.xml", single_workbook()),
        (
            "xl/_rels/workbook.xml.rels",
            r#"<Relationships><Relationship Id="r1" Target="worksheets/missing.xml"/></Relationships>"#,
        ),
    ])
}

pub(super) fn large_workbook() -> TestResult<Vec<u8>> {
    let mut writer = streaming_writer()?;
    writer.start_file("xl/worksheets/sheet1.xml", deflated())?;
    writer.write_all(br#"<worksheet><dimension ref="A1"/><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>large</t></is></c></row></sheetData>"#)?;
    write_padding(&mut writer, 129)?;
    writer.write_all(b"</worksheet>")?;
    Ok(writer.finish()?.into_inner())
}

pub(super) fn large_shared_strings_workbook() -> TestResult<Vec<u8>> {
    let mut writer = streaming_writer()?;
    writer.start_file("xl/worksheets/sheet1.xml", deflated())?;
    writer.write_all(worksheet("A1:C1").as_bytes())?;
    writer.start_file("xl/sharedStrings.xml", deflated())?;
    write_padding(&mut writer, 17)?;
    Ok(writer.finish()?.into_inner())
}

pub(super) fn corrupt_shared_strings() -> TestResult<[Vec<u8>; 2]> {
    let name = b"xl/sharedStrings.xml";
    let mut header = two_sheet_workbook()?;
    let header_offset = entry_name_offset(&header, name)?;
    header[header_offset - 30] = 0;
    let mut payload = two_sheet_workbook()?;
    let payload_offset = entry_name_offset(&payload, name)?;
    payload[payload_offset + name.len() + 1] = b'/';
    Ok([header, payload])
}

pub(super) fn corrupt_deflated_workbook() -> TestResult<Vec<u8>> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = zip::ZipWriter::new(cursor);
    writer.start_file("xl/workbook.xml", deflated())?;
    let payload = vec![b'A'; 1024 * 1024];
    writer.write_all(&payload)?;
    let mut bytes = writer.finish()?.into_inner();
    let name = b"xl/workbook.xml";
    let offset = entry_name_offset(&bytes, name)? + name.len();
    bytes[offset..offset + 16].fill(0xff);
    Ok(bytes)
}

fn zip_entries(entries: &[(&str, &str)]) -> TestResult<Vec<u8>> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = zip::ZipWriter::new(cursor);
    for (name, content) in entries {
        writer.start_file(*name, SimpleFileOptions::default())?;
        writer.write_all(content.as_bytes())?;
    }
    Ok(writer.finish()?.into_inner())
}

fn streaming_writer() -> TestResult<zip::ZipWriter<Cursor<Vec<u8>>>> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, content) in [
        ("xl/workbook.xml", single_workbook()),
        ("xl/_rels/workbook.xml.rels", single_relationship()),
    ] {
        writer.start_file(name, deflated())?;
        writer.write_all(content.as_bytes())?;
    }
    Ok(writer)
}

fn write_padding(writer: &mut zip::ZipWriter<Cursor<Vec<u8>>>, megabytes: usize) -> TestResult {
    let padding = [b' '; 1024 * 1024];
    for _ in 0..megabytes {
        writer.write_all(&padding)?;
    }
    Ok(())
}

fn entry_name_offset(bytes: &[u8], name: &[u8]) -> TestResult<usize> {
    bytes
        .windows(name.len())
        .position(|window| window == name)
        .ok_or_else(|| "ZIP entry name".into())
}

fn deflated() -> SimpleFileOptions {
    SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated)
}

fn single_workbook() -> &'static str {
    r#"<workbook><sheets><sheet name="One" id="r1"/></sheets></workbook>"#
}

fn single_relationship() -> &'static str {
    r#"<Relationships><Relationship Id="r1" Target="worksheets/sheet1.xml"/></Relationships>"#
}
