use seccompiler::{
    BackendError, BpfProgram, Error as SeccompError, SeccompAction, SeccompFilter, TargetArch,
    apply_filter,
};

pub(super) fn install() -> Result<bool, String> {
    let filter = build_filter()?;
    let program = compile_filter(filter)?;
    finish_install(apply_filter(&program))
}

fn build_filter() -> Result<SeccompFilter, String> {
    let rules = network_syscalls()
        .iter()
        .copied()
        .map(|syscall| (syscall, Vec::new()))
        .collect();
    let architecture = target_architecture()?;
    SeccompFilter::new(
        rules,
        SeccompAction::Allow,
        SeccompAction::Errno(libc::EPERM as u32),
        architecture,
    )
    .map_err(filter_failure)
}

fn target_architecture() -> Result<TargetArch, String> {
    std::env::consts::ARCH
        .try_into()
        .map_err(architecture_failure)
}

fn compile_filter(filter: SeccompFilter) -> Result<BpfProgram, String> {
    filter.try_into().map_err(compile_failure)
}

fn finish_install(result: seccompiler::Result<()>) -> Result<bool, String> {
    result.map_err(apply_failure)?;
    Ok(true)
}

fn architecture_failure(error: BackendError) -> String {
    format!("seccomp arch: {error:?}")
}

fn filter_failure(error: BackendError) -> String {
    format!("seccomp filter: {error}")
}

fn compile_failure(error: BackendError) -> String {
    format!("seccomp compile: {error}")
}

fn apply_failure(error: SeccompError) -> String {
    format!("seccomp apply: {error}")
}

const fn network_syscalls() -> &'static [libc::c_long] {
    &[
        libc::SYS_socket,
        libc::SYS_socketpair,
        libc::SYS_connect,
        libc::SYS_bind,
        libc::SYS_listen,
        libc::SYS_accept,
        libc::SYS_accept4,
        libc::SYS_sendto,
        libc::SYS_sendmsg,
        libc::SYS_sendmmsg,
        libc::SYS_recvfrom,
        libc::SYS_recvmsg,
        libc::SYS_recvmmsg,
        libc::SYS_shutdown,
        libc::SYS_setsockopt,
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        apply_failure, architecture_failure, build_filter, compile_failure, compile_filter,
        filter_failure, finish_install, install, network_syscalls,
    };
    use seccompiler::{BackendError, Error as SeccompError};
    use std::error::Error;
    use std::io;
    use std::net::TcpStream;
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
        let status = Command::new(std::env::current_exe()?)
            .args(["--exact", test_name, "--nocapture"])
            .env(CHILD_ENV, "1")
            .status()?;
        assert!(status.success(), "seccomp test child failed: {status}");
        Ok(())
    }
}
