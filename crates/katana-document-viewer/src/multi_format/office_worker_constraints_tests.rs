use super::OfficeWorkerConstraints;
use skarn_sandbox::{Backend, RestrictionReport, RestrictionStatus};

#[test]
fn sandbox_reports_require_full_enforcement() {
    let enforced = RestrictionReport {
        backend: Backend::None,
        status: RestrictionStatus::FullyEnforced,
        notes: Vec::new(),
    };
    assert_eq!(
        Ok(()),
        OfficeWorkerConstraints::validate_sandbox_report(&enforced, false)
    );
    let report = RestrictionReport {
        backend: Backend::None,
        status: RestrictionStatus::PartiallyEnforced,
        notes: vec!["network restriction unavailable".to_owned()],
    };
    let error = OfficeWorkerConstraints::validate_sandbox_report(&report, false);
    assert!(matches!(
        error,
        Err((stage, message))
            if stage == "sandbox"
                && message.contains("None")
                && message.contains("network restriction unavailable")
    ));
}

#[cfg(target_os = "linux")]
#[test]
fn partial_landlock_requires_both_seccomp_layers_and_clean_policy_paths() {
    let report = RestrictionReport {
        backend: Backend::Landlock,
        status: RestrictionStatus::PartiallyEnforced,
        notes: vec!["seccomp-bpf denylist applied".to_owned()],
    };
    assert_eq!(
        Ok(()),
        OfficeWorkerConstraints::validate_sandbox_report(&report, true)
    );
    assert!(OfficeWorkerConstraints::validate_sandbox_report(&report, false).is_err());
    let extra_note = RestrictionReport {
        notes: vec![
            "seccomp-bpf denylist applied".to_owned(),
            "unknown policy warning".to_owned(),
        ],
        ..report
    };
    assert!(OfficeWorkerConstraints::validate_sandbox_report(&extra_note, true).is_err());
}

#[test]
fn sandbox_failure_helpers_preserve_stage_context() {
    assert_eq!(
        ("limit".to_owned(), "failure".to_owned()),
        OfficeWorkerConstraints::failure("limit", "failure".to_owned())
    );
    assert_eq!(
        "cpu_limit",
        OfficeWorkerConstraints::cpu_limit_failure(std::io::Error::other("failure")).0
    );
    #[cfg(not(target_os = "macos"))]
    assert_eq!(
        "memory_limit",
        OfficeWorkerConstraints::memory_limit_failure(std::io::Error::other("failure")).0
    );
    assert_eq!(
        "sandbox",
        OfficeWorkerConstraints::sandbox_failure(skarn_sandbox::Error::sandbox("failure")).0
    );
    #[cfg(target_os = "linux")]
    assert_eq!(
        ("sandbox".to_owned(), "network failure".to_owned()),
        OfficeWorkerConstraints::network_seccomp_failure("network failure".to_owned())
    );
}
