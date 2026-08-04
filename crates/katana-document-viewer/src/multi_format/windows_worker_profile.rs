use super::{OfficeWorkerConfig, OfficeWorkerError};
use rappct::AppContainerProfile;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

const PROFILE_NAME: &str = "Katana.DocumentViewer.OfficeWorker";
const PROFILE_DISPLAY_NAME: &str = "KatanA document viewer office worker";
const PROFILE_DESCRIPTION: &str = "Network-denied Office document helper";

static APP_CONTAINER_PROFILE: OnceLock<Mutex<Option<AppContainerProfile>>> = OnceLock::new();

pub(super) fn app_container_profile(
    config: &OfficeWorkerConfig,
) -> Result<AppContainerProfile, OfficeWorkerError> {
    let cache = APP_CONTAINER_PROFILE.get_or_init(|| Mutex::new(None));
    let mut cached = cache.lock().map_err(|error| {
        OfficeWorkerError::unavailable(
            config,
            format!("Windows AppContainer profile cache is unavailable: {error}"),
        )
    })?;
    if let Some(profile) = cached.as_ref() {
        return Ok(profile.clone());
    }

    let profile = AppContainerProfile::ensure(
        PROFILE_NAME,
        PROFILE_DISPLAY_NAME,
        Some(PROFILE_DESCRIPTION),
    )
    .map_err(|error| OfficeWorkerError::unavailable(config, error.to_string()))?;
    *cached = Some(profile.clone());
    Ok(profile)
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
    let mut environment: Vec<_> = std::env::vars_os()
        .filter(|(name, _)| !is_worker_temp_variable(name))
        .collect();
    let workspace = workspace.as_os_str().to_owned();
    environment.push((OsString::from("TEMP"), workspace.clone()));
    environment.push((OsString::from("TMP"), workspace));
    environment.sort_by(|left, right| {
        left.0
            .to_string_lossy()
            .to_ascii_lowercase()
            .cmp(&right.0.to_string_lossy().to_ascii_lowercase())
    });
    environment
}

fn is_worker_temp_variable(name: &std::ffi::OsStr) -> bool {
    name.eq_ignore_ascii_case("TEMP") || name.eq_ignore_ascii_case("TMP")
}
