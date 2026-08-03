use crate::diagnostics::{KdvLintError, Violation};
use crate::workspace::{SourceFile, WorkspaceModel};
use std::path::{Path, PathBuf};

#[path = "kdv_ui_adapter_patterns.rs"]
mod patterns;
use patterns::StorybookAdapterPattern;

pub struct KdvUiAdapterOwnershipRule;

impl KdvUiAdapterOwnershipRule {
    pub fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KdvLintError> {
        let mut violations = Vec::new();
        violations.extend(KdvCoreAdapterDependencyChecker::new(workspace).violations()?);
        violations.extend(StorybookOwnedBridgeChecker::new(workspace.root()).violations());
        for file in workspace.storybook_files() {
            violations.extend(StorybookAdapterChecker::new(file).violations());
        }
        Ok(violations)
    }
}

struct KdvCoreAdapterDependencyChecker<'a> {
    workspace: &'a WorkspaceModel,
}

impl<'a> KdvCoreAdapterDependencyChecker<'a> {
    fn new(workspace: &'a WorkspaceModel) -> Self {
        Self { workspace }
    }

    fn violations(&self) -> Result<Vec<Violation>, KdvLintError> {
        let core_root = self
            .workspace
            .root()
            .join("crates")
            .join("katana-document-viewer");
        let mut violations = Vec::new();
        let manifest = core_root.join("Cargo.toml");
        if manifest.exists() {
            let source =
                std::fs::read_to_string(&manifest).map_err(|source| KdvLintError::Read {
                    path: manifest.clone(),
                    source,
                })?;
            violations.extend(Self::find_references(&manifest, &source));
        }
        let source_root = core_root.join("src");
        for file in self
            .workspace
            .rust_files()
            .iter()
            .filter(|file| file.is_under(&source_root))
        {
            violations.extend(Self::find_references(file.path(), file.source()));
        }
        Ok(violations)
    }

    fn find_references(path: &Path, source: &str) -> Vec<Violation> {
        source
            .lines()
            .enumerate()
            .filter_map(|(index, line)| {
                ["katana-document-viewer-kuc", "katana_document_viewer_kuc"]
                    .into_iter()
                    .find_map(|needle| line.find(needle))
                    .map(|column| {
                        Violation::new(
                            path.to_path_buf(),
                            index + 1,
                            column + 1,
                            "no_kdv_ui_adapter_ownership",
                            "Neutral KDV core must not depend on its optional KUC presentation adapter.",
                        )
                    })
            })
            .collect()
    }
}

struct StorybookOwnedBridgeChecker<'a> {
    root: &'a Path,
}

impl<'a> StorybookOwnedBridgeChecker<'a> {
    fn new(root: &'a Path) -> Self {
        Self { root }
    }

    fn violations(&self) -> Vec<Violation> {
        let module_path = self.module_path();
        if !module_path.exists() {
            return Vec::new();
        }
        vec![Violation::new(
            module_path,
            1,
            1,
            "no_kdv_ui_adapter_ownership",
            "KDV Storybook must not own a KUC viewer bridge module; move viewer projection/host contract to KUC.",
        )]
    }

    fn module_path(&self) -> PathBuf {
        self.root
            .join("tools")
            .join("kdv-storybook")
            .join("src")
            .join("kuc_bridge")
            .join("mod.rs")
    }
}

struct StorybookAdapterChecker<'a> {
    file: &'a SourceFile,
}

impl<'a> StorybookAdapterChecker<'a> {
    fn new(file: &'a SourceFile) -> Self {
        Self { file }
    }

    fn violations(&self) -> Vec<Violation> {
        if self.is_test_only_file() {
            return Vec::new();
        }
        let mut violations = Vec::new();
        for pattern in StorybookAdapterPattern::all() {
            violations.extend(self.find_pattern(*pattern));
        }
        violations
    }

    fn is_test_only_file(&self) -> bool {
        let path = self.file.path().to_string_lossy();
        path.contains("/tests/") || path.ends_with("_tests.rs") || path.ends_with("test_support.rs")
    }

    fn find_pattern(&self, pattern: StorybookAdapterPattern) -> Vec<Violation> {
        self.file
            .source()
            .lines()
            .enumerate()
            .filter_map(|(index, line)| self.violation_for_line(pattern, index, line))
            .collect()
    }

    fn violation_for_line(
        &self,
        pattern: StorybookAdapterPattern,
        index: usize,
        line: &str,
    ) -> Option<Violation> {
        line.find(pattern.needle()).map(|column| {
            Violation::new(
                PathBuf::from(self.file.path()),
                index + 1,
                column + 1,
                "no_kdv_ui_adapter_ownership",
                pattern.message(),
            )
        })
    }
}

#[cfg(test)]
#[path = "kdv_ui_adapter_ownership_tests.rs"]
mod tests;
