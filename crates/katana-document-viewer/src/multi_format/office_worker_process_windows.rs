use super::{OfficeDocumentFormat, OfficeWorkerConfig, OfficeWorkerError};
use crate::multi_format::office_worker_protocol::INPUT_NAME;
use crate::multi_format::windows_command_line::WindowsCommandLine;
use crate::multi_format::windows_worker_executable::stage_windows_worker;
use crate::multi_format::windows_worker_profile::{
    app_container_profile, launch_error, worker_environment,
};
use rappct::acl::{AccessMask, ResourcePath, grant_to_package};
use rappct::{AppContainerProfile, SecurityCapabilitiesBuilder};
use std::path::Path;

pub(super) struct OfficeWorkerWindowsProcess;

impl OfficeWorkerWindowsProcess {
    pub(super) fn run(
        workspace: &Path,
        format: OfficeDocumentFormat,
        config: &OfficeWorkerConfig,
    ) -> Result<Option<i64>, OfficeWorkerError> {
        let staged_executable = stage_windows_worker(workspace, config)?;
        let capabilities = windows_capabilities(workspace, &staged_executable, config)?;
        let options = build_options(workspace, &staged_executable, format, config);
        let child = {
            let _spawn = crate::multi_format::debug_trace::DebugTrace::start("office.worker_spawn");
            rappct::launch::launch_in_container_with_io(&capabilities, &options)
        }
        .map_err(|error| launch_error(config, &staged_executable, error))?;
        child
            .wait(Some(config.timeout))
            .map(|status| Some(i64::from(status)))
            .map_err(|error| map_wait_error(config, error))
    }
}

fn windows_capabilities(
    workspace: &Path,
    staged_executable: &Path,
    config: &OfficeWorkerConfig,
) -> Result<rappct::SecurityCapabilities, OfficeWorkerError> {
    let profile = app_container_profile(config)?;
    grant_access(
        ResourcePath::Directory(workspace.to_path_buf()),
        &profile,
        config,
    )?;
    grant_access(
        ResourcePath::File(workspace.join(INPUT_NAME)),
        &profile,
        config,
    )?;
    grant_access(
        ResourcePath::File(staged_executable.to_path_buf()),
        &profile,
        config,
    )?;
    SecurityCapabilitiesBuilder::new(&profile.sid)
        .build()
        .map_err(|error| OfficeWorkerError::unavailable(config, error.to_string()))
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

fn build_options(
    workspace: &Path,
    staged_executable: &Path,
    format: OfficeDocumentFormat,
    config: &OfficeWorkerConfig,
) -> rappct::LaunchOptions {
    use rappct::JobLimits;
    let debug_enabled = crate::multi_format::debug_trace::DebugTrace::enabled();
    rappct::LaunchOptions {
        exe: staged_executable.to_path_buf(),
        cmdline: Some(worker_command_line(
            workspace,
            staged_executable,
            format,
            config,
        )),
        cwd: Some(workspace.to_path_buf()),
        env: Some(worker_environment_with_trace(workspace, debug_enabled)),
        stdio: worker_stdio_config(debug_enabled),
        join_job: Some(JobLimits {
            memory_bytes: Some(config.max_memory_bytes),
            cpu_rate_percent: None,
            kill_on_job_close: true,
        }),
        ..rappct::LaunchOptions::default()
    }
}

fn worker_stdio_config(debug_enabled: bool) -> rappct::StdioConfig {
    if debug_enabled {
        rappct::StdioConfig::Inherit
    } else {
        rappct::StdioConfig::Null
    }
}

fn worker_environment_with_trace(
    workspace: &Path,
    debug_enabled: bool,
) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
    let mut environment = worker_environment(workspace);
    if debug_enabled {
        crate::multi_format::debug_trace::DebugTrace::configure_worker_environment(
            &mut environment,
        );
    }
    environment
}

fn worker_command_line(
    workspace: &Path,
    staged_executable: &Path,
    format: OfficeDocumentFormat,
    config: &OfficeWorkerConfig,
) -> String {
    WindowsCommandLine::from_arguments([
        staged_executable.to_string_lossy().into_owned(),
        workspace.to_string_lossy().into_owned(),
        super::format_argument(format).to_owned(),
        config.max_memory_bytes.to_string(),
        super::cpu_seconds(config.timeout).to_string(),
        config.max_output_bytes.to_string(),
    ])
}

fn map_wait_error(config: &OfficeWorkerConfig, error: rappct::AcError) -> OfficeWorkerError {
    if error.to_string().contains("timeout") {
        OfficeWorkerError::WorkerTimedOut
    } else {
        OfficeWorkerError::unavailable(config, error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::worker_stdio_config;
    use rappct::StdioConfig;

    #[test]
    fn debug_worker_stdio_is_inherited() {
        assert!(matches!(worker_stdio_config(true), StdioConfig::Inherit));
        assert!(matches!(worker_stdio_config(false), StdioConfig::Null));
    }
}
