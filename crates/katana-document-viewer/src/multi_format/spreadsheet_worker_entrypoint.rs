use super::office_worker_constraints::OfficeWorkerConstraints;
use super::office_worker_protocol::INPUT_NAME;
use super::spreadsheet_engine::SpreadsheetEngineSession;
use super::spreadsheet_worker_arguments::SpreadsheetWorkerArguments;
use super::spreadsheet_worker_protocol::{
    MAX_SPREADSHEET_REQUEST_BYTES, MAX_SPREADSHEET_RESPONSE_BYTES, SpreadsheetWorkerRequest,
    SpreadsheetWorkerResponse,
};
use std::ffi::OsString;
use std::io::{BufReader, BufWriter, Write};

#[path = "spreadsheet_worker_io.rs"]
mod io;
#[cfg(test)]
use io::{
    invalid_request, protocol_read_failure, protocol_write_failure, response_encoding_failure,
    write_encoded_response,
};
use io::{read_request, write_response, write_response_limited};

const EXIT_USAGE: i32 = 64;
const EXIT_FAILURE: i32 = 70;
type SpreadsheetWorker = fn(SpreadsheetWorkerArguments) -> Result<(), (String, String)>;
type ConstraintApplier = fn(&std::path::Path, u64, u64) -> Result<(), (String, String)>;

pub(super) struct SpreadsheetWorkerEntrypoint;

impl SpreadsheetWorkerEntrypoint {
    pub(super) fn run(arguments: Vec<OsString>) -> i32 {
        let mut writer = BufWriter::new(std::io::stdout());
        Self::run_with(arguments, SpreadsheetWorkerLoop::run, &mut writer)
    }

    fn run_with(
        arguments: Vec<OsString>,
        worker: SpreadsheetWorker,
        writer: &mut dyn Write,
    ) -> i32 {
        let arguments = match SpreadsheetWorkerArguments::parse(arguments) {
            Ok(arguments) => arguments,
            Err(_) => return EXIT_USAGE,
        };
        match worker(arguments) {
            Ok(()) => 0,
            Err((stage, message)) => {
                let response = SpreadsheetWorkerResponse::Failed {
                    request_id: None,
                    stage,
                    message,
                };
                let _ = write_response(writer, &response);
                EXIT_FAILURE
            }
        }
    }
}

struct SpreadsheetWorkerLoop {
    engine: SpreadsheetEngineSession,
    reader: BufReader<std::io::Stdin>,
    writer: BufWriter<std::io::Stdout>,
}

impl SpreadsheetWorkerLoop {
    fn run(arguments: SpreadsheetWorkerArguments) -> Result<(), (String, String)> {
        Self::run_with_constraints(arguments, OfficeWorkerConstraints::apply)
    }

    fn run_with_constraints(
        arguments: SpreadsheetWorkerArguments,
        apply_constraints: ConstraintApplier,
    ) -> Result<(), (String, String)> {
        apply_constraints(
            &arguments.workspace,
            arguments.max_memory_bytes,
            arguments.max_cpu_seconds,
        )?;
        let input = std::fs::read(arguments.workspace.join(INPUT_NAME)).map_err(input_failure)?;
        let name = arguments.workspace.join(INPUT_NAME);
        let engine =
            SpreadsheetEngineSession::open(input, &name.to_string_lossy(), arguments.limits)
                .map_err(spreadsheet_open_failure)?;
        let mut worker = Self {
            engine,
            reader: BufReader::new(std::io::stdin()),
            writer: BufWriter::new(std::io::stdout()),
        };
        worker
            .write(&SpreadsheetWorkerResponse::Opened {
                sheets: worker.engine.sheets().to_vec(),
            })
            .map_err(protocol_failure)?;
        worker.run_requests().map_err(protocol_failure)
    }

    fn run_requests(&mut self) -> Result<(), String> {
        loop {
            let request = self.read()?;
            match request {
                SpreadsheetWorkerRequest::Materialize {
                    request_id,
                    sheet_index,
                    coordinates,
                } => self.materialize(request_id, sheet_index, &coordinates)?,
                SpreadsheetWorkerRequest::Shutdown => {
                    self.write(&SpreadsheetWorkerResponse::Stopped)?;
                    return Ok(());
                }
            }
        }
    }

    fn materialize(
        &mut self,
        request_id: u64,
        sheet_index: usize,
        coordinates: &[super::SpreadsheetCoordinate],
    ) -> Result<(), String> {
        let response = match self.engine.materialize(sheet_index, coordinates) {
            Ok(cells) => SpreadsheetWorkerResponse::Materialized { request_id, cells },
            Err(error) => SpreadsheetWorkerResponse::Failed {
                request_id: Some(request_id),
                stage: "spreadsheet".to_owned(),
                message: error.to_string(),
            },
        };
        self.write(&response)
    }

    fn read(&mut self) -> Result<SpreadsheetWorkerRequest, String> {
        read_request(&mut self.reader, MAX_SPREADSHEET_REQUEST_BYTES)
    }

    fn write(&mut self, response: &SpreadsheetWorkerResponse) -> Result<(), String> {
        write_response_limited(&mut self.writer, response, MAX_SPREADSHEET_RESPONSE_BYTES)
    }
}

fn protocol_failure(message: String) -> (String, String) {
    ("protocol".to_owned(), message)
}

fn input_failure(error: std::io::Error) -> (String, String) {
    failure("input", error.to_string())
}

fn spreadsheet_open_failure(
    error: super::spreadsheet_engine::SpreadsheetEngineError,
) -> (String, String) {
    failure("spreadsheet_open", error.to_string())
}

fn failure(stage: &str, message: String) -> (String, String) {
    (stage.to_owned(), message)
}

#[cfg(test)]
#[path = "spreadsheet_worker_entrypoint_tests.rs"]
mod tests;
