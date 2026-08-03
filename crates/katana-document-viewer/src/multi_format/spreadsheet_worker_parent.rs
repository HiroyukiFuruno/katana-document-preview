use super::spreadsheet_worker_process::SpreadsheetWorkerProcess;
use super::spreadsheet_worker_protocol::{SpreadsheetWorkerRequest, SpreadsheetWorkerResponse};
use super::{
    OfficeDocumentFormat, OfficeDocumentSource, OfficePackagePreflight, OfficeWorkerConfig,
    OfficeWorkerError, SpreadsheetCellArtifact, SpreadsheetCoordinate, SpreadsheetDocumentArtifact,
    SpreadsheetSheetArtifact, ViewerQualityProfile,
};

pub struct SpreadsheetViewerSession {
    artifact: SpreadsheetDocumentArtifact,
    worker: SpreadsheetWorkerProcess,
    next_request_id: u64,
}

impl std::fmt::Debug for SpreadsheetViewerSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SpreadsheetViewerSession")
            .field("artifact", &self.artifact)
            .finish_non_exhaustive()
    }
}

impl SpreadsheetViewerSession {
    pub fn open(
        source: OfficeDocumentSource,
        config: OfficeWorkerConfig,
    ) -> Result<Self, OfficeWorkerError> {
        if source.format != OfficeDocumentFormat::Xlsx {
            return Err(OfficeWorkerError::UnsupportedFormat(source.format));
        }
        OfficePackagePreflight::inspect(&source, config.preflight_limits)?;
        let mut worker = SpreadsheetWorkerProcess::spawn(&source, &config)?;
        let sheets = opened_sheets(worker.receive()?)?;
        let profile = ViewerQualityProfile::interactive_grid();
        let diagnostics = profile.diagnostics();
        let artifact = SpreadsheetDocumentArtifact {
            identity: source.identity,
            mime: source.mime,
            sheet_count: sheets.len(),
            sheets,
            capabilities: profile.capabilities,
            diagnostics,
        };
        Ok(Self {
            artifact,
            worker,
            next_request_id: 1,
        })
    }

    #[must_use]
    pub const fn artifact(&self) -> &SpreadsheetDocumentArtifact {
        &self.artifact
    }

    pub fn materialize_cells(
        &mut self,
        sheet_index: usize,
        coordinates: Vec<SpreadsheetCoordinate>,
    ) -> Result<Vec<SpreadsheetCellArtifact>, OfficeWorkerError> {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1);
        self.worker.send(&SpreadsheetWorkerRequest::Materialize {
            request_id,
            sheet_index,
            coordinates,
        })?;
        materialized_cells(request_id, self.worker.receive()?)
    }
}

fn opened_sheets(
    response: SpreadsheetWorkerResponse,
) -> Result<Vec<SpreadsheetSheetArtifact>, OfficeWorkerError> {
    match response {
        SpreadsheetWorkerResponse::Opened { sheets } => Ok(sheets),
        response => Err(unexpected_response("open", response)),
    }
}

fn materialized_cells(
    request_id: u64,
    response: SpreadsheetWorkerResponse,
) -> Result<Vec<SpreadsheetCellArtifact>, OfficeWorkerError> {
    match response {
        SpreadsheetWorkerResponse::Materialized {
            request_id: response_id,
            cells,
        } if response_id == request_id => Ok(cells),
        SpreadsheetWorkerResponse::Failed {
            request_id: Some(response_id),
            stage,
            message,
        } if response_id == request_id => Err(OfficeWorkerError::EngineFailure { stage, message }),
        response => Err(unexpected_response("materialize", response)),
    }
}

fn unexpected_response(operation: &str, response: SpreadsheetWorkerResponse) -> OfficeWorkerError {
    OfficeWorkerError::protocol(format!(
        "unexpected spreadsheet response during {operation}: {response:?}"
    ))
}

#[cfg(test)]
#[path = "spreadsheet_worker_parent_tests.rs"]
mod tests;
