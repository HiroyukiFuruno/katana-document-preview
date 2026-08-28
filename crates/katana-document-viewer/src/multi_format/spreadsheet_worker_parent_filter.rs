use super::SpreadsheetViewerSession;
use crate::multi_format::spreadsheet_worker_protocol::{
    SpreadsheetWorkerRequest, SpreadsheetWorkerResponse,
};
use crate::multi_format::{OfficeWorkerError, spreadsheet_worker_parent::unexpected_response};
use crate::multi_format::{SpreadsheetFilterCommand, SpreadsheetFilterEvent};

impl SpreadsheetViewerSession {
    pub fn apply_filter(
        &mut self,
        command: SpreadsheetFilterCommand,
    ) -> Result<SpreadsheetFilterEvent, OfficeWorkerError> {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1);
        self.worker.send(&filter_request(request_id, command))?;
        let event = filter_event(request_id, self.worker.receive()?)?;
        self.update_filter_artifact(&event);
        Ok(event)
    }

    fn update_filter_artifact(&mut self, event: &SpreadsheetFilterEvent) {
        match event {
            SpreadsheetFilterEvent::Candidates {
                sheet_index,
                column,
                values,
                ..
            } => self.update_candidates(*sheet_index, *column, values),
            SpreadsheetFilterEvent::VisibilityChanged {
                sheet_index,
                filtered_out_rows,
                ..
            } => self.update_visibility(*sheet_index, filtered_out_rows),
        }
    }

    pub(super) fn update_candidates(
        &mut self,
        sheet_index: usize,
        column: usize,
        values: &[String],
    ) {
        let Some(filter) = self
            .artifact
            .sheets
            .get_mut(sheet_index)
            .and_then(|sheet| sheet.auto_filter.as_mut())
        else {
            return;
        };
        if let Some(filter_column) = filter
            .columns
            .iter_mut()
            .find(|candidate| candidate.column == column)
        {
            filter_column.candidates = values.to_vec();
        }
    }

    fn update_visibility(&mut self, sheet_index: usize, filtered_out_rows: &[usize]) {
        if let Some(filter) = self
            .artifact
            .sheets
            .get_mut(sheet_index)
            .and_then(|sheet| sheet.auto_filter.as_mut())
        {
            filter.filtered_out_rows = filtered_out_rows.to_vec();
        }
    }
}

fn filter_request(request_id: u64, command: SpreadsheetFilterCommand) -> SpreadsheetWorkerRequest {
    match command {
        SpreadsheetFilterCommand::Candidates {
            sheet_index,
            column,
            limit,
        } => candidate_request(request_id, sheet_index, column, limit),
        SpreadsheetFilterCommand::ApplyValues {
            sheet_index,
            column,
            values,
        } => apply_request(request_id, sheet_index, column, values),
        SpreadsheetFilterCommand::Clear {
            sheet_index,
            column,
        } => SpreadsheetWorkerRequest::ClearFilter {
            request_id,
            sheet_index,
            column,
        },
    }
}

const fn candidate_request(
    request_id: u64,
    sheet_index: usize,
    column: usize,
    limit: usize,
) -> SpreadsheetWorkerRequest {
    SpreadsheetWorkerRequest::FilterCandidates {
        request_id,
        sheet_index,
        column,
        limit,
    }
}

fn apply_request(
    request_id: u64,
    sheet_index: usize,
    column: usize,
    values: Vec<String>,
) -> SpreadsheetWorkerRequest {
    SpreadsheetWorkerRequest::ApplyFilter {
        request_id,
        sheet_index,
        column,
        values,
    }
}

pub(super) fn filter_event(
    request_id: u64,
    response: SpreadsheetWorkerResponse,
) -> Result<SpreadsheetFilterEvent, OfficeWorkerError> {
    match response {
        SpreadsheetWorkerResponse::Failed {
            request_id: Some(response_id),
            stage,
            message,
        } if response_id == request_id => Err(OfficeWorkerError::EngineFailure { stage, message }),
        response => filter_success_event(request_id, response),
    }
}

fn filter_success_event(
    request_id: u64,
    response: SpreadsheetWorkerResponse,
) -> Result<SpreadsheetFilterEvent, OfficeWorkerError> {
    match response {
        SpreadsheetWorkerResponse::FilterCandidates {
            request_id: response_id,
            sheet_index,
            column,
            values,
            truncated,
        } if response_id == request_id => {
            Ok(candidate_event(sheet_index, column, values, truncated))
        }
        SpreadsheetWorkerResponse::FilterVisibility {
            request_id: response_id,
            sheet_index,
            applied_columns,
            visible_row_count,
            filtered_out_rows,
        } if response_id == request_id => Ok(visibility_event(
            sheet_index,
            applied_columns,
            visible_row_count,
            filtered_out_rows,
        )),
        response => Err(unexpected_response("filter", response)),
    }
}

fn candidate_event(
    sheet_index: usize,
    column: usize,
    values: Vec<String>,
    truncated: bool,
) -> SpreadsheetFilterEvent {
    SpreadsheetFilterEvent::Candidates {
        sheet_index,
        column,
        values,
        truncated,
    }
}

fn visibility_event(
    sheet_index: usize,
    applied_columns: Vec<usize>,
    visible_row_count: usize,
    filtered_out_rows: Vec<usize>,
) -> SpreadsheetFilterEvent {
    SpreadsheetFilterEvent::VisibilityChanged {
        sheet_index,
        applied_columns,
        visible_row_count,
        filtered_out_rows,
    }
}
