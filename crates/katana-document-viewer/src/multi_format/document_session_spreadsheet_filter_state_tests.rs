use super::*;
use crate::ViewerSourceIdentity;
use std::path::PathBuf;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn scoped_filters_add_and_clear_their_own_frame_metadata_criteria() -> TestResult {
    let mut session = filter_session()?;
    assert_missing_filter_column(&mut session, 1)?;
    apply_scoped_filter(&mut session)?;
    assert_filter_criteria(&mut session, 1, &["98"])?;
    clear_scoped_filter(&mut session)?;
    assert_cleared_filter_criteria(&mut session, 1)?;
    assert_filter_criteria(&mut session, 0, &["North"])?;
    Ok(())
}

fn apply_scoped_filter(session: &mut SpreadsheetDocumentSession) -> TestResult {
    session.apply_filter(SpreadsheetFilterCommand::ApplyValues {
        sheet_index: 0,
        column: 1,
        values: vec!["98".to_owned()],
    })?;
    Ok(())
}

fn clear_scoped_filter(session: &mut SpreadsheetDocumentSession) -> TestResult {
    session.apply_filter(SpreadsheetFilterCommand::Clear {
        sheet_index: 0,
        column: Some(1),
    })?;
    Ok(())
}

fn assert_missing_filter_column(
    session: &mut SpreadsheetDocumentSession,
    column: usize,
) -> TestResult {
    let metadata = session.frame()?.spreadsheet.ok_or("metadata missing")?;
    let filter = metadata.auto_filter.ok_or("filter missing")?;
    assert!(
        !filter
            .columns
            .iter()
            .any(|candidate| candidate.column == column)
    );
    Ok(())
}

fn assert_filter_criteria(
    session: &mut SpreadsheetDocumentSession,
    column: usize,
    expected: &[&str],
) -> TestResult {
    let metadata = session.frame()?.spreadsheet.ok_or("metadata missing")?;
    let filter = metadata.auto_filter.ok_or("filter missing")?;
    let criteria = &filter
        .columns
        .iter()
        .find(|candidate| candidate.column == column)
        .ok_or("filter column missing")?
        .criteria;
    assert_eq!(
        [crate::SpreadsheetFilterCriterion::Values(
            expected.iter().map(|value| (*value).to_owned()).collect()
        )],
        criteria.as_slice()
    );
    Ok(())
}

fn assert_cleared_filter_criteria(
    session: &mut SpreadsheetDocumentSession,
    column: usize,
) -> TestResult {
    let metadata = session.frame()?.spreadsheet.ok_or("metadata missing")?;
    let filter = metadata.auto_filter.ok_or("filter missing")?;
    let criteria = &filter
        .columns
        .iter()
        .find(|candidate| candidate.column == column)
        .ok_or("filter column missing")?
        .criteria;
    assert!(criteria.is_empty());
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
