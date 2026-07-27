use super::BrowserSessionAdapterError;
use super::browser_session_types::BrowserSessionCommand;
#[path = "browser_session_command_coalescing.rs"]
mod browser_session_command_coalescing;
use browser_session_command_coalescing::enqueue_command;
#[cfg(test)]
use browser_session_command_coalescing::merge_command;
use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex, MutexGuard},
};

#[derive(Debug)]
pub(super) struct BrowserSessionCommandQueue {
    state: Mutex<CommandQueueState>,
    command_ready: Condvar,
}

#[derive(Debug)]
struct CommandQueueState {
    pending: VecDeque<BrowserSessionCommand>,
    accepting: bool,
    worker_running: bool,
    worker_busy: bool,
}

impl BrowserSessionCommandQueue {
    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new(CommandQueueState {
                pending: VecDeque::new(),
                accepting: true,
                worker_running: true,
                worker_busy: true,
            }),
            command_ready: Condvar::new(),
        }
    }

    pub(super) fn is_idle(&self) -> bool {
        let state = self.lock_state();
        state.worker_running && !state.worker_busy && state.pending.is_empty()
    }

    pub(super) fn enqueue(
        &self,
        command: BrowserSessionCommand,
    ) -> Result<(), BrowserSessionAdapterError> {
        let mut state = self.lock_state();
        if !state.accepting || !state.worker_running {
            return Err(BrowserSessionAdapterError::WorkerStopped);
        }
        enqueue_command(&mut state.pending, command);
        self.command_ready.notify_one();
        Ok(())
    }

    pub(super) fn receive(&self) -> Option<BrowserSessionCommand> {
        let mut state = self.lock_state();
        while state.pending.is_empty() && state.worker_running {
            state = self
                .command_ready
                .wait(state)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        let command = state.pending.pop_front();
        if command.is_some() {
            state.worker_busy = true;
        }
        command
    }

    pub(super) fn mark_worker_ready(&self) {
        let mut state = self.lock_state();
        if state.worker_running {
            state.worker_busy = false;
        }
        self.command_ready.notify_all();
    }

    pub(super) fn complete_command(&self) {
        let mut state = self.lock_state();
        state.worker_busy = false;
        self.command_ready.notify_all();
    }

    pub(super) fn close(&self) -> Result<(), BrowserSessionAdapterError> {
        let mut state = self.lock_state();
        if !state.worker_running {
            return Err(BrowserSessionAdapterError::WorkerStopped);
        }
        state.accepting = false;
        state.pending.clear();
        state.pending.push_back(BrowserSessionCommand::Close);
        self.command_ready.notify_one();
        Ok(())
    }

    pub(super) fn mark_worker_stopped(&self) {
        let mut state = self.lock_state();
        state.accepting = false;
        state.worker_running = false;
        state.worker_busy = false;
        state.pending.clear();
        self.command_ready.notify_all();
    }

    fn lock_state(&self) -> MutexGuard<'_, CommandQueueState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
#[path = "browser_session_command_queue_burst_tests.rs"]
mod burst_tests;

#[cfg(test)]
#[path = "browser_session_command_queue_idle_tests.rs"]
mod idle_tests;

#[cfg(test)]
#[path = "browser_session_worker_coalescing_tests.rs"]
mod tests;
