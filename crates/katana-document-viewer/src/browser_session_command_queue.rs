use super::BrowserSessionAdapterError;
use super::browser_session_types::BrowserSessionCommand;
use katana_render_runtime::HtmlBrowserInput;
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CoalescingKey {
    PointerMove,
    Scroll,
    Resize,
    Refresh,
}

impl BrowserSessionCommandQueue {
    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new(CommandQueueState {
                pending: VecDeque::new(),
                accepting: true,
                worker_running: true,
            }),
            command_ready: Condvar::new(),
        }
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
        state.pending.pop_front()
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
        state.pending.clear();
        self.command_ready.notify_all();
    }

    fn lock_state(&self) -> MutexGuard<'_, CommandQueueState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn enqueue_command(pending: &mut VecDeque<BrowserSessionCommand>, command: BrowserSessionCommand) {
    let Some(next_key) = coalescing_key(&command) else {
        pending.push_back(command);
        return;
    };
    let current = pending
        .iter_mut()
        .rev()
        .take_while(|current| coalescing_key(current).is_some())
        .find(|current| coalescing_key(current) == Some(next_key));
    let Some(current) = current else {
        pending.push_back(command);
        return;
    };
    let merged = merge_command(current, command);
    debug_assert!(merged.is_ok(), "matching mailbox commands must merge");
}

fn coalescing_key(command: &BrowserSessionCommand) -> Option<CoalescingKey> {
    match command {
        BrowserSessionCommand::Input(HtmlBrowserInput::PointerMove { .. }) => {
            Some(CoalescingKey::PointerMove)
        }
        BrowserSessionCommand::Input(HtmlBrowserInput::Scroll { .. }) => {
            Some(CoalescingKey::Scroll)
        }
        BrowserSessionCommand::Resize(_) => Some(CoalescingKey::Resize),
        BrowserSessionCommand::Refresh => Some(CoalescingKey::Refresh),
        BrowserSessionCommand::Input(_)
        | BrowserSessionCommand::Navigate(_)
        | BrowserSessionCommand::Close => None,
    }
}

fn merge_command(
    command: &mut BrowserSessionCommand,
    next: BrowserSessionCommand,
) -> Result<(), BrowserSessionCommand> {
    match (command, next) {
        (BrowserSessionCommand::Input(command), BrowserSessionCommand::Input(next)) => {
            merge_input(command, next).map_err(BrowserSessionCommand::Input)
        }
        (command @ BrowserSessionCommand::Resize(_), BrowserSessionCommand::Resize(viewport)) => {
            *command = BrowserSessionCommand::Resize(viewport);
            Ok(())
        }
        (BrowserSessionCommand::Refresh, BrowserSessionCommand::Refresh) => Ok(()),
        (_, next) => Err(next),
    }
}

fn merge_input(
    command: &mut HtmlBrowserInput,
    next: HtmlBrowserInput,
) -> Result<(), HtmlBrowserInput> {
    match (command, next) {
        (
            HtmlBrowserInput::Scroll { delta_x, delta_y },
            HtmlBrowserInput::Scroll {
                delta_x: next_x,
                delta_y: next_y,
            },
        ) => {
            *delta_x += next_x;
            *delta_y += next_y;
            Ok(())
        }
        (
            HtmlBrowserInput::PointerMove { x, y },
            HtmlBrowserInput::PointerMove {
                x: next_x,
                y: next_y,
            },
        ) => {
            *x = next_x;
            *y = next_y;
            Ok(())
        }
        (_, next) => Err(next),
    }
}

#[cfg(test)]
#[path = "browser_session_command_queue_burst_tests.rs"]
mod burst_tests;

#[cfg(test)]
#[path = "browser_session_worker_coalescing_tests.rs"]
mod tests;
