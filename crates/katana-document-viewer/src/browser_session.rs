//! Worker-backed adapter for the KRR browser session.

#[path = "browser_session_command_queue.rs"]
mod browser_session_command_queue;
#[path = "browser_session_runtime.rs"]
mod browser_session_runtime;
#[path = "browser_session_state.rs"]
mod browser_session_state;
#[path = "browser_session_types.rs"]
mod browser_session_types;
#[path = "browser_session_worker.rs"]
mod browser_session_worker;

use browser_session_command_queue::BrowserSessionCommandQueue;
use browser_session_state::BrowserSessionState;
use browser_session_types::BrowserSessionCommand;
pub use browser_session_types::{
    BrowserSessionAdapterError, BrowserSessionOperation, BrowserSessionRequest,
    BrowserSessionUpdate,
};
pub use katana_render_runtime::{
    HtmlBrowserInput, HtmlBrowserNavigation, HtmlBrowserSource, HtmlBrowserViewport,
};
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

/// Non-blocking handle for one KRR-owned persistent browser page.
#[derive(Debug)]
pub struct BrowserSessionAdapter {
    commands: Arc<BrowserSessionCommandQueue>,
    state: Arc<BrowserSessionState>,
    worker: Option<JoinHandle<()>>,
}

impl BrowserSessionAdapter {
    pub fn start(request: BrowserSessionRequest) -> Self {
        let adapter_started_at = std::time::Instant::now();
        let commands = Arc::new(BrowserSessionCommandQueue::new());
        let state = Arc::new(BrowserSessionState::default());
        let worker_commands = Arc::clone(&commands);
        let worker_state = Arc::clone(&state);
        let worker = thread::spawn(move || {
            let _worker_lifetime = WorkerLifetime::new(Arc::clone(&worker_commands));
            browser_session_worker::BrowserSessionWorker::run(
                request,
                worker_commands,
                worker_state,
                adapter_started_at,
            );
        });
        Self {
            commands,
            state,
            worker: Some(worker),
        }
    }

    pub fn dispatch_input(
        &self,
        input: katana_render_runtime::HtmlBrowserInput,
    ) -> Result<(), BrowserSessionAdapterError> {
        self.enqueue(BrowserSessionCommand::Input(input))
    }

    pub fn resize(
        &self,
        viewport: katana_render_runtime::HtmlBrowserViewport,
    ) -> Result<(), BrowserSessionAdapterError> {
        self.enqueue(BrowserSessionCommand::Resize(viewport))
    }

    pub fn navigate(
        &self,
        navigation: katana_render_runtime::HtmlBrowserNavigation,
    ) -> Result<(), BrowserSessionAdapterError> {
        self.enqueue(BrowserSessionCommand::Navigate(navigation))
    }

    pub fn refresh_frame(&self) -> Result<(), BrowserSessionAdapterError> {
        self.enqueue(BrowserSessionCommand::Refresh)
    }

    pub fn take_update(&self) -> Option<BrowserSessionUpdate> {
        self.state.take_update()
    }

    pub fn wait_for_update(&self, timeout: Duration) -> Option<BrowserSessionUpdate> {
        self.state.wait_for_update(timeout)
    }

    pub fn close(&mut self) -> Result<(), BrowserSessionAdapterError> {
        let Some(worker) = self.worker.take() else {
            return Ok(());
        };
        let close_result = self.commands.close();
        match worker.join() {
            Ok(()) => close_result,
            Err(_) => Err(BrowserSessionAdapterError::WorkerPanicked),
        }
    }

    fn enqueue(&self, command: BrowserSessionCommand) -> Result<(), BrowserSessionAdapterError> {
        self.commands.enqueue(command)
    }
}

impl Drop for BrowserSessionAdapter {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

struct WorkerLifetime {
    commands: Arc<BrowserSessionCommandQueue>,
}

impl WorkerLifetime {
    fn new(commands: Arc<BrowserSessionCommandQueue>) -> Self {
        Self { commands }
    }
}

impl Drop for WorkerLifetime {
    fn drop(&mut self) {
        self.commands.mark_worker_stopped();
    }
}

#[cfg(test)]
#[path = "browser_session_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "browser_session_adapter_tests.rs"]
mod adapter_tests;
