use katana_document_viewer::{
    BinaryDocumentSource, PdfPageRenderRequest, PdfResourceLimitKind, PdfViewerError,
    PdfViewerLimits, PdfViewerSession, ViewerFeature, ViewerFeatureStatus, ViewerSourceIdentity,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const SAMPLE_PDF: &[u8] = include_bytes!("../../../assets/reference/katana/pdf/sample.pdf");

fn source(bytes: Vec<u8>) -> BinaryDocumentSource {
    BinaryDocumentSource::new(
        ViewerSourceIdentity::new("file:///fixtures/sample.pdf", "sha256:sample"),
        "application/pdf",
        bytes,
    )
}

#[test]
fn hayro_pdf_session_exposes_page_geometry_and_capabilities() -> TestResult {
    let session = PdfViewerSession::open(source(SAMPLE_PDF.to_vec()))?;
    let artifact = session.artifact();

    assert_eq!(13, artifact.page_count);
    assert_eq!(13, artifact.pages.len());
    assert!(artifact.pages.iter().all(|page| page.width > 0.0));
    assert!(artifact.pages.iter().all(|page| page.height > 0.0));
    assert_eq!(
        ViewerFeatureStatus::Supported,
        artifact.capabilities.status(ViewerFeature::PageNavigation)
    );
    assert_eq!(
        ViewerFeatureStatus::Unsupported,
        artifact.capabilities.status(ViewerFeature::TextSelection)
    );
    Ok(())
}

#[test]
fn hayro_pdf_render_is_opaque_bounded_and_cached() -> TestResult {
    let mut session = PdfViewerSession::open(source(SAMPLE_PDF.to_vec()))?;
    let request = PdfPageRenderRequest::new(0, 1.0);

    let first = session.render_page(request)?;
    let second = session.render_page(request)?;

    assert_eq!(0, first.page_index);
    assert!(first.surface.width > 0);
    assert!(first.surface.height > 0);
    assert_eq!(
        usize::try_from(first.surface.width * first.surface.height * 4)?,
        first.surface.rgba.len()
    );
    assert!(
        first
            .surface
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| pixel[3] == 255)
    );
    assert_eq!(first, second);
    assert_eq!(1, session.cached_page_count());
    assert_eq!(first.surface.rgba.len(), session.cached_byte_count());
    Ok(())
}

#[test]
fn pdf_source_page_and_render_limits_fail_with_typed_kinds() -> TestResult {
    let mut source_limits = PdfViewerLimits::strict();
    source_limits.max_source_bytes = SAMPLE_PDF.len().saturating_sub(1);
    assert!(matches!(
        PdfViewerSession::open_with_limits(source(SAMPLE_PDF.to_vec()), source_limits),
        Err(PdfViewerError::ResourceLimitExceeded {
            kind: PdfResourceLimitKind::SourceBytes,
            ..
        })
    ));

    let mut page_limits = PdfViewerLimits::strict();
    page_limits.max_pages = 12;
    assert!(matches!(
        PdfViewerSession::open_with_limits(source(SAMPLE_PDF.to_vec()), page_limits),
        Err(PdfViewerError::ResourceLimitExceeded {
            kind: PdfResourceLimitKind::PageCount,
            ..
        })
    ));

    let mut render_limits = PdfViewerLimits::strict();
    render_limits.max_render_pixels = 1;
    let mut session =
        PdfViewerSession::open_with_limits(source(SAMPLE_PDF.to_vec()), render_limits)?;
    assert!(matches!(
        session.render_page(PdfPageRenderRequest::new(0, 1.0)),
        Err(PdfViewerError::ResourceLimitExceeded {
            kind: PdfResourceLimitKind::RenderPixels,
            ..
        })
    ));
    Ok(())
}

#[test]
fn pdf_cache_evicts_by_recency_and_can_be_disabled() -> TestResult {
    let mut limits = PdfViewerLimits::strict();
    limits.max_cached_pages = 1;
    let mut session = PdfViewerSession::open_with_limits(source(SAMPLE_PDF.to_vec()), limits)?;
    let first = session.render_page(PdfPageRenderRequest::new(0, 1.0))?;
    let second = session.render_page(PdfPageRenderRequest::new(1, 1.0))?;
    assert_eq!(1, session.cached_page_count());
    assert_eq!(second.surface.rgba.len(), session.cached_byte_count());
    assert_eq!(
        &first,
        &session.render_page(PdfPageRenderRequest::new(0, 1.0))?
    );
    assert_eq!(1, session.cached_page_count());

    let mut byte_bounded = PdfViewerLimits::strict();
    byte_bounded.max_cached_bytes = first.surface.rgba.len();
    let mut byte_bounded_session =
        PdfViewerSession::open_with_limits(source(SAMPLE_PDF.to_vec()), byte_bounded)?;
    byte_bounded_session.render_page(PdfPageRenderRequest::new(0, 1.0))?;
    byte_bounded_session.render_page(PdfPageRenderRequest::new(1, 1.0))?;
    assert_eq!(1, byte_bounded_session.cached_page_count());
    assert!(byte_bounded_session.cached_byte_count() <= byte_bounded.max_cached_bytes);

    let mut too_small = PdfViewerLimits::strict();
    too_small.max_cached_bytes = first.surface.rgba.len().saturating_sub(1);
    let mut byte_uncached =
        PdfViewerSession::open_with_limits(source(SAMPLE_PDF.to_vec()), too_small)?;
    byte_uncached.render_page(PdfPageRenderRequest::new(0, 1.0))?;
    assert_eq!(0, byte_uncached.cached_page_count());

    let mut disabled = PdfViewerLimits::strict();
    disabled.max_cached_pages = 0;
    let mut uncached = PdfViewerSession::open_with_limits(source(SAMPLE_PDF.to_vec()), disabled)?;
    uncached.render_page(PdfPageRenderRequest::new(0, 1.0))?;
    assert_eq!(0, uncached.cached_page_count());
    assert_eq!(0, uncached.cached_byte_count());
    Ok(())
}

#[test]
fn pdf_failures_are_typed_without_fallback() -> TestResult {
    let wrong_mime = BinaryDocumentSource::new(
        ViewerSourceIdentity::new("file:///fixtures/sample.pdf", "sha256:sample"),
        "application/octet-stream",
        SAMPLE_PDF.to_vec(),
    );
    assert!(matches!(
        PdfViewerSession::open(wrong_mime),
        Err(PdfViewerError::UnsupportedMime)
    ));
    assert!(matches!(
        PdfViewerSession::open(source(b"not a pdf".to_vec())),
        Err(PdfViewerError::InvalidDocument)
    ));

    let mut session = PdfViewerSession::open(source(SAMPLE_PDF.to_vec()))?;
    assert_eq!(
        Err(PdfViewerError::PageOutsideDocument {
            requested: 13,
            page_count: 13,
        }),
        session.render_page(PdfPageRenderRequest::new(13, 1.0))
    );
    assert_eq!(
        Err(PdfViewerError::InvalidScale),
        session.render_page(PdfPageRenderRequest::new(0, f32::NAN))
    );
    assert!(matches!(
        session.render_page(PdfPageRenderRequest::new(0, 100.0)),
        Err(PdfViewerError::ResourceLimitExceeded {
            kind: PdfResourceLimitKind::RenderDimension,
            ..
        })
    ));
    Ok(())
}
