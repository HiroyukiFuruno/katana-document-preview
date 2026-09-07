use super::{
    SpreadsheetProcessOwner, SpreadsheetWorkerProcess, SpreadsheetWorkerRequest,
    SpreadsheetWorkerResponse, encode_request,
};
#[cfg(target_os = "macos")]
use crate::multi_format::office_worker_monitor::MacOsMemoryMonitor;
use crate::multi_format::{OfficeWorkerError, SpreadsheetCoordinate};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

type WorkerResponseResult = Result<SpreadsheetWorkerResponse, String>;
type WorkerResponseChannel = (Sender<WorkerResponseResult>, Receiver<WorkerResponseResult>);

fn channel() -> WorkerResponseChannel {
    std::sync::mpsc::channel()
}

fn process(
    responses: Receiver<WorkerResponseResult>,
    timeout: Duration,
    workspace: tempfile::TempDir,
) -> SpreadsheetWorkerProcess {
    SpreadsheetWorkerProcess {
        input: Box::new(Vec::<u8>::new()),
        responses,
        reader: None,
        #[cfg(windows)]
        stderr_reader: None,
        owner: SpreadsheetProcessOwner { child: None },
        #[cfg(target_os = "macos")]
        memory_monitor: None,
        timeout,
        #[cfg(target_os = "macos")]
        max_memory_bytes: usize::MAX,
        _workspace: workspace,
        _resource_lease: super::super::resource_metrics::SpreadsheetWorkerLease::acquire(),
        trace_session: None,
        #[cfg(all(coverage, not(windows)))]
        coverage_profile: None,
    }
}

#[test]
fn request_encoding_and_failed_response_are_bounded() {
    let request = SpreadsheetWorkerRequest::Materialize {
        request_id: 1,
        sheet_index: 0,
        coordinates: vec![SpreadsheetCoordinate::new(1, 1); 100_000],
    };
    assert!(matches!(
        encode_request(&request),
        Err(OfficeWorkerError::Protocol { .. })
    ));
    let response = SpreadsheetWorkerResponse::Failed {
        request_id: None,
        stage: "open".to_owned(),
        message: "failed".to_owned(),
    };
    assert!(matches!(
        SpreadsheetWorkerProcess::response_or_error(response),
        Err(OfficeWorkerError::EngineFailure { .. })
    ));
}

#[test]
fn receive_reports_invalid_frame_and_timeout() {
    let workspaces = (tempfile::tempdir(), tempfile::tempdir());
    assert!(workspaces.0.is_ok() && workspaces.1.is_ok());
    if let (Ok(invalid_workspace), Ok(timeout_workspace)) = workspaces {
        let (sender, receiver) = channel();
        assert!(sender.send(Err("invalid frame".to_owned())).is_ok());
        let mut invalid = process(receiver, Duration::from_secs(1), invalid_workspace);
        assert!(matches!(
            invalid.receive(),
            Err(OfficeWorkerError::Protocol { .. })
        ));
        let (_sender, receiver) = channel();
        let mut timed_out = process(receiver, Duration::ZERO, timeout_workspace);
        assert_eq!(Err(OfficeWorkerError::WorkerTimedOut), timed_out.receive());
    }
}

#[test]
fn receive_reports_disconnect_and_accepts_completed_response() {
    let workspaces = (tempfile::tempdir(), tempfile::tempdir());
    assert!(workspaces.0.is_ok() && workspaces.1.is_ok());
    if let (Ok(disconnected_workspace), Ok(completed_workspace)) = workspaces {
        let (sender, receiver) = channel();
        drop(sender);
        let mut disconnected = process(receiver, Duration::from_secs(1), disconnected_workspace);
        assert_eq!(
            Err(OfficeWorkerError::WorkerCrashed { status: None }),
            disconnected.receive()
        );
        let (sender, receiver) = channel();
        assert!(sender.send(Ok(SpreadsheetWorkerResponse::Stopped)).is_ok());
        let mut completed = process(receiver, Duration::from_secs(1), completed_workspace);
        assert_eq!(Ok(SpreadsheetWorkerResponse::Stopped), completed.receive());
        assert_eq!(Ok(()), completed.send(&SpreadsheetWorkerRequest::Shutdown));
    }
}

#[cfg(target_os = "macos")]
#[test]
fn disconnected_worker_reports_memory_limit_when_monitor_killed_it() {
    let child = std::process::Command::new("/bin/sleep").arg("5").spawn();
    let workspace = tempfile::tempdir();
    assert!(child.is_ok() && workspace.is_ok());
    if let (Ok(child), Ok(workspace)) = (child, workspace) {
        let monitor = MacOsMemoryMonitor::start(child.id(), 0);
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while !monitor.exceeded() && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        let (sender, receiver) = channel();
        drop(sender);
        let mut worker = process(receiver, Duration::from_secs(1), workspace);
        worker.owner = SpreadsheetProcessOwner { child: Some(child) };
        worker.memory_monitor = Some(monitor);
        worker.max_memory_bytes = 0;
        assert!(matches!(
            worker.receive(),
            Err(OfficeWorkerError::WorkerMemoryLimitExceeded { limit: 0 })
        ));
    }
}
