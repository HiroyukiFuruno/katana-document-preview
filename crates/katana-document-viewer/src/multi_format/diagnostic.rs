use super::{ViewerFeature, ViewerFeatureStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewerDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewerDiagnosticCode {
    UnsupportedFormat,
    UnsupportedFeature,
    InvalidDocument,
    PasswordProtected,
    ResourceLimitExceeded,
    ActiveContentBlocked,
    ExternalResourceBlocked,
    WorkerUnavailable,
    WorkerTimedOut,
    WorkerCrashed,
    EngineFailure,
    DegradedRendering,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewerDiagnostic {
    pub code: ViewerDiagnosticCode,
    pub severity: ViewerDiagnosticSeverity,
    pub feature: Option<ViewerFeature>,
    pub status: Option<ViewerFeatureStatus>,
    pub message: String,
}

impl ViewerDiagnostic {
    #[must_use]
    pub fn unsupported(feature: ViewerFeature) -> Self {
        Self {
            code: ViewerDiagnosticCode::UnsupportedFeature,
            severity: ViewerDiagnosticSeverity::Warning,
            feature: Some(feature),
            status: Some(ViewerFeatureStatus::Unsupported),
            message: format!("viewer feature `{feature:?}` is unsupported by this profile"),
        }
    }
}
