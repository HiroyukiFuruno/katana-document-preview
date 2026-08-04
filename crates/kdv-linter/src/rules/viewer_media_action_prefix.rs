use crate::diagnostics::{KdvLintError, Violation};
use crate::workspace::{SourceFile, WorkspaceModel};
use std::path::PathBuf;

pub struct ViewerMediaActionPrefixRule;

impl ViewerMediaActionPrefixRule {
    pub fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KdvLintError> {
        let mut violations = Vec::new();
        for file in workspace.rust_files() {
            violations.extend(ViewerMediaActionPrefixChecker::new(file).violations());
        }
        Ok(violations)
    }
}

struct ViewerMediaActionPrefixChecker<'a> {
    file: &'a SourceFile,
}

impl<'a> ViewerMediaActionPrefixChecker<'a> {
    fn new(file: &'a SourceFile) -> Self {
        Self { file }
    }

    fn violations(&self) -> Vec<Violation> {
        if self.is_owner_or_linter_fixture() {
            return Vec::new();
        }
        self.file
            .source()
            .lines()
            .enumerate()
            .filter_map(|(index, line)| self.violation(index, line))
            .collect()
    }

    fn is_owner_or_linter_fixture(&self) -> bool {
        self.file
            .path_contains("crates/katana-document-viewer/src/viewer/media_action")
            || self
                .file
                .path_contains("crates/katana-document-viewer/src/viewer/media_control_spec")
            || self.file.path_contains("crates/kdv-linter/")
    }

    fn violation(&self, index: usize, line: &str) -> Option<Violation> {
        let (needle, column) = Self::forbidden_prefix(line)?;
        Some(Violation::new(
            PathBuf::from(self.file.path()),
            index + 1,
            column + 1,
            "no_duplicate_viewer_media_action_prefix",
            format!("{needle} is owned by KDV core ViewerMediaControlAction, not adapters."),
        ))
    }

    fn forbidden_prefix(line: &str) -> Option<(&'static str, usize)> {
        ["viewer.image.", "viewer.diagram.", "viewer.code."]
            .iter()
            .find_map(|needle| line.find(needle).map(|column| (*needle, column)))
    }
}

#[cfg(test)]
#[path = "viewer_media_action_prefix_tests.rs"]
mod tests;
