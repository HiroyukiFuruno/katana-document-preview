use super::spreadsheet_engine::SpreadsheetEngineError;
use super::{
    SpreadsheetCellArtifact, SpreadsheetCellBorderArtifact, SpreadsheetCellStyleArtifact,
    SpreadsheetCellValue, SpreadsheetConditionalFormattingArtifact, SpreadsheetCoordinate,
    SpreadsheetHorizontalAlignment, SpreadsheetVerticalAlignment,
};

#[derive(Clone, Copy)]
pub(super) enum Capture {
    None,
    Formula,
    Value,
    Text,
}

pub(super) struct CellAccumulator {
    result_index: usize,
    coordinate: SpreadsheetCoordinate,
    cell_type: String,
    value: String,
    text: String,
    formula: String,
}

impl CellAccumulator {
    pub(super) fn new(
        result_index: usize,
        coordinate: SpreadsheetCoordinate,
        cell_type: String,
    ) -> Self {
        Self {
            result_index,
            coordinate,
            cell_type,
            value: String::new(),
            text: String::new(),
            formula: String::new(),
        }
    }

    pub(super) const fn result_index(&self) -> usize {
        self.result_index
    }

    pub(super) fn append(&mut self, capture: Capture, value: &str) {
        match capture {
            Capture::None => {}
            Capture::Formula => self.formula.push_str(value),
            Capture::Value => self.value.push_str(value),
            Capture::Text => self.text.push_str(value),
        }
    }

    pub(super) fn finish(self, shared_strings: &[String]) -> SpreadsheetCellArtifact {
        let value = self.resolved_value(shared_strings);
        SpreadsheetCellArtifact {
            coordinate: self.coordinate,
            display_text: display_text(&value),
            value,
            formula: (!self.formula.is_empty()).then_some(self.formula),
            style: default_style(),
            conditional_formatting: SpreadsheetConditionalFormattingArtifact::default(),
        }
    }

    fn resolved_value(&self, shared_strings: &[String]) -> SpreadsheetCellValue {
        match self.cell_type.as_str() {
            "inlineStr" => SpreadsheetCellValue::Text(self.text.clone()),
            "str" => SpreadsheetCellValue::Text(self.value.clone()),
            "s" => self
                .value
                .parse::<usize>()
                .ok()
                .and_then(|index| shared_strings.get(index).cloned())
                .map_or(SpreadsheetCellValue::Empty, SpreadsheetCellValue::Text),
            "b" => SpreadsheetCellValue::Boolean(self.value == "1" || self.value == "true"),
            _ => self.value.parse::<f64>().map_or_else(
                |_| SpreadsheetCellValue::Text(self.value.clone()),
                SpreadsheetCellValue::Number,
            ),
        }
    }
}

fn display_text(value: &SpreadsheetCellValue) -> String {
    match value {
        SpreadsheetCellValue::Empty => String::new(),
        SpreadsheetCellValue::Text(value) => value.clone(),
        SpreadsheetCellValue::Number(value) => value.to_string(),
        SpreadsheetCellValue::Boolean(value) => value.to_string().to_uppercase(),
    }
}

pub(super) fn empty_cell(coordinate: SpreadsheetCoordinate) -> SpreadsheetCellArtifact {
    SpreadsheetCellArtifact {
        coordinate,
        display_text: String::new(),
        value: SpreadsheetCellValue::Empty,
        formula: None,
        style: default_style(),
        conditional_formatting: SpreadsheetConditionalFormattingArtifact::default(),
    }
}

fn default_style() -> SpreadsheetCellStyleArtifact {
    SpreadsheetCellStyleArtifact {
        font_name: "Calibri".to_owned(),
        font_size: 11.0,
        font_color: None,
        fill_color: None,
        bold: false,
        italic: false,
        underline: false,
        strike: false,
        horizontal_alignment: SpreadsheetHorizontalAlignment::General,
        vertical_alignment: SpreadsheetVerticalAlignment::Bottom,
        wrap_text: false,
        number_format: "General".to_owned(),
        borders: SpreadsheetCellBorderArtifact::default(),
    }
}

pub(super) fn import_error(error: impl ToString) -> SpreadsheetEngineError {
    SpreadsheetEngineError::Import(error.to_string())
}
