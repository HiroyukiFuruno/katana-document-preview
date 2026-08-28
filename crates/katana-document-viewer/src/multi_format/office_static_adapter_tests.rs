use super::{
    BinaryDocumentSource, OfficeDocumentFormat, OfficeDocumentSource, OfficeStaticDocumentArtifact,
    OfficeStaticViewerSession, OfficeWorkerConfig, OfficeWorkerError, PdfViewerError,
    PdfViewerSession, ViewerQualityProfile, static_profile,
};
use crate::multi_format::ViewerSourceIdentity;
use crate::multi_format::office_conversion_key::OfficeConversionKey;
use std::path::PathBuf;

type TestResult = Result<(), Box<dyn std::error::Error>>;
type SessionFixture = (
    ViewerSourceIdentity,
    OfficeConversionKey,
    OfficeStaticViewerSession,
);

#[test]
fn artifact_getter_returns_the_session_artifact() -> TestResult {
    let (identity, conversion_key, session) = test_session()?;

    assert_eq!(&identity, &session.artifact().identity);
    assert_eq!(&conversion_key, session.conversion_key());
    Ok(())
}

fn test_session() -> Result<SessionFixture, Box<dyn std::error::Error>> {
    let identity = ViewerSourceIdentity::new("file:///representative.pdf", "sha256:test");
    let pdf = representative_pdf(&identity)?;
    let artifact = representative_artifact(&identity);
    let source = representative_source(&identity);
    let conversion_key =
        OfficeConversionKey::new(&source, &OfficeWorkerConfig::new(PathBuf::from("worker")));
    Ok((
        identity,
        conversion_key.clone(),
        OfficeStaticViewerSession {
            artifact,
            pdf,
            conversion_key: conversion_key.clone(),
        },
    ))
}

fn representative_pdf(identity: &ViewerSourceIdentity) -> Result<PdfViewerSession, PdfViewerError> {
    PdfViewerSession::open(BinaryDocumentSource::new(
        identity.clone(),
        "application/pdf",
        include_bytes!("../../../../assets/reference/katana/pdf/sample.pdf").to_vec(),
    ))
}

fn representative_artifact(identity: &ViewerSourceIdentity) -> OfficeStaticDocumentArtifact {
    OfficeStaticDocumentArtifact {
        identity: identity.clone(),
        format: OfficeDocumentFormat::Docx,
        mime: "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_owned(),
        item_count: 0,
        items: Vec::new(),
        capabilities: ViewerQualityProfile::static_page().capabilities,
        diagnostics: Vec::new(),
    }
}

fn representative_source(identity: &ViewerSourceIdentity) -> OfficeDocumentSource {
    OfficeDocumentSource::new(
        identity.clone(),
        OfficeDocumentFormat::Docx,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        b"test".to_vec(),
    )
}

#[test]
fn static_profiles_cover_every_office_format() {
    assert_eq!(
        Ok(ViewerQualityProfile::static_page()),
        static_profile(OfficeDocumentFormat::Docx)
    );
    assert_eq!(
        Ok(ViewerQualityProfile::static_slide_with_chart_fallback()),
        static_profile(OfficeDocumentFormat::Pptx)
    );
    assert_eq!(
        Err(OfficeWorkerError::UnsupportedFormat(
            OfficeDocumentFormat::Xlsx
        )),
        static_profile(OfficeDocumentFormat::Xlsx)
    );
}
