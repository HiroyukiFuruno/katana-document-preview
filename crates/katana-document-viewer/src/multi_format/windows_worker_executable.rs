use super::{OfficeWorkerConfig, OfficeWorkerError};
use std::path::{Path, PathBuf};

const STAGED_WORKER_NAME: &str = "kdv-office-worker.exe";

pub(super) fn stage_windows_worker(
    workspace: &Path,
    config: &OfficeWorkerConfig,
) -> Result<PathBuf, OfficeWorkerError> {
    let destination = workspace.join(STAGED_WORKER_NAME);
    if config.executable == destination {
        return Ok(destination);
    }
    std::fs::copy(&config.executable, &destination)
        .map(|_| destination.clone())
        .map_err(|error| {
            OfficeWorkerError::unavailable(
                config,
                format!(
                    "Windows AppContainer worker staging failed: source=`{}`, destination=`{}`: {error}",
                    config.executable.display(),
                    destination.display()
                ),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::{STAGED_WORKER_NAME, stage_windows_worker};
    use crate::multi_format::{OfficeWorkerConfig, OfficeWorkerError};

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn stages_exact_worker_bytes_inside_the_session_workspace() -> TestResult {
        let source_directory = tempfile::tempdir()?;
        let workspace = tempfile::tempdir()?;
        let source = source_directory.path().join("external-worker.exe");
        std::fs::write(&source, b"worker-bytes")?;
        let config = OfficeWorkerConfig::new(source);

        let staged = stage_windows_worker(workspace.path(), &config)?;

        assert_eq!(workspace.path().join(STAGED_WORKER_NAME), staged);
        assert_eq!(b"worker-bytes", std::fs::read(staged)?.as_slice());
        Ok(())
    }

    #[test]
    fn staging_failure_preserves_original_worker_and_operation_context() -> TestResult {
        let workspace = tempfile::tempdir()?;
        let missing = workspace.path().join("missing-worker.exe");
        let config = OfficeWorkerConfig::new(missing.clone());

        let result = stage_windows_worker(workspace.path(), &config);

        assert!(matches!(
            result,
            Err(OfficeWorkerError::WorkerUnavailable { executable, reason })
                if executable == missing
                    && reason.contains("Windows AppContainer worker staging failed")
                    && reason.contains("missing-worker.exe")
                    && reason.contains(STAGED_WORKER_NAME)
        ));
        Ok(())
    }

    #[test]
    fn already_staged_worker_is_reused_without_copying_itself() -> TestResult {
        let workspace = tempfile::tempdir()?;
        let staged = workspace.path().join(STAGED_WORKER_NAME);
        std::fs::write(&staged, b"worker")?;
        let config = OfficeWorkerConfig::new(staged.clone());

        assert_eq!(staged, stage_windows_worker(workspace.path(), &config)?);
        assert_eq!(b"worker", std::fs::read(staged)?.as_slice());
        Ok(())
    }
}
