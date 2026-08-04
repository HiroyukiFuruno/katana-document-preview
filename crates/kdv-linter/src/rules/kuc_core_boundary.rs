use crate::diagnostics::{KdvLintError, Violation};
use crate::workspace::{SourceFile, WorkspaceModel};
use std::path::PathBuf;

pub struct KucCoreBoundaryRule;

impl KucCoreBoundaryRule {
    pub fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KdvLintError> {
        let mut violations = Vec::new();
        for file in workspace.kuc_core_files() {
            violations.extend(KucCoreBoundaryChecker::new(file).violations());
        }
        Ok(violations)
    }
}

struct KucCoreBoundaryChecker<'a> {
    file: &'a SourceFile,
}

impl<'a> KucCoreBoundaryChecker<'a> {
    fn new(file: &'a SourceFile) -> Self {
        Self { file }
    }

    fn violations(&self) -> Vec<Violation> {
        let mut violations = self
            .file
            .source()
            .lines()
            .enumerate()
            .filter_map(|(index, line)| self.violation(index, line))
            .collect::<Vec<_>>();
        violations.extend(self.file_tree_facade_violations());
        violations
    }

    fn violation(&self, index: usize, line: &str) -> Option<Violation> {
        let (needle, column) = forbidden_viewer_semantic(line)?;
        Some(Violation::new(
            PathBuf::from(self.file.path()),
            index + 1,
            column + 1,
            "kuc-core-boundary",
            format!("{needle} is a KDV viewer semantic and must not live in KUC core."),
        ))
    }
}

impl<'a> KucCoreBoundaryChecker<'a> {
    fn file_tree_facade_violations(&self) -> Vec<Violation> {
        if !self.is_file_tree_source() {
            return Vec::new();
        }
        let mut violations = Vec::new();
        violations.extend(self.required_file_tree_patterns());
        violations.extend(self.forbidden_file_tree_row_ui());
        violations
    }

    fn is_file_tree_source(&self) -> bool {
        if self.is_test_source() {
            return false;
        }
        self.file.source().contains("pub struct FileTree")
            && self.file.source().contains("impl FileTree")
    }

    fn is_test_source(&self) -> bool {
        self.file.path_ends_with("_tests.rs") || self.file.path_ends_with("/tests.rs")
    }

    fn required_file_tree_patterns(&self) -> Vec<Violation> {
        [
            ("TreeView::new", "FileTree must build KUC TreeView."),
            (
                "TreeViewHitTestInput",
                "FileTree must delegate hit-test to KUC TreeView.",
            ),
            (
                "TreeViewAction::SelectNode",
                "FileTree must map KUC TreeView selection action.",
            ),
            (
                "TreeViewAction::ToggleNode",
                "FileTree must map KUC TreeView directory toggle action.",
            ),
        ]
        .iter()
        .filter_map(|(needle, message)| self.missing_file_tree_pattern(needle, message))
        .collect()
    }

    fn missing_file_tree_pattern(&self, needle: &str, message: &str) -> Option<Violation> {
        if self.file.source().contains(needle) {
            return None;
        }
        Some(self.file_tree_violation(1, 1, message))
    }

    fn forbidden_file_tree_row_ui(&self) -> Vec<Violation> {
        ["Text::new", "Row::new", "Column::new", "UiNodeKind::Text"]
            .iter()
            .filter_map(|needle| self.find_file_tree_forbidden_row_ui(needle))
            .collect()
    }

    fn find_file_tree_forbidden_row_ui(&self, needle: &str) -> Option<Violation> {
        self.file
            .source()
            .lines()
            .enumerate()
            .find_map(|(index, line)| {
                line.find(needle).map(|column| {
                    self.file_tree_violation(
                        index + 1,
                        column + 1,
                        "FileTree must stay a TreeView facade, not build separate row UI.",
                    )
                })
            })
    }

    fn file_tree_violation(&self, line: usize, column: usize, message: &str) -> Violation {
        Violation::new(
            PathBuf::from(self.file.path()),
            line,
            column,
            "kuc-file-tree-facade",
            message.to_string(),
        )
    }
}

fn forbidden_viewer_semantic(line: &str) -> Option<(&'static str, usize)> {
    [
        "ViewerMediaControl",
        "UiMediaControl",
        "media_control_action",
        "viewer.image",
        "viewer.diagram",
        "viewer.code",
        "ui.diagram",
        "diagram.zoom",
        "kdv-",
        "MediaControl",
    ]
    .iter()
    .find_map(|needle| line.find(needle).map(|column| (*needle, column)))
}

#[cfg(test)]
#[path = "kuc_core_boundary_tests.rs"]
mod tests;
