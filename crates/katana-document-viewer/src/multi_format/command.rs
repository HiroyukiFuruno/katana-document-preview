use serde::{Deserialize, Serialize};
use thiserror::Error;

const DEFAULT_ZOOM: f32 = 1.0;
const MAX_ZOOM: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentFitMode {
    Width,
    Page,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DocumentViewerCommand {
    Previous,
    Next,
    JumpTo(usize),
    SetZoom(f32),
    Fit(DocumentFitMode),
    CopySelection,
    OpenTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DocumentViewerEvent {
    None,
    IndexChanged(usize),
    ZoomChanged(f32),
    FitChanged(DocumentFitMode),
    CopyRequested,
    OpenRequested,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DocumentViewerStateError {
    #[error("document index {requested} is outside item count {item_count}")]
    IndexOutsideDocument { requested: usize, item_count: usize },
    #[error("document zoom must be finite, greater than zero, and at most eight")]
    InvalidZoom,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DocumentViewerState {
    pub item_count: usize,
    pub active_index: usize,
    pub zoom: f32,
    pub fit: Option<DocumentFitMode>,
}

impl DocumentViewerState {
    #[must_use]
    pub const fn new(item_count: usize) -> Self {
        Self {
            item_count,
            active_index: 0,
            zoom: DEFAULT_ZOOM,
            fit: None,
        }
    }

    pub fn apply(
        &mut self,
        command: DocumentViewerCommand,
    ) -> Result<DocumentViewerEvent, DocumentViewerStateError> {
        match command {
            DocumentViewerCommand::Previous => {
                Ok(self.move_to(self.active_index.saturating_sub(1)))
            }
            DocumentViewerCommand::Next => Ok(self.move_to(
                self.active_index
                    .saturating_add(1)
                    .min(self.item_count.saturating_sub(1)),
            )),
            DocumentViewerCommand::JumpTo(index) => self.jump_to(index),
            DocumentViewerCommand::SetZoom(zoom) => self.set_zoom(zoom),
            DocumentViewerCommand::Fit(mode) => {
                self.fit = Some(mode);
                Ok(DocumentViewerEvent::FitChanged(mode))
            }
            DocumentViewerCommand::CopySelection => Ok(DocumentViewerEvent::CopyRequested),
            DocumentViewerCommand::OpenTarget => Ok(DocumentViewerEvent::OpenRequested),
        }
    }

    fn jump_to(&mut self, index: usize) -> Result<DocumentViewerEvent, DocumentViewerStateError> {
        if index >= self.item_count {
            return Err(DocumentViewerStateError::IndexOutsideDocument {
                requested: index,
                item_count: self.item_count,
            });
        }
        Ok(self.move_to(index))
    }

    fn set_zoom(&mut self, zoom: f32) -> Result<DocumentViewerEvent, DocumentViewerStateError> {
        if !zoom.is_finite() || zoom <= 0.0 || zoom > MAX_ZOOM {
            return Err(DocumentViewerStateError::InvalidZoom);
        }
        self.zoom = zoom;
        self.fit = None;
        Ok(DocumentViewerEvent::ZoomChanged(zoom))
    }

    fn move_to(&mut self, index: usize) -> DocumentViewerEvent {
        if self.item_count == 0 || self.active_index == index {
            return DocumentViewerEvent::None;
        }
        self.active_index = index;
        DocumentViewerEvent::IndexChanged(index)
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_ZOOM, DocumentViewerState};

    #[test]
    fn state_constructor_initializes_runtime_item_count() {
        let item_count = std::hint::black_box(3);
        let state = DocumentViewerState::new(item_count);

        assert_eq!(item_count, state.item_count);
        assert_eq!(0, state.active_index);
        assert_eq!(DEFAULT_ZOOM, state.zoom);
        assert_eq!(None, state.fit);
    }
}
