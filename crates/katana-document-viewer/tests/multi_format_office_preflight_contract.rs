use katana_document_viewer::{
    OfficeDocumentFormat, OfficeDocumentSource, OfficePackagePreflight, OfficePreflightError,
    OfficePreflightLimits, OfficeResourceLimitKind, ViewerDiagnosticCode, ViewerSourceIdentity,
};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn package(entries: &[(&str, &[u8])]) -> TestResult<Vec<u8>> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in entries {
        writer.start_file(
            *name,
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
        )?;
        writer.write_all(bytes)?;
    }
    Ok(writer.finish()?.into_inner())
}

fn duplicate_filename_package() -> TestResult<Vec<u8>> {
    const CANONICAL: &[u8] = b"word/document.xml";
    const ALIAS: &[u8] = b"word/documenz.xml";
    let mut bytes = package(&[
        ("word/document.xml", b"<w:document/>"),
        ("word/documenz.xml", b"duplicate"),
    ])?;
    let mut replacements = 0;
    for offset in 0..=bytes.len().saturating_sub(ALIAS.len()) {
        if bytes[offset..offset + ALIAS.len()] == *ALIAS {
            bytes[offset..offset + ALIAS.len()].copy_from_slice(CANONICAL);
            replacements += 1;
        }
    }
    if replacements != 2 {
        return Err(format!("expected two ZIP filename headers, found {replacements}").into());
    }
    Ok(bytes)
}

fn corrupt_local_header_package() -> TestResult<Vec<u8>> {
    let mut bytes = package(&[("word/document.xml", b"<w:document/>")])?;
    let name = b"word/document.xml";
    let name_offset = bytes
        .windows(name.len())
        .position(|window| window == name)
        .ok_or("document local header")?;
    bytes[name_offset - 30] = 0;
    Ok(bytes)
}

