use super::spreadsheet_worker_executable::SpreadsheetWorkerExecutable;
use super::spreadsheet_worker_owner::SpreadsheetProcessOwner;
#[cfg(not(windows))]
use super::spreadsheet_worker_protocol::SPREADSHEET_MODE;
use super::{OfficeWorkerConfig, OfficeWorkerError};
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

pub(crate) struct SpawnedSpreadsheetProcess {
    #[cfg(target_os = "macos")]
    pub(crate) process_id: u32,
    pub(crate) input: Box<dyn Write + Send>,
    pub(crate) output: Box<dyn Read + Send>,
    #[cfg(windows)]
    pub(crate) stderr_reader: Option<std::thread::JoinHandle<()>>,
    pub(crate) owner: SpreadsheetProcessOwner,
    #[cfg(all(coverage, not(windows)))]
    pub(crate) coverage_profile: Option<super::coverage_profile::ChildCoverageProfile>,
}

pub(crate) struct SpreadsheetWorkerSpawn;

impl SpreadsheetWorkerSpawn {
    #[cfg(not(windows))]
    pub(crate) fn spawn(
        workspace: &Path,
        config: &OfficeWorkerConfig,
    ) -> Result<SpawnedSpreadsheetProcess, OfficeWorkerError> {
        let resolved = SpreadsheetWorkerExecutable::resolve(config);
        let mut command = std::process::Command::new(&resolved.executable);
        configure_command(&mut command, workspace, &resolved);
        #[cfg(coverage)]
        let coverage_profile = super::coverage_profile::ChildCoverageProfile::configure(
            &mut command,
            workspace,
            "spreadsheet",
        );
        let mut child = spawn_child(&mut command, &resolved)?;
        #[cfg(target_os = "macos")]
        let process_id = child.id();
        let input = child.stdin.take().ok_or_else(stdin_unavailable)?;
        let output = child.stdout.take().ok_or_else(stdout_unavailable)?;
        Ok(SpawnedSpreadsheetProcess {
            #[cfg(target_os = "macos")]
            process_id,
            input: Box::new(input),
            output: Box::new(output),
            owner: SpreadsheetProcessOwner { child: Some(child) },
            #[cfg(coverage)]
            coverage_profile,
        })
    }

    #[cfg(windows)]
    pub(crate) fn spawn(
        workspace: &Path,
        config: &OfficeWorkerConfig,
    ) -> Result<SpawnedSpreadsheetProcess, OfficeWorkerError> {
        let resolved = SpreadsheetWorkerExecutable::resolve(config);
        let _spawn = super::debug_trace::DebugTrace::start("spreadsheet.worker_spawn");
        super::spreadsheet_worker_spawn_windows::spawn(workspace, &resolved)
    }
}

#[cfg(not(windows))]
fn spawn_child(
    command: &mut std::process::Command,
    config: &OfficeWorkerConfig,
) -> Result<std::process::Child, OfficeWorkerError> {
    let _spawn = super::debug_trace::DebugTrace::start("spreadsheet.worker_spawn");
    command
        .spawn()
        .map_err(|error| OfficeWorkerError::unavailable(config, error.to_string()))
}

#[cfg(not(windows))]
fn configure_command(
    command: &mut std::process::Command,
    workspace: &Path,
    config: &OfficeWorkerConfig,
) {
    configure_command_with_debug(
        command,
        workspace,
        config,
        super::debug_trace::DebugTrace::enabled(),
    );
}

#[cfg(not(windows))]
fn configure_command_with_debug(
    command: &mut std::process::Command,
    workspace: &Path,
    config: &OfficeWorkerConfig,
    debug_enabled: bool,
) {
    configure_worker_command(command, workspace, config);
    configure_debug_output(command, debug_enabled);
}

#[cfg(not(windows))]
fn configure_worker_command(
    command: &mut std::process::Command,
    workspace: &Path,
    config: &OfficeWorkerConfig,
) {
    let limits = config.spreadsheet_limits;
    command
        .arg(SPREADSHEET_MODE)
        .arg(workspace)
        .arg(config.max_memory_bytes.to_string())
        .arg(cpu_seconds(config.timeout).to_string())
        .arg(limits.max_sheets.to_string())
        .arg(limits.max_logical_cells.to_string())
        .arg(limits.max_materialized_cells.to_string())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .env_clear();
}

#[cfg(not(windows))]
fn configure_debug_output(command: &mut std::process::Command, debug_enabled: bool) {
    if !debug_enabled {
        command.stderr(std::process::Stdio::null());
        return;
    }
    command
        .stderr(std::process::Stdio::inherit())
        .env("DEBUG", "true");
    propagate_trace_environment(command);
}

#[cfg(not(windows))]
fn propagate_trace_environment(command: &mut std::process::Command) {
    if let Some((session, source)) = super::debug_trace::DebugTrace::worker_environment() {
        command
            .env("KDV_TRACE_SESSION", session)
            .env("KDV_TRACE_SOURCE", source);
    }
}

pub(super) fn stdin_unavailable() -> OfficeWorkerError {
    OfficeWorkerError::protocol("spreadsheet worker stdin is unavailable".to_owned())
}

pub(super) fn stdout_unavailable() -> OfficeWorkerError {
    OfficeWorkerError::protocol("spreadsheet worker stdout is unavailable".to_owned())
}

pub(super) fn cpu_seconds(timeout: Duration) -> u64 {
    timeout.as_secs().saturating_add(1).max(1)
}

#[cfg(test)]
#[path = "spreadsheet_worker_spawn_tests.rs"]
mod tests;
