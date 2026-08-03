use super::spreadsheet_worker_owner::SpreadsheetProcessOwner;
use super::spreadsheet_worker_protocol::SPREADSHEET_MODE;
#[cfg(windows)]
use super::windows_command_line::WindowsCommandLine;
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
        let mut command = std::process::Command::new(&config.executable);
        configure_command(&mut command, workspace, config);
        #[cfg(coverage)]
        let coverage_profile = super::coverage_profile::ChildCoverageProfile::configure(
            &mut command,
            workspace,
            "spreadsheet",
        );
        let mut child = command
            .spawn()
            .map_err(|error| OfficeWorkerError::unavailable(config, error.to_string()))?;
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
        let capabilities = windows_capabilities(workspace, config)?;
        let options = windows_options(workspace, config);
        let mut child = match rappct::launch::launch_in_container_with_io(&capabilities, &options) {
            Ok(child) => child,
            Err(error) => {
                return Err(OfficeWorkerError::unavailable(config, error.to_string()));
            }
        };
        let input = child.stdin.take().ok_or_else(stdin_unavailable)?;
        let output = child.stdout.take().ok_or_else(stdout_unavailable)?;
        Ok(SpawnedSpreadsheetProcess {
            input: Box::new(input),
            output: Box::new(output),
            owner: SpreadsheetProcessOwner { child: Some(child) },
        })
    }
}

#[cfg(not(windows))]
fn configure_command(
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
        .stderr(std::process::Stdio::null())
        .env_clear();
}

#[cfg(windows)]
fn windows_capabilities(
    workspace: &Path,
    config: &OfficeWorkerConfig,
) -> Result<rappct::SecurityCapabilities, OfficeWorkerError> {
    use rappct::{AppContainerProfile, SecurityCapabilitiesBuilder};
    let profile = AppContainerProfile::ensure(
        "Katana.DocumentViewer.OfficeWorker",
        "KatanA document viewer office worker",
        Some("Network-denied Office viewer helper"),
    )
    .map_err(|error| OfficeWorkerError::unavailable(config, error.to_string()))?;
    grant_windows_access(
        rappct::acl::ResourcePath::Directory(workspace.to_path_buf()),
        &profile,
        config,
    )?;
    grant_windows_access(
        rappct::acl::ResourcePath::File(config.executable.clone()),
        &profile,
        config,
    )?;
    SecurityCapabilitiesBuilder::new(&profile.sid)
        .build()
        .map_err(|error| OfficeWorkerError::unavailable(config, error.to_string()))
}

#[cfg(windows)]
fn grant_windows_access(
    resource: rappct::acl::ResourcePath,
    profile: &rappct::AppContainerProfile,
    config: &OfficeWorkerConfig,
) -> Result<(), OfficeWorkerError> {
    rappct::acl::grant_to_package(resource, &profile.sid, rappct::acl::AccessMask::GENERIC_ALL)
        .map_err(|error| OfficeWorkerError::unavailable(config, error.to_string()))
}

#[cfg(windows)]
fn windows_options(workspace: &Path, config: &OfficeWorkerConfig) -> rappct::LaunchOptions {
    use rappct::{JobLimits, StdioConfig};
    rappct::LaunchOptions {
        exe: config.executable.clone(),
        cmdline: Some(spreadsheet_command_line(workspace, config)),
        cwd: Some(workspace.to_path_buf()),
        env: Some(rappct::launch::merge_parent_env(Vec::new())),
        stdio: StdioConfig::Pipe,
        join_job: Some(JobLimits {
            memory_bytes: Some(config.max_memory_bytes),
            cpu_rate_percent: None,
            kill_on_job_close: true,
        }),
        ..rappct::LaunchOptions::default()
    }
}

#[cfg(windows)]
fn spreadsheet_command_line(workspace: &Path, config: &OfficeWorkerConfig) -> String {
    let limits = config.spreadsheet_limits;
    WindowsCommandLine::from_arguments([
        config.executable.to_string_lossy().into_owned(),
        SPREADSHEET_MODE.to_owned(),
        workspace.to_string_lossy().into_owned(),
        config.max_memory_bytes.to_string(),
        cpu_seconds(config.timeout).to_string(),
        limits.max_sheets.to_string(),
        limits.max_logical_cells.to_string(),
        limits.max_materialized_cells.to_string(),
    ])
}

fn stdin_unavailable() -> OfficeWorkerError {
    OfficeWorkerError::protocol("spreadsheet worker stdin is unavailable".to_owned())
}

fn stdout_unavailable() -> OfficeWorkerError {
    OfficeWorkerError::protocol("spreadsheet worker stdout is unavailable".to_owned())
}

fn cpu_seconds(timeout: Duration) -> u64 {
    timeout.as_secs().saturating_add(1).max(1)
}

#[cfg(test)]
mod tests {
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
}
