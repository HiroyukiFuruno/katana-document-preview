use super::{
    apply_failure, architecture_failure, build_filter, compile_failure, compile_filter,
    filter_failure, finish_install, install, network_syscalls,
};
use seccompiler::{BackendError, Error as SeccompError};
use std::error::Error;
use std::io;
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::Command;

const CHILD_ENV: &str = "KDV_NETWORK_SECCOMP_CHILD";

#[test]
fn network_filter_builds_for_the_runner_architecture() -> Result<(), String> {
    let filter = build_filter()?;
    assert!(compile_filter(filter).is_ok());
    assert!(network_syscalls().contains(&libc::SYS_socket));
    assert!(network_syscalls().contains(&libc::SYS_connect));
    assert!(network_syscalls().contains(&libc::SYS_bind));
    Ok(())
}

#[test]
fn network_filter_errors_keep_the_seccomp_stage() {
    assert_eq!(
        "seccomp arch: InvalidTargetArch(\"unknown\")",
        architecture_failure(BackendError::InvalidTargetArch("unknown".to_owned()))
    );
    assert_eq!(
        "seccomp filter: `match_action` and `mismatch_action` are equal.",
        filter_failure(BackendError::IdenticalActions)
    );
    assert_eq!(
        "seccomp compile: The condition vector of a rule cannot be empty.",
        compile_failure(BackendError::EmptyRule)
    );
    assert_eq!(
        "seccomp apply: Cannot install empty filter.",
        apply_failure(SeccompError::EmptyFilter)
    );
    assert_eq!(Ok(true), finish_install(Ok(())));
    assert_eq!(
        Err("seccomp apply: Cannot install empty filter.".to_owned()),
        finish_install(Err(SeccompError::EmptyFilter))
    );
}

#[test]
fn network_filter_denies_socket_creation_in_an_isolated_child() -> Result<(), Box<dyn Error>> {
    if std::env::var_os(CHILD_ENV).is_some() {
        assert!(install().map_err(io::Error::other)?);
        let error = match TcpStream::connect(("127.0.0.1", 9)) {
            Ok(_) => return Err(io::Error::other("socket was not denied").into()),
            Err(error) => error,
        };
        assert_eq!(Some(libc::EPERM), error.raw_os_error());
        return Ok(());
    }

    let test_thread = std::thread::current();
    let test_name = test_thread
        .name()
        .ok_or_else(|| io::Error::other("test thread must be named"))?;
    let mut command = Command::new(std::env::current_exe()?);
    command
        .args(["--exact", test_name, "--nocapture"])
        .env(CHILD_ENV, "1");
    configure_child_coverage_profile(&mut command);
    let status = command.status()?;
    assert!(status.success(), "seccomp test child failed: {status}");
    Ok(())
}

fn configure_child_coverage_profile(command: &mut Command) {
    if let Some(profile) = std::env::var_os("LLVM_PROFILE_FILE") {
        let mut child_profile = PathBuf::from(profile).into_os_string();
        child_profile.push(format!("-seccomp-child-{}.profraw", std::process::id()));
        command.env("LLVM_PROFILE_FILE", child_profile);
    }
}
