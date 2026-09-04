#[cfg(not(windows))]
use super::configure_command_with_debug;
use super::{cpu_seconds, stdin_unavailable, stdout_unavailable};
use crate::multi_format::{OfficeWorkerError, debug_trace::DebugTrace};
use std::time::Duration;

#[test]
fn pipe_failures_and_cpu_floor_are_typed() {
    assert!(matches!(
        stdin_unavailable(),
        OfficeWorkerError::Protocol { .. }
    ));
    assert!(matches!(
        stdout_unavailable(),
        OfficeWorkerError::Protocol { .. }
    ));
    assert_eq!(1, cpu_seconds(Duration::ZERO));
}

#[cfg(not(windows))]
#[test]
fn debug_environment_is_propagated_only_when_enabled() {
    for debug_enabled in [false, true] {
        let mut command = std::process::Command::new("worker");
        let config = crate::multi_format::OfficeWorkerConfig::new("worker".into());
        configure_command_with_debug(
            &mut command,
            std::path::Path::new("workspace"),
            &config,
            debug_enabled,
        );
        let has_debug = command
            .get_envs()
            .any(|(name, value)| name == "DEBUG" && value == Some(std::ffi::OsStr::new("true")));
        assert_eq!(debug_enabled, has_debug);
    }
}

#[cfg(not(windows))]
#[test]
fn debug_worker_environment_carries_the_spreadsheet_session_correlation() {
    let _trace_session = DebugTrace::session((42, 0x0123_4567_89ab_cdef));
    let mut command = std::process::Command::new("worker");
    let config = crate::multi_format::OfficeWorkerConfig::new("worker".into());
    configure_command_with_debug(
        &mut command,
        std::path::Path::new("workspace"),
        &config,
        true,
    );
    let environment: Vec<_> = command
        .get_envs()
        .filter_map(|(name, value)| value.map(|value| (name, value)))
        .collect();
    assert!(environment.iter().any(|(name, value)| {
        *name == std::ffi::OsStr::new("KDV_TRACE_SESSION") && *value != std::ffi::OsStr::new("0")
    }));
    assert!(environment.iter().any(|(name, value)| {
        *name == std::ffi::OsStr::new("KDV_TRACE_SOURCE") && value.len() == 16
    }));
}
