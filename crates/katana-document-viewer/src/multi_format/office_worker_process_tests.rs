#[cfg(target_os = "macos")]
use super::OfficeWorkerProcess;
#[cfg(not(windows))]
use super::configure_command_with_debug;
#[cfg(not(windows))]
use super::normalize_wait_result;
use super::{cpu_seconds, format_argument};
use crate::multi_format::{
    OfficeDocumentFormat, OfficeWorkerConfig, OfficeWorkerError, debug_trace::DebugTrace,
};
use std::path::PathBuf;
use std::time::Duration;

#[test]
fn office_worker_arguments_cover_all_formats_and_minimum_cpu_time() {
    assert_eq!("docx", format_argument(OfficeDocumentFormat::Docx));
    assert_eq!("pptx", format_argument(OfficeDocumentFormat::Pptx));
    assert_eq!("xlsx", format_argument(OfficeDocumentFormat::Xlsx));
    assert_eq!(1, cpu_seconds(Duration::ZERO));
}

#[cfg(not(windows))]
#[test]
fn normalized_wait_failures_remain_typed() {
    let config = OfficeWorkerConfig::new(PathBuf::from("worker"));
    assert!(matches!(
        normalize_wait_result(&config, Err(std::io::Error::other("wait failed"))),
        Err(OfficeWorkerError::WorkerUnavailable { .. })
    ));
    assert_eq!(
        Err(OfficeWorkerError::WorkerTimedOut),
        normalize_wait_result(&config, Ok(None))
    );
}

#[cfg(not(windows))]
#[test]
fn debug_environment_is_propagated_only_when_enabled() {
    for debug_enabled in [false, true] {
        let mut command = std::process::Command::new("worker");
        let config = OfficeWorkerConfig::new(PathBuf::from("worker"));
        configure_command_with_debug(
            &mut command,
            std::path::Path::new("workspace"),
            OfficeDocumentFormat::Docx,
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
fn debug_worker_environment_carries_the_office_session_correlation() {
    let _trace_session = DebugTrace::session((42, 0x0123_4567_89ab_cdef));
    let mut command = std::process::Command::new("worker");
    let config = OfficeWorkerConfig::new(PathBuf::from("worker"));
    configure_command_with_debug(
        &mut command,
        std::path::Path::new("workspace"),
        OfficeDocumentFormat::Docx,
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

#[cfg(not(windows))]
#[test]
fn parent_wait_returns_a_completed_worker_status() {
    let child = std::process::Command::new("/usr/bin/true").spawn();
    assert!(child.is_ok());
    if let Ok(mut child) = child {
        let config = OfficeWorkerConfig::new(PathBuf::from("/usr/bin/true"));
        assert_eq!(Ok(Some(0)), super::wait_for_worker(&mut child, &config));
    }
}

#[cfg(target_os = "macos")]
#[test]
fn parent_monitor_reports_worker_memory_limit() {
    let workspace = tempfile::tempdir();
    assert!(workspace.is_ok());
    if let Ok(workspace) = workspace {
        let mut config = OfficeWorkerConfig::new(PathBuf::from("/usr/bin/yes"));
        config.timeout = Duration::from_secs(2);
        config.max_memory_bytes = 0;
        assert_eq!(
            OfficeWorkerProcess::run(workspace.path(), OfficeDocumentFormat::Docx, &config),
            Err(OfficeWorkerError::WorkerMemoryLimitExceeded { limit: 0 })
        );
    }
}
