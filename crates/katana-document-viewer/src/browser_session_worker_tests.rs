use crate::browser_session::{
    BrowserSessionAdapter, BrowserSessionUpdate,
    browser_session_runtime::{dispatch, publish_updates, start_session},
    browser_session_state::BrowserSessionState,
    browser_session_types::{BrowserSessionCommand, BrowserSessionRequest},
};
use katana_render_runtime::{
    HtmlBrowserInput, HtmlBrowserNavigation, HtmlBrowserSession, HtmlBrowserSource,
    HtmlBrowserViewport,
};
use std::time::Duration;

const UPDATE_TIMEOUT: Duration = Duration::from_secs(10);
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn dispatch_handles_every_browser_command() -> TestResult {
    let mut session = start_session(&request("index")?)?;

    dispatch(
        &mut session,
        BrowserSessionCommand::Input(HtmlBrowserInput::Focus { focused: true }),
    )?;
    dispatch(
        &mut session,
        BrowserSessionCommand::Navigate(HtmlBrowserNavigation::new(source("next")?)?),
    )?;
    dispatch(&mut session, BrowserSessionCommand::Resize(viewport()?))?;
    dispatch(&mut session, BrowserSessionCommand::Refresh)?;
    dispatch(&mut session, BrowserSessionCommand::Close)?;
    session.close()?;
    Ok(())
}

#[test]
fn dispatch_propagates_errors_after_the_runtime_session_is_closed() -> TestResult {
    let mut session = start_session(&request("index")?)?;
    session.close()?;

    assert!(
        dispatch(
            &mut session,
            BrowserSessionCommand::Input(HtmlBrowserInput::Focus { focused: true }),
        )
        .is_err()
    );
    assert!(dispatch(&mut session, BrowserSessionCommand::Resize(viewport()?),).is_err());
    assert!(
        dispatch(
            &mut session,
            BrowserSessionCommand::Navigate(HtmlBrowserNavigation::new(source("next")?)?),
        )
        .is_err()
    );
    assert!(dispatch(&mut session, BrowserSessionCommand::Refresh).is_err());
    Ok(())
}

#[test]
fn worker_recovers_from_startup_error_on_navigation() -> TestResult {
    let mut adapter = BrowserSessionAdapter::start(BrowserSessionRequest::new(
        source("index")?,
        invalid_viewport(),
    ));

    assert_operation_error(adapter.wait_for_update(UPDATE_TIMEOUT), "start", "index")?;

    adapter.refresh_frame()?;
    adapter.navigate(HtmlBrowserNavigation::new(source("still-invalid")?)?)?;
    assert_operation_error(
        adapter.wait_for_update(UPDATE_TIMEOUT),
        "start",
        "still-invalid",
    )?;

    adapter.resize(invalid_viewport())?;
    assert_operation_error(
        adapter.wait_for_update(UPDATE_TIMEOUT),
        "resize",
        "still-invalid",
    )?;

    adapter.resize(viewport()?)?;
    adapter.navigate(HtmlBrowserNavigation::new(source("recovered")?)?)?;
    assert_frame(adapter.wait_for_update(UPDATE_TIMEOUT))?;
    adapter.close()?;
    Ok(())
}

#[test]
fn publishing_without_pending_browser_updates_is_a_noop() -> TestResult {
    let mut session = HtmlBrowserSession::new(source("index")?, viewport()?)?;
    let state = BrowserSessionState::default();
    let _ = session.take_frame_update();

    publish_updates(&mut session, &state);

    assert!(state.take_update().is_none());
    Ok(())
}

#[test]
fn worker_publishes_invalid_resize_errors() -> TestResult {
    let mut adapter = BrowserSessionAdapter::start(request("index")?);
    assert_frame(adapter.wait_for_update(UPDATE_TIMEOUT))?;
    adapter.resize(HtmlBrowserViewport {
        width: 0,
        height: 1,
        device_scale_factor: 1.0,
    })?;
    let error = assert_error(adapter.wait_for_update(UPDATE_TIMEOUT))?;
    assert!(error.contains("Operation: resize"));
    assert!(error.contains("Document: https://example.test/index.html"));
    assert!(error.contains("Cause: browser viewport dimensions must be non-zero"));
    adapter.close()?;
    Ok(())
}

fn request(page: &str) -> Result<BrowserSessionRequest, katana_render_runtime::HtmlBrowserError> {
    Ok(BrowserSessionRequest::new(source(page)?, viewport()?))
}

fn source(page: &str) -> Result<HtmlBrowserSource, katana_render_runtime::HtmlBrowserError> {
    HtmlBrowserSource::new(
        "<button>Run</button>",
        format!("https://example.test/{page}.html"),
    )
}

fn viewport() -> Result<HtmlBrowserViewport, katana_render_runtime::HtmlBrowserError> {
    HtmlBrowserViewport::new(320, 240, 1.0)
}

fn invalid_viewport() -> HtmlBrowserViewport {
    HtmlBrowserViewport {
        width: 0,
        height: 1,
        device_scale_factor: 1.0,
    }
}

fn assert_frame(update: Option<BrowserSessionUpdate>) -> TestResult {
    match update {
        Some(BrowserSessionUpdate::Frame(frame)) if !frame.pixels.is_empty() => Ok(()),
        _ => Err("expected initial browser frame".into()),
    }
}

fn assert_error(
    update: Option<BrowserSessionUpdate>,
) -> Result<String, Box<dyn std::error::Error>> {
    match update {
        Some(BrowserSessionUpdate::Error(error)) => Ok(error.to_string()),
        _ => Err("expected browser error update".into()),
    }
}

fn assert_operation_error(
    update: Option<BrowserSessionUpdate>,
    operation: &str,
    page: &str,
) -> TestResult {
    let error = assert_error(update)?;
    assert!(error.contains("Layer: KRR runtime"));
    assert!(error.contains(&format!("Operation: {operation}")));
    assert!(error.contains(&format!("Document: https://example.test/{page}.html")));
    assert!(error.contains("Cause: browser viewport dimensions must be non-zero"));
    Ok(())
}
