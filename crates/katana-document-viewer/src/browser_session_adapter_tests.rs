use super::{
    BrowserSessionAdapter, BrowserSessionAdapterError, BrowserSessionRequest, BrowserSessionUpdate,
    WorkerLifetime, browser_session_command_queue::BrowserSessionCommandQueue,
    browser_session_state::BrowserSessionState,
};
use katana_render_runtime::{HtmlBrowserSource, HtmlBrowserViewport};
use std::{
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

const UPDATE_TIMEOUT: Duration = Duration::from_secs(1);
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn take_update_delegates_to_adapter_state() -> TestResult {
    let commands = Arc::new(BrowserSessionCommandQueue::new());
    let state = Arc::new(BrowserSessionState::default());
    state.publish(BrowserSessionUpdate::Error(
        BrowserSessionAdapterError::WorkerStopped,
    ));
    let adapter = BrowserSessionAdapter {
        commands,
        state,
        worker: None,
    };

    assert!(matches!(
        adapter.take_update(),
        Some(BrowserSessionUpdate::Error(
            BrowserSessionAdapterError::WorkerStopped
        ))
    ));
    Ok(())
}

#[test]
fn start_publishes_an_in_process_runtime_frame() -> TestResult {
    let mut adapter =
        BrowserSessionAdapter::start(BrowserSessionRequest::new(source()?, viewport()?));

    assert!(matches!(
        adapter.wait_for_update(UPDATE_TIMEOUT),
        Some(BrowserSessionUpdate::Frame(frame)) if !frame.pixels.is_empty()
    ));
    assert!(wait_until_idle(&adapter));
    adapter.close()?;
    Ok(())
}

#[test]
fn close_is_idempotent_after_the_worker_has_stopped() -> TestResult {
    let mut adapter =
        BrowserSessionAdapter::start(BrowserSessionRequest::new(source()?, viewport()?));

    assert!(matches!(
        adapter.wait_for_update(UPDATE_TIMEOUT),
        Some(BrowserSessionUpdate::Frame(frame)) if !frame.pixels.is_empty()
    ));
    adapter.close()?;
    adapter.close()?;
    Ok(())
}

#[test]
fn commands_are_rejected_after_the_worker_stops() -> TestResult {
    let commands = BrowserSessionCommandQueue::new();
    commands.mark_worker_stopped();

    assert_eq!(
        commands.enqueue(super::BrowserSessionCommand::Refresh),
        Err(BrowserSessionAdapterError::WorkerStopped)
    );
    Ok(())
}

#[test]
fn close_reports_worker_panic() -> TestResult {
    let commands = Arc::new(BrowserSessionCommandQueue::new());
    let worker_commands = Arc::clone(&commands);
    let worker = thread::spawn(move || {
        let _worker_lifetime = WorkerLifetime::new(Arc::clone(&worker_commands));
        let _ = worker_commands.receive();
        std::panic::resume_unwind(Box::new("test worker panic"));
    });
    let mut adapter = BrowserSessionAdapter {
        commands,
        state: Default::default(),
        worker: Some(worker),
    };

    assert_eq!(
        adapter.close(),
        Err(BrowserSessionAdapterError::WorkerPanicked)
    );
    Ok(())
}

#[test]
fn close_reports_a_worker_that_stopped_before_receiving_close() -> TestResult {
    let commands = Arc::new(BrowserSessionCommandQueue::new());
    let worker_commands = Arc::clone(&commands);
    let (stopped, stopped_receiver) = mpsc::sync_channel(1);
    let worker = thread::spawn(move || {
        let worker_lifetime = WorkerLifetime::new(worker_commands);
        drop(worker_lifetime);
        let _ = stopped.send(());
    });
    stopped_receiver
        .recv()
        .map_err(|_| std::io::Error::other("worker did not stop"))?;
    let mut adapter = BrowserSessionAdapter {
        commands,
        state: Default::default(),
        worker: Some(worker),
    };

    assert_eq!(
        adapter.close(),
        Err(BrowserSessionAdapterError::WorkerStopped)
    );
    Ok(())
}

fn source() -> Result<HtmlBrowserSource, katana_render_runtime::HtmlBrowserError> {
    HtmlBrowserSource::new("<button>Run</button>", "https://example.test/index.html")
}

fn viewport() -> Result<HtmlBrowserViewport, katana_render_runtime::HtmlBrowserError> {
    HtmlBrowserViewport::new(320, 240, 1.0)
}

fn wait_until_idle(adapter: &BrowserSessionAdapter) -> bool {
    let deadline = std::time::Instant::now() + UPDATE_TIMEOUT;
    while std::time::Instant::now() < deadline {
        if adapter.is_idle() {
            return true;
        }
        thread::yield_now();
    }
    false
}
