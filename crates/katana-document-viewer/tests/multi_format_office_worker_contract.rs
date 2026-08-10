use katana_document_viewer::{
    OfficeDocumentFormat, OfficeDocumentSource, OfficePreflightError, OfficeStaticViewerSession,
    OfficeWorkerConfig, OfficeWorkerEntrypoint, OfficeWorkerError, PdfPageRenderRequest,
    PdfViewerError, ViewerDiagnosticCode, ViewerFeature, ViewerFeatureStatus, ViewerSourceIdentity,
};
use std::ffi::OsString;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::time::Duration;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const MAX_GLYPH_FRAGMENT_GAP_ROWS: u32 = 2;
const MIN_PARAGRAPH_CENTER_ADVANCE_TWICE: u32 = 38;
const MAX_PARAGRAPH_CENTER_ADVANCE_TWICE: u32 = 48;
const JAPANESE_BAND_PADDING_ROWS: u32 = 2;

fn fixture(name: &str, format: OfficeDocumentFormat) -> TestResult<OfficeDocumentSource> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/multi-format")
        .join(name);
    let bytes = std::fs::read(path)?;
    Ok(OfficeDocumentSource::new(
        ViewerSourceIdentity::new(format!("file:///fixtures/{name}"), format!("sha256:{name}")),
        format,
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
        },
        bytes,
    ))
}

fn worker_config() -> OfficeWorkerConfig {
    OfficeWorkerConfig::new(PathBuf::from(env!("CARGO_BIN_EXE_kdv-office-worker")))
}

fn dark_pixels(rgba: &[u8], width: u32, x_range: Range<u32>, y_range: Range<u32>) -> usize {
    y_range
        .flat_map(|y| x_range.clone().map(move |x| (y * width + x) as usize * 4))
        .filter(|offset| {
            rgba[*offset..*offset + 3]
                .iter()
                .any(|channel| *channel < 180)
        })
        .count()
}

fn paragraph_row_bands(rgba: &[u8], width: u32) -> Vec<(u32, u32)> {
    let mut ink_bands: Vec<(u32, u32)> = Vec::new();
    for y in 120..220 {
        if dark_pixels(rgba, width, 660..900, y..y + 1) < 25 {
            continue;
        }
        match ink_bands.last_mut() {
            Some((_, end)) if *end + 1 == y => *end = y,
            _ => ink_bands.push((y, y)),
        }
    }

    let mut paragraphs: Vec<(u32, u32)> = Vec::new();
    for (start, end) in ink_bands {
        match paragraphs.last_mut() {
            Some((_, paragraph_end))
                if start <= *paragraph_end + MAX_GLYPH_FRAGMENT_GAP_ROWS + 1 =>
            {
                *paragraph_end = end;
            }
            _ => paragraphs.push((start, end)),
        }
    }
    paragraphs
}

fn assert_pptx_text_layout(rgba: &[u8], width: u32, height: u32) {
    let bands = paragraph_row_bands(rgba, width);
    assert_eq!(
        4,
        bands.len(),
        "PPTX text lines must not overlap: {bands:?}"
    );
    assert!(
        bands.windows(2).all(|pair| {
            let center_advance_twice = pair[1].0 + pair[1].1 - pair[0].0 - pair[0].1;
            (MIN_PARAGRAPH_CENTER_ADVANCE_TWICE..=MAX_PARAGRAPH_CENTER_ADVANCE_TWICE)
                .contains(&center_advance_twice)
        }),
        "PPTX paragraphs must preserve the 21.6pt line advance: {bands:?}"
    );
    let japanese_band = bands[3];
    let japanese_y = japanese_band.0.saturating_sub(JAPANESE_BAND_PADDING_ROWS)
        ..(japanese_band.1 + JAPANESE_BAND_PADDING_ROWS + 1).min(height);
    for x_range in [744..756, 759..776, 777..793] {
        assert!(
            dark_pixels(rgba, width, x_range, japanese_y.clone()) > 70,
            "each Japanese glyph must render as ink instead of tofu"
        );
    }
}

