use super::office_worker_output::OfficeWorkerOutputReader;
use super::office_worker_process::OfficeWorkerProcess;
use super::office_worker_protocol::OfficeWorkerResponse;
use super::office_worker_workspace::OfficeWorkerWorkspace;
use super::{
    OfficeDocumentFormat, OfficeDocumentSource, OfficePreflightError, OfficePreflightLimits,
    PdfViewerError, SpreadsheetViewerLimits, ViewerDiagnostic, ViewerDiagnosticCode,
    ViewerDiagnosticSeverity,
};
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(45);
const DEFAULT_MAX_MEMORY_BYTES: usize = 2 * 1024 * 1024 * 1024;
const DEFAULT_MAX_OUTPUT_BYTES: u64 = 128 * 1024 * 1024;

#[path = "office_worker_parent_preflight.rs"]
mod preflight;
#[path = "office_worker_trace.rs"]
pub(super) mod trace;
use preflight::preflight_diagnostics;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfficeWorkerConfig {
    pub executable: PathBuf,
    pub timeout: Duration,
    pub max_memory_bytes: usize,
    pub max_output_bytes: u64,
    pub preflight_limits: OfficePreflightLimits,
    pub spreadsheet_limits: SpreadsheetViewerLimits,
}

impl OfficeWorkerConfig {
    #[must_use]
    pub fn new(executable: PathBuf) -> Self {
        Self {
            executable,
            timeout: DEFAULT_TIMEOUT,
            max_memory_bytes: DEFAULT_MAX_MEMORY_BYTES,
            max_output_bytes: DEFAULT_MAX_OUTPUT_BYTES,
            preflight_limits: OfficePreflightLimits::strict(),
            spreadsheet_limits: SpreadsheetViewerLimits::strict(),
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OfficeWorkerError {
    #[error(transparent)]
    Preflight(#[from] OfficePreflightError),
    #[error("Office format {0:?} is not supported by this viewer profile")]
    UnsupportedFormat(OfficeDocumentFormat),
    #[error("Office worker `{executable}` is unavailable: {reason}")]
    WorkerUnavailable { executable: PathBuf, reason: String },
    #[error("Office worker exceeded its time limit")]
    WorkerTimedOut,
    #[error("Office worker crashed with status {status:?}")]
    WorkerCrashed { status: Option<i64> },
    #[error("Office worker exceeded its {limit}-byte memory limit")]
    WorkerMemoryLimitExceeded { limit: usize },
    #[error("Office worker protocol failed: {message}")]
    Protocol { message: String },
    #[error("Office engine failed at {stage}: {message}")]
    EngineFailure { stage: String, message: String },
    #[error("Office worker output is {actual} bytes and exceeds {limit} bytes")]
    OutputLimitExceeded { actual: u64, limit: u64 },
    #[error(transparent)]
    Pdf(#[from] PdfViewerError),
}

impl OfficeWorkerError {
    #[must_use]
    pub fn diagnostic(&self) -> ViewerDiagnostic {
        let code = match self {
            Self::Preflight(error) => return error.diagnostic(),
            Self::UnsupportedFormat(_) => ViewerDiagnosticCode::UnsupportedFormat,
            Self::WorkerUnavailable { .. } => ViewerDiagnosticCode::WorkerUnavailable,
            Self::WorkerTimedOut => ViewerDiagnosticCode::WorkerTimedOut,
            Self::WorkerCrashed { .. } => ViewerDiagnosticCode::WorkerCrashed,
            Self::WorkerMemoryLimitExceeded { .. } | Self::OutputLimitExceeded { .. } => {
                ViewerDiagnosticCode::ResourceLimitExceeded
            }
            Self::Protocol { .. } | Self::EngineFailure { .. } | Self::Pdf(_) => {
                ViewerDiagnosticCode::EngineFailure
            }
        };
        ViewerDiagnostic {
            code,
            severity: ViewerDiagnosticSeverity::Error,
            feature: None,
            status: None,
            message: self.to_string(),
        }
    }

    pub(crate) fn unavailable(config: &OfficeWorkerConfig, reason: String) -> Self {
        Self::WorkerUnavailable {
            executable: config.executable.clone(),
            reason,
        }
    }

    pub(crate) fn protocol(message: String) -> Self {
        Self::Protocol { message }
    }

    pub(crate) fn protocol_io(error: std::io::Error) -> Self {
        Self::protocol(error.to_string())
    }

    pub(crate) fn protocol_json(error: serde_json::Error) -> Self {
        Self::protocol(error.to_string())
    }
}

pub(super) struct OfficeWorkerOutput {
    pub pdf: Vec<u8>,
    pub warnings: Vec<String>,
    pub preflight_diagnostics: Vec<ViewerDiagnostic>,
}
pub(super) struct OfficeWorkerRunner;

impl OfficeWorkerRunner {
    pub fn convert(
        source: &OfficeDocumentSource,
        config: &OfficeWorkerConfig,
    ) -> Result<OfficeWorkerOutput, OfficeWorkerError> {
        let _conversion = super::debug_trace::DebugTrace::start("office.total");
        let preflight_diagnostics = preflight_diagnostics(source, config)?;
        let workspace = {
            let _transfer = super::debug_trace::DebugTrace::start("office.transfer_to_worker");
            let _workspace = super::debug_trace::DebugTrace::start("office.workspace");
            OfficeWorkerWorkspace::prepare("kdv-office-worker-", &source.bytes, config)?
        };
        let status = {
            let _convert = super::debug_trace::DebugTrace::start("office.conversion");
            OfficeWorkerProcess::run(workspace.path(), source.format, config)?
        };
        let response = {
            let _transfer = super::debug_trace::DebugTrace::start("office.transfer_from_worker");
            let _decode = super::debug_trace::DebugTrace::start("office.response_decode");
            OfficeWorkerOutputReader::read_response(workspace.path())?
        };
        complete_conversion(workspace.path(), status, response, config).map(|mut output| {
            super::debug_trace::DebugTrace::event(
                "office.artifact",
                format_args!("bytes={}", output.pdf.len()),
            );
            output.preflight_diagnostics = preflight_diagnostics;
            output
        })
    }
}
fn complete_conversion(
    workspace: &Path,
    status: Option<i64>,
    response: OfficeWorkerResponse,
    config: &OfficeWorkerConfig,
) -> Result<OfficeWorkerOutput, OfficeWorkerError> {
    match response {
        OfficeWorkerResponse::Completed { warnings } => {
            if status != Some(0) {
                return Err(OfficeWorkerError::WorkerCrashed { status });
            }
            let pdf = OfficeWorkerOutputReader::read_pdf(workspace, config.max_output_bytes)?;
            Ok(OfficeWorkerOutput {
                pdf,
                warnings,
                preflight_diagnostics: Vec::new(),
            })
        }
        OfficeWorkerResponse::Failed { stage, message } => {
            failed_conversion(stage, message, config.max_output_bytes)
        }
    }
}

fn failed_conversion(
    stage: String,
    message: String,
    max_output_bytes: u64,
) -> Result<OfficeWorkerOutput, OfficeWorkerError> {
    if stage == "output_limit" {
        return Err(OfficeWorkerError::OutputLimitExceeded {
            actual: max_output_bytes.saturating_add(1),
            limit: max_output_bytes,
        });
    }
    Err(OfficeWorkerError::EngineFailure { stage, message })
}

#[cfg(test)]
#[path = "office_worker_parent_tests.rs"]
mod tests;
