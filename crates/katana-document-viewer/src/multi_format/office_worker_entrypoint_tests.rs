use super::failure::{engine_failure, failure, input_failure, output_failure};
use super::{
    EXIT_FAILURE, EXIT_USAGE, OfficeDocumentFormat, OfficeWorkerEntrypoint, OfficeWorkerResponse,
    WorkerArguments, conversion_options, engine_format, execute_with_constraints, parse_arguments,
    validate_output_size, write_response, write_response_with,
};
use office2pdf::config::Format;
use std::ffi::OsString;

fn arguments(workspace: &std::path::Path) -> Vec<OsString> {
    [
        OsString::from("worker"),
        workspace.as_os_str().to_owned(),
        OsString::from("docx"),
        OsString::from("1024"),
        OsString::from("1"),
        OsString::from("1024"),
    ]
    .into()
}

fn completed(_arguments: &WorkerArguments) -> Result<OfficeWorkerResponse, (String, String)> {
    Ok(OfficeWorkerResponse::Completed {
        warnings: Vec::new(),
    })
}

fn failed(_arguments: &WorkerArguments) -> Result<OfficeWorkerResponse, (String, String)> {
    Err(("engine".to_owned(), "failed".to_owned()))
}

fn constraints_denied(
    _workspace: &std::path::Path,
    _max_memory_bytes: u64,
    _max_cpu_seconds: u64,
) -> Result<(), (String, String)> {
    Err(("sandbox".to_owned(), "denied".to_owned()))
}

fn encoding_failed(_response: &OfficeWorkerResponse) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::from_slice(b"{")
}

#[test]
fn usage_and_executor_failures_return_stable_exit_codes() {
    assert_eq!(
        EXIT_USAGE,
        OfficeWorkerEntrypoint::run_office(vec![OsString::from("worker")], completed)
    );
    let workspace = tempfile::tempdir();
    assert!(workspace.is_ok());
    if let Ok(workspace) = workspace {
        assert_eq!(
            0,
            OfficeWorkerEntrypoint::run_office(arguments(workspace.path()), completed)
        );
        assert_eq!(
            EXIT_FAILURE,
            OfficeWorkerEntrypoint::run_office(arguments(workspace.path()), failed)
        );
    }
}

#[test]
fn argument_parser_rejects_invalid_paths_formats_and_arity() {
    let workspace = tempfile::tempdir();
    assert!(workspace.is_ok());
    if let Ok(workspace) = workspace {
        let mut trailing = arguments(workspace.path());
        trailing.push(OsString::from("extra"));
        assert!(parse_arguments(trailing).is_err());
        let mut relative = arguments(workspace.path());
        relative[1] = OsString::from("relative");
        assert!(parse_arguments(relative).is_err());
        let mut unknown = arguments(workspace.path());
        unknown[2] = OsString::from("xlsx");
        assert!(parse_arguments(unknown).is_err());
        let mut missing = arguments(workspace.path());
        missing.truncate(2);
        assert!(parse_arguments(missing).is_err());
    }
}

#[test]
fn argument_parser_accepts_pptx_and_rejects_invalid_limits() {
    let workspace = tempfile::tempdir();
    assert!(workspace.is_ok());
    if let Ok(workspace) = workspace {
        let mut invalid = arguments(workspace.path());
        invalid[3] = OsString::from("invalid");
        assert!(parse_arguments(invalid).is_err());
        let mut zero = arguments(workspace.path());
        zero[3] = OsString::from("0");
        assert!(parse_arguments(zero).is_err());
        let mut pptx = arguments(workspace.path());
        pptx[2] = OsString::from("pptx");
        let parsed = parse_arguments(pptx);
        assert!(matches!(parsed, Ok(value) if value.format == OfficeDocumentFormat::Pptx));
        let mut missing = arguments(workspace.path());
        missing.truncate(3);
        assert!(parse_arguments(missing).is_err());
    }
}

#[cfg(unix)]
#[test]
fn argument_parser_rejects_invalid_utf8_limits() {
    use std::os::unix::ffi::OsStringExt;
    let workspace = tempfile::tempdir();
    assert!(workspace.is_ok());
    if let Ok(workspace) = workspace {
        let mut invalid = arguments(workspace.path());
        invalid[3] = OsString::from_vec(vec![0xff]);
        assert!(parse_arguments(invalid).is_err());
    }
}

#[test]
fn constraints_output_and_engine_format_failures_are_typed() {
    let workspace = tempfile::tempdir();
    assert!(workspace.is_ok());
    if let Ok(workspace) = workspace {
        let arguments = WorkerArguments {
            workspace: workspace.path().to_path_buf(),
            format: OfficeDocumentFormat::Docx,
            max_memory_bytes: 1,
            max_cpu_seconds: 1,
            max_output_bytes: 1,
        };
        assert!(execute_with_constraints(&arguments, constraints_denied).is_err());
        assert!(validate_output_size(&arguments, 2).is_err());
        assert_eq!(Ok(()), validate_output_size(&arguments, 1));
    }
    assert_eq!(Format::Docx, engine_format(OfficeDocumentFormat::Docx));
    assert_eq!(Format::Pptx, engine_format(OfficeDocumentFormat::Pptx));
    assert_eq!(Format::Xlsx, engine_format(OfficeDocumentFormat::Xlsx));
    let font_path = std::path::PathBuf::from("fonts");
    assert_eq!(
        vec![font_path.clone()],
        conversion_options(font_path).font_paths
    );
}

#[test]
fn failure_helpers_and_response_writer_are_typed() {
    assert_eq!(
        ("input".to_owned(), "missing".to_owned()),
        failure("input", "missing".to_owned())
    );
    assert_eq!("input", input_failure(std::io::Error::other("missing")).0);
    assert_eq!("output", output_failure(std::io::Error::other("blocked")).0);
    assert_eq!(
        "engine",
        engine_failure(office2pdf::error::ConvertError::Parse("invalid".to_owned())).0
    );
    let response = OfficeWorkerResponse::Completed {
        warnings: Vec::new(),
    };
    let workspace = tempfile::tempdir();
    assert!(workspace.is_ok());
    if let Ok(workspace) = workspace {
        assert_eq!(
            EXIT_FAILURE,
            write_response_with(workspace.path(), &response, 0, encoding_failed)
        );
    }
    let not_directory = tempfile::NamedTempFile::new();
    assert!(not_directory.is_ok());
    if let Ok(file) = not_directory {
        assert_eq!(EXIT_FAILURE, write_response(file.path(), &response, 0));
    }
}
