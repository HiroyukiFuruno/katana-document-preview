use super::spreadsheet_engine::SpreadsheetEngineError;
use super::spreadsheet_streaming_cell_reader::StreamingCellReader;
use super::spreadsheet_streaming_cell_types::import_error;
use super::{SpreadsheetCellArtifact, SpreadsheetCoordinate};
use std::io::{BufReader, Cursor};
use zip::ZipArchive;

pub(super) struct StreamingCellMaterializer;

impl StreamingCellMaterializer {
    pub(super) fn materialize(
        bytes: &[u8],
        path: &str,
        coordinates: &[SpreadsheetCoordinate],
        shared_strings: &[String],
    ) -> Result<Vec<SpreadsheetCellArtifact>, SpreadsheetEngineError> {
        let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(import_error)?;
        let entry = archive.by_name(path).map_err(import_error)?;
        StreamingCellReader::read(BufReader::new(entry), coordinates, shared_strings)
    }
}
