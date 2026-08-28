use super::*;
use crate::OfficeWorkerConfig;
use std::path::Path;

#[test]
fn xlsx_session_routes_unified_runtime_operations() -> TestResult {
    let mut session = DocumentSession::open(xlsx_source()?, xlsx_config()?)?;

    assert_eq!(ViewerDocumentFormat::Xlsx, session.frame()?.format);
    assert!(
        session
            .apply_spreadsheet_filter(SpreadsheetFilterCommand::Candidates {
                sheet_index: 0,
                column: 0,
                limit: 8,
            })
            .is_err()
    );
    assert!(matches!(
        session.apply(DocumentSessionCommand::Surface(
            DocumentSurfaceCommand::Grid(DocumentGridCommand::ScrollTo { x: 1, y: 1 })
        )),
        Ok(DocumentSessionEvent::Grid(_))
    ));
    Ok(())
}

fn xlsx_source() -> Result<ViewerSource, Box<dyn std::error::Error>> {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/multi-format/representative.xlsx");
    Ok(ViewerSource::Office(OfficeDocumentSource::new(
        ViewerSourceIdentity::new("file:///unit.xlsx", "sha256:unit-xlsx"),
        OfficeDocumentFormat::Xlsx,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        std::fs::read(fixture)?,
    )))
}

fn xlsx_config() -> Result<DocumentSessionConfig, Box<dyn std::error::Error>> {
    let current_exe = std::env::current_exe()?;
    let worker = current_exe
        .parent()
        .and_then(Path::parent)
        .ok_or("unit test binary has no target directory")?
        .join(if cfg!(windows) {
            "kdv-office-worker.exe"
        } else {
            "kdv-office-worker"
        });
    Ok(DocumentSessionConfig::new(DocumentViewport::new(640, 480))
        .office_worker(OfficeWorkerConfig::new(worker)))
}