#[cfg(unix)]
fn worker_script(body: &str) -> TestResult<(tempfile::TempDir, PathBuf)> {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir()?;
    let path = directory.path().join("worker");
    std::fs::write(&path, format!("#!/bin/sh\nset -eu\n{body}\n"))?;
    let mut permissions = std::fs::metadata(&path)?.permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&path, permissions)?;
    Ok((directory, path))
}

#[test]
fn docx_isolated_worker_produces_bounded_static_pages() -> TestResult {
    let mut session = OfficeStaticViewerSession::open(
        fixture("representative.docx", OfficeDocumentFormat::Docx)?,
        worker_config(),
    )?;
    let artifact = session.artifact();

    assert_eq!(OfficeDocumentFormat::Docx, artifact.format);
    assert!(artifact.item_count > 0);
    assert_eq!(artifact.item_count, artifact.items.len());
    assert_eq!(
        ViewerFeatureStatus::Supported,
        artifact.capabilities.status(ViewerFeature::PageNavigation)
    );
    assert!(format!("{session:?}").contains("OfficeStaticViewerSession"));
    let first = session.render_item(PdfPageRenderRequest::new(0, 1.0))?;
    assert!(first.surface.width > 0);
    assert!(first.surface.height > 0);
    Ok(())
}

#[test]
fn pptx_isolated_worker_preserves_slide_profile_and_fallback_diagnostics() -> TestResult {
    let mut session = OfficeStaticViewerSession::open(
        fixture("representative.pptx", OfficeDocumentFormat::Pptx)?,
        worker_config(),
    )?;
    let artifact = session.artifact();

    assert_eq!(OfficeDocumentFormat::Pptx, artifact.format);
    assert!(artifact.item_count > 0);
    assert_eq!(
        ViewerFeatureStatus::Supported,
        artifact.capabilities.status(ViewerFeature::SlideNavigation)
    );
    assert!(artifact.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == ViewerDiagnosticCode::UnsupportedFeature
            && diagnostic.feature == Some(ViewerFeature::Chart)
    }));
    assert!(artifact.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == ViewerDiagnosticCode::DegradedRendering
            && diagnostic.message.to_ascii_lowercase().contains("fallback")
    }));
    let page = session.render_item(PdfPageRenderRequest::new(0, 1.0))?;
    assert_eq!((959, 540), (page.surface.width, page.surface.height));
    assert_pptx_text_layout(&page.surface.rgba, page.surface.width, page.surface.height);
    Ok(())
}

#[test]
fn missing_worker_is_a_typed_failure_without_in_process_fallback() -> TestResult {
    assert!(matches!(
        OfficeStaticViewerSession::open(
            fixture("representative.docx", OfficeDocumentFormat::Docx)?,
            OfficeWorkerConfig::new(PathBuf::from("/missing/kdv-office-worker")),
        ),
        Err(error)
            if matches!(&error, OfficeWorkerError::WorkerUnavailable { .. })
                && error.diagnostic().code == ViewerDiagnosticCode::WorkerUnavailable
    ));
    Ok(())
}

#[test]
fn xlsx_never_routes_through_the_static_office_worker() -> TestResult {
    assert!(matches!(
        OfficeStaticViewerSession::open(
            fixture("representative.xlsx", OfficeDocumentFormat::Xlsx)?,
            worker_config(),
        ),
        Err(OfficeWorkerError::UnsupportedFormat(
            OfficeDocumentFormat::Xlsx
        ))
    ));
    Ok(())
}

