use super::BrowserSessionState;
use crate::browser_session::BrowserSessionUpdate;
use katana_render_runtime::{
    HtmlBrowserFrame, HtmlBrowserNavigationEvent, HtmlBrowserOrigin, HtmlBrowserPixelFormat,
    HtmlBrowserViewport,
};
use std::{panic, sync::Arc, time::Duration};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn latest_frame_replaces_older_frame() -> TestResult {
    let state = BrowserSessionState::default();
    state.publish(BrowserSessionUpdate::Frame(frame(1)?));
    state.publish(BrowserSessionUpdate::Frame(frame(2)?));

    assert!(matches!(
        state.take_update(),
        Some(BrowserSessionUpdate::Frame(frame)) if frame.generation == 2
    ));
    assert!(state.take_update().is_none());
    Ok(())
}

#[test]
fn navigation_precedes_coalesced_frame() -> TestResult {
    let state = BrowserSessionState::default();
    state.publish(BrowserSessionUpdate::Frame(frame(1)?));
    state.publish(BrowserSessionUpdate::Navigation(
        HtmlBrowserNavigationEvent::new("https://example.test/next.html")?,
    ));

    assert!(matches!(
        state.take_update(),
        Some(BrowserSessionUpdate::Navigation(_))
    ));
    assert!(matches!(
        state.take_update(),
        Some(BrowserSessionUpdate::Frame(frame)) if frame.generation == 1
    ));
    Ok(())
}

#[test]
fn wait_returns_none_immediately_for_zero_timeout_without_update() -> TestResult {
    assert!(
        BrowserSessionState::default()
            .wait_for_update(Duration::ZERO)
            .is_none()
    );
    Ok(())
}

#[test]
fn wait_returns_none_after_timeout_without_update() -> TestResult {
    assert!(
        BrowserSessionState::default()
            .wait_for_update(Duration::from_millis(1))
            .is_none()
    );
    Ok(())
}

#[test]
fn wait_returns_queued_update_without_blocking() -> TestResult {
    let state = BrowserSessionState::default();
    state.publish(BrowserSessionUpdate::Frame(frame(4)?));

    assert!(matches!(
        state.wait_for_update(Duration::from_millis(1)),
        Some(BrowserSessionUpdate::Frame(frame)) if frame.generation == 4
    ));
    Ok(())
}

#[test]
fn poisoned_state_keeps_existing_update_available() -> TestResult {
    let state = Arc::new(BrowserSessionState::default());
    let poisoned = Arc::clone(&state);
    let _ = panic::catch_unwind(move || {
        let _guard = match poisoned.updates.lock() {
            Ok(guard) => guard,
            Err(error) => error.into_inner(),
        };
        panic::resume_unwind(Box::new("poison lock"));
    });
    state.publish(BrowserSessionUpdate::Frame(frame(3)?));

    assert!(matches!(
        state.take_update(),
        Some(BrowserSessionUpdate::Frame(frame)) if frame.generation == 3
    ));
    Ok(())
}

#[test]
fn poisoned_state_times_out_without_an_update() -> TestResult {
    let state = Arc::new(BrowserSessionState::default());
    let poisoned = Arc::clone(&state);
    let _ = panic::catch_unwind(move || {
        let _guard = match poisoned.updates.lock() {
            Ok(guard) => guard,
            Err(error) => error.into_inner(),
        };
        panic::resume_unwind(Box::new("poison lock"));
    });

    assert!(state.wait_for_update(Duration::from_millis(1)).is_none());
    Ok(())
}

fn frame(generation: u64) -> Result<HtmlBrowserFrame, katana_render_runtime::HtmlBrowserError> {
    let origin = HtmlBrowserOrigin::parse("https://example.test/index.html")?;
    let viewport = HtmlBrowserViewport::new(1, 1, 1.0)?;
    HtmlBrowserFrame::new(
        generation,
        origin,
        viewport,
        HtmlBrowserPixelFormat::Rgba8,
        vec![0, 0, 0, 255],
    )
}
