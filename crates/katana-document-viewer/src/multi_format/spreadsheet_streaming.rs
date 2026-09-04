use super::spreadsheet_engine::{SpreadsheetEngineError, SpreadsheetEngineSupport};
use super::spreadsheet_streaming_cells::StreamingCellMaterializer;
use super::spreadsheet_streaming_xml::{
    WorkbookSheet, parse_relationships, parse_workbook_sheets, read_zip_entry, worksheet_artifact,
};
use super::spreadsheet_streaming_xml_values::parse_shared_strings;
use super::{
    SpreadsheetCellArtifact, SpreadsheetCoordinate, SpreadsheetSheetArtifact,
    SpreadsheetViewerLimits,
};
use std::collections::HashMap;
use std::io::Cursor;
use zip::ZipArchive;
use zip::result::ZipError;

#[path = "spreadsheet_streaming_filter_reader.rs"]
mod filter_reader;

const STREAMING_WORKSHEET_THRESHOLD: u64 = 128 * 1024 * 1024;
const METADATA_ENTRY_LIMIT: u64 = 16 * 1024 * 1024;
type WorkbookMetadata = (Vec<WorkbookSheet>, HashMap<String, String>);
type StreamingSheets = (Vec<StreamingSheet>, Vec<SpreadsheetSheetArtifact>);

pub(super) struct StreamingSpreadsheetSession {
    bytes: Vec<u8>,
    sheets: Vec<StreamingSheet>,
    artifacts: Vec<SpreadsheetSheetArtifact>,
    shared_strings: Vec<String>,
    #[cfg(test)]
    filter_grid_scans: std::cell::Cell<usize>,
}

struct StreamingSheet {
    path: String,
}

impl StreamingSpreadsheetSession {
    pub(super) fn is_required(bytes: &[u8]) -> Result<bool, SpreadsheetEngineError> {
        let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(zip_error)?;
        for index in 0..archive.len() {
            let entry = archive.by_index(index).map_err(zip_error)?;
            if worksheet_requires_streaming(entry.name(), entry.size()) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(super) fn open(
        bytes: Vec<u8>,
        limits: SpreadsheetViewerLimits,
    ) -> Result<Self, SpreadsheetEngineError> {
        let mut archive = ZipArchive::new(Cursor::new(bytes.as_slice())).map_err(zip_error)?;
        let (workbook_sheets, relationship_targets) = workbook_metadata(&mut archive)?;
        SpreadsheetEngineSupport::check_limit(
            "sheet_count",
            workbook_sheets.len(),
            limits.max_sheets,
        )?;
        let (sheets, artifacts) = build_sheets(
            &mut archive,
            workbook_sheets,
            &relationship_targets,
            limits.max_logical_cells,
        )?;
        let shared_strings = read_shared_strings(&mut archive)?;
        drop(archive);
        Ok(Self {
            bytes,
            sheets,
            artifacts,
            shared_strings,
            #[cfg(test)]
            filter_grid_scans: std::cell::Cell::new(0),
        })
    }

    pub(super) fn sheets(&self) -> &[SpreadsheetSheetArtifact] {
        &self.artifacts
    }

    pub(super) fn materialize(
        &self,
        sheet_index: usize,
        coordinates: &[SpreadsheetCoordinate],
    ) -> Result<Vec<SpreadsheetCellArtifact>, SpreadsheetEngineError> {
        let sheet =
            self.sheets
                .get(sheet_index)
                .ok_or(SpreadsheetEngineError::SheetOutsideDocument {
                    requested: sheet_index,
                    sheet_count: self.sheets.len(),
                })?;
        StreamingCellMaterializer::materialize(
            &self.bytes,
            &sheet.path,
            coordinates,
            &self.shared_strings,
        )
    }
}

fn workbook_metadata(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
) -> Result<WorkbookMetadata, SpreadsheetEngineError> {
    let workbook = read_zip_entry(archive, "xl/workbook.xml", METADATA_ENTRY_LIMIT)?;
    let relationships =
        read_zip_entry(archive, "xl/_rels/workbook.xml.rels", METADATA_ENTRY_LIMIT)?;
    Ok((
        parse_workbook_sheets(&workbook)?,
        parse_relationships(&relationships)?,
    ))
}

fn build_sheets(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    workbook_sheets: Vec<WorkbookSheet>,
    relationship_targets: &HashMap<String, String>,
    max_logical_cells: usize,
) -> Result<StreamingSheets, SpreadsheetEngineError> {
    let mut logical_cells = 0_usize;
    let mut sheets = Vec::with_capacity(workbook_sheets.len());
    let mut artifacts = Vec::with_capacity(workbook_sheets.len());
    for (index, sheet) in workbook_sheets.into_iter().enumerate() {
        let path = worksheet_path(&sheet, relationship_targets)?;
        let artifact = worksheet_artifact(archive, &path, index, sheet.name, max_logical_cells)?;
        logical_cells =
            logical_cells.saturating_add(artifact.row_count.saturating_mul(artifact.column_count));
        SpreadsheetEngineSupport::check_limit(
            "logical_cell_count",
            logical_cells,
            max_logical_cells,
        )?;
        sheets.push(StreamingSheet { path });
        artifacts.push(artifact);
    }
    Ok((sheets, artifacts))
}

fn read_shared_strings(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
) -> Result<Vec<String>, SpreadsheetEngineError> {
    let mut entry = match archive.by_name("xl/sharedStrings.xml") {
        Ok(entry) => entry,
        Err(ZipError::FileNotFound) => return Ok(Vec::new()),
        Err(error) => return Err(zip_error(error)),
    };
    SpreadsheetEngineSupport::check_limit(
        "spreadsheet_metadata_bytes",
        entry.size() as usize,
        METADATA_ENTRY_LIMIT as usize,
    )?;
    let mut xml = Vec::with_capacity(entry.size() as usize);
    std::io::Read::read_to_end(&mut entry, &mut xml)
        .map_err(|error| SpreadsheetEngineError::Import(error.to_string()))?;
    parse_shared_strings(&xml)
}

fn worksheet_path(
    sheet: &WorkbookSheet,
    targets: &HashMap<String, String>,
) -> Result<String, SpreadsheetEngineError> {
    targets
        .get(&sheet.relationship_id)
        .map(|target| {
            let target = target.trim_start_matches('/').trim_start_matches("./");
            if target.starts_with("xl/") {
                target.to_owned()
            } else {
                format!("xl/{target}")
            }
        })
        .ok_or_else(|| SpreadsheetEngineError::Import("worksheet relationship is missing".into()))
}

fn zip_error(error: zip::result::ZipError) -> SpreadsheetEngineError {
    SpreadsheetEngineError::Import(error.to_string())
}

fn worksheet_requires_streaming(name: &str, size: u64) -> bool {
    name.starts_with("xl/worksheets/")
        && name.ends_with(".xml")
        && size > STREAMING_WORKSHEET_THRESHOLD
}

#[cfg(test)]
#[path = "spreadsheet_streaming_tests.rs"]
mod tests;
