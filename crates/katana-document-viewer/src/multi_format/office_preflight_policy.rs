use super::{
    OfficeDocumentFormat, OfficeDocumentSource,
    office_preflight::{
        OfficePreflightError, OfficePreflightLimits, OfficePreflightSupport,
        OfficeResourceLimitKind,
    },
};

pub(crate) struct OfficePreflightPolicy;

impl OfficePreflightPolicy {
    pub(crate) fn validate_source(
        source: &OfficeDocumentSource,
        limits: OfficePreflightLimits,
    ) -> Result<(), OfficePreflightError> {
        if source.mime != OfficePreflightSupport::expected_mime(source.format) {
            return Err(OfficePreflightError::UnsupportedMime {
                format: source.format,
                mime: source.mime.clone(),
            });
        }
        let actual = source.bytes.len() as u64;
        if actual > limits.max_source_bytes {
            return Err(OfficePreflightSupport::resource_limit(
                OfficeResourceLimitKind::SourceBytes,
                actual,
                limits.max_source_bytes,
                None,
            ));
        }
        Ok(())
    }

    pub(crate) fn validate_entry_count(
        count: usize,
        limits: OfficePreflightLimits,
    ) -> Result<(), OfficePreflightError> {
        if count > limits.max_entries {
            return Err(OfficePreflightSupport::resource_limit(
                OfficeResourceLimitKind::EntryCount,
                count as u64,
                limits.max_entries as u64,
                None,
            ));
        }
        Ok(())
    }

    pub(crate) fn validate_entry(
        name: &str,
        compressed: u64,
        uncompressed: u64,
        limits: OfficePreflightLimits,
    ) -> Result<(), OfficePreflightError> {
        validate_entry_name(name)?;
        validate_entry_size(name, compressed, uncompressed, limits)
    }

    pub(crate) fn checked_total(
        total: u64,
        amount: u64,
        kind: OfficeResourceLimitKind,
        limit: u64,
    ) -> Result<u64, OfficePreflightError> {
        let actual = total.saturating_add(amount);
        if actual > limit {
            return Err(OfficePreflightSupport::resource_limit(
                kind, actual, limit, None,
            ));
        }
        Ok(actual)
    }

    pub(crate) fn relationship_entry(name: &str) -> bool {
        let lower = name.to_ascii_lowercase();
        lower.ends_with(".rels") && lower.split('/').any(|segment| segment == "_rels")
    }

    pub(crate) fn nested_package_format(name: &str) -> Option<OfficeDocumentFormat> {
        nested_package_format(name)
    }

    pub(crate) fn active_content_entry(name: &str) -> bool {
        active_content_entry(name)
    }

    pub(crate) const fn main_part(format: OfficeDocumentFormat) -> &'static str {
        match format {
            OfficeDocumentFormat::Docx => "word/document.xml",
            OfficeDocumentFormat::Xlsx => "xl/workbook.xml",
            OfficeDocumentFormat::Pptx => "ppt/presentation.xml",
        }
    }
}

fn validate_entry_size(
    name: &str,
    compressed: u64,
    uncompressed: u64,
    limits: OfficePreflightLimits,
) -> Result<(), OfficePreflightError> {
    let entry_limit = if worksheet_entry(name) {
        super::office_preflight::MAX_WORKSHEET_UNCOMPRESSED_BYTES
    } else {
        limits.max_entry_uncompressed_bytes
    };
    if uncompressed > entry_limit {
        return Err(OfficePreflightSupport::resource_limit(
            OfficeResourceLimitKind::EntryBytes,
            uncompressed,
            entry_limit,
            Some(name.to_owned()),
        ));
    }
    let ratio_limit = compressed.saturating_mul(limits.max_compression_ratio);
    if uncompressed > ratio_limit && uncompressed > 0 {
        return Err(OfficePreflightSupport::resource_limit(
            OfficeResourceLimitKind::CompressionRatio,
            uncompressed,
            ratio_limit,
            Some(name.to_owned()),
        ));
    }
    Ok(())
}

fn worksheet_entry(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.starts_with("xl/worksheets/") && lower.ends_with(".xml")
}

fn validate_entry_name(name: &str) -> Result<(), OfficePreflightError> {
    let unsafe_name = name.is_empty()
        || name.starts_with('/')
        || name.starts_with('\\')
        || name.contains('\\')
        || name.contains('\0')
        || name
            .split('/')
            .any(|segment| segment == "." || segment == "..");
    if unsafe_name {
        return Err(OfficePreflightError::UnsafeEntryName {
            entry: name.to_owned(),
        });
    }
    Ok(())
}

fn active_content_entry(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with("/vbaproject.bin")
        || lower.contains("/activex/")
        || lower.contains("/macrosheets/")
        || lower.contains("/dialogsheets/")
        || lower.ends_with(".docm")
        || lower.ends_with(".dotm")
        || lower.ends_with(".xlsm")
        || lower.ends_with(".xlam")
        || lower.ends_with(".pptm")
        || lower.ends_with(".potm")
        || lower.ends_with(".ppam")
        || (lower.contains("/embeddings/") && nested_package_format(name).is_none())
}

fn nested_package_format(name: &str) -> Option<OfficeDocumentFormat> {
    let lower = name.to_ascii_lowercase();
    if !lower.contains("/embeddings/") {
        return None;
    }
    if lower.ends_with(".docx") {
        Some(OfficeDocumentFormat::Docx)
    } else if lower.ends_with(".xlsx") {
        Some(OfficeDocumentFormat::Xlsx)
    } else if lower.ends_with(".pptx") {
        Some(OfficeDocumentFormat::Pptx)
    } else {
        None
    }
}

#[cfg(test)]
#[path = "office_preflight_policy_tests.rs"]
mod tests;
