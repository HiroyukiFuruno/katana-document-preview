use super::super::{StreamingSpreadsheetSession, zip_error};
use super::StreamingFilterGridReader;
use crate::multi_format::SpreadsheetCellArtifact;
use crate::multi_format::spreadsheet_engine::SpreadsheetEngineError;
use std::io::{BufReader, Cursor};
use zip::ZipArchive;

pub(super) fn read_grid<Visitor>(
    session: &StreamingSpreadsheetSession,
    sheet_index: usize,
    columns: &[usize],
    rows: std::ops::Range<usize>,
    chunk_rows: usize,
    visitor: Visitor,
) -> Result<(), SpreadsheetEngineError>
where
    Visitor: FnMut(
        std::ops::Range<usize>,
        Vec<SpreadsheetCellArtifact>,
    ) -> Result<(), SpreadsheetEngineError>,
{
    let path = filter_sheet_path(session, sheet_index)?;
    let mut archive = ZipArchive::new(Cursor::new(session.bytes.as_slice())).map_err(zip_error)?;
    let entry = archive.by_name(path).map_err(zip_error)?;
    StreamingFilterGridReader::read(
        BufReader::new(entry),
        columns,
        rows,
        chunk_rows,
        &session.shared_strings,
        visitor,
    )
}

fn filter_sheet_path(
    session: &StreamingSpreadsheetSession,
    sheet_index: usize,
) -> Result<&str, SpreadsheetEngineError> {
    session
        .sheets
        .get(sheet_index)
        .map(|sheet| sheet.path.as_str())
        .ok_or(SpreadsheetEngineError::SheetOutsideDocument {
            requested: sheet_index,
            sheet_count: session.sheets.len(),
        })
}
