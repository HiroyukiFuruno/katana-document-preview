use super::office_preflight::{OfficePreflightError, OfficePreflightSupport};
use super::office_preflight_eocd::EndOfCentralDirectory;
use super::office_preflight_local_header::LocalHeader;
use std::io::{Cursor, sink};
use zip::ZipArchive;

pub(super) struct OfficeZipEntries;

impl OfficeZipEntries {
    pub(super) fn validate(
        bytes: &[u8],
        archive: &mut ZipArchive<Cursor<&[u8]>>,
    ) -> Result<(), OfficePreflightError> {
        Self::validate_declared_count(bytes, archive.len())?;
        (0..archive.len()).try_for_each(|index| Self::validate_entry(bytes, archive, index))
    }

    pub(super) fn validate_entry(
        bytes: &[u8],
        archive: &mut ZipArchive<Cursor<&[u8]>>,
        index: usize,
    ) -> Result<(), OfficePreflightError> {
        let mut file = archive
            .by_index(index)
            .map_err(OfficePreflightSupport::archive_error)?;
        LocalHeader::validate(bytes, &file)?;
        std::io::copy(&mut file, &mut sink()).map_err(OfficePreflightSupport::archive_error)?;
        Ok(())
    }

    pub(super) fn validate_declared_count(
        bytes: &[u8],
        exposed: usize,
    ) -> Result<(), OfficePreflightError> {
        let declared = EndOfCentralDirectory::entry_count(bytes)?.unwrap_or(exposed);
        if declared == exposed {
            return Ok(());
        }
        Err(OfficePreflightSupport::invalid_archive(format!(
            "central directory declares {declared} entries but archive exposes {exposed}"
        )))
    }
}
