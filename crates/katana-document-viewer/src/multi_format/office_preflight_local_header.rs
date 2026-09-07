use super::office_preflight::{OfficePreflightError, OfficePreflightSupport};
use std::io::Cursor;

const SIGNATURE: &[u8; 4] = b"PK\x03\x04";
const FIXED_BYTES: usize = 30;
pub(super) const DATA_DESCRIPTOR_FLAG: u16 = 1 << 3;

pub(super) struct LocalHeader;

impl LocalHeader {
    pub(super) fn validate(
        bytes: &[u8],
        file: &zip::read::ZipFile<'_, Cursor<&[u8]>>,
    ) -> Result<(), OfficePreflightError> {
        let fields = LocalHeaderFields::read(bytes, file.header_start())?;
        fields.validate_name_and_offset(bytes, file)?;
        if fields.flags & DATA_DESCRIPTOR_FLAG == 0 {
            fields.validate_inline_sizes(file)?;
        }
        Ok(())
    }
}

struct LocalHeaderFields {
    start: usize,
    flags: u16,
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    name_length: usize,
    extra_length: usize,
}

impl LocalHeaderFields {
    fn read(bytes: &[u8], start: u64) -> Result<Self, OfficePreflightError> {
        #[cfg(target_pointer_width = "64")]
        let start = start as usize;
        #[cfg(not(target_pointer_width = "64"))]
        let start = match usize::try_from(start) {
            Ok(start) => start,
            Err(error) => return Err(Self::invalid(&error.to_string())),
        };
        let Some(fixed) = bytes.get(start..start.saturating_add(FIXED_BYTES)) else {
            return Err(Self::invalid("truncated local header"));
        };
        if fixed.get(..4) != Some(SIGNATURE.as_slice()) {
            return Err(Self::invalid("invalid local header signature"));
        }
        Ok(Self::from_fixed(start, fixed))
    }

    fn from_fixed(start: usize, fixed: &[u8]) -> Self {
        Self {
            start,
            flags: u16::from_le_bytes([fixed[6], fixed[7]]),
            crc32: u32::from_le_bytes([fixed[14], fixed[15], fixed[16], fixed[17]]),
            compressed_size: u32::from_le_bytes([fixed[18], fixed[19], fixed[20], fixed[21]]),
            uncompressed_size: u32::from_le_bytes([fixed[22], fixed[23], fixed[24], fixed[25]]),
            name_length: usize::from(u16::from_le_bytes([fixed[26], fixed[27]])),
            extra_length: usize::from(u16::from_le_bytes([fixed[28], fixed[29]])),
        }
    }

    fn validate_name_and_offset(
        &self,
        bytes: &[u8],
        file: &zip::read::ZipFile<'_, Cursor<&[u8]>>,
    ) -> Result<(), OfficePreflightError> {
        let name_start = self.start.saturating_add(FIXED_BYTES);
        let name_end = name_start.saturating_add(self.name_length);
        let Some(name) = bytes.get(name_start..name_end) else {
            return Err(Self::invalid("truncated local entry name"));
        };
        if name != file.name_raw() {
            return Err(Self::invalid("local and central entry names differ"));
        }
        let data_start = name_end.saturating_add(self.extra_length);
        let expected = file
            .data_start()
            .and_then(|value| usize::try_from(value).ok());
        if Some(data_start) != expected {
            return Err(Self::invalid("local and central data offsets differ"));
        }
        Ok(())
    }

    fn validate_inline_sizes(
        &self,
        file: &zip::read::ZipFile<'_, Cursor<&[u8]>>,
    ) -> Result<(), OfficePreflightError> {
        if self.crc32 != file.crc32()
            || !Self::size_matches(self.compressed_size, file.compressed_size())
            || !Self::size_matches(self.uncompressed_size, file.size())
        {
            return Err(Self::invalid("local and central entry metadata differ"));
        }
        Ok(())
    }

    fn size_matches(local: u32, central: u64) -> bool {
        local == u32::MAX || u64::from(local) == central
    }

    fn invalid(reason: &str) -> OfficePreflightError {
        OfficePreflightSupport::invalid_archive(reason.to_owned())
    }
}

#[cfg(test)]
#[path = "office_preflight_local_header_tests.rs"]
mod tests;
