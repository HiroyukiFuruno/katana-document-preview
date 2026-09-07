mod asset_loader;
mod asset_loader_cache;
mod asset_loader_diagram;
#[cfg(test)]
mod asset_loader_disk_cache_tests;
#[cfg(test)]
mod asset_loader_engine_cache_test_engines;
#[cfg(test)]
mod asset_loader_engine_cache_tests;
mod asset_loader_media;
mod asset_loader_media_types;
mod asset_loader_parallel;
mod asset_loader_support;
mod direct_html_css;
mod direct_html_css_attrs;
mod direct_html_normalizer;
mod direct_html_preview_renderer;
mod direct_html_table_normalizer;
mod direct_html_visibility;
mod engine;
mod output_factory;
mod source_normalizer;
#[cfg(test)]
mod storybook_score_gate;
pub mod types;

pub use asset_loader::{PreviewAssetLoadReport, PreviewAssetLoader};
pub use direct_html_preview_renderer::DirectHtmlPreviewRenderer;
pub use engine::PreviewRenderEngine;
pub use output_factory::PreviewOutputFactory;
pub use types::{
    MarkdownSource, PreviewConfig, PreviewDiagnostics, PreviewError, PreviewOutput,
    PreviewSurfaceImage, PreviewTheme, RenderTarget,
};

pub trait MarkdownPreview {
    fn render(
        &self,
        source: &MarkdownSource,
        config: &PreviewConfig,
    ) -> Result<PreviewOutput, PreviewError>;
}

#[cfg(test)]
#[path = "asset_loader_cache_tests.rs"]
mod asset_loader_cache_tests;
#[cfg(test)]
#[path = "asset_loader_parallel_tests.rs"]
mod asset_loader_parallel_tests;
#[cfg(test)]
#[path = "asset_loader_support_tests.rs"]
mod asset_loader_support_tests;
#[cfg(test)]
#[path = "asset_loader_tests.rs"]
mod asset_loader_tests;
#[cfg(test)]
#[path = "direct_diagram_score_matrix_tests.rs"]
mod direct_diagram_score_matrix_tests;
#[cfg(test)]
#[path = "direct_html_preview_renderer_tests.rs"]
mod direct_html_preview_renderer_tests;
#[cfg(test)]
#[path = "direct_html_source_tests.rs"]
mod direct_html_source_tests;
#[cfg(test)]
#[path = "direct_html_static_css_source_tests.rs"]
mod direct_html_static_css_source_tests;
#[cfg(test)]
#[path = "direct_image_source_tests.rs"]
mod direct_image_source_tests;
#[cfg(test)]
#[path = "direct_markdown_source_tests.rs"]
mod direct_markdown_source_tests;
#[cfg(test)]
#[path = "direct_mermaid_source_tests.rs"]
mod direct_mermaid_source_tests;
#[cfg(test)]
#[path = "direct_source_tests.rs"]
mod direct_source_tests;
#[cfg(test)]
#[path = "fixture_feature_matrix_tests.rs"]
mod fixture_feature_matrix_tests;
#[cfg(test)]
#[path = "fixture_mermaid_matrix_tests.rs"]
mod fixture_mermaid_matrix_tests;
#[cfg(test)]
#[path = "fixture_score_matrix_assertions.rs"]
mod fixture_score_matrix_assertions;
#[cfg(test)]
#[path = "fixture_score_matrix_basic_tests.rs"]
mod fixture_score_matrix_basic_tests;
#[cfg(test)]
#[path = "fixture_score_matrix_full_tests.rs"]
mod fixture_score_matrix_full_tests;
#[cfg(test)]
#[path = "fixture_score_matrix_render_support.rs"]
mod fixture_score_matrix_render_support;
#[cfg(test)]
#[path = "fixture_score_matrix_requirements_tests.rs"]
mod fixture_score_matrix_requirements_tests;
#[cfg(test)]
#[path = "fixture_score_matrix_support.rs"]
mod fixture_score_matrix_support;
#[cfg(test)]
#[path = "fixture_score_matrix_tests.rs"]
mod fixture_score_matrix_tests;
#[cfg(test)]
#[path = "katana_fixture_inventory_tests.rs"]
mod katana_fixture_inventory_tests;
#[cfg(test)]
#[path = "katana_preview_image_parity_tests.rs"]
mod katana_preview_image_parity_tests;
#[cfg(test)]
#[path = "katana_preview_parity_tests.rs"]
mod katana_preview_parity_tests;
#[cfg(test)]
#[path = "katana_reference_artifact_tests.rs"]
mod katana_reference_artifact_tests;
#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;
#[cfg(test)]
#[path = "types_theme_tests.rs"]
mod types_theme_tests;
#[cfg(test)]
#[path = "viewer_node_plan_contract_tests.rs"]
mod viewer_node_plan_contract_tests;
