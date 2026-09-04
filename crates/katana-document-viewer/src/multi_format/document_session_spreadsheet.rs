use super::{
    DocumentFrame, DocumentSessionCommand, DocumentSessionError, DocumentSessionEvent,
    DocumentViewerState, OfficeDocumentSource, OfficeWorkerConfig, SpreadsheetFilterCommand,
    SpreadsheetFilterEvent, SpreadsheetFrameMetadata, SpreadsheetViewerSession, ViewerCapabilities,
    ViewerDiagnostic, ViewerDocumentFormat,
};
use crate::{DocumentSurfaceCommand, DocumentViewport, SpreadsheetGridSurface};

pub(super) struct SpreadsheetDocumentSession {
    engine: SpreadsheetViewerSession,
    surface: SpreadsheetGridSurface,
    state: DocumentViewerState,
    capabilities: ViewerCapabilities,
    diagnostics: Vec<ViewerDiagnostic>,
    viewport: DocumentViewport,
}

impl SpreadsheetDocumentSession {
    pub(super) fn open(
        source: OfficeDocumentSource,
        worker: OfficeWorkerConfig,
        viewport: DocumentViewport,
    ) -> Result<Self, DocumentSessionError> {
        let engine = SpreadsheetViewerSession::open(source, worker)?;
        let artifact = engine.artifact();
        let state = DocumentViewerState::new(artifact.sheet_count);
        let capabilities = artifact.capabilities.clone();
        let diagnostics = artifact.diagnostics.clone();
        let surface = surface_for(&engine, 0, viewport)?;
        Ok(Self {
            engine,
            surface,
            state,
            capabilities,
            diagnostics,
            viewport,
        })
    }

    pub(super) fn apply(
        &mut self,
        command: DocumentSessionCommand,
    ) -> Result<DocumentSessionEvent, DocumentSessionError> {
        Ok(match command {
            DocumentSessionCommand::Viewer(command) => {
                let previous = self.state.active_index;
                let event = self.state.apply(command)?;
                if self.state.active_index != previous {
                    self.replace_surface()?;
                }
                DocumentSessionEvent::Viewer(event)
            }
            DocumentSessionCommand::Surface(DocumentSurfaceCommand::Resize(viewport)) => {
                self.viewport = viewport;
                self.replace_surface()?;
                DocumentSessionEvent::None
            }
            DocumentSessionCommand::Surface(DocumentSurfaceCommand::Grid(command)) => {
                DocumentSessionEvent::Grid(self.surface.apply_command(command))
            }
        })
    }

    pub(super) fn frame(&mut self) -> Result<DocumentFrame, DocumentSessionError> {
        let _trace_scope = self.engine.trace_scope();
        let _publication = super::debug_trace::DebugTrace::start("spreadsheet.frame_publication");
        let coordinates = self.surface.materialization_request();
        let cells = self
            .engine
            .materialize_cells(self.surface.sheet_index(), coordinates)?;
        self.surface.supply_cells(cells)?;
        let item_labels = self
            .engine
            .artifact()
            .sheets
            .iter()
            .map(|sheet| sheet.name.clone())
            .collect();
        let spreadsheet = frame_metadata(self.engine.artifact(), self.state.active_index);
        Ok(DocumentFrame {
            surface: self
                .surface
                .frame()?
                .with_navigation_metadata(item_labels, Vec::new()),
            state: self.state,
            capabilities: self.capabilities.clone(),
            diagnostics: self.diagnostics.clone(),
            format: ViewerDocumentFormat::Xlsx,
            spreadsheet,
        })
    }

    pub(super) fn apply_filter(
        &mut self,
        command: SpreadsheetFilterCommand,
    ) -> Result<SpreadsheetFilterEvent, DocumentSessionError> {
        let _trace_scope = self.engine.trace_scope();
        let sheet_index = command.sheet_index();
        let event = self.engine.apply_filter(command)?;
        if sheet_index == self.state.active_index
            && matches!(event, SpreadsheetFilterEvent::VisibilityChanged { .. })
        {
            self.replace_surface_preserving_state()?;
        }
        Ok(event)
    }

    fn replace_surface(&mut self) -> Result<(), DocumentSessionError> {
        self.surface = surface_for(&self.engine, self.state.active_index, self.viewport)?;
        Ok(())
    }

    fn replace_surface_preserving_state(&mut self) -> Result<(), DocumentSessionError> {
        let sheet = self
            .engine
            .artifact()
            .sheets
            .get(self.state.active_index)
            .ok_or(super::DocumentViewerStateError::IndexOutsideDocument {
                requested: self.state.active_index,
                item_count: self.engine.artifact().sheet_count,
            })?;
        self.surface.replace_sheet(sheet, self.viewport)?;
        Ok(())
    }

    pub(super) fn info_parts(&self) -> super::document_session_types::DocumentRuntimeInfo<'_> {
        (
            ViewerDocumentFormat::Xlsx,
            &self.capabilities,
            &self.diagnostics,
        )
    }
}

fn frame_metadata(
    artifact: &super::SpreadsheetDocumentArtifact,
    sheet_index: usize,
) -> Option<SpreadsheetFrameMetadata> {
    let sheet = artifact.sheets.get(sheet_index)?;
    let visible_row_count = sheet
        .row_tracks
        .iter()
        .enumerate()
        .filter(|(row, track)| {
            !track.hidden
                && sheet
                    .auto_filter
                    .as_ref()
                    .is_none_or(|filter| filter.filtered_out_rows.binary_search(row).is_err())
        })
        .count();
    Some(SpreadsheetFrameMetadata {
        sheet_index: sheet.index,
        visible_row_count,
        auto_filter: sheet.auto_filter.clone(),
    })
}

fn surface_for(
    engine: &SpreadsheetViewerSession,
    sheet_index: usize,
    viewport: DocumentViewport,
) -> Result<SpreadsheetGridSurface, DocumentSessionError> {
    let sheet = engine.artifact().sheets.get(sheet_index).ok_or(
        super::DocumentViewerStateError::IndexOutsideDocument {
            requested: sheet_index,
            item_count: engine.artifact().sheet_count,
        },
    )?;
    Ok(SpreadsheetGridSurface::new(sheet, viewport)?)
}

#[cfg(test)]
#[path = "document_session_spreadsheet_filter_state_tests.rs"]
mod filter_state_tests;
#[cfg(test)]
#[path = "document_session_spreadsheet_filter_tests.rs"]
mod filter_tests;
#[cfg(test)]
#[path = "document_session_spreadsheet_tests.rs"]
mod tests;
