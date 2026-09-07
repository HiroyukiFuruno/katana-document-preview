use super::{
    OfficeWorkerConfig, OfficeWorkerError, complete_conversion, failed_conversion,
    trace::trace_source_fingerprint,
};
use crate::multi_format::office_worker_protocol::{OUTPUT_NAME, OfficeWorkerResponse};
use crate::multi_format::{
    OfficeDocumentFormat, OfficePreflightError, PdfViewerError, ViewerDiagnosticCode,
    ViewerSourceIdentity,
};
use std::path::PathBuf;

fn assert_diagnostic_cases(cases: Vec<(OfficeWorkerError, ViewerDiagnosticCode)>) {
    for (error, code) in cases {
        assert_eq!(code, error.diagnostic().code);
    }
}

#[test]
fn document_and_lifecycle_errors_map_to_stable_diagnostics() {
    assert_diagnostic_cases(vec![
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
            OfficeWorkerError::WorkerCrashed { status: None },
            ViewerDiagnosticCode::WorkerCrashed,
        ),
    ]);
}

#[test]
fn resource_errors_map_to_stable_diagnostics() {
    assert_diagnostic_cases(vec![
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
    ]);
}

#[test]
fn engine_errors_map_to_stable_diagnostics() {
    assert_diagnostic_cases(vec![
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
    ]);
}

#[test]
fn worker_error_helpers_remain_typed() {
    let config = OfficeWorkerConfig::new(PathBuf::from("worker"));
    assert!(matches!(
        OfficeWorkerError::unavailable(&config, "missing".to_owned()),
        OfficeWorkerError::WorkerUnavailable { .. }
    ));
    assert!(matches!(
        OfficeWorkerError::protocol("bad response".to_owned()),
        OfficeWorkerError::Protocol { .. }
    ));
    assert!(matches!(
        OfficeWorkerError::protocol_io(std::io::Error::other("broken pipe")),
        OfficeWorkerError::Protocol { .. }
    ));
    let json_error = serde_json::from_slice::<serde_json::Value>(b"{");
    assert!(json_error.is_err());
    if let Err(error) = json_error {
        assert!(matches!(
            OfficeWorkerError::protocol_json(error),
            OfficeWorkerError::Protocol { .. }
        ));
    }
}

#[test]
fn completed_conversion_returns_pdf_and_rejects_crashes() {
    let config = OfficeWorkerConfig::new(PathBuf::from("worker"));
    let workspace = tempfile::tempdir();
    assert!(workspace.is_ok());
    if let Ok(workspace) = workspace {
        assert!(std::fs::write(workspace.path().join(OUTPUT_NAME), b"%PDF-1.7").is_ok());
        let response = OfficeWorkerResponse::Completed {
            warnings: vec!["fallback".to_owned()],
        };
        let output = complete_conversion(workspace.path(), Some(0), response, &config);
        assert!(output.is_ok());
        if let Ok(output) = output {
            assert_eq!(b"%PDF-1.7", output.pdf.as_slice());
            assert_eq!(vec!["fallback".to_owned()], output.warnings);
        }
        let response = OfficeWorkerResponse::Completed {
            warnings: Vec::new(),
        };
        assert!(matches!(
            complete_conversion(workspace.path(), Some(9), response, &config),
            Err(OfficeWorkerError::WorkerCrashed { status: Some(9) })
        ));
    }
}

#[test]
fn failed_conversion_maps_output_and_engine_failures() {
    let config = OfficeWorkerConfig::new(PathBuf::from("worker"));
    assert!(matches!(
        failed_conversion("output_limit".to_owned(), "large".to_owned(), config.max_output_bytes),
        Err(OfficeWorkerError::OutputLimitExceeded { actual, limit })
            if actual == config.max_output_bytes + 1 && limit == config.max_output_bytes
    ));
    let workspace = std::path::Path::new(".");
    let response = OfficeWorkerResponse::Failed {
        stage: "engine".to_owned(),
        message: "conversion failed".to_owned(),
    };
    assert!(matches!(
        complete_conversion(workspace, Some(70), response, &config),
        Err(OfficeWorkerError::EngineFailure { .. })
    ));
}

#[test]
fn trace_source_fingerprint_separates_source_revisions_without_exposing_them() {
    let first = ViewerSourceIdentity::new("memory://report.xlsx", "rev-7");
    let second = ViewerSourceIdentity::new("memory://report.xlsx", "rev-8");
    assert_ne!(
        trace_source_fingerprint(&first),
        trace_source_fingerprint(&second)
    );
}
