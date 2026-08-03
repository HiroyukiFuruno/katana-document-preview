use super::OfficeWorkerWorkspace;
use crate::multi_format::office_worker_protocol::INPUT_NAME;
use crate::multi_format::{OfficeWorkerConfig, OfficeWorkerError};
use std::path::PathBuf;

#[test]
fn workspace_creation_writes_the_bounded_input() {
    let config = OfficeWorkerConfig::new(PathBuf::from("worker"));
    let workspace = OfficeWorkerWorkspace::prepare("kdv-workspace-test-", b"input", &config);
    assert!(workspace.is_ok());
    if let Ok(workspace) = workspace {
        let input = std::fs::read(workspace.path().join(INPUT_NAME));
        assert!(input.is_ok());
        if let Ok(input) = input {
            assert_eq!(b"input", input.as_slice());
        }
    }
}

#[test]
fn workspace_creation_and_input_failures_keep_worker_context() {
    let config = OfficeWorkerConfig::new(PathBuf::from("worker"));
    let creation_error = OfficeWorkerWorkspace::created(
        &config,
        Err(std::io::Error::other("workspace unavailable")),
    );
    assert!(creation_error.is_err());
    if let Err(creation_error) = creation_error {
        assert_eq!(
            creation_error,
            OfficeWorkerError::WorkerUnavailable {
                executable: PathBuf::from("worker"),
                reason: "workspace unavailable".to_owned(),
            }
        );
    }
    assert_eq!(
        OfficeWorkerWorkspace::input_written(&config, Err(std::io::Error::other("input blocked")),),
        Err(OfficeWorkerError::WorkerUnavailable {
            executable: PathBuf::from("worker"),
            reason: "input blocked".to_owned(),
        })
    );
}
