use super::office_preflight::{OfficePreflightError, OfficePreflightSupport};
use std::collections::HashSet;
use std::io::{Cursor, sink};
use zip::read::read_zipfile_from_stream;

pub(super) struct OfficeZipEntries;

impl OfficeZipEntries {
    pub(super) fn validate(
        bytes: &[u8],
        expected_entries: usize,
    ) -> Result<(), OfficePreflightError> {
        let mut cursor = Cursor::new(bytes);
        let mut names = HashSet::with_capacity(expected_entries);
        while let Some(mut file) =
            read_zipfile_from_stream(&mut cursor).map_err(OfficePreflightSupport::archive_error)?
        {
            Self::record_name(&mut names, &file)?;
            std::io::copy(&mut file, &mut sink()).map_err(OfficePreflightSupport::archive_error)?;
        }
        Self::validate_count(names.len(), expected_entries)
    }

    fn record_name(
        names: &mut HashSet<Vec<u8>>,
        file: &zip::read::ZipFile<'_, Cursor<&[u8]>>,
    ) -> Result<(), OfficePreflightError> {
        if names.insert(file.name_raw().to_vec()) {
            return Ok(());
        }
        Err(OfficePreflightSupport::invalid_archive(format!(
            "duplicate entry `{}`",
            file.name()
        )))
    }

    fn validate_count(found: usize, expected: usize) -> Result<(), OfficePreflightError> {
        if found == expected {
            return Ok(());
        }
        Err(OfficePreflightSupport::invalid_archive(format!(
            "local ZIP headers contain {found} entries but archive exposes {expected}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::OfficeZipEntries;
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;

    type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

    fn single_entry_zip() -> TestResult<Vec<u8>> {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer.start_file("word/document.xml", SimpleFileOptions::default())?;
        writer.write_all(b"payload")?;
        Ok(writer.finish()?.into_inner())
    }

    #[test]
    fn valid_local_entry_count_is_accepted() -> TestResult {
        assert!(OfficeZipEntries::validate(&single_entry_zip()?, 1).is_ok());
        Ok(())
    }

    #[test]
    fn local_entry_count_must_match_the_archive_directory() {
        assert!(matches!(
            OfficeZipEntries::validate_count(1, 2),
            Err(error)
                if error.to_string()
                    == "Office package is invalid: local ZIP headers contain 1 entries but archive exposes 2"
        ));
    }

    #[test]
    fn truncated_local_header_is_a_typed_archive_error() {
        assert!(matches!(
            OfficeZipEntries::validate(b"PK\x03\x04", 1),
            Err(error) if error.to_string().starts_with("Office package is invalid:")
        ));
    }

    #[test]
    fn corrupt_entry_payload_is_a_typed_archive_error() -> TestResult {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        writer.start_file("word/document.xml", options)?;
        writer.write_all(b"payload")?;
        let mut bytes = writer.finish()?.into_inner();
        let name_len = usize::from(u16::from_le_bytes([bytes[26], bytes[27]]));
        let extra_len = usize::from(u16::from_le_bytes([bytes[28], bytes[29]]));
        bytes[30 + name_len + extra_len] ^= 0xff;

        assert!(matches!(
            OfficeZipEntries::validate(&bytes, 1),
            Err(error) if error.to_string().starts_with("Office package is invalid:")
        ));
        Ok(())
    }

    #[test]
    fn duplicate_local_entry_name_is_a_typed_archive_error() -> TestResult {
        const CANONICAL: &[u8] = b"word/document.xml";
        const ALIAS: &[u8] = b"word/documenz.xml";
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, payload) in [
            ("word/document.xml", b"first".as_slice()),
            ("word/documenz.xml", b"second".as_slice()),
        ] {
            writer.start_file(name, SimpleFileOptions::default())?;
            writer.write_all(payload)?;
        }
        let mut bytes = writer.finish()?.into_inner();
        let mut replacements = 0;
        for offset in 0..=bytes.len().saturating_sub(ALIAS.len()) {
            if bytes[offset..offset + ALIAS.len()] == *ALIAS {
                bytes[offset..offset + ALIAS.len()].copy_from_slice(CANONICAL);
                replacements += 1;
            }
        }
        assert_eq!(2, replacements);

        assert!(matches!(
            OfficeZipEntries::validate(&bytes, 2),
            Err(error)
                if error.to_string()
                    == "Office package is invalid: duplicate entry `word/document.xml`"
        ));
        Ok(())
    }
}
