use katana_document_viewer::{
    BinaryDocumentSource, DocumentFitMode, DocumentGridCommand, DocumentGridEvent,
    DocumentGridNavigation, DocumentResourceSnapshot, DocumentSession, DocumentSessionCommand,
    DocumentSessionCommandKind, DocumentSessionConfig, DocumentSessionError, DocumentSessionEvent,
    DocumentSurfaceCommand, DocumentSurfaceKind, DocumentViewerCommand, DocumentViewerEvent,
    DocumentViewport, OfficeDocumentFormat, OfficeDocumentSource, OfficeWorkerConfig,
    SpreadsheetFilterCommand, ViewerDocumentFormat, ViewerFeature, ViewerFeatureStatus,
    ViewerSource, ViewerSourceIdentity,
};
use std::{
    path::{Path, PathBuf},
    time::Instant,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const SUPPLIED_PPTX_CYCLES: usize = 10;

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/multi-format")
        .join(name)
}

fn office_source(
    name: &str,
    format: OfficeDocumentFormat,
) -> Result<ViewerSource, Box<dyn std::error::Error>> {
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
    Ok(ViewerSource::Office(OfficeDocumentSource::new(
        ViewerSourceIdentity::new(format!("file:///fixtures/{name}"), format!("sha256:{name}")),
        format,
        mime,
        std::fs::read(fixture_path(name))?,
    )))
}

fn config() -> DocumentSessionConfig {
    let worker = std::env::var_os("KDV_ACCEPTANCE_WORKER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_kdv-office-worker")));
    DocumentSessionConfig::new(DocumentViewport::new(640, 480))
        .office_worker(OfficeWorkerConfig::new(worker))
}

#[test]
fn pdf_uses_the_unified_session_for_fit_zoom_resize_and_typed_errors() -> TestResult {
    let bytes = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/reference/katana/pdf/sample.pdf"),
    )?;
    let source = ViewerSource::Pdf(BinaryDocumentSource::new(
        ViewerSourceIdentity::new("file:///fixtures/representative.pdf", "sha256:pdf"),
        "application/pdf",
        bytes,
    ));
    let mut session = DocumentSession::open(source, config())?;
    let _resource_snapshot = DocumentSession::resource_snapshot();
    assert!(!session.is_closed());
    assert_eq!(
        "file:///fixtures/representative.pdf",
        session.info().identity.uri
    );
    assert_eq!("application/pdf", session.info().mime);
    assert_eq!(ViewerDocumentFormat::Pdf, session.info().format);
    assert_frame(
        &mut session,
        ViewerDocumentFormat::Pdf,
        DocumentSurfaceKind::Page,
    )?;

    for command in [
        DocumentSessionCommand::Viewer(DocumentViewerCommand::Fit(DocumentFitMode::Width)),
        DocumentSessionCommand::Viewer(DocumentViewerCommand::Fit(DocumentFitMode::Page)),
        DocumentSessionCommand::Viewer(DocumentViewerCommand::SetZoom(1.5)),
        DocumentSessionCommand::Surface(DocumentSurfaceCommand::Resize(DocumentViewport::new(
            900, 700,
        ))),
    ] {
        let _ = session.apply(command)?;
        assert_frame(
            &mut session,
            ViewerDocumentFormat::Pdf,
            DocumentSurfaceKind::Page,
        )?;
    }
    let state_before_rejected_command = session.frame()?.state;
    assert_eq!(
        Err(DocumentSessionError::UnsupportedCommand {
            format: ViewerDocumentFormat::Pdf,
            command: DocumentSessionCommandKind::Grid,
        }),
        session.apply(DocumentSessionCommand::Surface(
            DocumentSurfaceCommand::Grid(DocumentGridCommand::ScrollTo { x: 10, y: 20 })
        ))
    );
    assert_eq!(state_before_rejected_command, session.frame()?.state);
    assert!(matches!(
        session.apply(DocumentSessionCommand::Viewer(
            DocumentViewerCommand::JumpTo(usize::MAX)
        )),
        Err(DocumentSessionError::State(_))
    ));
    session.close();
    assert!(session.is_closed());
    assert_eq!(
        Err(DocumentSessionError::Closed),
        session.apply(DocumentSessionCommand::Surface(
            DocumentSurfaceCommand::Resize(DocumentViewport::new(1, 1)),
        ))
    );
    assert_eq!(Err(DocumentSessionError::Closed), session.frame());
    assert_eq!(
        Err(DocumentSessionError::Closed),
        session.apply_spreadsheet_filter(SpreadsheetFilterCommand::Clear {
            sheet_index: 0,
            column: None,
        })
    );
    Ok(())
}

