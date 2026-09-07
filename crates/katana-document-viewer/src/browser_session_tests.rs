use super::{BrowserSessionAdapter, BrowserSessionRequest, BrowserSessionUpdate};
use katana_render_runtime::{
    HtmlBrowserInput, HtmlBrowserNavigation, HtmlBrowserSource, HtmlBrowserViewport,
};
use std::time::{Duration, Instant};

const UPDATE_TIMEOUT: Duration = Duration::from_secs(10);
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn worker_returns_initial_and_refresh_frames() -> TestResult {
    let mut adapter = BrowserSessionAdapter::start(request("<button>Run</button>")?);

    assert_frame(adapter.wait_for_update(UPDATE_TIMEOUT))?;
    adapter.refresh_frame()?;
    assert_frame(adapter.wait_for_update(UPDATE_TIMEOUT))?;
    adapter.close()?;
    Ok(())
}

#[test]
fn worker_forwards_resize_and_explicit_navigation() -> TestResult {
    let mut adapter = BrowserSessionAdapter::start(request("<p>Initial</p>")?);

    assert_frame(adapter.wait_for_update(UPDATE_TIMEOUT))?;
    adapter.resize(HtmlBrowserViewport::new(480, 160, 1.0)?)?;
    assert_frame(adapter.wait_for_update(UPDATE_TIMEOUT))?;
    adapter.navigate(HtmlBrowserNavigation::new(HtmlBrowserSource::new(
        "<p>Next</p>",
        "https://example.test/next.html",
    )?)?)?;
    assert_frame(adapter.wait_for_update(UPDATE_TIMEOUT))?;
    adapter.close()?;
    Ok(())
}

#[test]
fn worker_forwards_input_and_publishes_runtime_link_navigation() -> TestResult {
    let mut adapter = BrowserSessionAdapter::start(request(
        "<a href=linked.html style=\"min-height: 80px; padding: 8px\">Next</a>",
    )?);

    assert_frame(adapter.wait_for_update(UPDATE_TIMEOUT))?;
    for input in [
        HtmlBrowserInput::PointerDown {
            x: 20.0,
            y: 20.0,
            button: 0,
        },
        HtmlBrowserInput::PointerUp {
            x: 20.0,
            y: 20.0,
            button: 0,
        },
    ] {
        adapter.dispatch_input(input)?;
    }

    assert_navigation(&adapter, "https://example.test/linked.html", UPDATE_TIMEOUT)?;
    adapter.close()?;
    Ok(())
}

fn request(html: &str) -> Result<BrowserSessionRequest, katana_render_runtime::HtmlBrowserError> {
    Ok(BrowserSessionRequest::new(source(html)?, viewport()?))
}

fn source(html: &str) -> Result<HtmlBrowserSource, katana_render_runtime::HtmlBrowserError> {
    HtmlBrowserSource::new(html, "https://example.test/index.html")
}

fn viewport() -> Result<HtmlBrowserViewport, katana_render_runtime::HtmlBrowserError> {
    HtmlBrowserViewport::new(320, 240, 1.0)
}

fn assert_frame(update: Option<BrowserSessionUpdate>) -> TestResult {
    match update {
        Some(BrowserSessionUpdate::Frame(frame)) if !frame.pixels.is_empty() => Ok(()),
        _ => Err(format!("expected browser frame, got {update:?}").into()),
    }
}

fn assert_navigation(
    adapter: &BrowserSessionAdapter,
    expected_url: &str,
    timeout: Duration,
) -> TestResult {
    let deadline = Instant::now() + timeout;
    let mut observed = Vec::new();
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let Some(update) = adapter.wait_for_update(remaining) else {
            return Err(format!("expected navigation to {expected_url}, got {observed:?}").into());
        };
        match update {
            BrowserSessionUpdate::Navigation(navigation)
                if navigation.url.as_str() == expected_url =>
            {
                return Ok(());
            }
            update => observed.push(update),
        }
    }
}