fn office(bytes: Vec<u8>, format: OfficeDocumentFormat) -> OfficeDocumentSource {
    let mime = match format {
        OfficeDocumentFormat::Docx => {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        }
        OfficeDocumentFormat::Xlsx => {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        }
        OfficeDocumentFormat::Pptx => {
            "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        }
    };
    OfficeDocumentSource::new(
        ViewerSourceIdentity::new("file:///fixtures/generated.office", "generated"),
        format,
        mime,
        bytes,
    )
}

fn docx(bytes: Vec<u8>) -> OfficeDocumentSource {
    office(bytes, OfficeDocumentFormat::Docx)
}

fn source(name: &str, format: OfficeDocumentFormat) -> TestResult<OfficeDocumentSource> {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/multi-format")
        .join(name);
    let bytes = std::fs::read(fixture)?;
    let mime = match format {
        OfficeDocumentFormat::Docx => {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        }
        OfficeDocumentFormat::Xlsx => {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        }
        OfficeDocumentFormat::Pptx => {
            "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        }
    };
    Ok(OfficeDocumentSource::new(
        ViewerSourceIdentity::new(format!("file:///fixtures/{name}"), format!("sha256:{name}")),
        format,
        mime,
        bytes,
    ))
}

#[test]
fn representative_packages_pass_bounded_preflight() -> TestResult {
    let cases = [
        ("representative.docx", OfficeDocumentFormat::Docx),
        ("representative.xlsx", OfficeDocumentFormat::Xlsx),
        ("representative.pptx", OfficeDocumentFormat::Pptx),
    ];

    for (name, format) in cases {
        let report = OfficePackagePreflight::inspect(
            &source(name, format)?,
            OfficePreflightLimits::strict(),
        )?;
        assert!(report.entry_count > 0);
        assert!(report.total_uncompressed_bytes > 0);
        assert_eq!(0, report.external_relationship_count);
    }
    Ok(())
}

#[test]
fn corrupt_local_entry_header_fails_during_archive_scan() -> TestResult {
    assert!(matches!(
        OfficePackagePreflight::inspect(
            &docx(corrupt_local_header_package()?),
            OfficePreflightLimits::strict(),
        ),
        Err(OfficePreflightError::InvalidArchive { .. })
    ));
    Ok(())
}

#[test]
fn external_hyperlink_is_accepted_without_granting_resource_access() -> TestResult {
    let relationship = br#"<Relationships><Relationship Id="rIdLink" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://example.invalid/documentation" TargetMode="External"/></Relationships>"#;
    let bytes = package(&[
        ("ppt/presentation.xml", b"<p:presentation/>"),
        ("ppt/slides/_rels/slide1.xml.rels", relationship),
    ])?;
    let report = OfficePackagePreflight::inspect(
        &office(bytes, OfficeDocumentFormat::Pptx),
        OfficePreflightLimits::strict(),
    )?;
    assert_eq!(1, report.external_relationship_count);
    Ok(())
}

#[test]
fn external_fetchable_relationship_is_rejected_before_an_engine_can_open_it() -> TestResult {
    assert!(matches!(
        OfficePackagePreflight::inspect(
            &source("external-image.docx", OfficeDocumentFormat::Docx)?,
            OfficePreflightLimits::strict(),
        ),
        Err(error)
            if matches!(&error, OfficePreflightError::ExternalResourceBlocked { .. })
                && error.diagnostic().code == ViewerDiagnosticCode::ExternalResourceBlocked
    ));
    Ok(())
}

#[test]
fn active_content_is_accepted_for_quarantined_static_display() -> TestResult {
    let report = OfficePackagePreflight::inspect(
        &source("macro-marker.docx", OfficeDocumentFormat::Docx)?,
        OfficePreflightLimits::strict(),
    )?;

    assert!(report.entry_count > 0);
    Ok(())
}

#[test]
fn high_expansion_package_is_rejected_without_inflating_the_large_entry() -> TestResult {
    assert!(matches!(
        OfficePackagePreflight::inspect(
            &source("oversized-document.docx", OfficeDocumentFormat::Docx)?,
            OfficePreflightLimits::strict(),
        ),
        Err(error)
            if matches!(&error, OfficePreflightError::ResourceLimitExceeded { .. })
                && error.diagnostic().code == ViewerDiagnosticCode::ResourceLimitExceeded
    ));
    Ok(())
}

#[test]
fn declared_format_and_mime_must_match() -> TestResult {
    let mut package = source("representative.docx", OfficeDocumentFormat::Docx)?;
    package.mime = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_owned();

    let expected = OfficePreflightError::UnsupportedMime {
        format: OfficeDocumentFormat::Docx,
        mime: package.mime.clone(),
    };
    let error = match OfficePackagePreflight::inspect(&package, OfficePreflightLimits::strict()) {
        Err(error) => error,
        Ok(_) => return Err("mismatched MIME did not fail".into()),
    };
    assert_eq!(expected, error);
    assert_eq!(
        ViewerDiagnosticCode::UnsupportedFormat,
        error.diagnostic().code
    );
    Ok(())
}

#[test]
fn every_resource_budget_is_enforced_through_the_public_preflight() -> TestResult {
    let main = b"<w:document>bounded content</w:document>";
    let relationship = b"<Relationships/>";
    let bytes = package(&[
        ("word/document.xml", main.as_slice()),
        ("word/_rels/document.xml.rels", relationship.as_slice()),
    ])?;

    let cases = [
        (
            OfficeResourceLimitKind::SourceBytes,
            OfficePreflightLimits {
                max_source_bytes: bytes.len() as u64 - 1,
                ..OfficePreflightLimits::strict()
            },
        ),
        (
            OfficeResourceLimitKind::EntryCount,
            OfficePreflightLimits {
                max_entries: 1,
                ..OfficePreflightLimits::strict()
            },
        ),
        (
            OfficeResourceLimitKind::EntryBytes,
            OfficePreflightLimits {
                max_entry_uncompressed_bytes: main.len() as u64 - 1,
                ..OfficePreflightLimits::strict()
            },
        ),
        (
            OfficeResourceLimitKind::TotalUncompressedBytes,
            OfficePreflightLimits {
                max_total_uncompressed_bytes: main.len() as u64,
                ..OfficePreflightLimits::strict()
            },
        ),
        (
            OfficeResourceLimitKind::CompressionRatio,
            OfficePreflightLimits {
                max_compression_ratio: 0,
                ..OfficePreflightLimits::strict()
            },
        ),
        (
            OfficeResourceLimitKind::RelationshipBytes,
            OfficePreflightLimits {
                max_relationship_bytes: relationship.len() as u64 - 1,
                ..OfficePreflightLimits::strict()
            },
        ),
    ];

    for (kind, limits) in cases {
        assert!(matches!(
            OfficePackagePreflight::inspect(&docx(bytes.clone()), limits),
            Err(OfficePreflightError::ResourceLimitExceeded {
                kind: actual,
                ..
            }) if actual == kind
        ));
    }
    Ok(())
}

#[test]
fn malformed_package_structures_fail_closed_through_the_public_preflight() -> TestResult {
    let main = b"<w:document/>";
    let cases = [
        package(&[("../escape.xml", b"unsafe"), ("word/document.xml", main)])?,
        duplicate_filename_package()?,
        package(&[("word/styles.xml", b"<w:styles/>")])?,
        package(&[
            ("word/document.xml", main),
            ("word/_rels/document.xml.rels", b"<"),
        ])?,
        package(&[
            ("word/document.xml", main),
            (
                "word/_rels/document.xml.rels",
                b"<Relationships><Relationship Target='a' Target='b'/></Relationships>",
            ),
        ])?,
        package(&[
            ("word/document.xml", main),
            (
                "word/_rels/document.xml.rels",
                b"<Relationships><Relationship Target='&bogus;'/></Relationships>",
            ),
        ])?,
    ];

    for (index, bytes) in cases.into_iter().enumerate() {
        let result = OfficePackagePreflight::inspect(&docx(bytes), OfficePreflightLimits::strict());
        assert!(
            matches!(
                &result,
                Err(error) if error.diagnostic().code == ViewerDiagnosticCode::InvalidDocument
            ),
            "malformed package case {index} returned {result:?}"
        );
    }
    Ok(())
}

#[test]
fn invalid_zip_bytes_and_default_limits_keep_typed_diagnostics() {
    let error = OfficePackagePreflight::inspect(
        &docx(b"not a zip".to_vec()),
        OfficePreflightLimits::default(),
    );
    assert!(matches!(
        error,
        Err(error) if error.diagnostic().code == ViewerDiagnosticCode::InvalidDocument
    ));
}

#[test]
fn nested_office_packages_are_bounded_recursively() -> TestResult {
    let leaf = package(&[("word/document.xml", b"<w:document/>")])?;
    let level_three = package(&[
        ("word/document.xml", b"<w:document/>"),
        ("word/embeddings/leaf.docx", leaf.as_slice()),
    ])?;
    let level_two = package(&[
        ("word/document.xml", b"<w:document/>"),
        ("word/embeddings/level-three.docx", level_three.as_slice()),
    ])?;
    let level_one = package(&[
        ("word/document.xml", b"<w:document/>"),
        ("word/embeddings/level-two.docx", level_two.as_slice()),
    ])?;
    let outer = package(&[
        ("word/document.xml", b"<w:document/>"),
        ("word/embeddings/level-one.docx", level_one.as_slice()),
    ])?;

    assert!(matches!(
        OfficePackagePreflight::inspect(&docx(outer), OfficePreflightLimits::strict()),
        Err(OfficePreflightError::ResourceLimitExceeded {
            kind: OfficeResourceLimitKind::EntryCount,
            ..
        })
    ));
    Ok(())
}
