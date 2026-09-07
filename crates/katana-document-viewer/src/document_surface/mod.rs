mod frame;
mod frame_grid;
mod frame_grid_style;
mod page_surface;
mod spreadsheet_grid;

use thiserror::Error;

pub use frame::{
    DocumentPageSurfaceFrame, DocumentSurfaceFrame, DocumentSurfaceKind, PdfOutlineItem,
};
pub use frame_grid::{
    DocumentGridCell, DocumentGridCoordinate, DocumentGridSurfaceFrame, DocumentGridViewport,
    DocumentRect,
};
pub use frame_grid_style::{
    DocumentGridBorderSide, DocumentGridCellAppearance, DocumentGridCellBorders,
    DocumentGridDataBar, DocumentGridHorizontalAlignment, DocumentGridIcon, DocumentGridRating,
    DocumentGridVerticalAlignment,
};
pub use spreadsheet_grid::SpreadsheetGridSurface;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocumentViewport {
    pub width: u32,
    pub height: u32,
}

impl DocumentViewport {
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self {
            width: if width == 0 { 1 } else { width },
            height: if height == 0 { 1 } else { height },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentGridNavigation {
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentGridCommand {
    SelectAt {
        x: i32,
        y: i32,
        extend: bool,
    },
    ScrollTo {
        x: u32,
        y: u32,
    },
    Select {
        row: usize,
        column: usize,
        extend: bool,
    },
    Navigate {
        intent: DocumentGridNavigation,
        extend: bool,
    },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DocumentGridEvent {
    #[default]
    None,
    SelectionChanged,
    Scrolled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentSurfaceCommand {
    Resize(DocumentViewport),
    Grid(DocumentGridCommand),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DocumentSurfaceError {
    #[error("document page surface is invalid: {detail}")]
    InvalidPage { detail: String },
    #[error("document spreadsheet surface is invalid: {detail}")]
    InvalidGrid { detail: String },
    #[error("document surface node kind is unsupported: {detail}")]
    UnsupportedNodeKind { detail: String },
}

#[cfg(test)]
#[path = "document_surface_tests.rs"]
mod tests;
