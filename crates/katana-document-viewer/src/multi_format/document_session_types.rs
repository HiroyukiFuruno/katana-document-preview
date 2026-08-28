use super::{
    DocumentViewerCommand, DocumentViewerState, OfficeDocumentFormat, OfficeWorkerConfig,
    OfficeWorkerError, PdfViewerError, SpreadsheetFrameMetadata, ViewerCapabilities,
    ViewerDiagnostic, ViewerSourceIdentity,
};
use crate::{
    DocumentGridEvent, DocumentSurfaceCommand, DocumentSurfaceError, DocumentSurfaceFrame,
    DocumentViewport,
};
use thiserror::Error;

pub(super) type DocumentRuntimeInfo<'a> = (
    ViewerDocumentFormat,
    &'a ViewerCapabilities,
    &'a [ViewerDiagnostic],
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewerDocumentFormat {
    Pdf,
    Docx,
    Xlsx,
    Pptx,
}

impl From<OfficeDocumentFormat> for ViewerDocumentFormat {
    fn from(value: OfficeDocumentFormat) -> Self {
        match value {
            OfficeDocumentFormat::Docx => Self::Docx,
            OfficeDocumentFormat::Xlsx => Self::Xlsx,
            OfficeDocumentFormat::Pptx => Self::Pptx,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSessionConfig {
    pub viewport: DocumentViewport,
    pub office_worker: Option<OfficeWorkerConfig>,
}

impl DocumentSessionConfig {
    #[must_use]
    pub const fn new(viewport: DocumentViewport) -> Self {
        Self {
            viewport,
            office_worker: None,
        }
    }

    #[must_use]
    pub fn office_worker(mut self, value: OfficeWorkerConfig) -> Self {
        self.office_worker = Some(value);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DocumentSessionCommand {
    Viewer(DocumentViewerCommand),
    Surface(DocumentSurfaceCommand),
}

impl DocumentSessionCommand {
    #[must_use]
    pub const fn kind(self) -> DocumentSessionCommandKind {
        match self {
            Self::Viewer(DocumentViewerCommand::Previous) => DocumentSessionCommandKind::Previous,
            Self::Viewer(DocumentViewerCommand::Next) => DocumentSessionCommandKind::Next,
            Self::Viewer(DocumentViewerCommand::JumpTo(_)) => DocumentSessionCommandKind::JumpTo,
            Self::Viewer(DocumentViewerCommand::SetZoom(_)) => DocumentSessionCommandKind::SetZoom,
            Self::Viewer(DocumentViewerCommand::Fit(_)) => DocumentSessionCommandKind::Fit,
            Self::Viewer(DocumentViewerCommand::CopySelection) => {
                DocumentSessionCommandKind::CopySelection
            }
            Self::Viewer(DocumentViewerCommand::OpenTarget) => {
                DocumentSessionCommandKind::OpenTarget
            }
            Self::Surface(DocumentSurfaceCommand::Resize(_)) => DocumentSessionCommandKind::Resize,
            Self::Surface(DocumentSurfaceCommand::Grid(_)) => DocumentSessionCommandKind::Grid,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentSessionCommandKind {
    Previous,
    Next,
    JumpTo,
    SetZoom,
    Fit,
    CopySelection,
    OpenTarget,
    Resize,
    Grid,
    SpreadsheetFilter,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DocumentSessionEvent {
    None,
    Viewer(super::DocumentViewerEvent),
    Grid(DocumentGridEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSessionInfo {
    pub identity: ViewerSourceIdentity,
    pub mime: String,
    pub format: ViewerDocumentFormat,
    pub capabilities: ViewerCapabilities,
    pub diagnostics: Vec<ViewerDiagnostic>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentFrame {
    pub surface: DocumentSurfaceFrame,
    pub state: DocumentViewerState,
    pub capabilities: ViewerCapabilities,
    pub diagnostics: Vec<ViewerDiagnostic>,
    pub format: ViewerDocumentFormat,
    pub spreadsheet: Option<SpreadsheetFrameMetadata>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DocumentSessionError {
    #[error("document session is closed")]
    Closed,
    #[error("Office worker configuration is required for {format:?}")]
    MissingOfficeWorker { format: OfficeDocumentFormat },
    #[error("command {command:?} is unsupported for {format:?}")]
    UnsupportedCommand {
        format: ViewerDocumentFormat,
        command: DocumentSessionCommandKind,
    },
    #[error(transparent)]
    Pdf(#[from] PdfViewerError),
    #[error(transparent)]
    Office(#[from] OfficeWorkerError),
    #[error(transparent)]
    State(#[from] super::DocumentViewerStateError),
    #[error(transparent)]
    Surface(#[from] DocumentSurfaceError),
}
