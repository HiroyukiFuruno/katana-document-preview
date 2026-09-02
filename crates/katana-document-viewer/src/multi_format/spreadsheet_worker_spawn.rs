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
    if debug_enabled {
        command
            .stderr(std::process::Stdio::inherit())
            .env("DEBUG", "true");
    } else {
        command.stderr(std::process::Stdio::null());
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
mod tests {
    #[cfg(not(windows))]
    use super::configure_command_with_debug;
    use super::{cpu_seconds, stdin_unavailable, stdout_unavailable};
    use crate::multi_format::OfficeWorkerError;
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
            let has_debug = command.get_envs().any(|(name, value)| {
                name == "DEBUG" && value == Some(std::ffi::OsStr::new("true"))
            });
            assert_eq!(debug_enabled, has_debug);
        }
    }
}
