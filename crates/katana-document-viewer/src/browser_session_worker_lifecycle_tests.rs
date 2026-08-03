use super::BrowserSessionWorker;
use crate::browser_session::{
    BrowserSessionUpdate, browser_session_command_queue::BrowserSessionCommandQueue,
    browser_session_state::BrowserSessionState, browser_session_types::BrowserSessionRequest,
};
use katana_render_runtime::{HtmlBrowserSource, HtmlBrowserViewport};
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const UPDATE_TIMEOUT: Duration = Duration::from_secs(10);
type TestResult = Result<(), Box<dyn std::error::Error>>;
type RunningWorker = (
    Arc<BrowserSessionCommandQueue>,
    Arc<BrowserSessionState>,
    JoinHandle<()>,
);

#[test]
fn worker_publishes_initial_frame_and_closes_from_the_mailbox() -> TestResult {
    let (commands, state, worker) = spawn_worker()?;

    assert_frame(state.wait_for_update(UPDATE_TIMEOUT))?;
    commands.close()?;
    join_worker(worker)
}

#[test]
fn worker_closes_when_the_mailbox_reports_a_stopped_owner() -> TestResult {
    let (commands, state, worker) = spawn_worker()?;

    assert_frame(state.wait_for_update(UPDATE_TIMEOUT))?;
    commands.mark_worker_stopped();
    join_worker(worker)
}

fn spawn_worker() -> Result<RunningWorker, katana_render_runtime::HtmlBrowserError> {
    let commands = Arc::new(BrowserSessionCommandQueue::new());
    let state = Arc::new(BrowserSessionState::default());
    let worker_commands = Arc::clone(&commands);
    let worker_state = Arc::clone(&state);
    let request = BrowserSessionRequest::new(source()?, viewport()?);
    let worker = thread::spawn(move || {
        BrowserSessionWorker::run(request, worker_commands, worker_state, Instant::now());
    });
    Ok((commands, state, worker))
}

fn source() -> Result<HtmlBrowserSource, katana_render_runtime::HtmlBrowserError> {
    HtmlBrowserSource::new("<button>Run</button>", "https://example.test/index.html")
}

fn viewport() -> Result<HtmlBrowserViewport, katana_render_runtime::HtmlBrowserError> {
    HtmlBrowserViewport::new(320, 240, 1.0)
}

fn assert_frame(update: Option<BrowserSessionUpdate>) -> TestResult {
    match update {
        Some(BrowserSessionUpdate::Frame(frame)) if !frame.pixels.is_empty() => Ok(()),
        _ => Err("expected initial browser frame".into()),
    }
}

fn join_worker(worker: JoinHandle<()>) -> TestResult {
    worker
        .join()
        .map_err(|_| std::io::Error::other("worker panicked"))?;
    Ok(())
}
