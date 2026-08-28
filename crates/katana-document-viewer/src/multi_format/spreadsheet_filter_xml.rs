use super::spreadsheet_engine::{SpreadsheetEngineError, SpreadsheetEngineSupport};
use super::spreadsheet_filter_xml_parser::parse_worksheet;
use super::spreadsheet_streaming_xml::{
    WorkbookSheet, parse_relationships, parse_workbook_sheets, read_zip_entry,
};
use super::{SpreadsheetAutoFilterArtifact, SpreadsheetSheetArtifact};
use std::collections::HashMap;
use std::io::{BufReader, Cursor};
use zip::ZipArchive;

const METADATA_ENTRY_LIMIT: u64 = 16 * 1024 * 1024;

pub(super) struct SpreadsheetFilterCatalog;

impl SpreadsheetFilterCatalog {
    pub(super) fn read(
        bytes: &[u8],
        max_sheets: usize,
    ) -> Result<Vec<Option<SpreadsheetAutoFilterArtifact>>, SpreadsheetEngineError> {
        let mut archive = match ZipArchive::new(Cursor::new(bytes)) {
            Ok(archive) => archive,
            Err(error) => return Err(SpreadsheetEngineError::Import(error.to_string())),
        };
        let workbook = read_zip_entry(&mut archive, "xl/workbook.xml", METADATA_ENTRY_LIMIT)?;
        let relationships = read_zip_entry(
            &mut archive,
            "xl/_rels/workbook.xml.rels",
            METADATA_ENTRY_LIMIT,
        )?;
        let sheets = parse_workbook_sheets(&workbook)?;
        SpreadsheetEngineSupport::check_limit("sheet_count", sheets.len(), max_sheets)?;
        let targets = parse_relationships(&relationships)?;
        sheets
            .iter()
            .map(|sheet| Self::read_sheet(&mut archive, sheet, &targets))
            .collect()
    }

    pub(super) fn attach(
        sheets: &mut [SpreadsheetSheetArtifact],
        filters: Vec<Option<SpreadsheetAutoFilterArtifact>>,
    ) {
        for (sheet, filter) in sheets.iter_mut().zip(filters) {
            sheet.auto_filter = filter;
        }
    }

    fn read_sheet(
        archive: &mut ZipArchive<Cursor<&[u8]>>,
        sheet: &WorkbookSheet,
        targets: &HashMap<String, String>,
    ) -> Result<Option<SpreadsheetAutoFilterArtifact>, SpreadsheetEngineError> {
        let path = worksheet_path(sheet, targets)?;
        let entry = match archive.by_name(&path) {
            Ok(entry) => entry,
            Err(error) => return Err(SpreadsheetEngineError::Import(error.to_string())),
        };
        parse_worksheet(BufReader::new(entry))
    }
}

fn worksheet_path(
    sheet: &WorkbookSheet,
    targets: &HashMap<String, String>,
) -> Result<String, SpreadsheetEngineError> {
    let target = match targets.get(&sheet.relationship_id) {
        Some(target) => target,
        None => {
            return Err(SpreadsheetEngineError::Import(
                "worksheet relationship is missing".into(),
            ));
        }
    };
    let target = target.trim_start_matches('/').trim_start_matches("./");
    if target.starts_with("xl/") {
        Ok(target.to_owned())
    } else {
        Ok(format!("xl/{target}"))
    }
}
