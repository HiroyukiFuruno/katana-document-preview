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
