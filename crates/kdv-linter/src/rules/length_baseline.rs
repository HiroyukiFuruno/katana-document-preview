use crate::diagnostics::Violation;
use crate::workspace::PortablePath;
use std::path::Path;

const FILE_LENGTH_BASELINE: &[&str] = &[
    "crates/katana-document-viewer/src/export_surface/export_surface_blocks_table.rs",
    "crates/katana-document-viewer/src/preview_runtime/types_tests.rs",
    "crates/katana-document-viewer/src/theme.rs",
    "crates/katana-document-viewer/src/forge_diagram_render.rs",
    "crates/katana-document-viewer/src/preview_runtime/katana_reference_artifact_tests.rs",
    "crates/katana-document-viewer/src/preview_runtime/storybook_score_gate_tests.rs",
    "crates/katana-document-viewer/src/viewer/commands_factory_tests.rs",
    "crates/katana-document-viewer/src/viewer/image_surface_factory.rs",
    "crates/katana-document-viewer/src/viewer/image_surface_tests.rs",
    "crates/katana-document-viewer/src/viewer/media_control_spec.rs",
    "crates/katana-document-viewer/src/viewer/node_plan/builder_media_asset_height.rs",
    "crates/katana-document-viewer/src/viewer/node_plan/builder_media_height.rs",
    "crates/katana-document-viewer/src/viewer/node_plan/builder_node_push.rs",
    "crates/katana-document-viewer/src/viewer/node_plan/builder_spacing_tests.rs",
    "crates/katana-document-viewer/src/viewer/node_plan/builder_tests.rs",
    "crates/katana-document-viewer/src/viewer/node_plan/metrics.rs",
    "crates/katana-document-viewer/src/viewer/node_plan/metrics_table.rs",
    "crates/katana-document-viewer/src/viewer/state.rs",
];

const FUNCTION_LENGTH_BASELINE: &[(&str, &str)] = &[
    (
        "crates/katana-document-viewer/src/forge_diagram_render_runtime_tests.rs",
        "krr_diagram_render_cache_options_include_runtime_asset_versions",
    ),
    (
        "crates/katana-document-viewer/src/preview_runtime/katana_reference_artifact_tests.rs",
        "katana_reference_artifacts_runtime_parity_audit_covers_reported_visual_and_performance_risks",
    ),
    (
        "crates/katana-document-viewer/src/preview_runtime/katana_reference_artifact_tests.rs",
        "katana_reference_artifact_parity_audit_declares_score_categories",
    ),
    (
        "crates/katana-document-viewer/src/preview_runtime/katana_reference_artifact_tests.rs",
        "openspec_requirements_are_connected_to_storybook_gates",
    ),
    (
        "crates/katana-document-viewer/src/preview_runtime/katana_reference_artifact_tests.rs",
        "openspec_requirements_are_connected_to_score_check",
    ),
    (
        "crates/katana-document-viewer/src/preview_runtime/katana_reference_artifact_tests.rs",
        "openspec_runtime_parity_audit_covers_reported_visual_and_performance_risks",
    ),
    (
        "crates/katana-document-viewer/src/preview_runtime/katana_reference_artifact_tests.rs",
        "assert_valid",
    ),
    (
        "crates/katana-document-viewer/src/preview_runtime/katana_reference_artifact_tests.rs",
        "gate_matches_category",
    ),
    (
        "crates/katana-document-viewer/src/preview_runtime/storybook_score_gate_tests.rs",
        "storybook_check_recipe_keeps_all_score_category_gates",
    ),
    (
        "crates/katana-document-viewer/src/preview_runtime/storybook_score_gate_tests.rs",
        "storybook_media_control_gate_keeps_diagram_min_height_contract",
    ),
    (
        "crates/katana-document-viewer/src/preview_runtime/storybook_score_gate_tests.rs",
        "storybook_score_gate_keeps_diagram_scale_and_scroll_flake_contract_sources",
    ),
    (
        "crates/katana-document-viewer/src/viewer/image_surface_tests.rs",
        "diagram_svg_artifact_with_background_matches_katana_texture_composite",
    ),
    (
        "crates/katana-document-viewer/src/viewer/commands_factory_tests.rs",
        "only_fullscreen_diagram_command_requires_host_propagation",
    ),
    (
        "crates/katana-document-viewer/src/viewer/media_control_spec.rs",
        "surface_control_svg",
    ),
    (
        "crates/katana-document-viewer/src/viewer/node_plan/builder_media_asset_height.rs",
        "diagram_height_uses_katana_minimum_container_height",
    ),
    (
        "crates/katana-document-viewer/src/viewer/node_plan/builder_media_asset_height.rs",
        "interactive_diagram_height_uses_preview_content_width",
    ),
    (
        "crates/katana-document-viewer/src/viewer/node_plan/builder_media_asset_height.rs",
        "interactive_diagram_height_uses_viewer_width_for_wide_diagram",
    ),
    (
        "crates/katana-document-viewer/src/viewer/node_plan/builder_media_asset_height.rs",
        "interactive_diagram_height_uses_katana_reference_width_cap_for_large_viewports",
    ),
    (
        "crates/katana-document-viewer/src/viewer/node_plan/builder_media_asset_height.rs",
        "export_surface_diagram_height_keeps_export_width",
    ),
    (
        "crates/katana-document-viewer/src/viewer/node_plan/builder_media_asset_height.rs",
        "export_surface_diagram_height_does_not_apply_interactive_minimum_container",
    ),
    (
        "crates/katana-document-viewer/src/viewer/node_plan/builder_media_height.rs",
        "kind_height",
    ),
    (
        "crates/katana-document-viewer/src/viewer/node_plan/builder_media_height.rs",
        "interactive_diagram_height_uses_viewer_row_width",
    ),
    (
        "crates/katana-document-viewer/src/viewer/node_plan/builder_media_height.rs",
        "interactive_diagram_height_uses_viewer_row_width_without_upscaling",
    ),
    (
        "crates/katana-document-viewer/src/viewer/node_plan/builder_spacing_tests.rs",
        "planner_uses_katana_context_gaps_around_html_rule_and_heading",
    ),
    (
        "crates/katana-document-viewer/src/viewer/node_plan/builder_spacing_tests.rs",
        "planner_preserves_katana_long_h2_row_height_without_affecting_short_h2",
    ),
    (
        "crates/katana-document-viewer/src/viewer/node_plan/metrics_table.rs",
        "allocate_katana_table_column_widths",
    ),
];

pub(super) struct LengthBaseline;

impl LengthBaseline {
    pub(super) fn contains(root: &Path, violation: &Violation) -> bool {
        let Some(relative) = PortablePath::new(&violation.file).relative_to(root) else {
            return false;
        };
        match violation.rule {
            "file-length" => FILE_LENGTH_BASELINE.contains(&relative.as_str()),
            "function-length" => function_is_baselined(&relative, &violation.message),
            _ => false,
        }
    }
}

fn function_is_baselined(relative: &str, message: &str) -> bool {
    FUNCTION_LENGTH_BASELINE.iter().any(|(path, function)| {
        relative == *path && message.contains(&format!("function `{function}`"))
    })
}

#[cfg(test)]
#[path = "length_baseline_tests.rs"]
mod length_baseline_tests;
