use super::LocalHeader;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn single_entry() -> TestResult<Vec<u8>> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    writer.start_file("a.txt", SimpleFileOptions::default())?;
    writer.write_all(b"content")?;
    Ok(writer.finish()?.into_inner())
}

fn rejects_mutation(mutated: &[u8], original: &[u8]) -> TestResult {
    let mut archive = zip::ZipArchive::new(Cursor::new(original))?;
    let file = archive.by_index(0)?;
    assert!(LocalHeader::validate(mutated, &file).is_err());
    Ok(())
}

#[test]
fn rejects_invalid_signature_name_and_data_offset() -> TestResult {
    let original = single_entry()?;
    rejects_mutation(&[], &original)?;
    rejects_mutation(&original[..32], &original)?;

    let mut signature = original.clone();
    signature[0] = b'X';
    rejects_mutation(&signature, &original)?;

    let mut name = original.clone();
    name[30] = b'b';
    rejects_mutation(&name, &original)?;

    let mut offset = original.clone();
    offset[28..30].copy_from_slice(&1_u16.to_le_bytes());
    rejects_mutation(&offset, &original)
}

#[test]
fn rejects_mismatched_inline_metadata() -> TestResult {
    let original = single_entry()?;
    let mut crc = original.clone();
    crc[14..18].copy_from_slice(&0_u32.to_le_bytes());
    rejects_mutation(&crc, &original)
}