#[test]
fn docx_and_pptx_use_the_unified_paged_session() -> TestResult {
    for (name, source_format, frame_format) in [
        (
            "representative.docx",
            OfficeDocumentFormat::Docx,
            ViewerDocumentFormat::Docx,
        ),
        (
            "representative.pptx",
            OfficeDocumentFormat::Pptx,
            ViewerDocumentFormat::Pptx,
        ),
    ] {
        let mut session = DocumentSession::open(office_source(name, source_format)?, config())?;
        assert_frame(&mut session, frame_format, DocumentSurfaceKind::Page)?;
        assert!(matches!(
            session.apply(DocumentSessionCommand::Viewer(DocumentViewerCommand::Next))?,
            DocumentSessionEvent::Viewer(_)
        ));
        assert_frame(&mut session, frame_format, DocumentSurfaceKind::Page)?;
        session.close();
    }
    Ok(())
}

#[test]
fn xlsx_uses_the_unified_session_for_sheet_grid_and_materialization() -> TestResult {
    let mut session = DocumentSession::open(
        office_source("representative.xlsx", OfficeDocumentFormat::Xlsx)?,
        config(),
    )?;
    assert_frame(
        &mut session,
        ViewerDocumentFormat::Xlsx,
        DocumentSurfaceKind::Grid,
    )?;
    assert_eq!(ViewerDocumentFormat::Xlsx, session.info().format);
    assert!(
        session
            .apply_spreadsheet_filter(SpreadsheetFilterCommand::Candidates {
                sheet_index: 0,
                column: 0,
                limit: 8,
            })
            .is_err()
    );
    assert_eq!(
        ViewerFeatureStatus::Supported,
        session
            .info()
            .capabilities
            .status(ViewerFeature::SheetNavigation)
    );

    assert_eq!(
        DocumentSessionEvent::Grid(DocumentGridEvent::Scrolled),
        session.apply(DocumentSessionCommand::Surface(
            DocumentSurfaceCommand::Grid(DocumentGridCommand::ScrollTo { x: 40, y: 20 })
        ))?
    );
    assert_eq!(
        DocumentSessionEvent::Grid(DocumentGridEvent::SelectionChanged),
        session.apply(DocumentSessionCommand::Surface(
            DocumentSurfaceCommand::Grid(DocumentGridCommand::Navigate {
                intent: DocumentGridNavigation::Down,
                extend: false,
            })
        ))?
    );
    assert_eq!(
        DocumentSessionEvent::None,
        session.apply(DocumentSessionCommand::Surface(
            DocumentSurfaceCommand::Resize(DocumentViewport::new(800, 600))
        ))?
    );
    assert!(matches!(
        session.apply(DocumentSessionCommand::Viewer(DocumentViewerCommand::Next))?,
        DocumentSessionEvent::Viewer(_)
    ));
    assert_eq!(
        DocumentSessionEvent::Viewer(DocumentViewerEvent::CopyRequested),
        session.apply(DocumentSessionCommand::Viewer(
            DocumentViewerCommand::CopySelection
        ))?
    );
    assert_frame(
        &mut session,
        ViewerDocumentFormat::Xlsx,
        DocumentSurfaceKind::Grid,
    )?;

    for (command, kind) in [
        (
            DocumentViewerCommand::SetZoom(1.25),
            DocumentSessionCommandKind::SetZoom,
        ),
        (
            DocumentViewerCommand::Fit(DocumentFitMode::Width),
            DocumentSessionCommandKind::Fit,
        ),
        (
            DocumentViewerCommand::OpenTarget,
            DocumentSessionCommandKind::OpenTarget,
        ),
    ] {
        let state_before_rejected_command = session.frame()?.state;
        assert_eq!(
            Err(DocumentSessionError::UnsupportedCommand {
                format: ViewerDocumentFormat::Xlsx,
                command: kind,
            }),
            session.apply(DocumentSessionCommand::Viewer(command))
        );
        assert_eq!(state_before_rejected_command, session.frame()?.state);
    }
    session.close();
    Ok(())
}

