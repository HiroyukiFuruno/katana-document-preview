use crate::diagnostics::{KdvLintError, Violation};
use crate::workspace::WorkspaceModel;

use super::vendor_boundary_manifest::VendorBoundaryManifestRule;
use super::vendor_boundary_source::VendorBoundarySourceRule;

pub struct VendorBoundaryRule;

impl VendorBoundaryRule {
    pub fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KdvLintError> {
        let mut violations = Vec::new();
        violations.extend(VendorBoundaryManifestRule::check(workspace.root())?);
        violations.extend(VendorBoundarySourceRule::check(workspace.root())?);
        Ok(violations)
    }
}

#[derive(Clone, Copy)]
pub(super) struct VendorScope {
    pub(super) path: &'static str,
}

impl VendorScope {
    pub(super) const fn core(path: &'static str) -> Self {
        Self { path }
    }
}

pub(super) fn is_vendor_ref(name: &str) -> bool {
    let normalized = name.replace('-', "_");
    matches!(
        normalized.as_str(),
        "eframe"
            | "egui"
            | "floem"
            | "floem_reactive"
            | "floem_renderer"
            | "gpui"
            | "katana_document_preview_egui"
            | "katana_ui_core_egui"
            | "katana_ui_core_floem"
            | "katana_ui_core_gpui"
            | "vello"
            | "winit"
    )
}

pub(super) const fn is_allowed_ref(_scope: VendorScope, _name: &str) -> bool {
    false
}

pub(super) fn is_allowed_source_ref(
    _scope: VendorScope,
    _path: &std::path::Path,
    _name: &str,
) -> bool {
    false
}

#[cfg(test)]
#[path = "vendor_boundary_tests.rs"]
mod tests;