#[test]
fn worker_entrypoint_rejects_malformed_invocations_before_sandboxing() {
    let absolute = std::env::temp_dir();
    let office_cases = [
        vec![OsString::from("worker")],
        vec![
            OsString::from("worker"),
            OsString::from("relative"),
            OsString::from("docx"),
            OsString::from("1"),
            OsString::from("1"),
            OsString::from("1"),
        ],
        vec![
            OsString::from("worker"),
            absolute.as_os_str().to_owned(),
            OsString::from("unknown"),
            OsString::from("1"),
            OsString::from("1"),
            OsString::from("1"),
        ],
        vec![
            OsString::from("worker"),
            absolute.as_os_str().to_owned(),
            OsString::from("docx"),
            OsString::from("invalid"),
            OsString::from("1"),
            OsString::from("1"),
        ],
        vec![
            OsString::from("worker"),
            absolute.as_os_str().to_owned(),
            OsString::from("docx"),
            OsString::from("0"),
            OsString::from("1"),
            OsString::from("1"),
        ],
    ];
    for arguments in office_cases {
        assert_eq!(64, OfficeWorkerEntrypoint::run(arguments));
    }

    let spreadsheet_cases = [
        vec![OsString::from("worker"), OsString::from("--spreadsheet")],
        vec![
            OsString::from("worker"),
            OsString::from("--spreadsheet"),
            OsString::from("relative"),
            OsString::from("1"),
            OsString::from("1"),
            OsString::from("1"),
            OsString::from("1"),
            OsString::from("1"),
        ],
        vec![
            OsString::from("worker"),
            OsString::from("--spreadsheet"),
            absolute.as_os_str().to_owned(),
            OsString::from("invalid"),
            OsString::from("1"),
            OsString::from("1"),
            OsString::from("1"),
            OsString::from("1"),
        ],
        vec![
            OsString::from("worker"),
            OsString::from("--spreadsheet"),
            absolute.as_os_str().to_owned(),
            OsString::from("1"),
            OsString::from("1"),
            OsString::from("0"),
            OsString::from("1"),
            OsString::from("1"),
        ],
        vec![
            OsString::from("worker"),
            OsString::from("--spreadsheet"),
            absolute.as_os_str().to_owned(),
            OsString::from("0"),
            OsString::from("1"),
            OsString::from("1"),
            OsString::from("1"),
            OsString::from("1"),
        ],
        vec![
            OsString::from("worker"),
            OsString::from("--spreadsheet"),
            absolute.as_os_str().to_owned(),
            OsString::from("1"),
            OsString::from("1"),
            OsString::from("invalid"),
            OsString::from("1"),
            OsString::from("1"),
        ],
        vec![
            OsString::from("worker"),
            OsString::from("--spreadsheet"),
            absolute.as_os_str().to_owned(),
        ],
        vec![
            OsString::from("worker"),
            OsString::from("--spreadsheet"),
            absolute.as_os_str().to_owned(),
            OsString::from("1"),
            OsString::from("1"),
            OsString::from("1"),
            OsString::from("1"),
            OsString::from("1"),
            OsString::from("extra"),
        ],
    ];
    for arguments in spreadsheet_cases {
        assert_eq!(64, OfficeWorkerEntrypoint::run(arguments));
    }

    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;

        assert_eq!(
            64,
            OfficeWorkerEntrypoint::run(vec![
                OsString::from("worker"),
                OsString::from("--spreadsheet"),
                absolute.as_os_str().to_owned(),
                OsString::from_vec(vec![0xff]),
                OsString::from("1"),
                OsString::from("1"),
                OsString::from("1"),
                OsString::from("1"),
            ])
        );
    }
}

