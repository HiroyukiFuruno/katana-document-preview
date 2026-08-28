use super::*;
use crate::{DocumentSurfaceKind, ViewerImageSurface, ViewerSourceIdentity};
use std::path::{Path, PathBuf};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn invalid_pdf_fails_at_the_unified_session_open_boundary() {
    let source = BinaryDocumentSource::new(
        ViewerSourceIdentity::new("file:///invalid.pdf", "sha256:invalid"),
        "application/pdf",
        b"not a pdf".to_vec(),
    );

    assert!(matches!(
        PagedDocumentSession::open_pdf(source, DocumentViewport::new(640, 480)),
        Err(DocumentSessionError::Pdf(_))
    ));
}

#[test]
fn corrupted_page_index_fails_closed_at_the_state_boundary() -> TestResult {
    let source = BinaryDocumentSource::new(
        ViewerSourceIdentity::new("file:///sample.pdf", "sha256:sample"),
        "application/pdf",
        include_bytes!("../../../../assets/reference/katana/pdf/sample.pdf").to_vec(),
    );
    let mut session = PagedDocumentSession::open_pdf(source, DocumentViewport::new(640, 480))?;
    assert!(matches!(
        session.apply(DocumentSessionCommand::Surface(
            DocumentSurfaceCommand::Grid(crate::DocumentGridCommand::ScrollTo { x: 1, y: 1 })
        )),
        Err(DocumentSessionError::UnsupportedCommand {
            format: ViewerDocumentFormat::Pdf,
            command: super::super::DocumentSessionCommandKind::Grid,
        })
    ));
    session.state.active_index = usize::MAX;

    assert!(matches!(
        session.frame(),
        Err(DocumentSessionError::State(
            super::super::DocumentViewerStateError::IndexOutsideDocument { .. }
        ))
    ));
    Ok(())
}

#[test]
fn invalid_engine_raster_fails_closed_at_the_neutral_frame_boundary() -> TestResult {
    let source = BinaryDocumentSource::new(
        ViewerSourceIdentity::new("file:///sample.pdf", "sha256:sample"),
        "application/pdf",
        include_bytes!("../../../../assets/reference/katana/pdf/sample.pdf").to_vec(),
    );
    let session = PagedDocumentSession::open_pdf(source, DocumentViewport::new(640, 480))?;
    let rendered = PdfRenderedPage {
        page_index: 0,
        scale: 1.0,
        surface: ViewerImageSurface {
            fingerprint: String::new(),
            width: 1,
            height: 1,
            display_width: 1.0,
            display_height: 1.0,
            content_scale: 100,
            rgba: vec![0, 0, 0, 255],
        },
    };

    assert!(matches!(
        session.frame_from_rendered(rendered),
        Err(DocumentSessionError::Surface(_))
    ));
    Ok(())
}

#[test]
fn office_engine_reuses_its_keyed_conversion_for_resize_and_repeat_frame() -> TestResult {
    let mut session = open_office_session()?;
    let conversion_key = office_conversion_key(&session)?;

    assert_eq!(DocumentSurfaceKind::Page, session.frame()?.surface.kind());
    assert_eq!(
        DocumentSessionEvent::None,
        session.apply(DocumentSessionCommand::Surface(
            DocumentSurfaceCommand::Resize(DocumentViewport::new(900, 700))
        ))?
    );
    assert_eq!(DocumentSurfaceKind::Page, session.frame()?.surface.kind());
    assert_eq!(conversion_key, office_conversion_key(&session)?);
    Ok(())
}

fn open_office_session() -> Result<PagedDocumentSession, Box<dyn std::error::Error>> {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/multi-format/representative.docx");
    let source = OfficeDocumentSource::new(
        ViewerSourceIdentity::new("file:///unit.docx", "sha256:unit-docx"),
        super::super::OfficeDocumentFormat::Docx,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        std::fs::read(fixture)?,
    );
    PagedDocumentSession::open_office(
        source,
        OfficeWorkerConfig::new(worker_binary_path()?),
        DocumentViewport::new(640, 480),
    )
    .map_err(Into::into)
}

fn office_conversion_key(
    session: &PagedDocumentSession,
) -> Result<super::super::office_conversion_key::OfficeConversionKey, Box<dyn std::error::Error>> {
    Ok(match &session.engine {
        PagedEngine::Office(office) => office.conversion_key(),
        PagedEngine::Pdf(_) => return Err("expected an Office engine".into()),
    }
    .clone())
}

fn worker_binary_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let current_exe = std::env::current_exe()?;
    let deps = current_exe
        .parent()
        .ok_or("unit test binary has no parent directory")?;
    let worker = deps
        .parent()
        .ok_or("unit test binary has no target directory")?
        .join("kdv-office-worker");
    #[cfg(windows)]
    let worker = worker.with_extension("exe");
    Ok(worker)
}
