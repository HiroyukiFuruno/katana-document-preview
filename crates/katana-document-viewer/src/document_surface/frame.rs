use super::{DocumentGridSurfaceFrame, DocumentSurfaceError};
use katana_ui_core::render_model::{UiImageSurfaceProps, UiNode, UiNodeKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentSurfaceKind {
    Page,
    Grid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSurfaceFrame {
    content: DocumentSurfaceContent,
    navigation: DocumentNavigationMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct DocumentNavigationMetadata {
    item_labels: Vec<String>,
    outline_items: Vec<PdfOutlineItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PdfOutlineItem {
    pub title: String,
    pub level: usize,
    pub page_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DocumentSurfaceContent {
    Page(DocumentPageSurfaceFrame),
    Grid(DocumentGridSurfaceFrame),
}

impl DocumentSurfaceFrame {
    pub(super) fn from_node(node: UiNode) -> Result<Self, DocumentSurfaceError> {
        let content = match node.kind() {
            UiNodeKind::ImageSurface => DocumentSurfaceContent::Page(
                DocumentPageSurfaceFrame::from(&node.props().image_surface),
            ),
            UiNodeKind::Grid => {
                DocumentSurfaceContent::Grid(DocumentGridSurfaceFrame::from(&node.props().grid))
            }
            kind => {
                return Err(DocumentSurfaceError::UnsupportedNodeKind {
                    detail: format!("{kind:?}"),
                });
            }
        };
        Ok(Self {
            content,
            navigation: DocumentNavigationMetadata::default(),
        })
    }

    #[must_use]
    pub const fn kind(&self) -> DocumentSurfaceKind {
        match self.content {
            DocumentSurfaceContent::Page(_) => DocumentSurfaceKind::Page,
            DocumentSurfaceContent::Grid(_) => DocumentSurfaceKind::Grid,
        }
    }

    #[must_use]
    pub const fn page(&self) -> Option<&DocumentPageSurfaceFrame> {
        match &self.content {
            DocumentSurfaceContent::Page(page) => Some(page),
            DocumentSurfaceContent::Grid(_) => None,
        }
    }

    #[must_use]
    pub const fn grid(&self) -> Option<&DocumentGridSurfaceFrame> {
        match &self.content {
            DocumentSurfaceContent::Grid(grid) => Some(grid),
            DocumentSurfaceContent::Page(_) => None,
        }
    }

    #[must_use]
    pub fn active_text(&self) -> Option<&str> {
        let grid = self.grid()?;
        let active = grid.active_cell?;
        grid.cells
            .iter()
            .find(|cell| cell.coordinate == active)
            .map(|cell| cell.text.as_str())
    }

    #[must_use]
    pub fn item_labels(&self) -> &[String] {
        &self.navigation.item_labels
    }

    #[must_use]
    pub fn outline_items(&self) -> &[PdfOutlineItem] {
        &self.navigation.outline_items
    }

    pub(crate) fn with_navigation_metadata(
        mut self,
        item_labels: Vec<String>,
        outline_items: Vec<PdfOutlineItem>,
    ) -> Self {
        self.navigation = DocumentNavigationMetadata {
            item_labels,
            outline_items,
        };
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentPageSurfaceFrame {
    pub fingerprint: String,
    pub width: u32,
    pub height: u32,
    pub display_width_milli: u32,
    pub display_height_milli: u32,
    pub content_scale: u32,
    pub accessibility_label: String,
    pub rgba: Vec<u8>,
}

impl From<&UiImageSurfaceProps> for DocumentPageSurfaceFrame {
    fn from(value: &UiImageSurfaceProps) -> Self {
        Self {
            fingerprint: value.fingerprint.clone(),
            width: value.width,
            height: value.height,
            display_width_milli: value.display_width_milli,
            display_height_milli: value.display_height_milli,
            content_scale: value.content_scale,
            accessibility_label: value.accessibility_label.clone(),
            rgba: value.rgba.clone(),
        }
    }
}

#[cfg(test)]
#[path = "frame_tests.rs"]
mod tests;
