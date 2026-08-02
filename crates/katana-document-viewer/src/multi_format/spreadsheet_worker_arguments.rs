use super::SpreadsheetViewerLimits;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct SpreadsheetWorkerArguments {
    pub(crate) workspace: PathBuf,
    pub(crate) max_memory_bytes: u64,
    pub(crate) max_cpu_seconds: u64,
    pub(crate) limits: SpreadsheetViewerLimits,
}

impl SpreadsheetWorkerArguments {
    pub(crate) fn parse(arguments: Vec<OsString>) -> Result<Self, String> {
        let mut values = arguments.into_iter();
        let _program = values.next();
        let _spreadsheet_mode = values.next();
        let workspace = Self::workspace(values.next())?;
        let max_memory_bytes = Self::positive_u64(values.next(), "max memory")?;
        let max_cpu_seconds = Self::positive_u64(values.next(), "max CPU seconds")?;
        let max_sheets = Self::positive_usize(values.next(), "max sheets")?;
        let max_logical_cells = Self::positive_usize(values.next(), "max logical cells")?;
        let max_materialized_cells = Self::positive_usize(values.next(), "max materialized cells")?;
        if values.next().is_some() {
            return Err("unexpected trailing arguments".to_owned());
        }
        Ok(Self {
            workspace,
            max_memory_bytes,
            max_cpu_seconds,
            limits: SpreadsheetViewerLimits {
                max_sheets,
                max_logical_cells,
                max_materialized_cells,
            },
        })
    }

    fn workspace(value: Option<OsString>) -> Result<PathBuf, String> {
        let path = match value {
            Some(value) => PathBuf::from(value),
            None => return Err("workspace argument is missing".to_owned()),
        };
        if path.is_absolute() {
            Ok(path)
        } else {
            Err("workspace must be absolute".to_owned())
        }
    }

    fn positive_u64(value: Option<OsString>, name: &str) -> Result<u64, String> {
        let value = Self::utf8(value, name)?;
        let parsed = match value.parse::<u64>() {
            Ok(parsed) => parsed,
            Err(_) => return Err(format!("{name} is not an unsigned integer")),
        };
        if parsed == 0 {
            Err(format!("{name} must be greater than zero"))
        } else {
            Ok(parsed)
        }
    }

    fn positive_usize(value: Option<OsString>, name: &str) -> Result<usize, String> {
        let value = Self::utf8(value, name)?;
        let parsed = match value.parse::<usize>() {
            Ok(parsed) => parsed,
            Err(_) => return Err(format!("{name} is not an unsigned integer")),
        };
        if parsed == 0 {
            Err(format!("{name} must be greater than zero"))
        } else {
            Ok(parsed)
        }
    }

    fn utf8(value: Option<OsString>, name: &str) -> Result<String, String> {
        match value {
            Some(value) => match value.into_string() {
                Ok(value) => Ok(value),
                Err(_) => Err(format!("{name} is missing or invalid UTF-8")),
            },
            None => Err(format!("{name} is missing or invalid UTF-8")),
        }
    }
}

#[cfg(test)]
#[path = "spreadsheet_worker_arguments_tests.rs"]
mod tests;
