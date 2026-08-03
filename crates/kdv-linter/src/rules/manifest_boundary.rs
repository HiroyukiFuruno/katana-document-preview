use crate::diagnostics::{KdvLintError, Violation};
use std::path::Path;

use super::architecture::VIEWER_CRATE;

pub(super) struct ManifestBoundaryRule;

impl ManifestBoundaryRule {
    pub(super) fn check(root: &Path) -> Result<Vec<Violation>, KdvLintError> {
        let mut violations = Vec::new();
        let viewer_manifest = root.join(VIEWER_CRATE).join("Cargo.toml");
        Self::check_neutral_manifest(&viewer_manifest, &mut violations)?;
        Ok(violations)
    }

    fn check_neutral_manifest(
        path: &Path,
        violations: &mut Vec<Violation>,
    ) -> Result<(), KdvLintError> {
        let manifest = ManifestReader::read(path)?;
        for dependency in ManifestReader::dependency_names(&manifest) {
            if !Self::is_neutral_boundary_violation(&dependency)
                || Self::is_feature_scoped_host_dependency(&manifest, &dependency)
            {
                continue;
            }
            violations.push(Self::manifest_violation(path, dependency));
        }
        Ok(())
    }

    fn is_neutral_boundary_violation(dependency: &str) -> bool {
        UiDependencyPolicy::is_ui_dependency(dependency)
            || dependency == "katana-document-preview-egui"
    }

    fn is_feature_scoped_host_dependency(manifest: &toml::Value, dependency: &str) -> bool {
        if dependency != "egui" {
            return false;
        }
        let optional = manifest
            .get("dependencies")
            .and_then(toml::Value::as_table)
            .and_then(|dependencies| dependencies.get(dependency))
            .and_then(toml::Value::as_table)
            .and_then(|dependency| dependency.get("optional"))
            .and_then(toml::Value::as_bool)
            == Some(true);
        let feature_dependency = manifest
            .get("features")
            .and_then(toml::Value::as_table)
            .and_then(|features| features.get("egui"))
            .and_then(toml::Value::as_array)
            .is_some_and(|members| {
                members
                    .iter()
                    .any(|member| member.as_str() == Some("dep:egui"))
            });
        optional && feature_dependency
    }

    fn manifest_violation(path: &Path, dependency: String) -> Violation {
        Violation::new(
            path.to_path_buf(),
            1,
            1,
            "preview-boundary",
            format!("neutral document crate must not depend on UI crate `{dependency}`."),
        )
    }
}

struct ManifestReader;

impl ManifestReader {
    fn read(path: &Path) -> Result<toml::Value, KdvLintError> {
        let source = std::fs::read_to_string(path).map_err(|source| KdvLintError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        toml::from_str(&source).map_err(|source| KdvLintError::TomlParse {
            path: path.to_path_buf(),
            source,
        })
    }

    fn dependency_names(manifest: &toml::Value) -> Vec<String> {
        let mut names = Vec::new();
        for table in ["dependencies", "dev-dependencies", "build-dependencies"] {
            Self::push_dependency_table(manifest, table, &mut names);
        }
        names
    }

    fn push_dependency_table(manifest: &toml::Value, table: &str, names: &mut Vec<String>) {
        let Some(dependencies) = manifest.get(table).and_then(toml::Value::as_table) else {
            return;
        };
        for (key, value) in dependencies {
            names.push(key.to_string());
            if let Some(package) = value.get("package").and_then(toml::Value::as_str) {
                names.push(package.to_string());
            }
        }
    }
}

struct UiDependencyPolicy;

impl UiDependencyPolicy {
    fn is_ui_dependency(name: &str) -> bool {
        let lower = name.to_ascii_lowercase();
        lower.ends_with("-ui")
            || lower.ends_with("_ui")
            || matches!(
                lower.as_str(),
                "dioxus"
                    | "eframe"
                    | "egui"
                    | "iced"
                    | "leptos"
                    | "tauri"
                    | "yew"
                    | "floem"
                    | "vello"
                    | "winit"
            )
    }
}

#[cfg(test)]
#[path = "manifest_boundary_tests.rs"]
mod tests;
