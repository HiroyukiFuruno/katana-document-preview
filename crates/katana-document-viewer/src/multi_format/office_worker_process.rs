#[cfg(target_os = "macos")]
use super::office_worker_monitor::MacOsMemoryMonitor;
#[cfg(windows)]
#[path = "office_worker_process_windows.rs"]
mod windows;
use super::{OfficeDocumentFormat, OfficeWorkerConfig, OfficeWorkerError};
#[cfg(not(windows))]
use process_control::{ChildExt, Control};
use std::path::Path;
#[cfg(not(windows))]
use std::process::{Command, Stdio};
use std::time::Duration;

pub(crate) struct OfficeWorkerProcess;

impl OfficeWorkerProcess {
    #[cfg(not(windows))]
    pub(crate) fn run(
        workspace: &Path,
        format: OfficeDocumentFormat,
        config: &OfficeWorkerConfig,
    ) -> Result<Option<i64>, OfficeWorkerError> {
        let mut command = Command::new(&config.executable);
        configure_command(&mut command, workspace, format, config);
        #[cfg(coverage)]
        let coverage_profile = super::coverage_profile::ChildCoverageProfile::configure(
            &mut command,
            workspace,
            "office",
        );
        let mut child = {
            let _spawn = super::debug_trace::DebugTrace::start("office.worker_spawn");
            spawn_worker(&mut command, config)?
        };
        #[cfg(target_os = "macos")]
        let memory_monitor = MacOsMemoryMonitor::start(child.id(), config.max_memory_bytes);
        let result = wait_for_worker(&mut child, config)?;
        #[cfg(target_os = "macos")]
        finish_memory_monitor(memory_monitor, config.max_memory_bytes)?;
        #[cfg(coverage)]
        if let Some(profile) = coverage_profile {
            let _ = profile.collect();
        }
        Ok(result)
    }

    #[cfg(windows)]
    pub(crate) fn run(
        workspace: &Path,
        format: OfficeDocumentFormat,
        config: &OfficeWorkerConfig,
    ) -> Result<Option<i64>, OfficeWorkerError> {
        windows::OfficeWorkerWindowsProcess::run(workspace, format, config)
    }
}

#[cfg(target_os = "macos")]
fn finish_memory_monitor(
    memory_monitor: MacOsMemoryMonitor,
    limit: usize,
) -> Result<(), OfficeWorkerError> {
    let _monitor = super::debug_trace::DebugTrace::start("office.monitor_finish");
    if memory_monitor.finish() {
        return Err(OfficeWorkerError::WorkerMemoryLimitExceeded { limit });
    }
    Ok(())
}

#[cfg(not(windows))]
fn spawn_worker(
    command: &mut Command,
    config: &OfficeWorkerConfig,
) -> Result<std::process::Child, OfficeWorkerError> {
    command
        .spawn()
        .map_err(|error| OfficeWorkerError::unavailable(config, error.to_string()))
}

#[cfg(not(windows))]
fn wait_for_worker(
    child: &mut std::process::Child,
    config: &OfficeWorkerConfig,
) -> Result<Option<i64>, OfficeWorkerError> {
    #[cfg(target_os = "linux")]
    let result = child
        .controlled()
        .memory_limit(config.max_memory_bytes)
        .time_limit(config.timeout)
        .terminate_for_timeout()
        .strict_errors()
        .wait();
    #[cfg(not(target_os = "linux"))]
    let result = child
        .controlled()
        .time_limit(config.timeout)
        .terminate_for_timeout()
        .strict_errors()
        .wait();
    normalize_wait_result(config, result)
}

#[cfg(not(windows))]
fn normalize_wait_result(
    config: &OfficeWorkerConfig,
    result: std::io::Result<Option<process_control::ExitStatus>>,
) -> Result<Option<i64>, OfficeWorkerError> {
    let status = match result {
        Ok(status) => status,
        Err(error) => return Err(OfficeWorkerError::unavailable(config, error.to_string())),
    };
    match status {
        Some(status) => Ok(status.code()),
        None => Err(OfficeWorkerError::WorkerTimedOut),
    }
}

#[cfg(not(windows))]
fn configure_command(
    command: &mut Command,
    workspace: &Path,
    format: OfficeDocumentFormat,
    config: &OfficeWorkerConfig,
) {
    configure_command_with_debug(
        command,
        workspace,
        format,
        config,
        super::debug_trace::DebugTrace::enabled(),
    );
}

#[cfg(not(windows))]
fn configure_command_with_debug(
    command: &mut Command,
    workspace: &Path,
    format: OfficeDocumentFormat,
    config: &OfficeWorkerConfig,
    debug_enabled: bool,
) {
    command
        .arg(workspace)
        .arg(format_argument(format))
        .arg(config.max_memory_bytes.to_string())
        .arg(cpu_seconds(config.timeout).to_string())
        .arg(config.max_output_bytes.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .env_clear();
    if debug_enabled {
        command.stderr(Stdio::inherit()).env("DEBUG", "true");
        if let Some((session, source)) = super::debug_trace::DebugTrace::worker_environment() {
            command
                .env("KDV_TRACE_SESSION", session)
                .env("KDV_TRACE_SOURCE", source);
        }
    } else {
        command.stderr(Stdio::null());
    }
}

const fn format_argument(format: OfficeDocumentFormat) -> &'static str {
    match format {
        OfficeDocumentFormat::Docx => "docx",
        OfficeDocumentFormat::Pptx => "pptx",
        OfficeDocumentFormat::Xlsx => "xlsx",
    }
}

fn cpu_seconds(timeout: Duration) -> u64 {
    timeout.as_secs().saturating_add(1).max(1)
}

#[cfg(test)]
#[path = "office_worker_process_tests.rs"]
mod tests;