#[test]
fn unified_session_preserves_worker_startup_failures_for_office_formats() -> TestResult {
    for (name, format) in [
        ("representative.docx", OfficeDocumentFormat::Docx),
        ("representative.xlsx", OfficeDocumentFormat::Xlsx),
    ] {
        let result = DocumentSession::open(
            office_source(name, format)?,
            DocumentSessionConfig::new(DocumentViewport::new(320, 240)).office_worker(
                OfficeWorkerConfig::new(PathBuf::from("/missing/kdv-office-worker")),
            ),
        );
        assert!(matches!(result, Err(DocumentSessionError::Office(_))));
    }
    Ok(())
}

#[test]
fn unified_session_preserves_invalid_pdf_failures() {
    let source = ViewerSource::Pdf(BinaryDocumentSource::new(
        ViewerSourceIdentity::new("file:///invalid.pdf", "sha256:invalid"),
        "application/pdf",
        b"not a pdf".to_vec(),
    ));
    assert!(matches!(
        DocumentSession::open(
            source,
            DocumentSessionConfig::new(DocumentViewport::new(320, 240))
        ),
        Err(DocumentSessionError::Pdf(_))
    ));
}

#[test]
#[ignore = "requires KDV_ACCEPTANCE_FIXTURE_DIR with user-supplied Office documents"]
fn user_supplied_office_fixtures_open_through_the_unified_session() -> TestResult {
    let mut failures = Vec::new();
    for path in acceptance_office_fixture_paths()? {
        let (source, format) = acceptance_office_source(&path)?;
        let first_frame_started = Instant::now();
        let mut session = match DocumentSession::open(source, config()) {
            Ok(session) => session,
            Err(error) => {
                failures.push(format!("{} failed to open: {error}", path.display()));
                continue;
            }
        };
        let result = match format {
            OfficeDocumentFormat::Pptx => match assert_pptx_first_frame(&mut session) {
                Ok(()) => {
                    trace_supplied_pptx_first_frame(first_frame_started);
                    assert_pptx_source_reuse_after_first_frame(&mut session)
                }
                Err(error) => Err(error),
            },
            OfficeDocumentFormat::Xlsx => {
                let scroll = DocumentSessionCommand::Surface(DocumentSurfaceCommand::Grid(
                    DocumentGridCommand::ScrollTo { x: 320, y: 20_000 },
                ));
                session
                    .frame()
                    .and_then(|_| session.apply(scroll))
                    .and_then(|_| session.frame())
                    .map(|_| ())
                    .map_err(Into::into)
            }
            OfficeDocumentFormat::Docx => session.frame().map(|_| ()).map_err(Into::into),
        };
        if let Err(error) = result {
            failures.push(format!("{} failed to render: {error}", path.display()));
        }
        session.close();
    }
    if !failures.is_empty() {
        return Err(failures.join("\n").into());
    }
    Ok(())
}

#[test]
#[ignore = "requires KDV_ACCEPTANCE_FIXTURE_DIR with the supplied PPTX corpus"]
fn supplied_pptx_reuses_its_source_and_cleans_up_after_ten_cycles() -> TestResult {
    let paths = acceptance_office_fixture_paths()?
        .into_iter()
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("pptx"))
        .collect::<Vec<_>>();
    if paths.is_empty() {
        return Err("no supplied PPTX fixtures found".into());
    }

    let baseline = DocumentResourceSnapshot::capture();
    for cycle in 0..SUPPLIED_PPTX_CYCLES {
        for path in &paths {
            let (source, format) = acceptance_office_source(path)?;
            assert_eq!(OfficeDocumentFormat::Pptx, format);
            let mut session = DocumentSession::open(source, config())?;
            assert_pptx_source_reuse(&mut session)?;
            session.close();
            assert_eq!(
                baseline,
                DocumentResourceSnapshot::capture(),
                "cycle {cycle}, fixture {} leaked resources",
                path.display()
            );
        }
    }
    Ok(())
}

