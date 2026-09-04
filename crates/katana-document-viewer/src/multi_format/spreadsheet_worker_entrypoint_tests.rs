use super::open::{failure, input_failure, spreadsheet_open_failure};
use super::{
    EXIT_FAILURE, EXIT_USAGE, OsString, SpreadsheetWorkerEntrypoint, SpreadsheetWorkerLoop,
    SpreadsheetWorkerRequest, SpreadsheetWorkerResponse, invalid_request, protocol_failure,
    protocol_read_failure, protocol_write_failure, read_request, response_encoding_failure,
    write_encoded_response, write_response,
};
use crate::multi_format::spreadsheet_engine::SpreadsheetEngineError;
use crate::multi_format::spreadsheet_worker_arguments::SpreadsheetWorkerArguments;
use crate::multi_format::spreadsheet_worker_protocol::SPREADSHEET_MODE;
use std::io::{BufReader, Cursor, Read, Write};

fn arguments(workspace: &std::path::Path) -> Vec<OsString> {
    [
        OsString::from("worker"),
        OsString::from(SPREADSHEET_MODE),
        workspace.as_os_str().to_owned(),
        OsString::from("1024"),
        OsString::from("1"),
        OsString::from("8"),
        OsString::from("1024"),
        OsString::from("128"),
    ]
    .into()
}

fn completed(_arguments: SpreadsheetWorkerArguments) -> Result<(), (String, String)> {
    Ok(())
}

fn failed(_arguments: SpreadsheetWorkerArguments) -> Result<(), (String, String)> {
    Err(("spreadsheet_open".to_owned(), "invalid".to_owned()))
}

fn constraints_denied(
    _workspace: &std::path::Path,
    _max_memory_bytes: u64,
    _max_cpu_seconds: u64,
) -> Result<(), (String, String)> {
    Err(("sandbox".to_owned(), "denied".to_owned()))
}

#[test]
fn top_level_usage_and_worker_failures_have_stable_exit_codes() {
    let mut output = Vec::new();
    assert_eq!(
        EXIT_USAGE,
        SpreadsheetWorkerEntrypoint::run_with(
            vec![OsString::from("worker")],
            completed,
            &mut output,
        )
    );
    let workspace = tempfile::tempdir();
    assert!(workspace.is_ok());
    if let Ok(workspace) = workspace {
        assert_eq!(
            0,
            SpreadsheetWorkerEntrypoint::run_with(
                arguments(workspace.path()),
                completed,
                &mut output
            )
        );
        assert_eq!(
            EXIT_FAILURE,
            SpreadsheetWorkerEntrypoint::run_with(arguments(workspace.path()), failed, &mut output)
        );
    }
    assert!(String::from_utf8_lossy(&output).contains("spreadsheet_open"));
}

#[test]
fn worker_constraints_fail_before_engine_open() {
    let workspace = tempfile::tempdir();
    assert!(workspace.is_ok());
    if let Ok(workspace) = workspace {
        let parsed = SpreadsheetWorkerArguments::parse(arguments(workspace.path()));
        assert!(parsed.is_ok());
        if let Ok(parsed) = parsed {
            assert!(
                SpreadsheetWorkerLoop::run_with_constraints(parsed, constraints_denied).is_err()
            );
        }
    }
}

#[test]
fn request_reader_rejects_invalid_frames_and_accepts_shutdown() {
    let mut empty = BufReader::new(Cursor::new(Vec::<u8>::new()));
    assert!(read_request(&mut empty, 16).is_err());
    let mut oversized = BufReader::new(Cursor::new(vec![b'x'; 17]));
    assert!(read_request(&mut oversized, 16).is_err());
    let mut valid = BufReader::new(Cursor::new(b"{\"command\":\"shutdown\"}\n".to_vec()));
    assert_eq!(
        Ok(SpreadsheetWorkerRequest::Shutdown),
        read_request(&mut valid, 32)
    );
    let mut invalid = BufReader::new(Cursor::new(b"not json\n".to_vec()));
    assert!(read_request(&mut invalid, 32).is_err());
    let mut failing = BufReader::new(FailingReader);
    assert!(read_request(&mut failing, 32).is_err());
}

#[test]
fn response_writer_rejects_size_and_write_failures() {
    let response = SpreadsheetWorkerResponse::Stopped;
    assert!(write_encoded_response(&mut Vec::new(), &[1], 0).is_err());
    assert!(write_response(&mut FailingWriter, &response).is_err());
    assert!(FailingWriter.flush().is_err());
}

#[test]
fn protocol_failure_helpers_preserve_stage_context() {
    assert_eq!(
        ("protocol".to_owned(), "failure".to_owned()),
        protocol_failure("failure".to_owned())
    );
    assert_eq!(
        ("input".to_owned(), "missing".to_owned()),
        failure("input", "missing".to_owned())
    );
    assert_eq!("input", input_failure(std::io::Error::other("missing")).0);
    assert_eq!(
        "spreadsheet_open",
        spreadsheet_open_failure(SpreadsheetEngineError::Model("invalid".to_owned())).0
    );
    assert!(protocol_read_failure(std::io::Error::other("read")).contains("read"));
    assert!(protocol_write_failure(std::io::Error::other("write")).contains("write"));
}

#[test]
fn protocol_json_failure_helpers_preserve_operation_context() {
    let invalid = serde_json::from_slice::<serde_json::Value>(b"{");
    assert!(invalid.is_err());
    if let Err(error) = invalid {
        assert!(invalid_request(error).contains("invalid request"));
    }
    let encoding = serde_json::from_slice::<serde_json::Value>(b"{");
    assert!(encoding.is_err());
    if let Err(error) = encoding {
        assert!(response_encoding_failure(error).contains("response encoding"));
    }
}

struct FailingWriter;
struct FailingReader;

impl Read for FailingReader {
    fn read(&mut self, _bytes: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("read failed"))
    }
}

impl Write for FailingWriter {
    fn write(&mut self, _bytes: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("write failed"))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Err(std::io::Error::other("flush failed"))
    }
}
