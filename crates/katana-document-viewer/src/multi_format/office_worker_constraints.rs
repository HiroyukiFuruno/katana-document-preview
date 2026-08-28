#[cfg(target_os = "linux")]
use skarn_sandbox::Backend;
use skarn_sandbox::{NetPolicy, Policy, RestrictionReport, RestrictionStatus};
use std::path::Path;

#[cfg(target_os = "linux")]
#[path = "office_worker_network_seccomp.rs"]
mod network_seccomp;

pub(super) struct OfficeWorkerConstraints;

impl OfficeWorkerConstraints {
    pub(super) fn apply(
        workspace: &Path,
        max_memory_bytes: u64,
        max_cpu_seconds: u64,
    ) -> Result<(), (String, String)> {
        {
            let _limits = super::debug_trace::DebugTrace::start("worker.resource_limits");
            Self::apply_resource_limits(max_memory_bytes, max_cpu_seconds)?;
        }
        let _sandbox = super::debug_trace::DebugTrace::start("worker.sandbox");
        Self::apply_sandbox(workspace)
    }

    #[cfg(unix)]
    fn apply_resource_limits(
        max_memory_bytes: u64,
        max_cpu_seconds: u64,
    ) -> Result<(), (String, String)> {
        #[cfg(target_os = "macos")]
        let _memory_limit_enforced_by_parent = max_memory_bytes;
        #[cfg(not(target_os = "macos"))]
        rlimit::setrlimit(rlimit::Resource::AS, max_memory_bytes, max_memory_bytes)
            .map_err(Self::memory_limit_failure)?;
        rlimit::setrlimit(rlimit::Resource::CPU, max_cpu_seconds, max_cpu_seconds)
            .map_err(Self::cpu_limit_failure)
    }

    #[cfg(windows)]
    fn apply_resource_limits(
        _max_memory_bytes: u64,
        _max_cpu_seconds: u64,
    ) -> Result<(), (String, String)> {
        let token = rappct::token::query_current_process_token()
            .map_err(|error| Self::failure("sandbox", error.to_string()))?;
        if token.is_appcontainer {
            Ok(())
        } else {
            Err(Self::failure(
                "sandbox",
                "worker is not running inside an AppContainer".to_owned(),
            ))
        }
    }

    #[cfg(not(any(unix, windows)))]
    fn apply_resource_limits(
        _max_memory_bytes: u64,
        _max_cpu_seconds: u64,
    ) -> Result<(), (String, String)> {
        Err(Self::failure(
            "sandbox",
            "the current platform has no supported office worker sandbox".to_owned(),
        ))
    }

    #[cfg(unix)]
    fn apply_sandbox(workspace: &Path) -> Result<(), (String, String)> {
        let policy = Policy::builder()
            .workspace(workspace)
            .net(NetPolicy::DenyAll)
            .fail_closed(true)
            .build();
        let report = policy
            .apply_to_current_process()
            .map_err(Self::sandbox_failure)?;
        #[cfg(target_os = "linux")]
        let network_seccomp = network_seccomp::install().map_err(Self::network_seccomp_failure)?;
        #[cfg(not(target_os = "linux"))]
        let network_seccomp = false;
        Self::validate_sandbox_report(&report, network_seccomp)
    }

    #[cfg(unix)]
    fn validate_sandbox_report(
        report: &RestrictionReport,
        network_seccomp: bool,
    ) -> Result<(), (String, String)> {
        if report.status == RestrictionStatus::FullyEnforced
            || Self::accept_partial_linux_report(report, network_seccomp)
        {
            Ok(())
        } else {
            Err(Self::failure(
                "sandbox",
                format!(
                    "{} sandbox did not fully enforce the policy: {:?}",
                    report.backend, report.notes
                ),
            ))
        }
    }

    #[cfg(target_os = "linux")]
    fn accept_partial_linux_report(report: &RestrictionReport, network_seccomp: bool) -> bool {
        report.backend == Backend::Landlock
            && report.status == RestrictionStatus::PartiallyEnforced
            && network_seccomp
            && report.notes == ["seccomp-bpf denylist applied"]
    }

    #[cfg(not(target_os = "linux"))]
    const fn accept_partial_linux_report(
        _report: &RestrictionReport,
        _network_seccomp: bool,
    ) -> bool {
        false
    }

    #[cfg(windows)]
    fn apply_sandbox(_workspace: &Path) -> Result<(), (String, String)> {
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    fn apply_sandbox(_workspace: &Path) -> Result<(), (String, String)> {
        Err(Self::failure(
            "sandbox",
            "the current platform has no supported office worker sandbox".to_owned(),
        ))
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    fn memory_limit_failure(error: std::io::Error) -> (String, String) {
        Self::failure("memory_limit", error.to_string())
    }

    #[cfg(unix)]
    fn cpu_limit_failure(error: std::io::Error) -> (String, String) {
        Self::failure("cpu_limit", error.to_string())
    }

    #[cfg(unix)]
    fn sandbox_failure(error: skarn_sandbox::Error) -> (String, String) {
        Self::failure("sandbox", error.to_string())
    }

    #[cfg(target_os = "linux")]
    fn network_seccomp_failure(message: String) -> (String, String) {
        Self::failure("sandbox", message)
    }

    fn failure(stage: &str, message: String) -> (String, String) {
        (stage.to_owned(), message)
    }
}

#[cfg(all(test, unix))]
#[path = "office_worker_constraints_tests.rs"]
mod tests;
