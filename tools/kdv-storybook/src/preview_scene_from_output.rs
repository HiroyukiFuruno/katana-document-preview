use super::PreviewBuilder;
use crate::KucViewerPlan;
use crate::preview_build_request::PreviewBuildRequest;
use crate::preview_build_support::{KucConfigState, PreviewBuildSupport};
use crate::preview_scene::{
    PreviewScene, scroll_redraw_diagram_boundary_rects, scroll_redraw_sensitive_rects,
    viewer_internal_anchor_lookup, viewer_target_lookup, viewer_targets,
};
use crate::preview_search_targets::StorybookSearchTargets;
use crate::preview_theme_bridge::KucThemeBridge;
use katana_document_viewer::{
    Artifact, ArtifactId, MarkdownSource, PreviewAssetLoadReport, PreviewOutput, ViewerNodeKind,
    ViewerNodePlan,
};
use std::collections::BTreeSet;

impl PreviewBuilder {
    pub(crate) fn scene_from_output(
        &self,
        source: &MarkdownSource,
        request: &PreviewBuildRequest<'_>,
        mut output: PreviewOutput,
        _asset_report: PreviewAssetLoadReport,
    ) -> Result<PreviewScene, Box<dyn std::error::Error>> {
        let theme_snapshot = request
            .theme
            .clone()
            .unwrap_or_else(|| PreviewBuildSupport::kdv_theme(request.dark));
        let config = PreviewBuildSupport::preview_config_for_theme(
            request.viewport,
            request.scene_scroll_y(),
            theme_snapshot.clone(),
            request.interaction.clone(),
            request.mode.clone(),
            request.typography,
            request.search.clone(),
        );
        if request.attach_surface {
            self.engine.attach_surface(&mut output, &config);
        }
        let kuc_theme = if request.export_surface {
            KucThemeBridge::from_kdv_export_surface(&theme_snapshot)?
        } else {
            KucThemeBridge::from_kdv(&theme_snapshot)?
        };
        let theme = PreviewBuildSupport::with_document_typography(kuc_theme, request.typography);
        let KucViewerPlan {
            paint_request,
            node_plan,
            content_height,
        } = self.host.project(
            &output,
            &PreviewBuildSupport::kuc_config(
                &config,
                theme.clone(),
                request.typography,
                KucConfigState {
                    diagram_viewports: request.diagram_viewports.clone(),
                    image_viewports: request.image_viewports.clone(),
                    task_state_overrides: request.task_state_overrides.clone(),
                    accordion_open_overrides: request.accordion_open_overrides.clone(),
                    copied_code_node_ids: request.copied_code_node_ids.clone(),
                },
            )?
            .export_surface(request.export_surface),
        )?;
        let tree = paint_request.into_tree();
        let asset_request_count = Self::pending_asset_count(&output, &node_plan);
        let asset_request_key = Self::pending_asset_key(&output, &node_plan);
        let asset_report = Self::scene_asset_report(&output, &node_plan);
        let image_surface_count = PreviewBuildSupport::count_image_surfaces(tree.root());
        let targets = viewer_targets(
            &node_plan,
            &tree,
            &theme,
            config.viewport.width,
            content_height,
        );
        let content_height = Self::content_height_for_targets(
            content_height,
            &targets,
            &config,
            request.export_surface,
        );
        let target_lookup = viewer_target_lookup(&targets);
        let internal_anchor_lookup = viewer_internal_anchor_lookup(&node_plan, &targets);
        let scroll_redraw_sensitive_rects = scroll_redraw_sensitive_rects(&node_plan);
        let scroll_redraw_diagram_boundary_rects =
            scroll_redraw_diagram_boundary_rects(&node_plan, &targets);
        let search_targets = StorybookSearchTargets::collect(
            &node_plan,
            &output.input.artifacts,
            &output.input.search.query,
        );
        let diagram_node_ids = diagram_node_ids(&node_plan);
        Ok(PreviewScene {
            document_id: Self::document_id(source, request),
            image_surface_count,
            tree,
            theme,
            host_action_cache: Default::default(),
            node_count: node_plan.nodes.len(),
            mode: output.state.mode,
            typography: request.typography,
            asset_request_count,
            asset_request_key,
            loaded_asset_count: asset_report.loaded_artifact_count,
            failed_asset_count: asset_report.failed_artifact_count,
            surface: output.surface.clone(),
            content_height,
            scroll_redraw_sensitive_rects,
            scroll_redraw_diagram_boundary_rects,
            slideshow_current_page: output.state.slideshow.current_page_index,
            slideshow_max_page: output.state.slideshow.max_page_index,
            diagram_viewports: request.diagram_viewports.clone(),
            diagram_node_ids,
            search_targets,
            targets,
            target_lookup,
            internal_anchor_lookup,
            warnings: output.diagnostics.warnings,
        })
    }

    fn content_height_for_targets(
        content_height: f32,
        targets: &[katana_document_viewer::ViewerTarget],
        config: &katana_document_viewer::PreviewConfig,
        export_surface: bool,
    ) -> f32 {
        if export_surface {
            return content_height;
        }
        let Some(last_target_y) = targets
            .iter()
            .map(|target| target.rect.y)
            .max_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal))
        else {
            return content_height;
        };
        content_height.max(last_target_y + config.viewport.height)
    }

    fn document_id(source: &MarkdownSource, request: &PreviewBuildRequest<'_>) -> String {
        source
            .document_id
            .clone()
            .unwrap_or_else(|| request.fixture.label.clone())
    }

    fn scene_asset_report(output: &PreviewOutput, plan: &ViewerNodePlan) -> PreviewAssetLoadReport {
        let mut report = PreviewAssetLoadReport::default();
        for request in plan
            .asset_requests
            .iter()
            .filter(|request| Self::loads_in_storybook_scope(request))
        {
            let Some(artifact) = Self::artifact_for_request(output, &request.artifact_id) else {
                continue;
            };
            if Self::artifact_failed(artifact) {
                report.failed_artifact_count += 1;
            } else {
                report.loaded_artifact_count += 1;
            }
        }
        report
    }

    fn artifact_for_request<'a>(
        output: &'a PreviewOutput,
        artifact_id: &ArtifactId,
    ) -> Option<&'a Artifact> {
        output
            .input
            .artifacts
            .iter()
            .find(|artifact| artifact.manifest.id == *artifact_id)
    }

    fn artifact_failed(artifact: &Artifact) -> bool {
        !artifact.manifest.diagnostics.entries.is_empty()
    }
}

fn diagram_node_ids(node_plan: &ViewerNodePlan) -> BTreeSet<String> {
    node_plan
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, ViewerNodeKind::Diagram { .. }))
        .map(|node| node.node_id.0.clone())
        .collect()
}
