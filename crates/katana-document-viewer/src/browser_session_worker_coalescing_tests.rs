use super::{BrowserSessionCommandQueue, merge_command};
use crate::browser_session::browser_session_types::BrowserSessionCommand;
use katana_render_runtime::{
    HtmlBrowserInput, HtmlBrowserNavigation, HtmlBrowserSource, HtmlBrowserViewport,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn adjacent_scroll_commands_are_coalesced_without_reordering_pointer_input() -> TestResult {
    let commands = BrowserSessionCommandQueue::new();
    commands.enqueue(scroll(2.0, 3.0))?;
    commands.enqueue(scroll(5.0, 7.0))?;
    commands.enqueue(BrowserSessionCommand::Input(
        HtmlBrowserInput::PointerDown {
            x: 10.0,
            y: 20.0,
            button: 0,
        },
    ))?;

    assert_scroll(
        commands.receive().ok_or("missing scroll command")?,
        7.0,
        10.0,
    )?;
    assert!(matches!(
        commands.receive(),
        Some(BrowserSessionCommand::Input(
            HtmlBrowserInput::PointerDown { .. }
        ))
    ));
    Ok(())
}

#[test]
fn adjacent_resize_commands_keep_only_the_latest_viewport() -> TestResult {
    let initial = HtmlBrowserViewport::new(320, 240, 1.0)?;
    let mut command = BrowserSessionCommand::Resize(initial);
    let latest = HtmlBrowserViewport::new(640, 480, 2.0)?;

    merge_command(&mut command, BrowserSessionCommand::Resize(latest))
        .map_err(|_| "resize commands did not merge")?;

    assert!(matches!(command, BrowserSessionCommand::Resize(value) if value == latest));
    Ok(())
}

#[test]
fn different_command_kinds_are_not_merged() -> TestResult {
    let viewport = HtmlBrowserViewport::new(640, 480, 1.0)?;
    let mut refresh = BrowserSessionCommand::Refresh;
    let rejected = merge_command(&mut refresh, BrowserSessionCommand::Resize(viewport));
    assert!(matches!(rejected, Err(BrowserSessionCommand::Resize(value)) if value == viewport));

    let mut scroll = scroll(1.0, 2.0);
    let pointer = pointer_move(3.0, 4.0);
    let rejected = merge_command(&mut scroll, pointer);
    assert!(matches!(
        rejected,
        Err(BrowserSessionCommand::Input(
            HtmlBrowserInput::PointerMove { x: 3.0, y: 4.0 }
        ))
    ));
    Ok(())
}

#[test]
fn adjacent_pointer_moves_and_refreshes_keep_only_the_latest_state() -> TestResult {
    let commands = BrowserSessionCommandQueue::new();
    commands.enqueue(pointer_move(1.0, 2.0))?;
    commands.enqueue(pointer_move(3.0, 4.0))?;
    commands.enqueue(BrowserSessionCommand::Refresh)?;
    commands.enqueue(BrowserSessionCommand::Refresh)?;

    assert!(matches!(
        commands.receive(),
        Some(BrowserSessionCommand::Input(
            HtmlBrowserInput::PointerMove { x: 3.0, y: 4.0 }
        ))
    ));
    assert!(matches!(
        commands.receive(),
        Some(BrowserSessionCommand::Refresh)
    ));
    Ok(())
}

#[test]
fn discrete_input_and_navigation_keep_their_exact_order() -> TestResult {
    let commands = BrowserSessionCommandQueue::new();
    let inputs = discrete_inputs();
    for input in inputs.clone() {
        commands.enqueue(BrowserSessionCommand::Input(input))?;
    }
    let navigation = navigation()?;
    commands.enqueue(BrowserSessionCommand::Navigate(navigation.clone()))?;

    for expected in inputs {
        assert_input(commands.receive(), expected)?;
    }
    assert!(matches!(
        commands.receive(),
        Some(BrowserSessionCommand::Navigate(actual)) if actual == navigation
    ));
    Ok(())
}

fn discrete_inputs() -> [HtmlBrowserInput; 6] {
    [
        HtmlBrowserInput::Focus { focused: true },
        HtmlBrowserInput::KeyDown {
            key: "Enter".into(),
        },
        HtmlBrowserInput::Text {
            text: "Katana".into(),
        },
        HtmlBrowserInput::KeyUp {
            key: "Enter".into(),
        },
        HtmlBrowserInput::PointerDown {
            x: 10.0,
            y: 20.0,
            button: 0,
        },
        HtmlBrowserInput::PointerUp {
            x: 10.0,
            y: 20.0,
            button: 0,
        },
    ]
}

fn navigation() -> Result<HtmlBrowserNavigation, katana_render_runtime::HtmlBrowserError> {
    HtmlBrowserNavigation::new(HtmlBrowserSource::new(
        "<p>Next</p>",
        "https://example.test/next.html",
    )?)
}

#[test]
fn close_discards_pending_work_and_stops_new_commands() -> TestResult {
    let commands = BrowserSessionCommandQueue::new();
    commands.enqueue(scroll(1.0, 2.0))?;

    commands.close()?;

    assert!(matches!(
        commands.receive(),
        Some(BrowserSessionCommand::Close)
    ));
    assert_eq!(
        commands.enqueue(BrowserSessionCommand::Refresh),
        Err(crate::browser_session::BrowserSessionAdapterError::WorkerStopped)
    );
    Ok(())
}

#[test]
fn stopped_worker_unblocks_a_waiting_receiver() -> TestResult {
    let commands = std::sync::Arc::new(BrowserSessionCommandQueue::new());
    let receiver_commands = std::sync::Arc::clone(&commands);
    let receiver = std::thread::spawn(move || receiver_commands.receive());

    commands.mark_worker_stopped();

    assert!(receiver.join().map_err(|_| "receiver panicked")?.is_none());
    Ok(())
}

fn scroll(delta_x: f32, delta_y: f32) -> BrowserSessionCommand {
    BrowserSessionCommand::Input(HtmlBrowserInput::Scroll { delta_x, delta_y })
}

fn pointer_move(x: f32, y: f32) -> BrowserSessionCommand {
    BrowserSessionCommand::Input(HtmlBrowserInput::PointerMove { x, y })
}

fn assert_scroll(command: BrowserSessionCommand, expected_x: f32, expected_y: f32) -> TestResult {
    match command {
        BrowserSessionCommand::Input(HtmlBrowserInput::Scroll { delta_x, delta_y }) => {
            assert_eq!((delta_x, delta_y), (expected_x, expected_y));
            Ok(())
        }
        _ => Err("expected a coalesced scroll command".into()),
    }
}

fn assert_input(command: Option<BrowserSessionCommand>, expected: HtmlBrowserInput) -> TestResult {
    match command {
        Some(BrowserSessionCommand::Input(actual)) => {
            assert_eq!(actual, expected);
            Ok(())
        }
        _ => Err("expected a discrete input command".into()),
    }
}
