use crate::browser_session::{
    BrowserSessionUpdate, browser_session_runtime::start_session,
    browser_session_state::BrowserSessionState, browser_session_types::BrowserSessionRequest,
};
use katana_render_runtime::HtmlBrowserSession;
use std::time::Instant;

pub(super) fn open_session(
    request: &BrowserSessionRequest,
    state: &BrowserSessionState,
    adapter_started_at: Instant,
) -> (Option<HtmlBrowserSession>, u128, u128) {
    let runtime_open_started_at = Instant::now();
    let worker_queue_ms = runtime_open_started_at
        .saturating_duration_since(adapter_started_at)
        .as_millis();
    let session = start_session(request)
        .map_err(|error| state.publish(BrowserSessionUpdate::Error(error)))
        .ok();
    (
        session,
        worker_queue_ms,
        runtime_open_started_at.elapsed().as_millis(),
    )
}
