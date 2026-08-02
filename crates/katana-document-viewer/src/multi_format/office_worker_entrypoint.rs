use super::OfficeDocumentFormat;
use super::office_worker_constraints::OfficeWorkerConstraints;
use super::office_worker_protocol::{INPUT_NAME, OUTPUT_NAME, OfficeWorkerResponse};
use super::spreadsheet_worker_entrypoint::SpreadsheetWorkerEntrypoint;
use super::spreadsheet_worker_protocol::SPREADSHEET_MODE;
use office2pdf::config::{ConvertOptions, Format};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[path = "office_worker_failure.rs"]
mod failure;
use failure::{engine_failure, input_failure, output_failure};

#[path = "office_worker_response_writer.rs"]
mod response_writer;
use response_writer::write_response;
#[cfg(test)]
use response_writer::write_response_with;

const EXIT_USAGE: i32 = 64;
const EXIT_FAILURE: i32 = 70;

#[derive(Debug)]
struct WorkerArguments {
    workspace: PathBuf,
    format: OfficeDocumentFormat,
    max_memory_bytes: u64,
    max_cpu_seconds: u64,
    max_output_bytes: u64,
}

type OfficeExecutor = fn(&WorkerArguments) -> Result<OfficeWorkerResponse, (String, String)>;
type ConstraintApplier = fn(&Path, u64, u64) -> Result<(), (String, String)>;

pub struct OfficeWorkerEntrypoint;

impl OfficeWorkerEntrypoint {
    #[must_use]
    pub fn run_from_env() -> i32 {
        Self::run(std::env::args_os().collect())
    }

    #[must_use]
    pub fn run(arguments: Vec<OsString>) -> i32 {
        if arguments.get(1).and_then(|value| value.to_str()) == Some(SPREADSHEET_MODE) {
            return SpreadsheetWorkerEntrypoint::run(arguments);
        }
        Self::run_office(arguments, execute)
    }

    fn run_office(arguments: Vec<OsString>, executor: OfficeExecutor) -> i32 {
        let arguments = match parse_arguments(arguments) {
            Ok(arguments) => arguments,
            Err(message) => {
                eprintln!("KDV office worker usage error: {message}");
                return EXIT_USAGE;
            }
        };
        match executor(&arguments) {
            Ok(response) => write_response(&arguments.workspace, &response, 0),
            Err((stage, message)) => write_response(
                &arguments.workspace,
                &OfficeWorkerResponse::Failed { stage, message },
                EXIT_FAILURE,
            ),
        }
    }
}

fn execute(arguments: &WorkerArguments) -> Result<OfficeWorkerResponse, (String, String)> {
    execute_with_constraints(arguments, OfficeWorkerConstraints::apply)
}

fn execute_with_constraints(
    arguments: &WorkerArguments,
    apply_constraints: ConstraintApplier,
) -> Result<OfficeWorkerResponse, (String, String)> {
    apply_constraints(
        &arguments.workspace,
        arguments.max_memory_bytes,
        arguments.max_cpu_seconds,
    )?;
    let input = std::fs::read(arguments.workspace.join(INPUT_NAME)).map_err(input_failure)?;
    let result = convert_document(arguments, &input)?;
    validate_output_size(arguments, result.pdf.len())?;
    std::fs::write(arguments.workspace.join(OUTPUT_NAME), result.pdf).map_err(output_failure)?;
    Ok(OfficeWorkerResponse::Completed {
        warnings: result
            .warnings
            .into_iter()
            .map(|warning| warning.to_string())
            .collect(),
    })
}

fn convert_document(
    arguments: &WorkerArguments,
    input: &[u8],
) -> Result<office2pdf::error::ConvertResult, (String, String)> {
    office2pdf::convert_bytes(
        input,
        engine_format(arguments.format),
        &ConvertOptions::default(),
    )
    .map_err(engine_failure)
}

fn validate_output_size(
    arguments: &WorkerArguments,
    output_bytes: usize,
) -> Result<(), (String, String)> {
    if output_bytes as u64 > arguments.max_output_bytes {
        return Err((
            "output_limit".to_owned(),
            format!(
                "converted PDF is {} bytes and exceeds {} bytes",
                output_bytes, arguments.max_output_bytes
            ),
        ));
    }
    Ok(())
}

fn parse_arguments(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<WorkerArguments, String> {
    let mut values = arguments.into_iter();
    let _program = values.next();
    let workspace = values
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "workspace argument is missing".to_owned())?;
    let format = parse_format(values.next())?;
    let max_memory_bytes = parse_u64(values.next(), "max memory")?;
    let max_cpu_seconds = parse_u64(values.next(), "max CPU seconds")?;
    let max_output_bytes = parse_u64(values.next(), "max output bytes")?;
    if values.next().is_some() {
        return Err("unexpected trailing arguments".to_owned());
    }
    if !workspace.is_absolute() {
        return Err("workspace must be absolute".to_owned());
    }
    Ok(WorkerArguments {
        workspace,
        format,
        max_memory_bytes,
        max_cpu_seconds,
        max_output_bytes,
    })
}

fn parse_format(value: Option<OsString>) -> Result<OfficeDocumentFormat, String> {
    match value.as_deref().and_then(|value| value.to_str()) {
        Some("docx") => Ok(OfficeDocumentFormat::Docx),
        Some("pptx") => Ok(OfficeDocumentFormat::Pptx),
        Some(value) => Err(format!("unsupported worker format `{value}`")),
        None => Err("format argument is missing or invalid UTF-8".to_owned()),
    }
}

fn parse_u64(value: Option<OsString>, name: &str) -> Result<u64, String> {
    let value = match value {
        Some(value) => match value.into_string() {
            Ok(value) => value,
            Err(_) => return Err(format!("{name} argument is missing or invalid UTF-8")),
        },
        None => return Err(format!("{name} argument is missing or invalid UTF-8")),
    };
    let parsed = match value.parse::<u64>() {
        Ok(parsed) => parsed,
        Err(_) => return Err(format!("{name} argument is not an unsigned integer")),
    };
    if parsed == 0 {
        return Err(format!("{name} must be greater than zero"));
    }
    Ok(parsed)
}

const fn engine_format(format: OfficeDocumentFormat) -> Format {
    match format {
        OfficeDocumentFormat::Docx => Format::Docx,
        OfficeDocumentFormat::Pptx => Format::Pptx,
        OfficeDocumentFormat::Xlsx => Format::Xlsx,
    }
}

#[cfg(test)]
#[path = "office_worker_entrypoint_tests.rs"]
mod tests;
