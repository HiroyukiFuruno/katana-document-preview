use super::OfficeWorkerConfig;

#[cfg(windows)]
const SPREADSHEET_WORKER_NAME: &str = "kdv-spreadsheet-worker.exe";
#[cfg(not(windows))]
const SPREADSHEET_WORKER_NAME: &str = "kdv-spreadsheet-worker";

pub(super) struct SpreadsheetWorkerExecutable;

impl SpreadsheetWorkerExecutable {
    pub(super) fn resolve(config: &OfficeWorkerConfig) -> OfficeWorkerConfig {
        let candidate = config.executable.with_file_name(SPREADSHEET_WORKER_NAME);
        if candidate.is_file() {
            let mut resolved = config.clone();
            resolved.executable = candidate;
            resolved
        } else {
            config.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SPREADSHEET_WORKER_NAME, SpreadsheetWorkerExecutable};
    use crate::multi_format::OfficeWorkerConfig;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn prefers_a_packaged_spreadsheet_worker_sibling() -> TestResult {
        let directory = tempfile::tempdir()?;
        let office = directory.path().join("kdv-office-worker");
        let spreadsheet = directory.path().join(SPREADSHEET_WORKER_NAME);
        std::fs::write(&office, b"office")?;
        std::fs::write(&spreadsheet, b"spreadsheet")?;

        let resolved = SpreadsheetWorkerExecutable::resolve(&OfficeWorkerConfig::new(office));

        assert_eq!(spreadsheet, resolved.executable);
        Ok(())
    }

    #[test]
    fn preserves_the_compatible_office_worker_fallback() {
        let configured = PathBuf::from("/missing/kdv-office-worker");
        let resolved =
            SpreadsheetWorkerExecutable::resolve(&OfficeWorkerConfig::new(configured.clone()));

        assert_eq!(configured, resolved.executable);
    }

    use std::path::PathBuf;
}
