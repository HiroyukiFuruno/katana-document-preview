use super::office_worker_protocol::INPUT_NAME;
use super::{OfficeWorkerConfig, OfficeWorkerError};

pub(crate) struct OfficeWorkerWorkspace;

impl OfficeWorkerWorkspace {
    pub(crate) fn prepare(
        prefix: &str,
        bytes: &[u8],
        config: &OfficeWorkerConfig,
    ) -> Result<tempfile::TempDir, OfficeWorkerError> {
        let workspace = Self::created(config, tempfile::Builder::new().prefix(prefix).tempdir())?;
        Self::input_written(
            config,
            std::fs::write(workspace.path().join(INPUT_NAME), bytes),
        )
        .map(|()| workspace)
    }

    fn created(
        config: &OfficeWorkerConfig,
        result: std::io::Result<tempfile::TempDir>,
    ) -> Result<tempfile::TempDir, OfficeWorkerError> {
        match result {
            Ok(workspace) => Ok(workspace),
            Err(error) => Err(OfficeWorkerError::unavailable(config, error.to_string())),
        }
    }

    fn input_written(
        config: &OfficeWorkerConfig,
        result: std::io::Result<()>,
    ) -> Result<(), OfficeWorkerError> {
        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(OfficeWorkerError::unavailable(config, error.to_string())),
        }
    }
}

#[cfg(test)]
#[path = "office_worker_workspace_tests.rs"]
mod tests;
