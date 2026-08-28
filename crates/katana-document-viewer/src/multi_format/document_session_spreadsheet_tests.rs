use super::*;
use crate::ViewerSourceIdentity;
use std::path::{Path, PathBuf};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn corrupted_sheet_index_fails_closed_before_surface_replacement() -> TestResult {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/multi-format/representative.xlsx");
    let source = OfficeDocumentSource::new(
        ViewerSourceIdentity::new("file:///unit.xlsx", "sha256:unit-xlsx"),
        super::super::OfficeDocumentFormat::Xlsx,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        std::fs::read(fixture)?,
    );
    let mut session = SpreadsheetDocumentSession::open(
        source,
        OfficeWorkerConfig::new(worker_binary_path()?),
        DocumentViewport::new(640, 480),
    )?;
    session.state.active_index = usize::MAX;

    assert!(matches!(
        session.replace_surface(),
        Err(DocumentSessionError::State(
            super::super::DocumentViewerStateError::IndexOutsideDocument { .. }
        ))
    ));
    Ok(())
}

#[test]
fn spreadsheet_frame_exposes_worksheet_names_for_host_tabs() -> TestResult {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/multi-format/representative.xlsx");
    let source = OfficeDocumentSource::new(
        ViewerSourceIdentity::new("file:///unit.xlsx", "sha256:unit-xlsx"),
        super::super::OfficeDocumentFormat::Xlsx,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        std::fs::read(fixture)?,
    );
    let mut session = SpreadsheetDocumentSession::open(
        source,
        OfficeWorkerConfig::new(worker_binary_path()?),
        DocumentViewport::new(640, 480),
    )?;

    let frame = session.frame()?;
    assert_eq!(frame.state.item_count, frame.surface.item_labels().len());
    assert!(
        frame
            .surface
            .item_labels()
            .iter()
            .all(|name| !name.trim().is_empty())
    );
    Ok(())
}

#[test]
fn spreadsheet_filter_worker_updates_event_artifact_and_frame_visibility() -> TestResult {
    let mut session = filter_session()?;
    let candidates = session.apply_filter(SpreadsheetFilterCommand::Candidates {
        sheet_index: 0,
        column: 0,
        limit: 16,
    })?;
    assert!(matches!(
        candidates,
        SpreadsheetFilterEvent::Candidates { .. }
    ));
    assert_filter_visibility(&mut session)?;
    assert_invalid_filter_requests(&mut session);
    Ok(())
}

fn assert_invalid_filter_requests(session: &mut SpreadsheetDocumentSession) {
    assert!(
        session
            .apply_filter(SpreadsheetFilterCommand::Candidates {
                sheet_index: usize::MAX,
                column: 0,
                limit: 8,
            })
            .is_err()
    );
    assert!(
        session
            .apply_filter(SpreadsheetFilterCommand::ApplyValues {
                sheet_index: 0,
                column: usize::MAX,
                values: vec!["missing".to_owned()],
            })
            .is_err()
    );
    assert!(
        session
            .apply_filter(SpreadsheetFilterCommand::Clear {
                sheet_index: usize::MAX,
                column: None,
            })
            .is_err()
    );
}

fn assert_filter_visibility(session: &mut SpreadsheetDocumentSession) -> TestResult {
    let applied = session.apply_filter(SpreadsheetFilterCommand::ApplyValues {
        sheet_index: 0,
        column: 0,
        values: vec!["North".to_owned()],
    })?;
    assert!(matches!(
        applied,
        SpreadsheetFilterEvent::VisibilityChanged {
            visible_row_count: 4,
            ..
        }
    ));
    assert_frame_metadata_and_clear(session)?;
    Ok(())
}

fn assert_frame_metadata_and_clear(session: &mut SpreadsheetDocumentSession) -> TestResult {
    let metadata = session.frame()?.spreadsheet.ok_or("metadata missing")?;
    assert_eq!(4, metadata.visible_row_count);
    assert_eq!(
        vec![4, 5, 6],
        metadata
            .auto_filter
            .ok_or("filter missing")?
            .filtered_out_rows
    );
    let cleared = session.apply_filter(SpreadsheetFilterCommand::Clear {
        sheet_index: 0,
        column: None,
    })?;
    assert!(matches!(
        cleared,
        SpreadsheetFilterEvent::VisibilityChanged {
            visible_row_count: 7,
            ..
        }
    ));
    Ok(())
}

fn filter_session() -> Result<SpreadsheetDocumentSession, Box<dyn std::error::Error>> {
    let bytes = super::super::spreadsheet_filter_test_support::representative_with_auto_filter()?;
    let source = OfficeDocumentSource::new(
        ViewerSourceIdentity::new("file:///filter.xlsx", "sha256:filter-xlsx"),
        super::super::OfficeDocumentFormat::Xlsx,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        bytes,
    );
    Ok(SpreadsheetDocumentSession::open(
        source,
        OfficeWorkerConfig::new(worker_binary_path()?),
        DocumentViewport::new(640, 480),
    )?)
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
