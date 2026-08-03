use super::SpreadsheetWorkerArguments;
use crate::multi_format::spreadsheet_worker_protocol::SPREADSHEET_MODE;
use std::ffi::OsString;

fn arguments(workspace: &std::path::Path) -> Vec<OsString> {
    [
        OsString::from("worker"),
        OsString::from(SPREADSHEET_MODE),
        workspace.as_os_str().to_owned(),
        OsString::from("1024"),
        OsString::from("1"),
        OsString::from("8"),
        OsString::from("1024"),
        OsString::from("128"),
    ]
    .into()
}

#[test]
fn parser_accepts_valid_limits() {
    let workspace = tempfile::tempdir();
    assert!(workspace.is_ok());
    if let Ok(workspace) = workspace {
        let parsed = SpreadsheetWorkerArguments::parse(arguments(workspace.path()));
        assert!(parsed.is_ok());
        if let Ok(parsed) = parsed {
            assert_eq!(workspace.path(), parsed.workspace);
            assert_eq!(1024, parsed.max_memory_bytes);
            assert_eq!(1, parsed.max_cpu_seconds);
            assert_eq!(8, parsed.limits.max_sheets);
            assert_eq!(1024, parsed.limits.max_logical_cells);
            assert_eq!(128, parsed.limits.max_materialized_cells);
        }
    }
}

#[test]
fn parser_rejects_malformed_workspace_arguments() {
    let workspace = tempfile::tempdir();
    assert!(workspace.is_ok());
    if let Ok(workspace) = workspace {
        let mut trailing = arguments(workspace.path());
        trailing.push(OsString::from("extra"));
        assert!(SpreadsheetWorkerArguments::parse(trailing).is_err());

        let mut missing_workspace = arguments(workspace.path());
        missing_workspace.truncate(2);
        assert!(SpreadsheetWorkerArguments::parse(missing_workspace).is_err());

        let mut relative = arguments(workspace.path());
        relative[2] = OsString::from("relative");
        assert!(SpreadsheetWorkerArguments::parse(relative).is_err());
    }
}

#[test]
fn parser_rejects_malformed_numeric_limits() {
    let workspace = tempfile::tempdir();
    assert!(workspace.is_ok());
    if let Ok(workspace) = workspace {
        let mut zero_memory = arguments(workspace.path());
        zero_memory[3] = OsString::from("0");
        assert!(SpreadsheetWorkerArguments::parse(zero_memory).is_err());

        let mut invalid_memory = arguments(workspace.path());
        invalid_memory[3] = OsString::from("invalid");
        assert!(SpreadsheetWorkerArguments::parse(invalid_memory).is_err());

        let mut invalid_limit = arguments(workspace.path());
        invalid_limit[5] = OsString::from("invalid");
        assert!(SpreadsheetWorkerArguments::parse(invalid_limit).is_err());

        let mut zero_limit = arguments(workspace.path());
        zero_limit[5] = OsString::from("0");
        assert!(SpreadsheetWorkerArguments::parse(zero_limit).is_err());

        let mut missing_limit = arguments(workspace.path());
        missing_limit.truncate(5);
        assert!(SpreadsheetWorkerArguments::parse(missing_limit).is_err());
    }
}

#[cfg(unix)]
#[test]
fn parser_rejects_non_utf8_numeric_limits() {
    use std::os::unix::ffi::OsStringExt;
    let workspace = tempfile::tempdir();
    assert!(workspace.is_ok());
    if let Ok(workspace) = workspace {
        let mut invalid = arguments(workspace.path());
        invalid[3] = OsString::from_vec(vec![0xff]);
        assert!(SpreadsheetWorkerArguments::parse(invalid).is_err());
    }
}
