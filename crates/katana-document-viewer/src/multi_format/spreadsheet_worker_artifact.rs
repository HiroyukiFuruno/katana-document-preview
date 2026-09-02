use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpreadsheetCoordinate {
    pub row: usize,
    pub column: usize,
}

impl SpreadsheetCoordinate {
    #[must_use]
    pub const fn new(row: usize, column: usize) -> Self {
        Self { row, column }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpreadsheetTrackArtifact {
    pub size: f32,
    pub hidden: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpreadsheetHorizontalAlignment {
    General,
    Left,
    Center,
    CenterContinuous,
    Right,
    Fill,
    Justify,
    Distributed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpreadsheetVerticalAlignment {
    Bottom,
    Center,
    Top,
    Justify,
    Distributed,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpreadsheetCellBorderArtifact {
    pub left: Option<SpreadsheetBorderSideArtifact>,
    pub right: Option<SpreadsheetBorderSideArtifact>,
    pub top: Option<SpreadsheetBorderSideArtifact>,
    pub bottom: Option<SpreadsheetBorderSideArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpreadsheetBorderSideArtifact {
    pub style: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpreadsheetCellStyleArtifact {
    pub font_name: String,
    pub font_size: f32,
    pub font_color: Option<String>,
    pub fill_color: Option<String>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub horizontal_alignment: SpreadsheetHorizontalAlignment,
    pub vertical_alignment: SpreadsheetVerticalAlignment,
    pub wrap_text: bool,
    pub number_format: String,
    #[serde(default)]
    pub borders: SpreadsheetCellBorderArtifact,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpreadsheetDataBarArtifact {
    pub positive_color: Option<String>,
    pub negative_color: Option<String>,
    pub value: f64,
    pub axis_position: f64,
    pub gradient: bool,
    pub show_value: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpreadsheetIconArtifact {
    pub name: String,
    pub color: Option<String>,
    pub show_value: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpreadsheetRatingArtifact {
    pub icon_name: String,
    pub count: u32,
    pub maximum: u32,
    pub color: Option<String>,
    pub show_value: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpreadsheetConditionalFormattingArtifact {
    pub applied: bool,
    pub data_bar: Option<SpreadsheetDataBarArtifact>,
    pub icon: Option<SpreadsheetIconArtifact>,
    pub rating: Option<SpreadsheetRatingArtifact>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SpreadsheetCellValue {
    Empty,
    Text(String),
    Number(f64),
    Boolean(bool),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpreadsheetCellArtifact {
    pub coordinate: SpreadsheetCoordinate,
    pub display_text: String,
    pub value: SpreadsheetCellValue,
    pub formula: Option<String>,
    pub style: SpreadsheetCellStyleArtifact,
    pub conditional_formatting: SpreadsheetConditionalFormattingArtifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpreadsheetMergedCellArtifact {
    pub anchor: SpreadsheetCoordinate,
    pub row_span: usize,
    pub column_span: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpreadsheetFilterRange {
    pub start: SpreadsheetCoordinate,
    pub end: SpreadsheetCoordinate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SpreadsheetFilterCriterion {
    Values(Vec<String>),
    Blank,
    NonBlank,
    Unsupported(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpreadsheetFilterColumnArtifact {
    pub column: usize,
    pub criteria: Vec<SpreadsheetFilterCriterion>,
    pub candidates: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpreadsheetAutoFilterArtifact {
    pub range: SpreadsheetFilterRange,
    pub columns: Vec<SpreadsheetFilterColumnArtifact>,
    pub filtered_out_rows: Vec<usize>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpreadsheetSheetArtifact {
    pub index: usize,
    pub name: String,
    pub row_count: usize,
    pub column_count: usize,
    pub row_tracks: Vec<SpreadsheetTrackArtifact>,
    pub column_tracks: Vec<SpreadsheetTrackArtifact>,
    pub frozen_rows: usize,
    pub frozen_columns: usize,
    pub merged_cells: Vec<SpreadsheetMergedCellArtifact>,
    pub show_grid_lines: bool,
    #[serde(default)]
    pub auto_filter: Option<SpreadsheetAutoFilterArtifact>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpreadsheetViewerLimits {
    pub max_sheets: usize,
    pub max_logical_cells: usize,
    pub max_materialized_cells: usize,
}

impl SpreadsheetViewerLimits {
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            max_sheets: 256,
            max_logical_cells: 25_000_000,
            max_materialized_cells: 4_096,
        }
    }
}
