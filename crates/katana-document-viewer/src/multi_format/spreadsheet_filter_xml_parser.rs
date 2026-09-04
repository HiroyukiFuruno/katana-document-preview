use super::spreadsheet_engine::{SpreadsheetEngineError, SpreadsheetEngineSupport};
use super::spreadsheet_streaming_xml_values::{attribute, attribute_usize, xml_error};
use super::{
    SpreadsheetAutoFilterArtifact, SpreadsheetCoordinate, SpreadsheetFilterColumnArtifact,
    SpreadsheetFilterCriterion, SpreadsheetFilterRange,
};
use ironcalc::base::expressions::utils::parse_reference_a1;
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::io::BufRead;

pub(super) fn parse_worksheet(
    input: impl BufRead,
) -> Result<Option<SpreadsheetAutoFilterArtifact>, SpreadsheetEngineError> {
    let mut parser = FilterParser::default();
    let mut reader = Reader::from_reader(input);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(event)) => parser.start(&reader, &event)?,
            Ok(Event::Empty(event)) => parser.empty(&reader, &event)?,
            Ok(Event::End(event)) if event.local_name().as_ref() == "filterColumn" => {
                parser.finish_column();
            }
            Ok(Event::Eof) => return Ok(parser.filter),
            Ok(_) => {}
            Err(error) => return Err(xml_error(error)),
        }
        buffer.clear();
    }
}

#[derive(Default)]
struct FilterParser {
    filter: Option<SpreadsheetAutoFilterArtifact>,
    column: Option<SpreadsheetFilterColumnArtifact>,
}

impl FilterParser {
    fn start(
        &mut self,
        reader: &Reader<impl BufRead>,
        event: &BytesStart<'_>,
    ) -> Result<(), SpreadsheetEngineError> {
        match event.local_name().as_ref() {
            "autoFilter" => self.filter = auto_filter(reader, event)?,
            "filterColumn" => self.column = filter_column(reader, event, self.filter.as_ref())?,
            "filters" => add_blank(reader, event, self.column.as_mut())?,
            "filter" => add_value(reader, event, self.column.as_mut())?,
            kind if unsupported_filter(kind) => self.add_unsupported(kind),
            _ => {}
        }
        Ok(())
    }

    fn empty(
        &mut self,
        reader: &Reader<impl BufRead>,
        event: &BytesStart<'_>,
    ) -> Result<(), SpreadsheetEngineError> {
        self.start(reader, event)?;
        if event.local_name().as_ref() == "filterColumn" {
            self.finish_column();
        }
        Ok(())
    }

    fn finish_column(&mut self) {
        if let (Some(column), Some(filter)) = (self.column.take(), self.filter.as_mut()) {
            filter.columns.push(column);
        }
    }

    fn add_unsupported(&mut self, kind: &str) {
        let kind = kind.to_owned();
        if let Some(column) = self.column.as_mut() {
            column
                .criteria
                .push(SpreadsheetFilterCriterion::Unsupported(kind.clone()));
        }
        if let Some(filter) = self.filter.as_mut() {
            filter
                .diagnostics
                .push(format!("unsupported AutoFilter criterion `{kind}`"));
        }
    }
}

fn auto_filter(
    reader: &Reader<impl BufRead>,
    event: &BytesStart<'_>,
) -> Result<Option<SpreadsheetAutoFilterArtifact>, SpreadsheetEngineError> {
    attribute(reader, event, b"ref")?
        .map(|reference| {
            Ok(SpreadsheetAutoFilterArtifact {
                range: filter_range(&reference)?,
                columns: Vec::new(),
                filtered_out_rows: Vec::new(),
                diagnostics: Vec::new(),
            })
        })
        .transpose()
}

fn filter_range(reference: &str) -> Result<SpreadsheetFilterRange, SpreadsheetEngineError> {
    let (start, end) = reference.split_once(':').unwrap_or((reference, reference));
    Ok(SpreadsheetFilterRange {
        start: coordinate(start)?,
        end: coordinate(end)?,
    })
}

fn coordinate(reference: &str) -> Result<SpreadsheetCoordinate, SpreadsheetEngineError> {
    let parsed = parse_reference_a1(reference)
        .ok_or_else(|| SpreadsheetEngineError::Import("invalid AutoFilter range".into()))?;
    Ok(SpreadsheetCoordinate::new(
        SpreadsheetEngineSupport::zero_based(parsed.row)?,
        SpreadsheetEngineSupport::zero_based(parsed.column)?,
    ))
}

fn filter_column(
    reader: &Reader<impl BufRead>,
    event: &BytesStart<'_>,
    filter: Option<&SpreadsheetAutoFilterArtifact>,
) -> Result<Option<SpreadsheetFilterColumnArtifact>, SpreadsheetEngineError> {
    let Some(filter) = filter else {
        return Ok(None);
    };
    let relative = attribute_usize(reader, event, b"colId")?.unwrap_or(0);
    Ok(Some(SpreadsheetFilterColumnArtifact {
        column: filter.range.start.column.saturating_add(relative),
        criteria: Vec::new(),
        candidates: Vec::new(),
    }))
}

fn add_blank(
    reader: &Reader<impl BufRead>,
    event: &BytesStart<'_>,
    column: Option<&mut SpreadsheetFilterColumnArtifact>,
) -> Result<(), SpreadsheetEngineError> {
    if matches!(
        attribute(reader, event, b"blank")?.as_deref(),
        Some("1" | "true")
    ) && let Some(column) = column
    {
        column.criteria.push(SpreadsheetFilterCriterion::Blank);
    }
    Ok(())
}

fn add_value(
    reader: &Reader<impl BufRead>,
    event: &BytesStart<'_>,
    column: Option<&mut SpreadsheetFilterColumnArtifact>,
) -> Result<(), SpreadsheetEngineError> {
    if let (Some(column), Some(value)) = (column, attribute(reader, event, b"val")?) {
        match column.criteria.last_mut() {
            Some(SpreadsheetFilterCriterion::Values(values)) => values.push(value),
            _ => column
                .criteria
                .push(SpreadsheetFilterCriterion::Values(vec![value])),
        }
    }
    Ok(())
}

fn unsupported_filter(kind: &str) -> bool {
    matches!(
        kind,
        "customFilters"
            | "dynamicFilter"
            | "top10"
            | "colorFilter"
            | "iconFilter"
            | "dateGroupItem"
    )
}
