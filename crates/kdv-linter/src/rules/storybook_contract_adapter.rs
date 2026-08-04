use crate::diagnostics::Violation;
use crate::workspace::SourceFile;
use std::path::PathBuf;

pub(super) struct AdapterContractChecker<'a> {
    file: &'a SourceFile,
}

impl<'a> AdapterContractChecker<'a> {
    pub(super) fn new(file: &'a SourceFile) -> Self {
        Self { file }
    }

    pub(super) fn violations(&self) -> Vec<Violation> {
        if !self.is_document_surface_source() {
            return Vec::new();
        }
        let mut violations = Vec::new();
        violations.extend(self.style_class_contract_violations());
        violations.extend(self.interactive_preset_override_violations());
        violations
    }

    fn is_document_surface_source(&self) -> bool {
        self.file
            .path_contains("crates/katana-document-viewer/src/document_surface")
    }

    fn style_class_contract_violations(&self) -> Vec<Violation> {
        if self.is_test_source() {
            return Vec::new();
        }
        self.file
            .source()
            .lines()
            .enumerate()
            .filter_map(|(index, line)| self.style_class_violation(index, line))
            .collect()
    }

    fn style_class_violation(&self, index: usize, line: &str) -> Option<Violation> {
        forbidden_media_style_class(line).map(|(needle, column)| {
            Violation::new(
                PathBuf::from(self.file.path()),
                index + 1,
                column + 1,
                "no_style_class_action_contract",
                format!("{needle} must not be a media control contract."),
            )
        })
    }

    fn interactive_preset_override_violations(&self) -> Vec<Violation> {
        if self.is_test_source() {
            return Vec::new();
        }
        self.file
            .source()
            .lines()
            .enumerate()
            .filter_map(|(index, line)| self.interactive_preset_violation(index, line))
            .collect()
    }

    fn is_test_source(&self) -> bool {
        self.file.path_ends_with("_tests.rs") || self.file.path_ends_with("/tests.rs")
    }

    fn interactive_preset_violation(&self, index: usize, line: &str) -> Option<Violation> {
        line.find("cursor(UiCursor::Pointer)").map(|column| {
            Violation::new(
                PathBuf::from(self.file.path()),
                index + 1,
                column + 1,
                "no_manual_interactive_preset_override",
                "KDV document surface must use KUC interactive presets instead of a manual pointer cursor.",
            )
        })
    }
}

fn forbidden_media_style_class(line: &str) -> Option<(&'static str, usize)> {
    ["kdv-diagram-", "kdv-image-control", "kdv-code-control"]
        .iter()
        .find_map(|needle| line.find(needle).map(|column| (*needle, column)))
}
