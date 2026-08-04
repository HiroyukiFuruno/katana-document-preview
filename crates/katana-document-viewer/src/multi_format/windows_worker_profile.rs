use super::{OfficeWorkerConfig, OfficeWorkerError};
use rappct::AppContainerProfile;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

const PROFILE_NAME: &str = "Katana.DocumentViewer.OfficeWorker";
const PROFILE_DISPLAY_NAME: &str = "KatanA document viewer office worker";
const PROFILE_DESCRIPTION: &str = "Network-denied Office document helper";

pub(super) fn app_container_profile(
    config: &OfficeWorkerConfig,
) -> Result<AppContainerProfile, OfficeWorkerError> {
    AppContainerProfile::ensure(
        PROFILE_NAME,
        PROFILE_DISPLAY_NAME,
        Some(PROFILE_DESCRIPTION),
    )
    .map_err(|error| OfficeWorkerError::unavailable(config, error.to_string()))
}

pub(super) fn workspace_root(config: &OfficeWorkerConfig) -> Result<PathBuf, OfficeWorkerError> {
    let _profile = app_container_profile(config)?;
    let local_app_data = std::env::var_os("LOCALAPPDATA").ok_or_else(|| {
        OfficeWorkerError::unavailable(config, "LOCALAPPDATA is unavailable".to_owned())
    })?;
    let root = PathBuf::from(local_app_data)
        .join("Packages")
        .join(PROFILE_NAME)
        .join("AC")
        .join("Temp");
    std::fs::create_dir_all(&root)
        .map_err(|error| OfficeWorkerError::unavailable(config, error.to_string()))?;
    Ok(root)
}

pub(super) fn launch_error(
    config: &OfficeWorkerConfig,
    executable: &Path,
    error: rappct::AcError,
) -> OfficeWorkerError {
    let source = std::error::Error::source(&error)
        .map(ToString::to_string)
        .unwrap_or_else(|| "source unavailable".to_owned());
    OfficeWorkerError::unavailable(
        config,
        format!(
            "Windows AppContainer staged worker launch failed: executable=`{}`: {error}: source={source}",
            executable.display()
        ),
    )
}

pub(super) fn worker_environment(workspace: &Path) -> Vec<(OsString, OsString)> {
    let workspace = workspace.as_os_str().to_owned();
    rappct::launch::merge_parent_env(vec![
        (OsString::from("TEMP"), workspace.clone()),
        (OsString::from("TMP"), workspace),
    ])
}
