mod grid;
mod grid_cell_text;
mod grid_conditional;
mod grid_paint;
mod page;

#[cfg(test)]
#[path = "host_test_support.rs"]
mod test_support;
#[cfg(test)]
#[path = "host_tests.rs"]
mod tests;

use super::{DocumentSurfaceCommand, DocumentSurfaceFrame, DocumentSurfaceKind};

#[derive(Default)]
pub struct DocumentSurfaceHost {
    pub(super) texture: Option<egui::TextureHandle>,
    pub(super) texture_fingerprint: Option<String>,
}

impl std::fmt::Debug for DocumentSurfaceHost {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DocumentSurfaceHost")
            .field("texture_fingerprint", &self.texture_fingerprint)
            .finish_non_exhaustive()
    }
}

impl DocumentSurfaceHost {
    #[must_use]
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        frame: &DocumentSurfaceFrame,
        surface_id: u64,
    ) -> DocumentSurfaceHostOutput {
        match frame.kind() {
            DocumentSurfaceKind::Page => page::show(self, ui, frame, surface_id),
            DocumentSurfaceKind::Grid => grid::show(ui, frame),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DocumentSurfaceHostOutput {
    commands: Vec<DocumentSurfaceCommand>,
}

impl DocumentSurfaceHostOutput {
    fn push(&mut self, command: DocumentSurfaceCommand) {
        self.commands.push(command);
    }

    #[must_use]
    pub fn commands(&self) -> &[DocumentSurfaceCommand] {
        &self.commands
    }

    #[must_use]
    pub fn into_commands(self) -> Vec<DocumentSurfaceCommand> {
        self.commands
    }
}
