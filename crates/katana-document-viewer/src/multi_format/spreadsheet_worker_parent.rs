use super::spreadsheet_cell_cache::SpreadsheetCellCache;
use super::spreadsheet_materialization_validation::SpreadsheetMaterializationValidator;
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
    cell_cache: SpreadsheetCellCache,
    materialized_cell_limit: usize,
    trace_session: Option<super::debug_trace::TraceSession>,
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
        Self::open_xlsx(source, config)
    }

    fn open_xlsx(
        source: OfficeDocumentSource,
        config: OfficeWorkerConfig,
    ) -> Result<Self, OfficeWorkerError> {
        let trace_session =
            super::office_worker_parent::trace::start_trace_session(&source.identity);
        let _trace_scope = trace_session.map(super::debug_trace::DebugTrace::session);
        let _open = super::debug_trace::DebugTrace::start("spreadsheet.session_open");
        let (_, preflight_diagnostics) =
            OfficePackagePreflight::inspect_with_diagnostics(&source, config.preflight_limits)?;
        let mut worker = SpreadsheetWorkerProcess::spawn(&source, &config)?;
        let sheets = {
            let _engine = super::debug_trace::DebugTrace::start("spreadsheet.engine_open");
            opened_sheets(worker.receive()?)?
        };
        let materialized_cell_limit = config.spreadsheet_limits.max_materialized_cells;
        let artifact = spreadsheet_artifact(source, sheets, preflight_diagnostics);
        Ok(Self {
            artifact,
            worker,
            next_request_id: 1,
            cell_cache: SpreadsheetCellCache::new(),
            materialized_cell_limit,
            trace_session,
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
        let _trace_scope = self.trace_scope();
        let _materialize = super::debug_trace::DebugTrace::start("spreadsheet.materialize");
        SpreadsheetMaterializationValidator::validate(
            &self.artifact,
            self.materialized_cell_limit,
            sheet_index,
            &coordinates,
        )?;
        let missing = self.cell_cache.missing(sheet_index, &coordinates);
        if missing.is_empty() {
            super::debug_trace::DebugTrace::event("spreadsheet.cell_cache", "hit=true");
            return self.cell_cache.resolve(sheet_index, &coordinates);
        }
        let materialized = self.materialize_missing(sheet_index, missing)?;
        self.cell_cache
            .resolve_materialized(sheet_index, &coordinates, materialized)
    }

    pub(super) fn trace_scope(&self) -> Option<super::debug_trace::TraceCorrelationGuard> {
        self.trace_session
            .map(super::debug_trace::DebugTrace::session)
    }

    fn materialize_missing(
        &mut self,
        sheet_index: usize,
        missing: Vec<SpreadsheetCoordinate>,
    ) -> Result<Vec<SpreadsheetCellArtifact>, OfficeWorkerError> {
        super::debug_trace::DebugTrace::event(
            "spreadsheet.cell_cache",
            format_args!("hit=false missing={}", missing.len()),
        );
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1);
        self.worker.send(&SpreadsheetWorkerRequest::Materialize {
            request_id,
            sheet_index,
            coordinates: missing,
        })?;
        materialized_cells(request_id, self.worker.receive()?)
    }
}

fn spreadsheet_artifact(
    source: OfficeDocumentSource,
    sheets: Vec<SpreadsheetSheetArtifact>,
    preflight_diagnostics: Vec<super::ViewerDiagnostic>,
) -> SpreadsheetDocumentArtifact {
    let profile = ViewerQualityProfile::interactive_grid();
    let mut diagnostics = profile.diagnostics();
    diagnostics.extend(preflight_diagnostics);
    SpreadsheetDocumentArtifact {
        identity: source.identity,
        mime: source.mime,
        sheet_count: sheets.len(),
        sheets,
        capabilities: profile.capabilities,
        diagnostics,
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

#[path = "spreadsheet_worker_parent_filter.rs"]
mod filter;

#[cfg(test)]
#[path = "spreadsheet_worker_parent_tests.rs"]
mod tests;
