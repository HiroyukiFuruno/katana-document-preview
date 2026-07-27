use super::BrowserSessionCommandQueue;
use crate::browser_session::browser_session_types::BrowserSessionCommand;
use katana_render_runtime::{HtmlBrowserInput, HtmlBrowserViewport};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn interleaved_continuous_burst_is_bounded_by_a_discrete_barrier() -> TestResult {
    let commands = BrowserSessionCommandQueue::new();
    enqueue_interleaved_burst(&commands)?;
    commands.enqueue(BrowserSessionCommand::Input(HtmlBrowserInput::Focus {
        focused: true,
    }))?;
    commands.enqueue(pointer_move(9_000.0, 10.0))?;

    assert_eq!(pending_len(&commands), 6);
    assert_pointer_move(commands.receive(), 4_096.0, 10.0);
    assert_scroll(commands.receive(), 4_097.0, -4_097.0);
    assert_resize(commands.receive(), 4_736);
    assert!(matches!(
        commands.receive(),
        Some(BrowserSessionCommand::Refresh)
    ));
    assert!(matches!(
        commands.receive(),
        Some(BrowserSessionCommand::Input(HtmlBrowserInput::Focus {
            focused: true
        }))
    ));
    assert_pointer_move(commands.receive(), 9_000.0, 10.0);
    Ok(())
}

fn enqueue_interleaved_burst(commands: &BrowserSessionCommandQueue) -> TestResult {
    for index in 0..4_097 {
        commands.enqueue(pointer_move(index as f32, 10.0))?;
        commands.enqueue(scroll(1.0, -1.0))?;
        commands.enqueue(BrowserSessionCommand::Resize(HtmlBrowserViewport::new(
            640 + index,
            480,
            1.0,
        )?))?;
        commands.enqueue(BrowserSessionCommand::Refresh)?;
    }
    Ok(())
}

fn pointer_move(x: f32, y: f32) -> BrowserSessionCommand {
    BrowserSessionCommand::Input(HtmlBrowserInput::PointerMove { x, y })
}

fn scroll(delta_x: f32, delta_y: f32) -> BrowserSessionCommand {
    BrowserSessionCommand::Input(HtmlBrowserInput::Scroll { delta_x, delta_y })
}

fn pending_len(commands: &BrowserSessionCommandQueue) -> usize {
    commands.lock_state().pending.len()
}

fn assert_pointer_move(command: Option<BrowserSessionCommand>, expected_x: f32, expected_y: f32) {
    assert!(matches!(
        command,
        Some(BrowserSessionCommand::Input(HtmlBrowserInput::PointerMove { x, y }))
            if (x, y) == (expected_x, expected_y)
    ));
}

fn assert_scroll(command: Option<BrowserSessionCommand>, expected_x: f32, expected_y: f32) {
    assert!(matches!(
        command,
        Some(BrowserSessionCommand::Input(HtmlBrowserInput::Scroll { delta_x, delta_y }))
            if (delta_x, delta_y) == (expected_x, expected_y)
    ));
}

fn assert_resize(command: Option<BrowserSessionCommand>, expected_width: u32) {
    assert!(matches!(
        command,
        Some(BrowserSessionCommand::Resize(viewport))
            if viewport.width == expected_width && viewport.height == 480
    ));
}
