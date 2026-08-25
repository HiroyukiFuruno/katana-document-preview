use super::spreadsheet_engine::SpreadsheetEngineError;

pub(crate) struct SpreadsheetEngineSupport;

impl SpreadsheetEngineSupport {
    pub(crate) fn check_limit(
        kind: &'static str,
        actual: usize,
        limit: usize,
    ) -> Result<(), SpreadsheetEngineError> {
        if actual <= limit {
            return Ok(());
        }
        Err(SpreadsheetEngineError::ResourceLimit {
            kind,
            actual,
            limit,
        })
    }

    pub(crate) fn track_size(size: f64) -> f32 {
        if size.is_finite() && size > 0.0 {
            size.min(f64::from(f32::MAX)) as f32
        } else {
            0.0
        }
    }

    pub(crate) fn positive_count(value: i32) -> Result<usize, SpreadsheetEngineError> {
        usize::try_from(value.max(1)).map_err(Self::model_error)
    }

    pub(crate) fn non_negative(value: i32) -> Result<usize, SpreadsheetEngineError> {
        usize::try_from(value.max(0)).map_err(Self::model_error)
    }

    pub(crate) fn zero_based(value: i32) -> Result<usize, SpreadsheetEngineError> {
        usize::try_from(value.saturating_sub(1)).map_err(Self::model_error)
    }

    pub(crate) fn span(start: i32, end: i32) -> Result<usize, SpreadsheetEngineError> {
        usize::try_from(end.saturating_sub(start).saturating_add(1)).map_err(Self::model_error)
    }

    pub(crate) fn engine_index(index: usize) -> Result<i32, String> {
        match i32::try_from(index.saturating_add(1)) {
            Ok(index) => Ok(index),
            Err(error) => Err(format!("cell index conversion failed: {error}")),
        }
    }

    pub(crate) fn model_error(error: std::num::TryFromIntError) -> SpreadsheetEngineError {
        SpreadsheetEngineError::Model(error.to_string())
    }

    pub(crate) fn engine_error(error: String) -> SpreadsheetEngineError {
        SpreadsheetEngineError::Model(error)
    }
}
