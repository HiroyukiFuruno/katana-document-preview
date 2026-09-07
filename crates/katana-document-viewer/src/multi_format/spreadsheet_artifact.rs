use super::{SpreadsheetSheetArtifact, ViewerCapabilities, ViewerDiagnostic, ViewerSourceIdentity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpreadsheetDocumentArtifact {
    pub identity: ViewerSourceIdentity,
    pub mime: String,
    pub sheet_count: usize,
    pub sheets: Vec<SpreadsheetSheetArtifact>,
    pub capabilities: ViewerCapabilities,
    pub diagnostics: Vec<ViewerDiagnostic>,
}
