use super::BrowserSessionUpdate;
use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex, MutexGuard},
    time::{Duration, Instant},
};

#[derive(Debug, Default)]
pub(crate) struct BrowserSessionState {
    updates: Mutex<PendingUpdates>,
    changed: Condvar,
}

#[derive(Debug, Default)]
struct PendingUpdates {
    latest_frame: Option<BrowserSessionUpdate>,
    events: VecDeque<BrowserSessionUpdate>,
}

impl BrowserSessionState {
    pub(crate) fn publish(&self, update: BrowserSessionUpdate) {
        let mut updates = self.lock_updates();
        match update {
            BrowserSessionUpdate::Frame(_) => updates.latest_frame = Some(update),
            BrowserSessionUpdate::Navigation(_) | BrowserSessionUpdate::Error(_) => {
                updates.events.push_back(update);
            }
        }
        self.changed.notify_all();
    }

    pub(crate) fn take_update(&self) -> Option<BrowserSessionUpdate> {
        self.take_from(&mut self.lock_updates())
    }

    pub(crate) fn wait_for_update(&self, timeout: Duration) -> Option<BrowserSessionUpdate> {
        let deadline = Instant::now() + timeout;
        let mut updates = self.lock_updates();
        while !updates_available(&updates) {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return None;
            }
            let (next, timeout) = self
                .changed
                .wait_timeout(updates, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            updates = next;
            if timeout.timed_out() && !updates_available(&updates) {
                return None;
            }
        }
        self.take_from(&mut updates)
    }

    fn take_from(&self, updates: &mut PendingUpdates) -> Option<BrowserSessionUpdate> {
        updates
            .events
            .pop_front()
            .or_else(|| updates.latest_frame.take())
    }

    fn lock_updates(&self) -> MutexGuard<'_, PendingUpdates> {
        self.updates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn updates_available(updates: &PendingUpdates) -> bool {
    updates.latest_frame.is_some() || !updates.events.is_empty()
}

#[cfg(test)]
#[path = "browser_session_state_tests.rs"]
mod tests;
