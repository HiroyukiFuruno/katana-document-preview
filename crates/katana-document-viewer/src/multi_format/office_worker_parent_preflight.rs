use super::super::{OfficeDocumentSource, OfficePackagePreflight, ViewerDiagnostic};
use super::{OfficeWorkerConfig, OfficeWorkerError};

pub(super) fn preflight_diagnostics(
    source: &OfficeDocumentSource,
    config: &OfficeWorkerConfig,
) -> Result<Vec<ViewerDiagnostic>, OfficeWorkerError> {
    let _archive_intake = super::super::debug_trace::DebugTrace::start("office.archive_intake");
    let _package_parse = super::super::debug_trace::DebugTrace::start("office.package_parse");
    let _preflight = super::super::debug_trace::DebugTrace::start("office.preflight");
    let (_, diagnostics) =
        OfficePackagePreflight::inspect_with_diagnostics(source, config.preflight_limits)?;
    Ok(diagnostics)
}
