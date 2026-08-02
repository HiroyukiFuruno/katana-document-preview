use katana_document_viewer::browser_session::{
    BrowserSessionAdapter, BrowserSessionRequest, BrowserSessionUpdate, HtmlBrowserInput,
    HtmlBrowserNavigation, HtmlBrowserSource, HtmlBrowserViewport,
};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

const UPDATE_TIMEOUT: Duration = Duration::from_secs(1);
const INITIAL_FRAME_TIMEOUT: Duration = Duration::from_secs(10);
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn public_adapter_forwards_in_process_runtime_commands() -> TestResult {
    let mut adapter = BrowserSessionAdapter::start(request("<p>Initial</p>")?);

    wait_for_frame(&adapter, INITIAL_FRAME_TIMEOUT)?;
    assert_idle(&adapter)?;
    adapter.resize(HtmlBrowserViewport::new(480, 160, 1.0)?)?;
    wait_for_frame(&adapter, UPDATE_TIMEOUT)?;
    assert_idle(&adapter)?;
    adapter.refresh_frame()?;
    wait_for_frame(&adapter, UPDATE_TIMEOUT)?;
    assert_idle(&adapter)?;
    adapter.navigate(HtmlBrowserNavigation::new(HtmlBrowserSource::new(
        "<p>Next</p>",
        "https://example.test/next.html",
    )?)?)?;
    wait_for_frame(&adapter, UPDATE_TIMEOUT)?;
    assert_idle(&adapter)?;
    adapter.close()?;
    Ok(())
}

#[test]
fn adapter_preserves_the_frame_after_a_non_fatal_lifecycle_listener_error() -> TestResult {
    let mut adapter = BrowserSessionAdapter::start(request(
        "<script>document.addEventListener('DOMContentLoaded', () => { throw new Error('lifecycle failed'); });</script>",
    )?);

    wait_for_frame(&adapter, INITIAL_FRAME_TIMEOUT)?;
    assert_idle(&adapter)?;
    while let Some(update) = adapter.take_update() {
        if let BrowserSessionUpdate::Error(error) = update {
            return Err(format!("non-fatal KRR lifecycle error leaked into KDV: {error}").into());
        }
    }
    adapter.close()?;
    Ok(())
}

#[test]
fn burst_continuous_input_preserves_discrete_input_and_frame_progress() -> TestResult {
    let mut adapter = BrowserSessionAdapter::start(request(
        "<a href=linked.html style=\"min-height:80px;padding:8px\">Next</a>",
    )?);
    let initial_generation = wait_for_frame(&adapter, INITIAL_FRAME_TIMEOUT)?;

    for index in 0..4_097 {
        let x = if index % 2 == 0 { 10.0 } else { 310.0 };
        adapter.dispatch_input(HtmlBrowserInput::PointerMove { x, y: 10.0 })?;
    }
    adapter.dispatch_input(HtmlBrowserInput::PointerDown {
        x: 20.0,
        y: 20.0,
        button: 0,
    })?;
    adapter.dispatch_input(HtmlBrowserInput::PointerUp {
        x: 20.0,
        y: 20.0,
        button: 0,
    })?;

    wait_for_navigation(&adapter, "https://example.test/linked.html")?;
    let mut latest_generation = initial_generation;
    while let Some(update) = adapter.take_update() {
        match update {
            BrowserSessionUpdate::Frame(frame) => latest_generation = frame.generation,
            BrowserSessionUpdate::Error(error) => return Err(error.into()),
            BrowserSessionUpdate::Navigation(_) => {}
        }
    }
    adapter.refresh_frame()?;
    let deadline = Instant::now() + UPDATE_TIMEOUT;
    let refresh_baseline = latest_generation;
    while Instant::now() < deadline && latest_generation <= refresh_baseline {
        match adapter.wait_for_update(Duration::from_millis(20)) {
            Some(BrowserSessionUpdate::Frame(frame)) => latest_generation = frame.generation,
            Some(BrowserSessionUpdate::Error(error)) => return Err(error.into()),
            Some(BrowserSessionUpdate::Navigation(_)) | None => {}
        }
    }
    assert!(
        latest_generation > refresh_baseline,
        "browser frame did not progress after burst input"
    );
    adapter.close()?;
    Ok(())
}

#[test]
fn adapter_boundary_does_not_reintroduce_html_semantics_or_an_external_browser() -> TestResult {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for source_name in [
        "src/browser_session.rs",
        "src/browser_session_command_coalescing.rs",
        "src/browser_session_command_queue.rs",
        "src/browser_session_state.rs",
        "src/browser_session_types.rs",
        "src/browser_session_worker.rs",
        "src/browser_session_worker_startup.rs",
    ] {
        let source = fs::read_to_string(crate_root.join(source_name))?;
        for forbidden in [
            "html5ever",
            "markup5ever",
            "cssparser",
            "v8::",
            "HtmlParser",
            "HtmlRenderer",
            "HtmlBrowserProcess",
            "headless_chrome",
            "Chromium",
            "WebView",
        ] {
            assert!(
                !source.contains(forbidden),
                "KDV browser-session adapter must not own {forbidden}: {source_name}"
            );
        }
    }
    Ok(())
}

fn request(html: &str) -> Result<BrowserSessionRequest, Box<dyn std::error::Error>> {
    Ok(BrowserSessionRequest::new(
        HtmlBrowserSource::new(html, "https://example.test/index.html")?,
        HtmlBrowserViewport::new(320, 240, 1.0)?,
    ))
}

fn assert_idle(adapter: &BrowserSessionAdapter) -> TestResult {
    let deadline = Instant::now() + UPDATE_TIMEOUT;
    while Instant::now() < deadline {
        if adapter.is_idle() {
            return Ok(());
        }
        std::thread::yield_now();
    }
    Err("browser adapter did not become idle".into())
}

fn wait_for_frame(
    adapter: &BrowserSessionAdapter,
    timeout: Duration,
) -> Result<u64, Box<dyn std::error::Error>> {
    let deadline = Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(format!("browser frame did not arrive within {timeout:?}").into());
        }
        match adapter.wait_for_update(remaining) {
            Some(BrowserSessionUpdate::Frame(frame)) if !frame.pixels.is_empty() => {
                return Ok(frame.generation);
            }
            Some(BrowserSessionUpdate::Error(error)) => return Err(error.into()),
            Some(BrowserSessionUpdate::Frame(_)) | Some(BrowserSessionUpdate::Navigation(_)) => {}
            None => {
                return Err(format!("browser frame did not arrive within {timeout:?}").into());
            }
        }
    }
}

fn wait_for_navigation(adapter: &BrowserSessionAdapter, expected: &str) -> TestResult {
    let deadline = Instant::now() + UPDATE_TIMEOUT;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match adapter.wait_for_update(remaining) {
            Some(BrowserSessionUpdate::Navigation(navigation))
                if navigation.url.as_str() == expected =>
            {
                return Ok(());
            }
            Some(BrowserSessionUpdate::Error(error)) => return Err(error.into()),
            Some(BrowserSessionUpdate::Frame(_)) | Some(BrowserSessionUpdate::Navigation(_)) => {}
            None => return Err(format!("expected navigation to {expected}").into()),
        }
    }
}
