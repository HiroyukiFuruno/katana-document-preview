use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;

type TestResult<T> = Result<T, Box<dyn std::error::Error>>;

pub(super) fn representative_with_auto_filter() -> TestResult<Vec<u8>> {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/multi-format/representative.xlsx");
    inject_auto_filter(&std::fs::read(fixture)?)
}

fn inject_auto_filter(bytes: &[u8]) -> TestResult<Vec<u8>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_owned();
        let options = SimpleFileOptions::default().compression_method(entry.compression());
        if entry.is_dir() {
            output.add_directory(name, options)?;
            continue;
        }
        let mut content = Vec::new();
        entry.read_to_end(&mut content)?;
        if name == "xl/worksheets/sheet1.xml" {
            content = worksheet_with_filter(content)?;
        }
        output.start_file(name, options)?;
        output.write_all(&content)?;
    }
    Ok(output.finish()?.into_inner())
}

fn worksheet_with_filter(content: Vec<u8>) -> TestResult<Vec<u8>> {
    let xml = String::from_utf8(content)?;
    let filter = r#"<autoFilter ref="A3:F7"><filterColumn colId="0"><filters><filter val="North"/></filters></filterColumn></autoFilter>"#;
    if !xml.contains("</worksheet>") {
        return Err("worksheet closing tag is missing".into());
    }
    Ok(xml
        .replacen("</worksheet>", &format!("{filter}</worksheet>"), 1)
        .into_bytes())
}
