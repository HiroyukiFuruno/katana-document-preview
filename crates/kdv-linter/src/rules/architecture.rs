use crate::diagnostics::{KdvLintError, Violation};
use crate::workspace::WorkspaceModel;

use super::manifest_boundary::ManifestBoundaryRule;

pub const VIEWER_CRATE: &str = "crates/katana-document-viewer";

pub struct ArchitectureRule;

impl ArchitectureRule {
    pub fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KdvLintError> {
        let mut violations = Vec::new();
        violations.extend(ManifestBoundaryRule::check(workspace.root())?);
        Ok(violations)
    }
}

#[cfg(test)]
#[path = "architecture_tests.rs"]
mod tests;
