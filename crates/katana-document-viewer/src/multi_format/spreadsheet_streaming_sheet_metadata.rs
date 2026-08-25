use super::spreadsheet_engine::{SpreadsheetEngineError, SpreadsheetEngineSupport};
use super::spreadsheet_streaming_xml_values::{
    attribute, attribute_f32, attribute_usize, xml_error,
};
use ironcalc::base::expressions::utils::parse_reference_a1;
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::io::BufRead;

pub(super) struct WorksheetMetadata {
    pub(super) row_count: usize,
    pub(super) column_count: usize,
    pub(super) row_height: f32,
    pub(super) column_width: f32,
    pub(super) frozen_rows: usize,
    pub(super) frozen_columns: usize,
    pub(super) show_grid_lines: bool,
}

impl WorksheetMetadata {
    pub(super) fn read(input: impl BufRead) -> Result<Self, SpreadsheetEngineError> {
        let mut metadata = Self::default();
        let mut reader = Reader::from_reader(input);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(event)) | Ok(Event::Empty(event)) => {
                    if event.local_name().as_ref() == "sheetData" {
                        return Ok(metadata);
                    }
                    metadata.update(&reader, &event)?;
                }
                Ok(Event::Eof) => return Ok(metadata),
                Ok(_) => {}
                Err(error) => return Err(xml_error(error)),
            }
            buffer.clear();
        }
    }

    fn update(
        &mut self,
        reader: &Reader<impl BufRead>,
        event: &BytesStart<'_>,
    ) -> Result<(), SpreadsheetEngineError> {
        match event.local_name().as_ref() {
            "dimension" => self.update_dimension(reader, event)?,
            "sheetFormatPr" => self.update_tracks(reader, event)?,
            "sheetView" => self.update_grid_lines(reader, event)?,
            "pane" => self.update_frozen(reader, event)?,
            _ => {}
        }
        Ok(())
    }

    fn update_dimension(
        &mut self,
        reader: &Reader<impl BufRead>,
        event: &BytesStart<'_>,
    ) -> Result<(), SpreadsheetEngineError> {
        if let Some(reference) = attribute(reader, event, b"ref")? {
            (self.row_count, self.column_count) = dimension_end(&reference)?;
        }
        Ok(())
    }

    fn update_tracks(
        &mut self,
        reader: &Reader<impl BufRead>,
        event: &BytesStart<'_>,
    ) -> Result<(), SpreadsheetEngineError> {
        if let Some(value) = attribute_f32(reader, event, b"defaultRowHeight")? {
            self.row_height = value * 4.0 / 3.0;
        }
        if let Some(value) = attribute_f32(reader, event, b"defaultColWidth")? {
            self.column_width = value * 7.0;
        }
        Ok(())
    }

    fn update_grid_lines(
        &mut self,
        reader: &Reader<impl BufRead>,
        event: &BytesStart<'_>,
    ) -> Result<(), SpreadsheetEngineError> {
        if let Some(value) = attribute(reader, event, b"showGridLines")? {
            self.show_grid_lines = !matches!(value.as_str(), "0" | "false");
        }
        Ok(())
    }

    fn update_frozen(
        &mut self,
        reader: &Reader<impl BufRead>,
        event: &BytesStart<'_>,
    ) -> Result<(), SpreadsheetEngineError> {
        self.frozen_rows = attribute_usize(reader, event, b"ySplit")?.unwrap_or(0);
        self.frozen_columns = attribute_usize(reader, event, b"xSplit")?.unwrap_or(0);
        Ok(())
    }
}

impl Default for WorksheetMetadata {
    fn default() -> Self {
        Self {
            row_count: 1,
            column_count: 1,
            row_height: 20.0,
            column_width: 64.0,
            frozen_rows: 0,
            frozen_columns: 0,
            show_grid_lines: true,
        }
    }
}

fn dimension_end(reference: &str) -> Result<(usize, usize), SpreadsheetEngineError> {
    let end = reference.rsplit_once(':').map_or(reference, |(_, end)| end);
    let parsed = parse_reference_a1(end)
        .ok_or_else(|| SpreadsheetEngineError::Import("invalid worksheet dimension".into()))?;
    Ok((
        SpreadsheetEngineSupport::positive_count(parsed.row)?,
        SpreadsheetEngineSupport::positive_count(parsed.column)?,
    ))
}
