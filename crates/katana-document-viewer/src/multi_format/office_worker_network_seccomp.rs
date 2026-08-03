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
#[path = "office_worker_network_seccomp_tests.rs"]
mod tests;
