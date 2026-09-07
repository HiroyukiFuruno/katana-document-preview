#[path = "debug_trace_correlation.rs"]
mod correlation;

use correlation::TraceCorrelation;
#[cfg(windows)]
use std::ffi::OsString;
use std::time::Instant;

pub(crate) use correlation::{TraceCorrelationGuard, TraceSession};

pub(crate) struct DebugTrace {
    stage: &'static str,
    started_at: Option<Instant>,
    correlation: Option<TraceCorrelation>,
}

impl DebugTrace {
    pub(crate) fn session(session: TraceSession) -> TraceCorrelationGuard {
        correlation::session(session)
    }

    pub(crate) fn session_from_environment() -> Option<TraceCorrelationGuard> {
        let correlation = TraceCorrelation::from_environment()?;
        let expected_environment = correlation.worker_environment();
        let expected_session = correlation.session();
        let guard = Self::session(expected_session);
        (Self::current_session() == Some(expected_session)
            && Self::worker_environment() == Some(expected_environment))
        .then_some(guard)
    }

    pub(crate) fn session_from_environment_or_workspace(
        workspace: &std::path::Path,
    ) -> Option<TraceCorrelationGuard> {
        Self::session_from_environment().or_else(|| {
            Self::enabled()
                .then(|| correlation::workspace_session(workspace))
                .map(Self::session)
        })
    }

    pub(crate) fn current_session() -> Option<TraceSession> {
        correlation::current_session()
    }

    pub(crate) fn worker_environment() -> Option<(String, String)> {
        correlation::worker_environment()
    }

    #[cfg(windows)]
    pub(crate) fn configure_worker_environment(environment: &mut Vec<(OsString, OsString)>) {
        correlation::configure_worker_environment(environment);
    }

    #[must_use]
    pub(crate) fn start(stage: &'static str) -> Self {
        Self {
            stage,
            started_at: Self::enabled().then(Instant::now),
            correlation: correlation::current(),
        }
    }

    pub(crate) fn enabled() -> bool {
        std::env::var("DEBUG")
            .ok()
            .is_some_and(|value| debug_value_enabled(&value))
    }

    pub(crate) fn event(stage: &'static str, detail: impl std::fmt::Display) {
        if Self::enabled() {
            eprintln!("{}", Self::stage_row(stage, detail, correlation::current()));
        }
    }

    fn stage_row(
        stage: &str,
        detail: impl std::fmt::Display,
        correlation: Option<TraceCorrelation>,
    ) -> String {
        let correlation =
            correlation.map_or_else(String::new, |value| format!(" {}", value.fields()));
        format!("[KDV_TRACE] stage={stage}{correlation} {detail}")
    }
}

impl Drop for DebugTrace {
    fn drop(&mut self) {
        if let Some(started_at) = self.started_at {
            eprintln!(
                "{}",
                Self::stage_row(
                    self.stage,
                    format_args!("elapsed_ms={}", started_at.elapsed().as_millis()),
                    self.correlation.clone(),
                )
            );
        }
    }
}

fn debug_value_enabled(value: &str) -> bool {
    value.eq_ignore_ascii_case("true") || value == "1"
}

#[cfg(test)]
mod tests {
    use super::{DebugTrace, debug_value_enabled};

    #[test]
    fn debug_trace_requires_an_explicit_true_value() {
        assert!(debug_value_enabled("true"));
        assert!(debug_value_enabled("TRUE"));
        assert!(debug_value_enabled("1"));
        assert!(!debug_value_enabled("false"));
        assert!(!debug_value_enabled(""));
    }

    #[test]
    fn office_and_spreadsheet_stage_rows_share_session_source_correlation() {
        let _session = DebugTrace::session((42, 0x0123_4567_89ab_cdef));
        let office_trace = DebugTrace::start("office.worker_total");
        let spreadsheet_trace = DebugTrace::start("spreadsheet.package_parse");
        let office_row = DebugTrace::stage_row(
            office_trace.stage,
            "elapsed_ms=15",
            office_trace.correlation.clone(),
        );
        let spreadsheet_row = DebugTrace::stage_row(
            spreadsheet_trace.stage,
            "elapsed_ms=15",
            spreadsheet_trace.correlation.clone(),
        );
        for row in [&office_row, &spreadsheet_row] {
            assert!(row.contains("session=42"));
            assert!(row.contains("source="));
            assert!(!row.contains("report.xlsx"));
            assert!(!row.contains("rev-7"));
        }
        assert!(office_row.contains("stage=office.worker_total"));
        assert!(spreadsheet_row.contains("stage=spreadsheet.package_parse"));
    }
}
