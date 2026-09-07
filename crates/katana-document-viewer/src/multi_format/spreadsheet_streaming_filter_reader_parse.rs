use super::{FilterGridVisitor, StreamingFilterGridReader};
use crate::multi_format::SpreadsheetCoordinate;
use crate::multi_format::spreadsheet_engine::SpreadsheetEngineError;
use crate::multi_format::spreadsheet_streaming_cell_types::{
    Capture, CellAccumulator, import_error,
};
use crate::multi_format::spreadsheet_streaming_xml_values::decode_text;
use ironcalc::base::expressions::utils::parse_reference_a1;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};
use std::io::BufRead;

impl<'a, 'visitor> StreamingFilterGridReader<'a, 'visitor> {
    pub(super) fn read(
        input: &mut dyn BufRead,
        columns: &'a [usize],
        rows: std::ops::Range<usize>,
        chunk_rows: usize,
        shared_strings: &'a [String],
        visitor: &'visitor mut FilterGridVisitor<'visitor>,
    ) -> Result<(), SpreadsheetEngineError> {
        let mut state = Self::new(columns, rows, chunk_rows, shared_strings, visitor);
        let mut reader = Reader::from_reader(input);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(event)) => state.start(&event)?,
                Ok(Event::Text(text)) => state.text(text.as_ref().as_bytes())?,
                Ok(Event::End(event)) => state.end(event.local_name().as_ref().as_bytes()),
                Ok(Event::Eof) => return state.finish(),
                Ok(_) => {}
                Err(error) => return Err(import_error(error)),
            }
            buffer.clear();
        }
    }

    fn start(&mut self, event: &BytesStart<'_>) -> Result<(), SpreadsheetEngineError> {
        match event.local_name().as_ref() {
            "row" => self.start_row(event)?,
            "c" => self.current = self.requested_cell(event)?,
            "f" if self.current.is_some() => self.capture = Capture::Formula,
            "v" if self.current.is_some() => self.capture = Capture::Value,
            "t" if self.current.is_some() => self.capture = Capture::Text,
            _ => {}
        }
        Ok(())
    }

    fn start_row(&mut self, event: &BytesStart<'_>) -> Result<(), SpreadsheetEngineError> {
        let row = filter_row(event)?.unwrap_or(self.current_row);
        self.current_row = row.saturating_add(1);
        self.flush_before(row)
    }

    fn text(&mut self, bytes: &[u8]) -> Result<(), SpreadsheetEngineError> {
        let Some(cell) = self.current.as_mut() else {
            return Ok(());
        };
        cell.append(self.capture, &decode_text(bytes)?);
        Ok(())
    }

    fn end(&mut self, name: &[u8]) {
        match name {
            b"f" | b"v" | b"t" => self.capture = Capture::None,
            b"c" => self.finish_cell(),
            _ => {}
        }
    }

    fn requested_cell(
        &mut self,
        event: &BytesStart<'_>,
    ) -> Result<Option<CellAccumulator>, SpreadsheetEngineError> {
        let (coordinate, cell_type) = cell_attributes(event)?;
        let Some(coordinate) = coordinate else {
            return Ok(None);
        };
        let Some(column_index) = self.requested_column(coordinate) else {
            return Ok(None);
        };
        self.ensure_chunk_cells();
        let index = self.cell_result_index(coordinate.row, column_index);
        Ok(Some(CellAccumulator::new(index, coordinate, cell_type)))
    }

    fn requested_column(&self, coordinate: SpreadsheetCoordinate) -> Option<usize> {
        if coordinate.row < self.next_chunk_start || coordinate.row >= self.chunk_end() {
            return None;
        }
        self.columns
            .iter()
            .position(|column| *column == coordinate.column)
    }

    fn cell_result_index(&self, row: usize, column_index: usize) -> usize {
        (row - self.next_chunk_start)
            .saturating_mul(self.columns.len())
            .saturating_add(column_index)
    }
}

fn cell_attributes(
    event: &BytesStart<'_>,
) -> Result<(Option<SpreadsheetCoordinate>, String), SpreadsheetEngineError> {
    let mut reference = None;
    let mut cell_type = String::new();
    for attribute in event.attributes() {
        let attribute = attribute.map_err(import_error)?;
        let value = attribute
            .normalized_value(XmlVersion::Implicit1_0)
            .map_err(import_error)?
            .into_owned();
        match attribute.key.local_name().as_ref() {
            "r" => reference = Some(value),
            "t" => cell_type = value,
            _ => {}
        }
    }
    Ok((reference.as_deref().and_then(filter_coordinate), cell_type))
}

fn filter_coordinate(reference: &str) -> Option<SpreadsheetCoordinate> {
    let parsed = parse_reference_a1(reference)?;
    Some(SpreadsheetCoordinate::new(
        usize::try_from(parsed.row.saturating_sub(1)).ok()?,
        usize::try_from(parsed.column.saturating_sub(1)).ok()?,
    ))
}

fn filter_row(event: &BytesStart<'_>) -> Result<Option<usize>, SpreadsheetEngineError> {
    let mut one_based_row = None;
    for attribute in event.attributes() {
        let attribute = attribute.map_err(import_error)?;
        if attribute.key.local_name().as_ref() == "r" {
            one_based_row = Some(
                attribute
                    .normalized_value(XmlVersion::Implicit1_0)
                    .map_err(import_error)?
                    .parse::<usize>()
                    .map_err(import_error)?,
            );
            break;
        }
    }
    Ok(one_based_row.and_then(|row| row.checked_sub(1)))
}

#[cfg(test)]
#[path = "spreadsheet_streaming_filter_reader_parse_tests.rs"]
mod tests;
