use super::{OfficePreflightPolicy, nested_package_format, validate_entry_name};
use crate::multi_format::{
    OfficeDocumentFormat, OfficeDocumentSource, OfficePreflightError, OfficePreflightLimits,
    OfficeResourceLimitKind, ViewerSourceIdentity,
};

fn source(bytes: usize) -> OfficeDocumentSource {
    OfficeDocumentSource::new(
        ViewerSourceIdentity::new("file:///sample.docx", "sample"),
        OfficeDocumentFormat::Docx,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        vec![0; bytes],
    )
}

#[test]
fn source_size_limit_fails_closed() {
    let mut limits = OfficePreflightLimits::strict();
    limits.max_source_bytes = 1;
    assert!(matches!(
        OfficePreflightPolicy::validate_source(&source(2), limits),
        Err(OfficePreflightError::ResourceLimitExceeded {
            kind: OfficeResourceLimitKind::SourceBytes,
            ..
        })
    ));
}

#[test]
fn entry_count_limit_fails_closed() {
    let mut limits = OfficePreflightLimits::strict();
    limits.max_entries = 1;
    assert!(matches!(
        OfficePreflightPolicy::validate_entry_count(2, limits),
        Err(OfficePreflightError::ResourceLimitExceeded {
            kind: OfficeResourceLimitKind::EntryCount,
            ..
        })
    ));
}

#[test]
fn accumulated_size_limit_fails_closed() {
    assert!(matches!(
        OfficePreflightPolicy::checked_total(
            2,
            2,
            OfficeResourceLimitKind::TotalUncompressedBytes,
            3,
        ),
        Err(OfficePreflightError::ResourceLimitExceeded {
            kind: OfficeResourceLimitKind::TotalUncompressedBytes,
            ..
        })
    ));
}

#[test]
fn entry_policy_rejects_unsafe_names_and_high_compression() {
    assert!(matches!(
        validate_entry_name("../escape.xml"),
        Err(OfficePreflightError::UnsafeEntryName { .. })
    ));
    let mut limits = OfficePreflightLimits::strict();
    limits.max_compression_ratio = 2;
    assert!(matches!(
        OfficePreflightPolicy::validate_entry("word/document.xml", 2, 5, limits),
        Err(OfficePreflightError::ResourceLimitExceeded {
            kind: OfficeResourceLimitKind::CompressionRatio,
            ..
        })
    ));
}

#[test]
fn nested_package_formats_are_explicit() {
    let cases = [
        (
            "word/embeddings/report.docx",
            Some(OfficeDocumentFormat::Docx),
        ),
        (
            "word/embeddings/data.xlsx",
            Some(OfficeDocumentFormat::Xlsx),
        ),
        (
            "word/embeddings/slides.pptx",
            Some(OfficeDocumentFormat::Pptx),
        ),
        ("word/embeddings/unsupported.bin", None),
    ];
    for (name, format) in cases {
        assert_eq!(format, nested_package_format(name));
    }
}
