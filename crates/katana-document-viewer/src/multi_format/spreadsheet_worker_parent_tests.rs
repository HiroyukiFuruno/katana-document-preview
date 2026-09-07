use super::{SpreadsheetViewerSession, filter::filter_event, materialized_cells, opened_sheets};
use crate::multi_format::spreadsheet_worker_protocol::SpreadsheetWorkerResponse;
use crate::multi_format::{
    OfficeDocumentFormat, OfficeDocumentSource, OfficeWorkerConfig, OfficeWorkerError,
    SpreadsheetCoordinate, ViewerSourceIdentity, debug_trace::DebugTrace,
};
use std::path::{Path, PathBuf};

#[test]
fn response_decoders_accept_expected_states() {
    let sheets = Vec::new();
    assert_eq!(
        Ok(sheets.clone()),
        opened_sheets(SpreadsheetWorkerResponse::Opened {
            sheets: sheets.clone(),
        })
    );
    assert_eq!(
        Ok(Vec::new()),
        materialized_cells(
            1,
            SpreadsheetWorkerResponse::Materialized {
                request_id: 1,
                cells: Vec::new(),
            },
        )
    );
}

#[test]
fn response_decoders_reject_wrong_states_and_request_ids() {
    assert_eq!(
        opened_sheets(SpreadsheetWorkerResponse::Stopped),
        Err(OfficeWorkerError::Protocol {
            message: "unexpected spreadsheet response during open: Stopped".to_owned(),
        })
    );
    assert_eq!(
        materialized_cells(
            1,
            SpreadsheetWorkerResponse::Materialized {
                request_id: 2,
                cells: Vec::new(),
            },
        ),
        Err(OfficeWorkerError::Protocol {
            message: "unexpected spreadsheet response during materialize: Materialized { request_id: 2, cells: [] }".to_owned(),
        })
    );
}

#[test]
fn response_decoders_preserve_worker_failures() {
    assert_eq!(
        materialized_cells(
            1,
            SpreadsheetWorkerResponse::Failed {
                request_id: Some(1),
                stage: "spreadsheet".to_owned(),
                message: "failed".to_owned(),
            },
        ),
        Err(OfficeWorkerError::EngineFailure {
            stage: "spreadsheet".to_owned(),
            message: "failed".to_owned(),
        })
    );
}

#[test]
fn debug_output_is_covered_with_the_isolated_worker() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/multi-format/representative.xlsx");
    let source = OfficeDocumentSource::new(
        ViewerSourceIdentity::new("file:///unit.xlsx", "sha256:unit-xlsx"),
        OfficeDocumentFormat::Xlsx,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        std::fs::read(fixture)?,
    );
    let mut session =
        SpreadsheetViewerSession::open(source, OfficeWorkerConfig::new(worker_binary_path()?))?;

    assert_trace_scope(&mut session);
    assert!(format!("{session:?}").contains("SpreadsheetViewerSession"));
    assert_eq!(2, session.artifact().sheet_count);
    session.update_candidates(0, 0, &["ignored".to_owned()]);
    session
        .worker
        .send(&super::SpreadsheetWorkerRequest::Materialize {
            request_id: 99,
            sheet_index: usize::MAX,
            coordinates: vec![SpreadsheetCoordinate::new(0, 0)],
        })?;
    assert!(matches!(
        materialized_cells(99, session.worker.receive()?),
        Err(OfficeWorkerError::EngineFailure { .. })
    ));
    Ok(())
}

fn assert_trace_scope(session: &mut SpreadsheetViewerSession) {
    session.trace_session = Some((42, 0x0123_4567_89ab_cdef));
    let _trace_scope = session.trace_scope();
    assert_eq!(
        Some((42, 0x0123_4567_89ab_cdef)),
        DebugTrace::current_session()
    );
}

#[test]
fn filter_response_decoder_preserves_failures_and_rejects_wrong_states() {
    assert!(matches!(
        filter_event(
            1,
            SpreadsheetWorkerResponse::Failed {
                request_id: Some(1),
                stage: "spreadsheet".to_owned(),
                message: "failed".to_owned(),
            },
        ),
        Err(OfficeWorkerError::EngineFailure { .. })
    ));
    assert!(matches!(
        filter_event(1, SpreadsheetWorkerResponse::Stopped),
        Err(OfficeWorkerError::Protocol { .. })
    ));
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
    let worker = {
        let mut worker = worker;
        worker.set_extension("exe");
        worker
    };
    Ok(worker)
}
