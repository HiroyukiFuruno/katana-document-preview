use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum SpreadsheetEngineError {
    #[error("XLSX import failed: {0}")]
    Import(String),
    #[error("spreadsheet model failed: {0}")]
    Model(String),
    #[error("spreadsheet resource limit `{kind}` exceeded: {actual} > {limit}")]
    ResourceLimit {
        kind: &'static str,
        actual: usize,
        limit: usize,
    },
    #[error("invalid merged-cell range `{0}`")]
    InvalidMergedCell(String),
    #[error("sheet index {requested} is outside the {sheet_count}-sheet workbook")]
    SheetOutsideDocument {
        requested: usize,
        sheet_count: usize,
    },
    #[error("cell ({row}, {column}) is outside sheet {sheet_index}")]
    CellOutsideSheet {
        sheet_index: usize,
        row: usize,
        column: usize,
    },
    #[error("cell ({row}, {column}) was requested more than once")]
    DuplicateCell { row: usize, column: usize },
    #[error("sheet {sheet_index} does not define an AutoFilter range")]
    FilterUnavailable { sheet_index: usize },
    #[error("column {column} is outside the AutoFilter range on sheet {sheet_index}")]
    FilterColumnOutsideRange { sheet_index: usize, column: usize },
    #[error("spreadsheet filter value limit exceeded: {actual} > {limit}")]
    FilterValueLimit { actual: usize, limit: usize },
}