#[test]
fn every_worker_failure_category_has_a_stable_diagnostic() {
    let cases = [
        (
            OfficeWorkerError::Preflight(OfficePreflightError::InvalidArchive {
                reason: "invalid".to_owned(),
            }),
            ViewerDiagnosticCode::InvalidDocument,
        ),
        (
            OfficeWorkerError::UnsupportedFormat(OfficeDocumentFormat::Xlsx),
            ViewerDiagnosticCode::UnsupportedFormat,
        ),
        (
            OfficeWorkerError::WorkerUnavailable {
                executable: PathBuf::from("worker"),
                reason: "missing".to_owned(),
            },
            ViewerDiagnosticCode::WorkerUnavailable,
        ),
        (
            OfficeWorkerError::WorkerTimedOut,
            ViewerDiagnosticCode::WorkerTimedOut,
        ),
        (
            OfficeWorkerError::WorkerCrashed { status: Some(9) },
            ViewerDiagnosticCode::WorkerCrashed,
        ),
        (
            OfficeWorkerError::WorkerMemoryLimitExceeded { limit: 1 },
            ViewerDiagnosticCode::ResourceLimitExceeded,
        ),
        (
            OfficeWorkerError::OutputLimitExceeded {
                actual: 2,
                limit: 1,
            },
            ViewerDiagnosticCode::ResourceLimitExceeded,
        ),
        (
            OfficeWorkerError::Protocol {
                message: "protocol".to_owned(),
            },
            ViewerDiagnosticCode::EngineFailure,
        ),
        (
            OfficeWorkerError::EngineFailure {
                stage: "engine".to_owned(),
                message: "failure".to_owned(),
            },
            ViewerDiagnosticCode::EngineFailure,
        ),
        (
            OfficeWorkerError::Pdf(PdfViewerError::InvalidDocument),
            ViewerDiagnosticCode::EngineFailure,
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(expected, error.diagnostic().code);
    }
}

#[cfg(target_os = "macos")]
#[test]
fn parent_memory_monitor_fails_closed_through_the_public_viewer() -> TestResult {
    let mut config = worker_config();
    config.timeout = Duration::from_secs(2);
    config.max_memory_bytes = 0;
    let result = OfficeStaticViewerSession::open(
        fixture("representative.docx", OfficeDocumentFormat::Docx)?,
        config,
    );
    let error = match result {
        Err(error) => error,
        Ok(_) => return Err("zero memory budget did not fail closed".into()),
    };
    assert_eq!(
        OfficeWorkerError::WorkerMemoryLimitExceeded { limit: 0 },
        error
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn isolated_worker_timeout_crash_and_protocol_failures_are_typed() -> TestResult {
    let cases = [
        (
            "/bin/sleep 5",
            Duration::from_millis(10),
            ViewerDiagnosticCode::WorkerTimedOut,
        ),
        (
            "workspace=\"$1\"\nprintf '%s' '{\"status\":\"completed\",\"warnings\":[]}' > \"$workspace/response.json\"\nexit 9",
            Duration::from_secs(30),
            ViewerDiagnosticCode::WorkerCrashed,
        ),
        (
            "workspace=\"$1\"\nprintf '%s' 'not json' > \"$workspace/response.json\"",
            Duration::from_secs(30),
            ViewerDiagnosticCode::EngineFailure,
        ),
        (
            "workspace=\"$1\"\nprintf '%s' '{\"status\":\"completed\",\"warnings\":[]}' > \"$workspace/response.json\"\nprintf '%s' 'not a pdf' > \"$workspace/output.pdf\"",
            Duration::from_secs(30),
            ViewerDiagnosticCode::EngineFailure,
        ),
    ];

    for (body, timeout, expected) in cases {
        let (_directory, executable) = worker_script(body)?;
        let mut config = OfficeWorkerConfig::new(executable);
        config.timeout = timeout;
        let result = OfficeStaticViewerSession::open(
            fixture("representative.docx", OfficeDocumentFormat::Docx)?,
            config,
        );
        let error = match result {
            Err(error) => error,
            Ok(_) => return Err("fault worker did not fail closed".into()),
        };
        assert_eq!(expected, error.diagnostic().code);
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn isolated_worker_engine_and_output_limits_are_typed() -> TestResult {
    let cases = [
        (
            "workspace=\"$1\"\nprintf '%s' '{\"status\":\"failed\",\"stage\":\"engine\",\"message\":\"conversion failed\"}' > \"$workspace/response.json\"",
            u64::MAX,
            ViewerDiagnosticCode::EngineFailure,
        ),
        (
            "workspace=\"$1\"\nprintf '%s' '{\"status\":\"failed\",\"stage\":\"output_limit\",\"message\":\"large\"}' > \"$workspace/response.json\"",
            1,
            ViewerDiagnosticCode::ResourceLimitExceeded,
        ),
        (
            "workspace=\"$1\"\nprintf '%s' '{\"status\":\"completed\",\"warnings\":[]}' > \"$workspace/response.json\"\nprintf '%s' '%PDF-large' > \"$workspace/output.pdf\"",
            1,
            ViewerDiagnosticCode::ResourceLimitExceeded,
        ),
    ];

    for (body, max_output_bytes, expected) in cases {
        let (_directory, executable) = worker_script(body)?;
        let mut config = OfficeWorkerConfig::new(executable);
        config.max_output_bytes = max_output_bytes;
        let result = OfficeStaticViewerSession::open(
            fixture("representative.docx", OfficeDocumentFormat::Docx)?,
            config,
        );
        let error = match result {
            Err(error) => error,
            Ok(_) => return Err("fault worker did not fail closed".into()),
        };
        assert_eq!(expected, error.diagnostic().code);
    }
    Ok(())
}
