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
        if !supported_worker(command) {
            return None;
        }
        let report_file = report_file(worker_kind)?;
        let workspace_file = workspace.join(format!(".coverage-{worker_kind}.profraw"));
        command.env("LLVM_PROFILE_FILE", &workspace_file);
        Some(Self {
            workspace_file,
            report_file,
        })
    }

    pub(crate) fn collect(self) -> std::io::Result<()> {
        std::fs::copy(self.workspace_file, self.report_file).map(|_| ())
    }
}

fn supported_worker(command: &Command) -> bool {
    let worker_name = Path::new(command.get_program())
        .file_name()
        .and_then(|name| name.to_str());
    matches!(
        worker_name,
        Some(
            "kdv-office-worker"
                | "kdv-office-worker.exe"
                | "kdv-spreadsheet-worker"
                | "kdv-spreadsheet-worker.exe"
        )
    )
}

fn report_file(worker_kind: &str) -> Option<PathBuf> {
    let mut report_file = std::env::var_os("LLVM_PROFILE_FILE")?;
    let sequence = CHILD_PROFILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let parent_process = std::process::id();
    report_file.push(format!(
        "-worker-{worker_kind}-{parent_process}-{sequence}.profraw"
    ));
    Some(PathBuf::from(report_file))
}
