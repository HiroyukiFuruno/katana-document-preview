use super::{
    DocumentSessionCommand, DocumentSessionConfig, DocumentSessionError, DocumentSessionEvent,
    DocumentSessionInfo, OfficeDocumentFormat, ViewerDocumentFormat, ViewerFeature,
    ViewerFeatureStatus, ViewerSource, document_session_paged::PagedDocumentSession,
    document_session_spreadsheet::SpreadsheetDocumentSession,
};
use crate::DocumentFrame;

pub struct DocumentSession {
    runtime: DocumentSessionRuntime,
    info: DocumentSessionInfo,
}

enum DocumentSessionRuntime {
    Paged(Box<PagedDocumentSession>),
    Spreadsheet(Box<SpreadsheetDocumentSession>),
}

impl DocumentSession {
    pub fn open(
        source: ViewerSource,
        config: DocumentSessionConfig,
    ) -> Result<Self, DocumentSessionError> {
        let identity = source.identity().clone();
        let mime = source.mime().to_owned();
        let runtime = open_runtime(source, &config)?;
        let info = session_info(identity, mime, &runtime);
        Ok(Self { runtime, info })
    }

    pub fn apply(
        &mut self,
        command: DocumentSessionCommand,
    ) -> Result<DocumentSessionEvent, DocumentSessionError> {
        self.ensure_supported(command)?;
        match &mut self.runtime {
            DocumentSessionRuntime::Paged(runtime) => runtime.apply(command),
            DocumentSessionRuntime::Spreadsheet(runtime) => runtime.apply(command),
        }
    }

    pub fn frame(&mut self) -> Result<DocumentFrame, DocumentSessionError> {
        match &mut self.runtime {
            DocumentSessionRuntime::Paged(runtime) => runtime.frame(),
            DocumentSessionRuntime::Spreadsheet(runtime) => runtime.frame(),
        }
    }

    #[must_use]
    pub const fn info(&self) -> &DocumentSessionInfo {
        &self.info
    }

    pub fn close(self) {}

    fn ensure_supported(
        &self,
        command: DocumentSessionCommand,
    ) -> Result<(), DocumentSessionError> {
        let Some(feature) = command_feature(self.info.format, command) else {
            return Ok(());
        };
        if self.info.capabilities.status(feature) == ViewerFeatureStatus::Supported {
            return Ok(());
        }
        Err(DocumentSessionError::UnsupportedCommand {
            format: self.info.format,
            command: command.kind(),
        })
    }
}

fn open_runtime(
    source: ViewerSource,
    config: &DocumentSessionConfig,
) -> Result<DocumentSessionRuntime, DocumentSessionError> {
    match source {
        ViewerSource::Pdf(source) => Ok(DocumentSessionRuntime::Paged(Box::new(
            PagedDocumentSession::open_pdf(source, config.viewport)?,
        ))),
        ViewerSource::Office(source) if source.format == OfficeDocumentFormat::Xlsx => {
            let worker = required_worker(config, source.format)?;
            Ok(DocumentSessionRuntime::Spreadsheet(Box::new(
                SpreadsheetDocumentSession::open(source, worker, config.viewport)?,
            )))
        }
        ViewerSource::Office(source) => {
            let worker = required_worker(config, source.format)?;
            Ok(DocumentSessionRuntime::Paged(Box::new(
                PagedDocumentSession::open_office(source, worker, config.viewport)?,
            )))
        }
    }
}

fn session_info(
    identity: super::ViewerSourceIdentity,
    mime: String,
    runtime: &DocumentSessionRuntime,
) -> DocumentSessionInfo {
    DocumentSessionInfo {
        identity,
        mime,
        format: runtime.format(),
        capabilities: runtime.capabilities().clone(),
        diagnostics: runtime.diagnostics().to_vec(),
    }
}

impl DocumentSessionRuntime {
    fn format(&self) -> ViewerDocumentFormat {
        match self {
            Self::Paged(runtime) => runtime.info_parts().0,
            Self::Spreadsheet(runtime) => runtime.info_parts().0,
        }
    }

    fn capabilities(&self) -> &super::ViewerCapabilities {
        match self {
            Self::Paged(runtime) => runtime.info_parts().1,
            Self::Spreadsheet(runtime) => runtime.info_parts().1,
        }
    }

    fn diagnostics(&self) -> &[super::ViewerDiagnostic] {
        match self {
            Self::Paged(runtime) => runtime.info_parts().2,
            Self::Spreadsheet(runtime) => runtime.info_parts().2,
        }
    }
}

const fn command_feature(
    format: ViewerDocumentFormat,
    command: DocumentSessionCommand,
) -> Option<ViewerFeature> {
    match command {
        DocumentSessionCommand::Viewer(super::DocumentViewerCommand::Previous)
        | DocumentSessionCommand::Viewer(super::DocumentViewerCommand::Next)
        | DocumentSessionCommand::Viewer(super::DocumentViewerCommand::JumpTo(_)) => {
            Some(navigation_feature(format))
        }
        DocumentSessionCommand::Viewer(super::DocumentViewerCommand::SetZoom(_)) => {
            Some(ViewerFeature::Zoom)
        }
        DocumentSessionCommand::Viewer(super::DocumentViewerCommand::Fit(_)) => {
            Some(ViewerFeature::Fit)
        }
        DocumentSessionCommand::Viewer(super::DocumentViewerCommand::CopySelection) => {
            Some(ViewerFeature::CopyText)
        }
        DocumentSessionCommand::Viewer(super::DocumentViewerCommand::OpenTarget) => {
            Some(ViewerFeature::OpenLink)
        }
        DocumentSessionCommand::Surface(crate::DocumentSurfaceCommand::Grid(_)) => {
            Some(ViewerFeature::GridNavigation)
        }
        DocumentSessionCommand::Surface(crate::DocumentSurfaceCommand::Resize(_)) => None,
    }
}

const fn navigation_feature(format: ViewerDocumentFormat) -> ViewerFeature {
    match format {
        ViewerDocumentFormat::Pdf | ViewerDocumentFormat::Docx => ViewerFeature::PageNavigation,
        ViewerDocumentFormat::Xlsx => ViewerFeature::SheetNavigation,
        ViewerDocumentFormat::Pptx => ViewerFeature::SlideNavigation,
    }
}

fn required_worker(
    config: &DocumentSessionConfig,
    format: OfficeDocumentFormat,
) -> Result<super::OfficeWorkerConfig, DocumentSessionError> {
    config
        .office_worker
        .clone()
        .ok_or(DocumentSessionError::MissingOfficeWorker { format })
}

#[cfg(test)]
#[path = "document_session_tests.rs"]
mod tests;
