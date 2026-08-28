use super::office_preflight::{OfficePreflightError, OfficePreflightSupport};

const SIGNATURE: &[u8; 4] = b"PK\x05\x06";
const RECORD_BYTES: usize = 22;

pub(super) struct EndOfCentralDirectory;

impl EndOfCentralDirectory {
    pub(super) fn entry_count(bytes: &[u8]) -> Result<Option<usize>, OfficePreflightError> {
        let Some(start) = Self::offset(bytes) else {
            return Err(Self::invalid("end of central directory is missing"));
        };
        let record = &bytes[start..start + RECORD_BYTES];
        let disk = u16::from_le_bytes([record[4], record[5]]);
        let central_disk = u16::from_le_bytes([record[6], record[7]]);
        let disk_entries = u16::from_le_bytes([record[8], record[9]]);
        let total_entries = u16::from_le_bytes([record[10], record[11]]);
        if disk != 0 || central_disk != 0 || disk_entries != total_entries {
            return Err(Self::invalid("multi-disk ZIP archives are unsupported"));
        }
        if total_entries == u16::MAX {
            return Ok(None);
        }
        Ok(Some(usize::from(total_entries)))
    }

    fn offset(bytes: &[u8]) -> Option<usize> {
        bytes
            .windows(SIGNATURE.len())
            .rposition(|window| window == SIGNATURE)
            .filter(|start| Self::record_ends_archive(bytes, *start))
    }

    fn record_ends_archive(bytes: &[u8], start: usize) -> bool {
        let Some(record) = bytes.get(start..start.saturating_add(RECORD_BYTES)) else {
            return false;
        };
        let comment_length = usize::from(u16::from_le_bytes([record[20], record[21]]));
        start
            .saturating_add(RECORD_BYTES)
            .saturating_add(comment_length)
            == bytes.len()
    }

    fn invalid(reason: &str) -> OfficePreflightError {
        OfficePreflightSupport::invalid_archive(reason.to_owned())
    }
}

#[cfg(test)]
#[path = "office_preflight_eocd_tests.rs"]
mod tests;
