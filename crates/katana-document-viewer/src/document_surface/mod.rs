mod host;
mod page_surface;
mod spreadsheet_grid;

use katana_ui_core::render_model::{UiNode, UiNodeKind};
use thiserror::Error;

pub use host::{DocumentSurfaceHost, DocumentSurfaceHostOutput};
pub use spreadsheet_grid::SpreadsheetGridSurface;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentSurfaceKind {
    Page,
    Grid,
}

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSurfaceFrame {
    node: UiNode,
}

impl DocumentSurfaceFrame {
    fn from_node(node: UiNode) -> Self {
        Self { node }
    }

    #[must_use]
    pub fn kind(&self) -> DocumentSurfaceKind {
        match self.node.kind() {
            UiNodeKind::ImageSurface => DocumentSurfaceKind::Page,
            UiNodeKind::Grid => DocumentSurfaceKind::Grid,
            kind => unreachable!("KDV produced unsupported document surface kind: {kind:?}"),
        }
    }

    #[must_use]
    pub fn active_text(&self) -> Option<&str> {
        if self.kind() != DocumentSurfaceKind::Grid {
            return None;
        }
        let grid = &self.node.props().grid;
        let active = grid.active_cell?;
        grid.cells
            .iter()
            .find(|cell| cell.coordinate == active)
            .map(|cell| cell.text.as_str())
    }

    fn node(&self) -> &UiNode {
        &self.node
    }
}

#[cfg(test)]
#[path = "document_surface_tests.rs"]
mod tests;
