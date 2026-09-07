use super::SpreadsheetAutoFilterArtifact;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum SpreadsheetFilterCommand {
    Candidates {
        sheet_index: usize,
        column: usize,
        limit: usize,
    },
    ApplyValues {
        sheet_index: usize,
        column: usize,
        values: Vec<String>,
    },
    Clear {
        sheet_index: usize,
        column: Option<usize>,
    },
}

impl SpreadsheetFilterCommand {
    #[must_use]
    pub const fn sheet_index(&self) -> usize {
        match self {
            Self::Candidates { sheet_index, .. }
            | Self::ApplyValues { sheet_index, .. }
            | Self::Clear { sheet_index, .. } => *sheet_index,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum SpreadsheetFilterEvent {
    Candidates {
        sheet_index: usize,
        column: usize,
        values: Vec<String>,
        truncated: bool,
    },
    VisibilityChanged {
        sheet_index: usize,
        applied_columns: Vec<usize>,
        visible_row_count: usize,
        filtered_out_rows: Vec<usize>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpreadsheetFrameMetadata {
    pub sheet_index: usize,
    pub visible_row_count: usize,
    pub auto_filter: Option<SpreadsheetAutoFilterArtifact>,
}
