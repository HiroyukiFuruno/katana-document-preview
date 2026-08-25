use super::{
    OfficeStaticViewerSession, PdfViewerSession, ViewerCapabilities, ViewerDiagnostic,
    ViewerDocumentFormat,
};
use crate::PdfOutlineItem;

pub(super) struct PagedDocumentMetadata {
    pub(super) format: ViewerDocumentFormat,
    pub(super) item_count: usize,
    pub(super) capabilities: ViewerCapabilities,
    pub(super) diagnostics: Vec<ViewerDiagnostic>,
    pub(super) item_sizes: Vec<(f32, f32)>,
    pub(super) outline_items: Vec<PdfOutlineItem>,
}

pub(super) fn pdf_metadata(session: &PdfViewerSession) -> PagedDocumentMetadata {
    let artifact = session.artifact();
    PagedDocumentMetadata {
        format: ViewerDocumentFormat::Pdf,
        item_count: artifact.page_count,
        capabilities: artifact.capabilities.clone(),
        diagnostics: artifact.diagnostics.clone(),
        item_sizes: artifact
            .pages
            .iter()
            .map(|page| (page.width, page.height))
            .collect(),
        outline_items: session.outline().to_vec(),
    }
}

pub(super) fn office_metadata(
    session: &OfficeStaticViewerSession,
    format: ViewerDocumentFormat,
) -> PagedDocumentMetadata {
    let artifact = session.artifact();
    PagedDocumentMetadata {
        format,
        item_count: artifact.item_count,
        capabilities: artifact.capabilities.clone(),
        diagnostics: artifact.diagnostics.clone(),
        item_sizes: super::document_session_paged_office::office_item_sizes(session),
        outline_items: Vec::new(),
    }
}
