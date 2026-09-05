#[cfg(target_os = "macos")]
use super::office_worker_monitor::MacOsMemoryMonitor;
use super::office_worker_workspace::OfficeWorkerWorkspace;
use super::spreadsheet_worker_owner::SpreadsheetProcessOwner;
use super::spreadsheet_worker_protocol::{
    MAX_SPREADSHEET_REQUEST_BYTES, SpreadsheetWorkerRequest, SpreadsheetWorkerResponse,
};
use super::spreadsheet_worker_reader::SpreadsheetResponseReader;
use super::spreadsheet_worker_spawn::{SpawnedSpreadsheetProcess, SpreadsheetWorkerSpawn};
use super::{OfficeDocumentSource, OfficeWorkerConfig, OfficeWorkerError};
use std::io::Write;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::thread::JoinHandle;
use std::time::Duration;

const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(1);
type SpreadsheetResponseChannel = (
    Receiver<Result<SpreadsheetWorkerResponse, String>>,
    JoinHandle<()>,
);

pub(crate) struct SpreadsheetWorkerProcess {
    input: Box<dyn Write + Send>,
    responses: Receiver<Result<SpreadsheetWorkerResponse, String>>,
    reader: Option<JoinHandle<()>>,
    #[cfg(windows)]
    stderr_reader: Option<JoinHandle<()>>,
    owner: SpreadsheetProcessOwner,
    #[cfg(target_os = "macos")]
    memory_monitor: Option<MacOsMemoryMonitor>,
    timeout: Duration,
    #[cfg(target_os = "macos")]
    max_memory_bytes: usize,
    _workspace: tempfile::TempDir,
    _resource_lease: super::resource_metrics::SpreadsheetWorkerLease,
    trace_session: Option<super::debug_trace::TraceSession>,
    #[cfg(all(coverage, not(windows)))]
    coverage_profile: Option<super::coverage_profile::ChildCoverageProfile>,
}

impl SpreadsheetWorkerProcess {
    pub(crate) fn spawn(
        source: &OfficeDocumentSource,
        config: &OfficeWorkerConfig,
    ) -> Result<Self, OfficeWorkerError> {
        let _spawn = super::debug_trace::DebugTrace::start("spreadsheet.spawn");
        let (workspace, spawned) = prepare_worker_process(source, config)?;
        let (responses, reader) = spawn_response_reader(spawned.output);
        #[cfg(target_os = "macos")]
        let memory_monitor = Some(start_memory_monitor(spawned.process_id, config));
        Ok(Self {
            input: spawned.input,
            responses,
            reader: Some(reader),
            #[cfg(windows)]
            stderr_reader: spawned.stderr_reader,
            owner: spawned.owner,
            #[cfg(target_os = "macos")]
            memory_monitor,
            timeout: config.timeout,
            #[cfg(target_os = "macos")]
            max_memory_bytes: config.max_memory_bytes,
            _workspace: workspace,
            _resource_lease: super::resource_metrics::SpreadsheetWorkerLease::acquire(),
            trace_session: super::debug_trace::DebugTrace::current_session(),
            #[cfg(all(coverage, not(windows)))]
            coverage_profile: spawned.coverage_profile,
        })
    }

    pub(crate) fn send(
        &mut self,
        request: &SpreadsheetWorkerRequest,
    ) -> Result<(), OfficeWorkerError> {
        let bytes = encode_request(request)?;
        self.input
            .write_all(&bytes)
            .map_err(OfficeWorkerError::protocol_io)?;
        self.input.flush().map_err(OfficeWorkerError::protocol_io)
    }

    pub(crate) fn receive(&mut self) -> Result<SpreadsheetWorkerResponse, OfficeWorkerError> {
        match self.responses.recv_timeout(self.timeout) {
            Ok(Ok(response)) => Self::response_or_error(response),
            Ok(Err(message)) => Err(OfficeWorkerError::protocol(message)),
            Err(RecvTimeoutError::Timeout) => {
                self.owner.terminate();
                Err(OfficeWorkerError::WorkerTimedOut)
            }
            Err(RecvTimeoutError::Disconnected) => Err(self.disconnected_error()),
        }
    }

    fn response_or_error(
        response: SpreadsheetWorkerResponse,
    ) -> Result<SpreadsheetWorkerResponse, OfficeWorkerError> {
        if let SpreadsheetWorkerResponse::Failed {
            request_id: None,
            stage,
            message,
        } = response
        {
            Err(OfficeWorkerError::EngineFailure { stage, message })
        } else {
            Ok(response)
        }
    }

    fn disconnected_error(&mut self) -> OfficeWorkerError {
        #[cfg(target_os = "macos")]
        if self
            .memory_monitor
            .as_ref()
            .is_some_and(MacOsMemoryMonitor::exceeded)
        {
            return OfficeWorkerError::WorkerMemoryLimitExceeded {
                limit: self.max_memory_bytes,
            };
        }
        OfficeWorkerError::WorkerCrashed {
            status: self.owner.status(),
        }
    }
}

fn prepare_worker_process(
    source: &OfficeDocumentSource,
    config: &OfficeWorkerConfig,
) -> Result<(tempfile::TempDir, SpawnedSpreadsheetProcess), OfficeWorkerError> {
    let workspace =
        OfficeWorkerWorkspace::prepare("kdv-spreadsheet-worker-", &source.bytes, config)?;
    let spawned = SpreadsheetWorkerSpawn::spawn(workspace.path(), config)?;
    Ok((workspace, spawned))
}

fn encode_request(request: &SpreadsheetWorkerRequest) -> Result<Vec<u8>, OfficeWorkerError> {
    let mut bytes = serde_json::to_vec(request).map_err(OfficeWorkerError::protocol_json)?;
    bytes.push(b'\n');
    if bytes.len() > MAX_SPREADSHEET_REQUEST_BYTES {
        return Err(OfficeWorkerError::protocol(
            "spreadsheet request exceeds its byte limit".to_owned(),
        ));
    }
    Ok(bytes)
}

fn spawn_response_reader(output: Box<dyn std::io::Read + Send>) -> SpreadsheetResponseChannel {
    let reader = SpreadsheetResponseReader::spawn(output);
    (reader.receiver, reader.worker)
}

#[cfg(target_os = "macos")]
fn start_memory_monitor(process_id: u32, config: &OfficeWorkerConfig) -> MacOsMemoryMonitor {
    MacOsMemoryMonitor::start(process_id, config.max_memory_bytes)
}

impl Drop for SpreadsheetWorkerProcess {
    fn drop(&mut self) {
        let _trace_scope = self
            .trace_session
            .map(super::debug_trace::DebugTrace::session);
        let _drop = super::debug_trace::DebugTrace::start("spreadsheet.drop");
        let _ = self.send(&SpreadsheetWorkerRequest::Shutdown);
        self.owner.finish(GRACEFUL_SHUTDOWN_TIMEOUT);
        #[cfg(all(coverage, not(windows)))]
        if let Some(profile) = self.coverage_profile.take() {
            let _ = profile.collect();
        }
        #[cfg(target_os = "macos")]
        if let Some(monitor) = self.memory_monitor.take() {
            let _ = monitor.finish();
        }
        #[cfg(windows)]
        if let Some(stderr_reader) = self.stderr_reader.take() {
            let _ = stderr_reader.join();
        }
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

#[cfg(test)]
#[path = "spreadsheet_worker_process_tests.rs"]
mod tests;
