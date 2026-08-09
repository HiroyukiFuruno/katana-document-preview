use super::DocumentGridCellAppearance;
use katana_ui_core::render_model::{UiGridCell, UiGridProps, UiGridViewport, UiRect};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentGridSurfaceFrame {
    pub row_count: usize,
    pub column_count: usize,
    pub total_width: u32,
    pub total_height: u32,
    pub viewport: DocumentGridViewport,
    pub active_cell: Option<DocumentGridCoordinate>,
    pub show_grid_lines: bool,
    pub cells: Vec<DocumentGridCell>,
}

impl DocumentGridSurfaceFrame {
    #[must_use]
    pub const fn scroll_x(&self) -> u32 {
        self.viewport.scroll_x
    }

    #[must_use]
    pub const fn scroll_y(&self) -> u32 {
        self.viewport.scroll_y
    }
}

impl From<&UiGridProps> for DocumentGridSurfaceFrame {
    fn from(value: &UiGridProps) -> Self {
        Self {
            row_count: value.row_count,
            column_count: value.column_count,
            total_width: value.total_width,
            total_height: value.total_height,
            viewport: DocumentGridViewport::from(value.viewport),
            active_cell: value.active_cell.map(DocumentGridCoordinate::from),
            show_grid_lines: value.show_grid_lines,
            cells: value.cells.iter().map(DocumentGridCell::from).collect(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DocumentGridViewport {
    pub width: u32,
    pub height: u32,
    pub scroll_x: u32,
    pub scroll_y: u32,
}

impl From<UiGridViewport> for DocumentGridViewport {
    fn from(value: UiGridViewport) -> Self {
        Self {
            width: value.width,
            height: value.height,
            scroll_x: value.scroll_x,
            scroll_y: value.scroll_y,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DocumentGridCoordinate {
    pub row: usize,
    pub column: usize,
}

impl From<katana_ui_core::render_model::UiGridCoordinate> for DocumentGridCoordinate {
    fn from(value: katana_ui_core::render_model::UiGridCoordinate) -> Self {
        Self {
            row: value.row,
            column: value.column,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DocumentRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl From<UiRect> for DocumentRect {
    fn from(value: UiRect) -> Self {
        Self {
            x: value.x,
            y: value.y,
            width: value.width,
            height: value.height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentGridCell {
    pub coordinate: DocumentGridCoordinate,
    pub bounds: DocumentRect,
    pub clipped_bounds: DocumentRect,
    pub text: String,
    pub appearance: DocumentGridCellAppearance,
    pub row_span: usize,
    pub column_span: usize,
    pub selected: bool,
    pub active: bool,
    pub frozen_row: bool,
    pub frozen_column: bool,
    pub accessibility_row_index: usize,
    pub accessibility_column_index: usize,
}

impl From<&UiGridCell> for DocumentGridCell {
    fn from(value: &UiGridCell) -> Self {
        Self {
            coordinate: DocumentGridCoordinate::from(value.coordinate),
            bounds: DocumentRect::from(value.bounds),
            clipped_bounds: DocumentRect::from(value.clipped_bounds),
            text: value.text.clone(),
            appearance: DocumentGridCellAppearance::from(&value.appearance),
            row_span: value.row_span,
            column_span: value.column_span,
            selected: value.selected,
            active: value.active,
            frozen_row: value.frozen_row,
            frozen_column: value.frozen_column,
            accessibility_row_index: value.accessibility_row_index,
            accessibility_column_index: value.accessibility_column_index,
        }
    }
}
