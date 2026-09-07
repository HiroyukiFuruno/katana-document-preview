use super::office_worker_protocol::INPUT_NAME;
use super::spreadsheet_worker_owner::SpreadsheetProcessOwner;
use super::spreadsheet_worker_protocol::SPREADSHEET_MODE;
use super::spreadsheet_worker_spawn::{
    SpawnedSpreadsheetProcess, cpu_seconds, stdin_unavailable, stdout_unavailable,
};
use super::spreadsheet_worker_spawn_windows_stderr::{spawn_stderr_reader, stderr_unavailable};
use super::windows_command_line::WindowsCommandLine;
use super::windows_worker_executable::stage_windows_worker;
use super::windows_worker_profile::{app_container_profile, launch_error, worker_environment};
use super::{OfficeWorkerConfig, OfficeWorkerError};
use rappct::acl::{AccessMask, ResourcePath, grant_to_package};
use rappct::{AppContainerProfile, SecurityCapabilitiesBuilder};
use std::path::Path;

pub(super) fn spawn(
    workspace: &Path,
    config: &OfficeWorkerConfig,
) -> Result<SpawnedSpreadsheetProcess, OfficeWorkerError> {
    let staged_executable = stage_windows_worker(workspace, config)?;
    let capabilities = windows_capabilities(workspace, &staged_executable, config)?;
    let debug_enabled = super::debug_trace::DebugTrace::enabled();
    let options = windows_options(workspace, &staged_executable, config, debug_enabled);
    let mut child = rappct::launch::launch_in_container_with_io(&capabilities, &options)
        .map_err(|error| launch_error(config, &staged_executable, error))?;
    let stderr = child.stderr.take().ok_or_else(stderr_unavailable)?;
    let stderr_reader = spawn_stderr_reader(stderr, debug_enabled);
    let input = child.stdin.take().ok_or_else(stdin_unavailable)?;
    let output = child.stdout.take().ok_or_else(stdout_unavailable)?;
    Ok(SpawnedSpreadsheetProcess {
        input: Box::new(input),
        output: Box::new(output),
        stderr_reader: Some(stderr_reader),
        owner: SpreadsheetProcessOwner { child: Some(child) },
    })
}

fn windows_capabilities(
    workspace: &Path,
    staged_executable: &Path,
    config: &OfficeWorkerConfig,
) -> Result<rappct::SecurityCapabilities, OfficeWorkerError> {
    let profile = app_container_profile(config)?;
    grant_worker_resources(workspace, staged_executable, &profile, config)?;
    SecurityCapabilitiesBuilder::new(&profile.sid)
        .build()
        .map_err(|error| OfficeWorkerError::unavailable(config, error.to_string()))
}

fn grant_worker_resources(
    workspace: &Path,
    staged_executable: &Path,
    profile: &AppContainerProfile,
    config: &OfficeWorkerConfig,
) -> Result<(), OfficeWorkerError> {
    grant_access(
        ResourcePath::Directory(workspace.to_path_buf()),
        profile,
        config,
    )?;
    grant_access(
        ResourcePath::File(workspace.join(INPUT_NAME)),
        profile,
        config,
    )?;
    grant_access(
        ResourcePath::File(staged_executable.to_path_buf()),
        profile,
        config,
    )
}

fn grant_access(
    resource: ResourcePath,
    profile: &AppContainerProfile,
    config: &OfficeWorkerConfig,
) -> Result<(), OfficeWorkerError> {
    let resource_label = format!("{resource:?}");
    grant_to_package(resource, &profile.sid, AccessMask::GENERIC_ALL).map_err(|error| {
        OfficeWorkerError::unavailable(
            config,
            format!("Windows AppContainer ACL grant failed for {resource_label}: {error}"),
        )
    })
}

fn windows_options(
    workspace: &Path,
    staged_executable: &Path,
    config: &OfficeWorkerConfig,
    debug_enabled: bool,
) -> rappct::LaunchOptions {
    use rappct::{JobLimits, StdioConfig};
    rappct::LaunchOptions {
        exe: staged_executable.to_path_buf(),
        cmdline: Some(spreadsheet_command_line(
            workspace,
            staged_executable,
            config,
        )),
        cwd: Some(workspace.to_path_buf()),
        env: Some(worker_environment_with_trace(workspace, debug_enabled)),
        stdio: StdioConfig::Pipe,
        join_job: Some(JobLimits {
            memory_bytes: Some(config.max_memory_bytes),
            cpu_rate_percent: None,
            kill_on_job_close: true,
        }),
        ..rappct::LaunchOptions::default()
    }
}

fn worker_environment_with_trace(
    workspace: &Path,
    debug_enabled: bool,
) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
    let mut environment = worker_environment(workspace);
    if debug_enabled {
        super::debug_trace::DebugTrace::configure_worker_environment(&mut environment);
    }
    environment
}

fn spreadsheet_command_line(
    workspace: &Path,
    staged_executable: &Path,
    config: &OfficeWorkerConfig,
) -> String {
    let limits = config.spreadsheet_limits;
    WindowsCommandLine::from_arguments([
        staged_executable.to_string_lossy().into_owned(),
        SPREADSHEET_MODE.to_owned(),
        workspace.to_string_lossy().into_owned(),
        config.max_memory_bytes.to_string(),
        cpu_seconds(config.timeout).to_string(),
        limits.max_sheets.to_string(),
        limits.max_logical_cells.to_string(),
        limits.max_materialized_cells.to_string(),
    ])
}
