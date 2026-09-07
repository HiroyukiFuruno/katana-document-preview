use super::super::office_worker_protocol::INPUT_NAME;
use super::super::spreadsheet_engine::{SpreadsheetEngineError, SpreadsheetEngineSession};
use super::super::spreadsheet_worker_arguments::SpreadsheetWorkerArguments;

pub(in crate::multi_format) fn open_engine(
    arguments: &SpreadsheetWorkerArguments,
) -> Result<SpreadsheetEngineSession, (String, String)> {
    let _open = super::super::debug_trace::DebugTrace::start("spreadsheet.package_parse");
    let input = std::fs::read(arguments.workspace.join(INPUT_NAME)).map_err(input_failure)?;
    let name = arguments.workspace.join(INPUT_NAME);
    SpreadsheetEngineSession::open(input, &name.to_string_lossy(), arguments.limits)
        .map_err(spreadsheet_open_failure)
}

pub(in crate::multi_format) fn protocol_failure(message: String) -> (String, String) {
    ("protocol".to_owned(), message)
}

pub(in crate::multi_format) fn input_failure(error: std::io::Error) -> (String, String) {
    failure("input", error.to_string())
}

pub(in crate::multi_format) fn spreadsheet_open_failure(
    error: SpreadsheetEngineError,
) -> (String, String) {
    failure("spreadsheet_open", error.to_string())
}

pub(in crate::multi_format) fn failure(stage: &str, message: String) -> (String, String) {
    (stage.to_owned(), message)
}
