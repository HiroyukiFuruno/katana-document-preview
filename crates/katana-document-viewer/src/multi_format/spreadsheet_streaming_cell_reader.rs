use super::spreadsheet_engine::SpreadsheetEngineError;
use super::spreadsheet_streaming_cell_types::{Capture, CellAccumulator, empty_cell, import_error};
use super::spreadsheet_streaming_xml_values::decode_text;
use super::{SpreadsheetCellArtifact, SpreadsheetCoordinate};
use ironcalc::base::expressions::utils::parse_reference_a1;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};
use std::collections::HashMap;
use std::io::BufRead;

pub(super) struct StreamingCellReader {
    requested: HashMap<SpreadsheetCoordinate, usize>,
    cells: Vec<SpreadsheetCellArtifact>,
    max_row: usize,
    current_row: usize,
    current: Option<CellAccumulator>,
    capture: Capture,
}

impl StreamingCellReader {
    pub(super) fn read(
        input: impl BufRead,
        coordinates: &[SpreadsheetCoordinate],
        shared_strings: &[String],
    ) -> Result<Vec<SpreadsheetCellArtifact>, SpreadsheetEngineError> {
        let Some(mut state) = Self::new(coordinates) else {
            return Ok(Vec::new());
        };
        let mut reader = Reader::from_reader(input);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(event)) => state.start(&reader, &event)?,
                Ok(Event::Text(text)) => state.text(text.as_ref().as_bytes())?,
                Ok(Event::End(event))
                    if state.end(event.local_name().as_ref().as_bytes(), shared_strings) =>
                {
                    return Ok(state.cells);
                }
                Ok(Event::Eof) => return Ok(state.cells),
                Ok(_) => {}
                Err(error) => return Err(import_error(error)),
            }
            buffer.clear();
        }
    }

    fn new(coordinates: &[SpreadsheetCoordinate]) -> Option<Self> {
        let max_row = coordinates.iter().map(|coordinate| coordinate.row).max()?;
        let requested = coordinates
            .iter()
            .copied()
            .enumerate()
            .map(|(index, coordinate)| (coordinate, index))
            .collect();
        Some(Self {
            requested,
            cells: coordinates.iter().copied().map(empty_cell).collect(),
            max_row,
            current_row: 0,
            current: None,
            capture: Capture::None,
        })
    }

    fn start(
        &mut self,
        reader: &Reader<impl BufRead>,
        event: &BytesStart<'_>,
    ) -> Result<(), SpreadsheetEngineError> {
        match event.local_name().as_ref() {
            "row" => self.current_row = explicit_row(reader, event)?.unwrap_or(self.current_row),
            "c" => self.current = requested_cell(reader, event, &self.requested)?,
            "f" if self.current.is_some() => self.capture = Capture::Formula,
            "v" if self.current.is_some() => self.capture = Capture::Value,
            "t" if self.current.is_some() => self.capture = Capture::Text,
            _ => {}
        }
        Ok(())
    }

    fn text(&mut self, bytes: &[u8]) -> Result<(), SpreadsheetEngineError> {
        let Some(cell) = self.current.as_mut() else {
            return Ok(());
        };
        cell.append(self.capture, &decode_text(bytes)?);
        Ok(())
    }

    fn end(&mut self, name: &[u8], shared_strings: &[String]) -> bool {
        match name {
            b"f" | b"v" | b"t" => self.capture = Capture::None,
            b"c" => self.finish_cell(shared_strings),
            b"row" if self.current_row >= self.max_row.saturating_add(1) => return true,
            _ => {}
        }
        false
    }

    fn finish_cell(&mut self, shared_strings: &[String]) {
        let Some(cell) = self.current.take() else {
            return;
        };
        let index = cell.result_index();
        self.cells[index] = cell.finish(shared_strings);
    }
}

fn requested_cell(
    _reader: &Reader<impl BufRead>,
    event: &BytesStart<'_>,
    requested: &HashMap<SpreadsheetCoordinate, usize>,
) -> Result<Option<CellAccumulator>, SpreadsheetEngineError> {
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
    let Some(coordinate) = reference.as_deref().and_then(coordinate) else {
        return Ok(None);
    };
    Ok(requested
        .get(&coordinate)
        .copied()
        .map(|index| CellAccumulator::new(index, coordinate, cell_type)))
}

fn coordinate(reference: &str) -> Option<SpreadsheetCoordinate> {
    let parsed = parse_reference_a1(reference)?;
    Some(SpreadsheetCoordinate::new(
        usize::try_from(parsed.row.saturating_sub(1)).ok()?,
        usize::try_from(parsed.column.saturating_sub(1)).ok()?,
    ))
}

fn explicit_row(
    _reader: &Reader<impl BufRead>,
    event: &BytesStart<'_>,
) -> Result<Option<usize>, SpreadsheetEngineError> {
    for attribute in event.attributes() {
        let attribute = attribute.map_err(import_error)?;
        if attribute.key.local_name().as_ref() == "r" {
            let value = attribute
                .normalized_value(XmlVersion::Implicit1_0)
                .map_err(import_error)?
                .parse()
                .map_err(import_error)?;
            return Ok(Some(value));
        }
    }
    Ok(None)
}
