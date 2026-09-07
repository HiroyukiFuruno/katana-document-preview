use super::document_session_paged_metadata::{office_metadata, pdf_metadata};
use super::{
    BinaryDocumentSource, DocumentFitMode, DocumentFrame, DocumentSessionCommand,
    DocumentSessionError, DocumentSessionEvent, DocumentViewerCommand, DocumentViewerState,
    OfficeDocumentSource, OfficeStaticViewerSession, OfficeWorkerConfig, PdfPageRenderRequest,
    PdfRenderedPage, PdfViewerSession, ViewerCapabilities, ViewerDiagnostic, ViewerDocumentFormat,
};
use crate::{DocumentSurfaceCommand, DocumentSurfaceFrame, DocumentViewport, PdfOutlineItem};

const VIEWPORT_HORIZONTAL_CHROME: u32 = 32;
const VIEWPORT_VERTICAL_CHROME: u32 = 72;
const MIN_RENDER_SCALE: f32 = 0.25;
const MAX_RENDER_SCALE: f32 = 4.0;

enum PagedEngine {
    Pdf(Box<PdfViewerSession>),
    Office(Box<OfficeStaticViewerSession>),
}

pub(super) struct PagedDocumentSession {
    engine: PagedEngine,
    format: ViewerDocumentFormat,
    state: DocumentViewerState,
    capabilities: ViewerCapabilities,
    diagnostics: Vec<ViewerDiagnostic>,
    item_sizes: Vec<(f32, f32)>,
    outline_items: Vec<PdfOutlineItem>,
    viewport: DocumentViewport,
}

impl PagedDocumentSession {
    pub(super) fn open_pdf(
        source: BinaryDocumentSource,
        viewport: DocumentViewport,
    ) -> Result<Self, DocumentSessionError> {
        let session = PdfViewerSession::open(source)?;
        let metadata = pdf_metadata(&session);
        Ok(Self::new(
            PagedEngine::Pdf(Box::new(session)),
            metadata,
            viewport,
        ))
    }
    pub(super) fn open_office(
        source: OfficeDocumentSource,
        worker: OfficeWorkerConfig,
        viewport: DocumentViewport,
    ) -> Result<Self, DocumentSessionError> {
        let format = ViewerDocumentFormat::from(source.format);
        let session = OfficeStaticViewerSession::open(source, worker)?;
        let metadata = office_metadata(&session, format);
        Ok(Self::new(
            PagedEngine::Office(Box::new(session)),
            metadata,
            viewport,
        ))
    }
    fn new(
        engine: PagedEngine,
        metadata: super::document_session_paged_metadata::PagedDocumentMetadata,
        viewport: DocumentViewport,
    ) -> Self {
        let mut state = DocumentViewerState::new(metadata.item_count);
        let _ = state.apply(DocumentViewerCommand::Fit(DocumentFitMode::Page));
        Self {
            engine,
            format: metadata.format,
            state,
            capabilities: metadata.capabilities,
            diagnostics: metadata.diagnostics,
            item_sizes: metadata.item_sizes,
            outline_items: metadata.outline_items,
            viewport,
        }
    }
    pub(super) fn apply(
        &mut self,
        command: DocumentSessionCommand,
    ) -> Result<DocumentSessionEvent, DocumentSessionError> {
        Ok(match command {
            DocumentSessionCommand::Viewer(command) => {
                DocumentSessionEvent::Viewer(self.state.apply(command)?)
            }
            DocumentSessionCommand::Surface(DocumentSurfaceCommand::Resize(viewport)) => {
                self.viewport = viewport;
                DocumentSessionEvent::None
            }
            DocumentSessionCommand::Surface(DocumentSurfaceCommand::Grid(_)) => {
                return Err(DocumentSessionError::UnsupportedCommand {
                    format: self.format,
                    command: super::DocumentSessionCommandKind::Grid,
                });
            }
        })
    }
    pub(super) fn frame(&mut self) -> Result<DocumentFrame, DocumentSessionError> {
        let _trace_scope = match &self.engine {
            PagedEngine::Pdf(_) => None,
            PagedEngine::Office(session) => session.trace_scope(),
        };
        let page_index = self.state.active_index;
        self.render_scale().and_then(|scale| {
            let request = PdfPageRenderRequest::new(page_index, scale);
            let rendered = match &mut self.engine {
                PagedEngine::Pdf(session) => session.render_page(request),
                PagedEngine::Office(session) => session.render_item(request),
            };
            rendered
                .map_err(DocumentSessionError::from)
                .and_then(|rendered| self.frame_from_rendered(rendered))
        })
    }
    fn frame_from_rendered(
        &self,
        rendered: PdfRenderedPage,
    ) -> Result<DocumentFrame, DocumentSessionError> {
        let stage = match &self.engine {
            PagedEngine::Pdf(_) => "pdf.frame_publication",
            PagedEngine::Office(_) => "office.frame_publication",
        };
        let _publication = super::debug_trace::DebugTrace::start(stage);
        DocumentSurfaceFrame::from_rendered_page("Document page", rendered)
            .map(|surface| DocumentFrame {
                surface: surface.with_navigation_metadata(Vec::new(), self.outline_items.clone()),
                state: self.state,
                capabilities: self.capabilities.clone(),
                diagnostics: self.diagnostics.clone(),
                format: self.format,
                spreadsheet: None,
            })
            .map_err(DocumentSessionError::from)
    }
    fn render_scale(&self) -> Result<f32, DocumentSessionError> {
        self.item_sizes
            .get(self.state.active_index)
            .copied()
            .ok_or(super::DocumentViewerStateError::IndexOutsideDocument {
                requested: self.state.active_index,
                item_count: self.item_sizes.len(),
            })
            .map(|(width, height)| {
                let available_width =
                    self.viewport
                        .width
                        .saturating_sub(VIEWPORT_HORIZONTAL_CHROME) as f32;
                let available_height =
                    self.viewport
                        .height
                        .saturating_sub(VIEWPORT_VERTICAL_CHROME) as f32;
                let fit_width = available_width / width.max(1.0);
                let fit_page = fit_width.min(available_height / height.max(1.0));
                match self.state.fit {
                    Some(DocumentFitMode::Width) => fit_width,
                    Some(DocumentFitMode::Page) => fit_page,
                    None => self.state.zoom,
                }
                .clamp(MIN_RENDER_SCALE, MAX_RENDER_SCALE)
            })
            .map_err(DocumentSessionError::from)
    }

    pub(super) fn info_parts(&self) -> super::document_session_types::DocumentRuntimeInfo<'_> {
        (self.format, &self.capabilities, &self.diagnostics)
    }
}

#[cfg(test)]
#[path = "document_session_paged_tests.rs"]
mod tests;
