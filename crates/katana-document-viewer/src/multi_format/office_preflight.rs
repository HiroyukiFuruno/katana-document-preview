use super::{
    OfficeDocumentFormat, OfficeDocumentSource, ViewerDiagnostic, ViewerDiagnosticCode,
    ViewerDiagnosticSeverity, ViewerFeature, ViewerFeatureStatus,
    office_preflight_archive::OfficePreflightArchive,
};
use thiserror::Error;

pub(crate) const MAX_NESTED_PACKAGE_DEPTH: usize = 2;
pub(crate) const MAX_WORKSHEET_UNCOMPRESSED_BYTES: u64 = 640 * 1024 * 1024;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OfficePreflightLimits {
    pub max_source_bytes: u64,
    pub max_entries: usize,
    pub max_entry_uncompressed_bytes: u64,
    pub max_total_uncompressed_bytes: u64,
    pub max_compression_ratio: u64,
    pub max_relationship_bytes: u64,
}

impl OfficePreflightLimits {
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            max_source_bytes: 128 * 1024 * 1024,
            max_entries: 4_096,
            max_entry_uncompressed_bytes: 16 * 1024 * 1024,
            max_total_uncompressed_bytes: 768 * 1024 * 1024,
            max_compression_ratio: 200,
            max_relationship_bytes: 2 * 1024 * 1024,
        }
    }
}

impl Default for OfficePreflightLimits {
    fn default() -> Self {
        Self::strict()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfficeResourceLimitKind {
    SourceBytes,
    EntryCount,
    EntryBytes,
    TotalUncompressedBytes,
    CompressionRatio,
    RelationshipBytes,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OfficePreflightError {
    #[error("Office MIME `{mime}` does not match declared format {format:?}")]
    UnsupportedMime {
        format: OfficeDocumentFormat,
        mime: String,
    },
    #[error("Office package is invalid: {reason}")]
    InvalidArchive { reason: String },
    #[error("Office package entry path is unsafe: {entry}")]
    UnsafeEntryName { entry: String },
    #[error("Office package contains blocked active content: {entry}")]
    ActiveContentBlocked { entry: String },
    #[error("Office package contains an external relationship in {entry}: {target}")]
    ExternalResourceBlocked { entry: String, target: String },
    #[error(
        "Office package resource limit exceeded: {kind:?}, actual={actual}, limit={limit}, entry={entry:?}"
    )]
    ResourceLimitExceeded {
        kind: OfficeResourceLimitKind,
        actual: u64,
        limit: u64,
        entry: Option<String>,
    },
}

impl OfficePreflightError {
    #[must_use]
    pub fn diagnostic(&self) -> ViewerDiagnostic {
        let (code, feature, status) = diagnostic_details(self);
        ViewerDiagnostic {
            code,
            severity: if matches!(self, Self::ActiveContentBlocked { .. }) {
                ViewerDiagnosticSeverity::Warning
            } else {
                ViewerDiagnosticSeverity::Error
            },
            feature,
            status,
            message: self.to_string(),
        }
    }
}

fn diagnostic_details(
    error: &OfficePreflightError,
) -> (
    ViewerDiagnosticCode,
    Option<ViewerFeature>,
    Option<ViewerFeatureStatus>,
) {
    match error {
        OfficePreflightError::UnsupportedMime { .. } => {
            (ViewerDiagnosticCode::UnsupportedFormat, None, None)
        }
        OfficePreflightError::InvalidArchive { .. }
        | OfficePreflightError::UnsafeEntryName { .. } => {
            (ViewerDiagnosticCode::InvalidDocument, None, None)
        }
        OfficePreflightError::ActiveContentBlocked { .. } => (
            ViewerDiagnosticCode::ActiveContentBlocked,
            Some(ViewerFeature::Macro),
            Some(ViewerFeatureStatus::Blocked),
        ),
        OfficePreflightError::ExternalResourceBlocked { .. } => (
            ViewerDiagnosticCode::ExternalResourceBlocked,
            Some(ViewerFeature::ExternalResource),
            Some(ViewerFeatureStatus::Blocked),
        ),
        OfficePreflightError::ResourceLimitExceeded { .. } => {
            (ViewerDiagnosticCode::ResourceLimitExceeded, None, None)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfficePreflightReport {
    pub entry_count: usize,
    pub total_compressed_bytes: u64,
    pub total_uncompressed_bytes: u64,
    pub external_relationship_count: usize,
}

pub struct OfficePackagePreflight;

impl OfficePackagePreflight {
    pub fn inspect(
        source: &OfficeDocumentSource,
        limits: OfficePreflightLimits,
    ) -> Result<OfficePreflightReport, OfficePreflightError> {
        OfficePreflightArchive::inspect(source, limits, 0).map(|(report, _)| report)
    }

    pub(crate) fn inspect_with_diagnostics(
        source: &OfficeDocumentSource,
        limits: OfficePreflightLimits,
    ) -> Result<(OfficePreflightReport, Vec<ViewerDiagnostic>), OfficePreflightError> {
        OfficePreflightArchive::inspect(source, limits, 0)
    }
}

pub(crate) struct OfficePreflightSupport;

impl OfficePreflightSupport {
    pub(crate) const fn expected_mime(format: OfficeDocumentFormat) -> &'static str {
        match format {
            OfficeDocumentFormat::Docx => {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            }
            OfficeDocumentFormat::Xlsx => {
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            }
            OfficeDocumentFormat::Pptx => {
                "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            }
        }
    }

    pub(crate) fn resource_limit(
        kind: OfficeResourceLimitKind,
        actual: u64,
        limit: u64,
        entry: Option<String>,
    ) -> OfficePreflightError {
        OfficePreflightError::ResourceLimitExceeded {
            kind,
            actual,
            limit,
            entry,
        }
    }

    pub(crate) fn invalid_archive(reason: String) -> OfficePreflightError {
        OfficePreflightError::InvalidArchive { reason }
    }

    pub(crate) fn archive_error(error: impl ToString) -> OfficePreflightError {
        Self::invalid_archive(error.to_string())
    }
}

#[cfg(test)]
#[path = "office_preflight_tests.rs"]
mod tests;
