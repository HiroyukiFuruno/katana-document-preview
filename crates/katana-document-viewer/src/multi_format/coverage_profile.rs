use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static CHILD_PROFILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) struct ChildCoverageProfile {
    workspace_file: PathBuf,
    report_file: PathBuf,
}

impl ChildCoverageProfile {
    pub(crate) fn configure(
        command: &mut Command,
        workspace: &Path,
        worker_kind: &str,
    ) -> Option<Self> {
        let worker_name = Path::new(command.get_program())
            .file_name()
            .and_then(|name| name.to_str());
        if !matches!(
            worker_name,
            Some("kdv-office-worker" | "kdv-office-worker.exe")
        ) {
            return None;
        }
        let mut report_file = std::env::var_os("LLVM_PROFILE_FILE")?;
        let sequence = CHILD_PROFILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        report_file.push(format!("-worker-{worker_kind}-{sequence}.profraw"));
        let workspace_file = workspace.join(format!(".coverage-{worker_kind}.profraw"));
        command.env("LLVM_PROFILE_FILE", &workspace_file);
        Some(Self {
            workspace_file,
            report_file: PathBuf::from(report_file),
        })
    }

    pub(crate) fn collect(self) -> std::io::Result<()> {
        std::fs::copy(self.workspace_file, self.report_file).map(|_| ())
    }
}
