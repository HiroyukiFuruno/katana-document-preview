use super::BrowserSessionCommandQueue;
use crate::browser_session::browser_session_types::BrowserSessionCommand;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn idle_requires_startup_and_in_flight_commands_to_complete() -> TestResult {
    let commands = BrowserSessionCommandQueue::new();
    assert!(!commands.is_idle());

    commands.mark_worker_ready();
    assert!(commands.is_idle());

    commands.enqueue(BrowserSessionCommand::Refresh)?;
    assert!(!commands.is_idle());
    assert!(matches!(
        commands.receive(),
        Some(BrowserSessionCommand::Refresh)
    ));
    assert!(!commands.is_idle());

    commands.complete_command();
    assert!(commands.is_idle());

    commands.mark_worker_stopped();
    assert!(!commands.is_idle());
    Ok(())
}
