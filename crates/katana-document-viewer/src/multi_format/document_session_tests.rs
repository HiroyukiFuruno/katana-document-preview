use super::*;
use crate::{
    BinaryDocumentSource, DocumentFitMode, DocumentGridCommand, DocumentSessionCommandKind,
    DocumentSurfaceCommand, DocumentSurfaceKind, DocumentViewerCommand, DocumentViewerEvent,
    DocumentViewport, OfficeDocumentSource, ViewerDocumentFormat, ViewerSourceIdentity,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const COMMAND_KINDS: [(DocumentSessionCommand, DocumentSessionCommandKind); 9] = [
    (
        DocumentSessionCommand::Viewer(DocumentViewerCommand::Previous),
        DocumentSessionCommandKind::Previous,
    ),
    (
        DocumentSessionCommand::Viewer(DocumentViewerCommand::Next),
        DocumentSessionCommandKind::Next,
    ),
    (
        DocumentSessionCommand::Viewer(DocumentViewerCommand::JumpTo(2)),
        DocumentSessionCommandKind::JumpTo,
    ),
    (
        DocumentSessionCommand::Viewer(DocumentViewerCommand::SetZoom(2.0)),
        DocumentSessionCommandKind::SetZoom,
    ),
    (
        DocumentSessionCommand::Viewer(DocumentViewerCommand::Fit(DocumentFitMode::Page)),
        DocumentSessionCommandKind::Fit,
    ),
    (
        DocumentSessionCommand::Viewer(DocumentViewerCommand::CopySelection),
        DocumentSessionCommandKind::CopySelection,
    ),
    (
        DocumentSessionCommand::Viewer(DocumentViewerCommand::OpenTarget),
        DocumentSessionCommandKind::OpenTarget,
    ),
    (
        DocumentSessionCommand::Surface(DocumentSurfaceCommand::Resize(DocumentViewport::new(
            1, 1,
        ))),
        DocumentSessionCommandKind::Resize,
    ),
    (
        DocumentSessionCommand::Surface(DocumentSurfaceCommand::Grid(
            DocumentGridCommand::ScrollTo { x: 1, y: 1 },
        )),
        DocumentSessionCommandKind::Grid,
    ),
];

#[test]
fn pdf_session_owns_navigation_surface_and_rendering() -> TestResult {
    let _baseline = DocumentSession::resource_snapshot();
    let mut session = DocumentSession::open(
        ViewerSource::Pdf(BinaryDocumentSource::new(
            ViewerSourceIdentity::new("file:///sample.pdf", "sha256:sample"),
            "application/pdf",
            include_bytes!("../../../../assets/reference/katana/pdf/sample.pdf").to_vec(),
        )),
        DocumentSessionConfig::new(DocumentViewport::new(640, 480)),
    )?;

    let initial = session.frame()?;
    assert_eq!(ViewerDocumentFormat::Pdf, initial.format);
    assert_eq!(DocumentSurfaceKind::Page, initial.surface.kind());
    assert!(initial.state.item_count > 0);
    assert!(initial.spreadsheet.is_none());

    assert_pdf_filter_is_rejected(&mut session);

    assert_pdf_commands(&mut session)?;
    assert_eq!("file:///sample.pdf", session.info().identity.uri);
    assert_idempotent_close(&mut session);
    Ok(())
}

fn assert_idempotent_close(session: &mut DocumentSession) {
    session.close();
    session.close();
    assert!(session.is_closed());
    assert_eq!(Err(DocumentSessionError::Closed), session.frame());
    assert_eq!(
        Err(DocumentSessionError::Closed),
        session.apply(DocumentSessionCommand::Surface(
            DocumentSurfaceCommand::Resize(DocumentViewport::new(1, 1)),
        ))
    );
    assert_eq!(
        Err(DocumentSessionError::Closed),
        session.apply_spreadsheet_filter(SpreadsheetFilterCommand::Clear {
            sheet_index: 0,
            column: None,
        })
    );
}

fn assert_pdf_filter_is_rejected(session: &mut DocumentSession) {
    assert_eq!(
        Err(DocumentSessionError::UnsupportedCommand {
            format: ViewerDocumentFormat::Pdf,
            command: DocumentSessionCommandKind::SpreadsheetFilter,
        }),
        session.apply_spreadsheet_filter(SpreadsheetFilterCommand::Clear {
            sheet_index: 0,
            column: None,
        })
    );
}

fn assert_pdf_commands(session: &mut DocumentSession) -> TestResult {
    assert_eq!(
        DocumentSessionEvent::None,
        session.apply(DocumentSessionCommand::Surface(
            DocumentSurfaceCommand::Resize(DocumentViewport::new(800, 600)),
        ))?
    );
    assert_eq!(
        Err(DocumentSessionError::UnsupportedCommand {
            format: ViewerDocumentFormat::Pdf,
            command: DocumentSessionCommandKind::Grid,
        }),
        session.apply(DocumentSessionCommand::Surface(
            DocumentSurfaceCommand::Grid(DocumentGridCommand::ScrollTo { x: 1, y: 1 })
        ))
    );
    assert_eq!(
        DocumentSessionEvent::Viewer(DocumentViewerEvent::ZoomChanged(1.25)),
        session.apply(DocumentSessionCommand::Viewer(
            DocumentViewerCommand::SetZoom(1.25),
        ))?
    );
    let zoomed = session.frame()?;
    assert_eq!(1.25, zoomed.state.zoom);
    Ok(())
}

#[test]
fn office_session_requires_an_explicit_worker_resource() {
    for format in [
        OfficeDocumentFormat::Docx,
        OfficeDocumentFormat::Xlsx,
        OfficeDocumentFormat::Pptx,
    ] {
        let source = ViewerSource::Office(OfficeDocumentSource::new(
            ViewerSourceIdentity::new("file:///office", "sha256:office"),
            format,
            "application/octet-stream",
            vec![1],
        ));
        assert!(matches!(
            DocumentSession::open(
                source,
                DocumentSessionConfig::new(DocumentViewport::new(320, 240)),
            ),
            Err(DocumentSessionError::MissingOfficeWorker {
                format: actual_format
            }) if actual_format == format
        ));
    }
}

#[test]
fn office_formats_map_to_kdv_owned_document_formats() {
    assert_eq!(
        ViewerDocumentFormat::Docx,
        ViewerDocumentFormat::from(OfficeDocumentFormat::Docx)
    );
    assert_eq!(
        ViewerDocumentFormat::Xlsx,
        ViewerDocumentFormat::from(OfficeDocumentFormat::Xlsx)
    );
    assert_eq!(
        ViewerDocumentFormat::Pptx,
        ViewerDocumentFormat::from(OfficeDocumentFormat::Pptx)
    );
}

#[test]
fn every_document_session_command_has_a_stable_kind() {
    for (command, expected) in COMMAND_KINDS {
        assert_eq!(expected, command.kind());
    }
}

#[path = "document_session_tests_xlsx.rs"]
mod xlsx;
