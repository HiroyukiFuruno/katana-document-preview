use super::{
    BinaryDocumentSource, OfficeDocumentFormat, OfficeStaticDocumentArtifact,
    OfficeStaticViewerSession, OfficeWorkerError, PdfViewerSession, ViewerQualityProfile,
    static_profile,
};
use crate::multi_format::ViewerSourceIdentity;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn artifact_getter_returns_the_session_artifact() -> TestResult {
    let identity = ViewerSourceIdentity::new("file:///representative.pdf", "sha256:test");
    let pdf = PdfViewerSession::open(BinaryDocumentSource::new(
        identity.clone(),
        "application/pdf",
        include_bytes!("../../../../assets/reference/katana/pdf/sample.pdf").to_vec(),
    ))?;
    let artifact = OfficeStaticDocumentArtifact {
        identity: identity.clone(),
        format: OfficeDocumentFormat::Docx,
        mime: "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_owned(),
        item_count: 0,
        items: Vec::new(),
        capabilities: ViewerQualityProfile::static_page().capabilities,
        diagnostics: Vec::new(),
    };
    let session = OfficeStaticViewerSession { artifact, pdf };

    assert_eq!(&identity, &session.artifact().identity);
    Ok(())
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
