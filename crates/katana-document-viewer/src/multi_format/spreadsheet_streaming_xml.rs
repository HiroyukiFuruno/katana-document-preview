use super::spreadsheet_engine::{SpreadsheetEngineError, SpreadsheetEngineSupport};
use super::spreadsheet_streaming_sheet_metadata::WorksheetMetadata;
use super::spreadsheet_streaming_xml_values::{attribute, required_attribute, xml_error};
use super::{SpreadsheetSheetArtifact, SpreadsheetTrackArtifact};
use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::HashMap;
use std::io::{BufReader, Cursor, Read};
use zip::ZipArchive;

pub(super) struct WorkbookSheet {
    pub(super) name: String,
    pub(super) relationship_id: String,
}

pub(super) fn read_zip_entry(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    name: &str,
    limit: u64,
) -> Result<Vec<u8>, SpreadsheetEngineError> {
    let mut entry = archive
        .by_name(name)
        .map_err(|error| SpreadsheetEngineError::Import(error.to_string()))?;
    if entry.size() > limit {
        return Err(SpreadsheetEngineError::ResourceLimit {
            kind: "spreadsheet_metadata_bytes",
            actual: entry.size() as usize,
            limit: limit as usize,
        });
    }
    let mut bytes = Vec::with_capacity(entry.size() as usize);
    entry
        .read_to_end(&mut bytes)
        .map_err(|error| SpreadsheetEngineError::Import(error.to_string()))?;
    Ok(bytes)
}

pub(super) fn parse_workbook_sheets(
    xml: &[u8],
) -> Result<Vec<WorkbookSheet>, SpreadsheetEngineError> {
    let mut reader = Reader::from_reader(xml);
    let mut sheets = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) | Ok(Event::Empty(event))
                if event.local_name().as_ref() == "sheet" =>
            {
                sheets.push(WorkbookSheet {
                    name: required_attribute(&reader, &event, b"name")?,
                    relationship_id: required_attribute(&reader, &event, b"id")?,
                });
            }
            Ok(Event::Eof) => return Ok(sheets),
            Ok(_) => {}
            Err(error) => return Err(xml_error(error)),
        }
    }
}

pub(super) fn parse_relationships(
    xml: &[u8],
) -> Result<HashMap<String, String>, SpreadsheetEngineError> {
    let mut reader = Reader::from_reader(xml);
    let mut relationships = HashMap::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) | Ok(Event::Empty(event))
                if event.local_name().as_ref() == "Relationship" =>
            {
                if let (Some(id), Some(target)) = (
                    attribute(&reader, &event, b"Id")?,
                    attribute(&reader, &event, b"Target")?,
                ) {
                    relationships.insert(id, target);
                }
            }
            Ok(Event::Eof) => return Ok(relationships),
            Ok(_) => {}
            Err(error) => return Err(xml_error(error)),
        }
    }
}

pub(super) fn worksheet_artifact(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    path: &str,
    index: usize,
    name: String,
    max_logical_cells: usize,
) -> Result<SpreadsheetSheetArtifact, SpreadsheetEngineError> {
    let entry = archive
        .by_name(path)
        .map_err(|error| SpreadsheetEngineError::Import(error.to_string()))?;
    let metadata = WorksheetMetadata::read(BufReader::new(entry))?;
    SpreadsheetEngineSupport::check_limit(
        "logical_cell_count",
        metadata.row_count.saturating_mul(metadata.column_count),
        max_logical_cells,
    )?;
    Ok(sheet_artifact(index, name, metadata))
}

fn sheet_artifact(
    index: usize,
    name: String,
    metadata: WorksheetMetadata,
) -> SpreadsheetSheetArtifact {
    SpreadsheetSheetArtifact {
        index,
        name,
        row_count: metadata.row_count,
        column_count: metadata.column_count,
        row_tracks: tracks(metadata.row_count, metadata.row_height),
        column_tracks: tracks(metadata.column_count, metadata.column_width),
        frozen_rows: metadata.frozen_rows,
        frozen_columns: metadata.frozen_columns,
        merged_cells: Vec::new(),
        show_grid_lines: metadata.show_grid_lines,
        auto_filter: None,
    }
}

fn tracks(count: usize, size: f32) -> Vec<SpreadsheetTrackArtifact> {
    vec![
        SpreadsheetTrackArtifact {
            size,
            hidden: false
        };
        count
    ]
}
