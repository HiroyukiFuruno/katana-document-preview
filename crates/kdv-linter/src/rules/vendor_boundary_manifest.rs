use crate::diagnostics::{KdvLintError, Violation};
use std::path::Path;

use super::vendor_boundary::{VendorScope, is_allowed_ref, is_vendor_ref};

pub(super) struct VendorBoundaryManifestRule;

impl VendorBoundaryManifestRule {
    pub(super) fn check(root: &Path) -> Result<Vec<Violation>, KdvLintError> {
        let mut violations = Vec::new();
        for scope in &MANIFEST_SCOPES {
            let path = root.join(scope.path).join("Cargo.toml");
            if !path.exists() {
                continue;
            }
            let manifest = ManifestReader::read(&path)?;
            for dependency in ManifestReader::dependency_names(&manifest) {
                if !is_manifest_violation(*scope, &dependency) {
                    continue;
                }
                violations.push(Violation::new(
                    path.clone(),
                    1,
                    1,
                    "vendor-boundary-manifest",
                    manifest_message(*scope, &dependency),
                ));
            }
        }
        Ok(violations)
    }
}

const MANIFEST_SCOPES: [VendorScope; 2] = [
    VendorScope::core("crates/katana-document-viewer"),
    VendorScope::core("tools/kdv-storybook"),
];

fn is_manifest_violation(scope: VendorScope, dependency: &str) -> bool {
    is_vendor_ref(dependency) && !is_allowed_ref(scope, dependency)
}

fn manifest_message(scope: VendorScope, dependency: &str) -> String {
    let _ = scope;
    format!("core crate must not depend on vendor crate `{dependency}`.")
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