#[test]
#[ignore = "used as the matched no-op process baseline by measure-office-first-frame.py"]
fn supplied_pptx_measurement_noop_baseline() {}

fn acceptance_office_fixture_paths() -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let directory = PathBuf::from(std::env::var("KDV_ACCEPTANCE_FIXTURE_DIR")?);
    let requested_name = std::env::var("KDV_ACCEPTANCE_FIXTURE_NAME").ok();
    let mut paths = std::fs::read_dir(&directory)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("docx" | "xlsx" | "pptx")
            )
        })
        .filter(|path| {
            requested_name.as_ref().is_none_or(|requested| {
                path.file_name().and_then(|name| name.to_str()) == Some(requested.as_str())
            })
        })
        .collect::<Vec<_>>();
    paths.sort();
    if paths.is_empty() {
        return Err(format!("no Office fixtures found in {}", directory.display()).into());
    }
    Ok(paths)
}

fn acceptance_office_source(
    path: &Path,
) -> Result<(ViewerSource, OfficeDocumentFormat), Box<dyn std::error::Error>> {
    let format = match path.extension().and_then(|value| value.to_str()) {
        Some("docx") => OfficeDocumentFormat::Docx,
        Some("xlsx") => OfficeDocumentFormat::Xlsx,
        Some("pptx") => OfficeDocumentFormat::Pptx,
        _ => return Err(format!("unsupported Office fixture: {}", path.display()).into()),
    };
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("fixture name is not UTF-8: {}", path.display()))?;
    let source = ViewerSource::Office(OfficeDocumentSource::new(
        ViewerSourceIdentity::new(
            format!("file://{}", path.display()),
            format!("acceptance:{name}"),
        ),
        format,
        mime_for_office_format(format),
        std::fs::read(path)?,
    ));
    Ok((source, format))
}

const fn mime_for_office_format(format: OfficeDocumentFormat) -> &'static str {
    match format {
        OfficeDocumentFormat::Docx => {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        }
        OfficeDocumentFormat::Xlsx => {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        }
        OfficeDocumentFormat::Pptx => {
            "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        }
    }
}

fn assert_pptx_source_reuse(session: &mut DocumentSession) -> TestResult {
    assert_pptx_first_frame(session)?;
    assert_pptx_source_reuse_after_first_frame(session)
}

fn assert_pptx_first_frame(session: &mut DocumentSession) -> TestResult {
    assert_frame(
        session,
        ViewerDocumentFormat::Pptx,
        DocumentSurfaceKind::Page,
    )
}

fn assert_pptx_source_reuse_after_first_frame(session: &mut DocumentSession) -> TestResult {
    assert_eq!(
        DocumentSessionEvent::None,
        session.apply(DocumentSessionCommand::Surface(
            DocumentSurfaceCommand::Resize(DocumentViewport::new(900, 700))
        ))?
    );
    assert_frame(
        session,
        ViewerDocumentFormat::Pptx,
        DocumentSurfaceKind::Page,
    )?;
    assert_frame(
        session,
        ViewerDocumentFormat::Pptx,
        DocumentSurfaceKind::Page,
    )
}

fn trace_supplied_pptx_first_frame(first_frame_started: Instant) {
    if std::env::var("DEBUG")
        .ok()
        .is_some_and(|value| value.eq_ignore_ascii_case("true") || value == "1")
    {
        eprintln!(
            "[KDV_ACCEPTANCE] stage=first_frame elapsed_ms={}",
            first_frame_started.elapsed().as_millis()
        );
    }
}

fn assert_frame(
    session: &mut DocumentSession,
    format: ViewerDocumentFormat,
    kind: DocumentSurfaceKind,
) -> TestResult {
    let frame = session.frame()?;
    assert_eq!(format, frame.format);
    assert_eq!(kind, frame.surface.kind());
    assert!(frame.state.item_count > 0);
    Ok(())
}
