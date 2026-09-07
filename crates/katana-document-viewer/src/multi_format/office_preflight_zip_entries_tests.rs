use super::office_preflight_zip_entries::OfficeZipEntries;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn duplicate_names_are_rejected_after_local_header_validation() -> TestResult {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    writer.start_file("duplicate.txt", SimpleFileOptions::default())?;
    writer.write_all(b"first")?;
    assert!(
        writer
            .start_file("duplicate.txt", SimpleFileOptions::default())
            .is_err()
    );
    Ok(())
}

#[test]
fn zip64_sentinel_skips_legacy_declared_count_comparison() {
    let mut bytes = vec![0; 22];
    bytes[..4].copy_from_slice(b"PK\x05\x06");
    bytes[8..12].copy_from_slice(&[0xff; 4]);
    assert!(OfficeZipEntries::validate_declared_count(&bytes, 7).is_ok());

    let mut mismatched = vec![0; 22];
    mismatched[..4].copy_from_slice(b"PK\x05\x06");
    mismatched[8..10].copy_from_slice(&1_u16.to_le_bytes());
    mismatched[10..12].copy_from_slice(&1_u16.to_le_bytes());
    assert!(OfficeZipEntries::validate_declared_count(&mismatched, 2).is_err());
}

#[test]
fn entry_and_eocd_failures_propagate_without_hiding_archive_errors() -> TestResult {
    assert!(OfficeZipEntries::validate_declared_count(b"missing", 0).is_err());

    let original = single_stored_entry()?;
    let mut archive = zip::ZipArchive::new(Cursor::new(original.as_slice()))?;
    assert!(OfficeZipEntries::validate_entry(&original, &mut archive, 1).is_err());

    let mut invalid_header = original.clone();
    invalid_header[0] = b'X';
    let mut archive = zip::ZipArchive::new(Cursor::new(original.as_slice()))?;
    assert!(OfficeZipEntries::validate_entry(&invalid_header, &mut archive, 0).is_err());

    let mut invalid_crc = original.clone();
    invalid_crc[35] ^= 0xff;
    let mut archive = zip::ZipArchive::new(Cursor::new(invalid_crc.as_slice()))?;
    assert!(OfficeZipEntries::validate_entry(&invalid_crc, &mut archive, 0).is_err());
    Ok(())
}

fn single_stored_entry() -> TestResult<Vec<u8>> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    writer.start_file(
        "a.txt",
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored),
    )?;
    writer.write_all(b"content")?;
    Ok(writer.finish()?.into_inner())
}
