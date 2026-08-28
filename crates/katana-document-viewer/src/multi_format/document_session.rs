use super::{
    DocumentSessionCommand, DocumentSessionConfig, DocumentSessionError, DocumentSessionEvent,
    DocumentSessionInfo, OfficeDocumentFormat, SpreadsheetFilterCommand, SpreadsheetFilterEvent,
    ViewerDocumentFormat, ViewerFeatureStatus, ViewerSource,
    document_session_paged::PagedDocumentSession,
    document_session_spreadsheet::SpreadsheetDocumentSession,
};
use crate::DocumentFrame;
use command_support::command_feature;

pub struct DocumentSession {
    runtime: Option<DocumentSessionRuntime>,
    info: DocumentSessionInfo,
    resource_lease: Option<super::resource_metrics::DocumentSessionLease>,
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
        Ok(Self {
            runtime: Some(runtime),
            info,
            resource_lease: Some(super::resource_metrics::DocumentSessionLease::acquire()),
        })
    }

    pub fn apply(
        &mut self,
        command: DocumentSessionCommand,
    ) -> Result<DocumentSessionEvent, DocumentSessionError> {
        self.ensure_supported(command)?;
        let runtime = match self.runtime.as_mut() {
            Some(runtime) => runtime,
            None => return Err(DocumentSessionError::Closed),
        };
        match runtime {
            DocumentSessionRuntime::Paged(runtime) => runtime.apply(command),
            DocumentSessionRuntime::Spreadsheet(runtime) => runtime.apply(command),
        }
    }

    pub fn frame(&mut self) -> Result<DocumentFrame, DocumentSessionError> {
        let runtime = match self.runtime.as_mut() {
            Some(runtime) => runtime,
            None => return Err(DocumentSessionError::Closed),
        };
        match runtime {
            DocumentSessionRuntime::Paged(runtime) => runtime.frame(),
            DocumentSessionRuntime::Spreadsheet(runtime) => runtime.frame(),
        }
    }

    pub fn apply_spreadsheet_filter(
        &mut self,
        command: SpreadsheetFilterCommand,
    ) -> Result<SpreadsheetFilterEvent, DocumentSessionError> {
        let runtime = match self.runtime.as_mut() {
            Some(runtime) => runtime,
            None => return Err(DocumentSessionError::Closed),
        };
        match runtime {
            DocumentSessionRuntime::Spreadsheet(runtime) => runtime.apply_filter(command),
            DocumentSessionRuntime::Paged(_) => Err(DocumentSessionError::UnsupportedCommand {
                format: self.info.format,
                command: super::DocumentSessionCommandKind::SpreadsheetFilter,
            }),
        }
    }

    #[must_use]
    pub const fn info(&self) -> &DocumentSessionInfo {
        &self.info
    }

    #[must_use]
    pub fn resource_snapshot() -> super::DocumentResourceSnapshot {
        super::DocumentResourceSnapshot::capture()
    }

    #[must_use]
    pub const fn is_closed(&self) -> bool {
        self.runtime.is_none()
    }

    pub fn close(&mut self) {
        let _close = super::debug_trace::DebugTrace::start("document.close");
        self.runtime.take();
        self.resource_lease.take();
    }

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

impl Drop for DocumentSession {
    fn drop(&mut self) {
        let _drop = super::debug_trace::DebugTrace::start("document.drop");
        self.close();
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

fn required_worker(
    config: &DocumentSessionConfig,
    format: OfficeDocumentFormat,
) -> Result<super::OfficeWorkerConfig, DocumentSessionError> {
    config
        .office_worker
        .clone()
        .ok_or(DocumentSessionError::MissingOfficeWorker { format })
}

#[path = "document_session_command.rs"]
mod command_support;
#[cfg(test)]
#[path = "document_session_tests.rs"]
mod tests;
