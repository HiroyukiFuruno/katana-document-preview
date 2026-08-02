use super::{
    ArchiveScan, MAX_NESTED_PACKAGE_DEPTH, OfficeDocumentFormat, OfficePreflightError,
    OfficePreflightLimits, OfficePreflightSupport, OfficeResourceLimitKind, record_entry,
    validate_depth, validate_main_part,
};

#[test]
fn archive_io_and_depth_failures_are_typed() {
    assert!(matches!(
        OfficePreflightSupport::archive_error(zip::result::ZipError::FileNotFound),
        OfficePreflightError::InvalidArchive { .. }
    ));
    assert!(matches!(
        OfficePreflightSupport::archive_error(std::io::Error::other("read failed")),
        OfficePreflightError::InvalidArchive { .. }
    ));
    assert!(matches!(
        validate_depth(MAX_NESTED_PACKAGE_DEPTH + 1),
        Err(OfficePreflightError::ResourceLimitExceeded {
            kind: OfficeResourceLimitKind::EntryCount,
            ..
        })
    ));
}

#[test]
fn duplicate_and_missing_main_parts_fail_closed() {
    let mut scan = ArchiveScan::new(2);
    let limits = OfficePreflightLimits::strict();
    assert!(
        record_entry(
            &mut scan,
            "word/document.xml",
            1,
            1,
            OfficeDocumentFormat::Docx,
            limits,
        )
        .is_ok()
    );
    assert!(matches!(
        record_entry(
            &mut scan,
            "word/document.xml",
            1,
            1,
            OfficeDocumentFormat::Docx,
            limits,
        ),
        Err(OfficePreflightError::InvalidArchive { .. })
    ));
    assert!(matches!(
        validate_main_part(OfficeDocumentFormat::Pptx, false),
        Err(OfficePreflightError::InvalidArchive { .. })
    ));
}

#[test]
fn compressed_archive_budget_is_enforced() {
    let mut limits = OfficePreflightLimits::strict();
    limits.max_source_bytes = 1;
    let mut scan = ArchiveScan::new(1);
    assert!(matches!(
        record_entry(
            &mut scan,
            "word/document.xml",
            2,
            1,
            OfficeDocumentFormat::Docx,
            limits,
        ),
        Err(OfficePreflightError::ResourceLimitExceeded {
            kind: OfficeResourceLimitKind::SourceBytes,
            ..
        })
    ));
}

#[test]
fn expanded_archive_budget_is_enforced() {
    let mut limits = OfficePreflightLimits::strict();
    limits.max_total_uncompressed_bytes = 1;
    let mut scan = ArchiveScan::new(1);
    assert!(matches!(
        record_entry(
            &mut scan,
            "word/document.xml",
            1,
            2,
            OfficeDocumentFormat::Docx,
            limits,
        ),
        Err(OfficePreflightError::ResourceLimitExceeded {
            kind: OfficeResourceLimitKind::TotalUncompressedBytes,
            ..
        })
    ));
}
