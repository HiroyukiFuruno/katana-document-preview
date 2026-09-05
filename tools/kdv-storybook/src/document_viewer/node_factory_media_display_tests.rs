use super::KucNodeFactory;
use super::media_fixture::*;
use crate::test_assert::KucTestExpect;
use katana_document_viewer::{
    ArtifactBytes, ArtifactDiagnostics, ArtifactFactory, ArtifactFormat, ArtifactId, DocumentId,
    SourceRevision,
};
use katana_ui_core::render_model::{UiNode, UiNodeKind, UiVisualRole};

#[test]
fn diagram_artifact_surface_keeps_katana_display_size_with_retina_raster_density() {
    let artifact_id = ArtifactId("diagram-svg".to_string());
    let artifact = ArtifactFactory::image_asset_with_id(
        artifact_id.clone(),
        ArtifactFormat::Svg,
        DocumentId("document".to_string()),
        SourceRevision("rev".to_string()),
        ArtifactBytes {
            bytes: br##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20">
<rect x="0" y="0" width="40" height="20" fill="#222222"/>
<path d="M2 18 L38 2" stroke="#ffffff" stroke-width="1"/>
</svg>"##
                .to_vec(),
        },
        "test",
        ArtifactDiagnostics {
            entries: Vec::new(),
        },
    );
    let mut node = diagram_node();
    node.artifact_id = Some(artifact_id);
    let artifacts = [artifact];
    let factory = KucNodeFactory::new(&artifacts, STORYBOOK_MEDIA_MAX_WIDTH);

    let ui_node = factory.media_node(&node);
    let image_node = find_image_surface(&ui_node).kuc_expect("diagram image surface missing");
    let displayed_width = image_node.props().image_surface.display_width_milli as f32 / 1000.0;
    let effective_scale = image_node.props().image_surface.width as f32 / displayed_width;

    assert_eq!(40.0, displayed_width);
    assert_eq!(
        20.0,
        image_node.props().image_surface.display_height_milli as f32 / 1000.0
    );
    assert!(
        effective_scale >= 2.0,
        "diagram artifact must stay retina while keeping KatanA display size: physical={} displayed={displayed_width}",
        image_node.props().image_surface.width
    );
}

#[test]
fn diagram_artifact_surface_aligns_to_retina_preview_target_pixels() {
    let artifact_id = ArtifactId("diagram-odd-width-svg".to_string());
    let artifact = ArtifactFactory::image_asset_with_id(
        artifact_id.clone(),
        ArtifactFormat::Svg,
        DocumentId("document".to_string()),
        SourceRevision("rev".to_string()),
        ArtifactBytes {
            bytes: br##"<svg xmlns="http://www.w3.org/2000/svg" width="41" height="20">
<path d="M1 19 L40 1" stroke="#ffffff" stroke-width="1"/>
</svg>"##
                .to_vec(),
        },
        "test",
        ArtifactDiagnostics {
            entries: Vec::new(),
        },
    );
    let mut node = diagram_node();
    node.artifact_id = Some(artifact_id);
    let artifacts = [artifact];
    let factory = KucNodeFactory::new(&artifacts, STORYBOOK_MEDIA_MAX_WIDTH);

    let ui_node = factory.media_node(&node);
    let image_node = find_image_surface(&ui_node).kuc_expect("diagram image surface missing");
    let displayed_width = image_node.props().image_surface.display_width_milli as f32 / 1000.0;
    let retina_target_width = (displayed_width.round() as u32).saturating_mul(2);

    assert!(
        image_node.props().image_surface.width >= retina_target_width,
        "diagram SVG should not be resampled again by the 2x Storybook canvas"
    );
}

#[test]
fn diagram_artifact_surface_composites_over_viewer_background_for_texture_parity() {
    let artifact_id = ArtifactId("diagram-transparent-svg".to_string());
    let artifact = ArtifactFactory::image_asset_with_id(
        artifact_id.clone(),
        ArtifactFormat::Svg,
        DocumentId("document".to_string()),
        SourceRevision("rev".to_string()),
        ArtifactBytes {
            bytes: br##"<svg xmlns="http://www.w3.org/2000/svg" width="2" height="1">
<rect x="1" width="1" height="1" fill="#d4d4d4"/>
</svg>"##
                .to_vec(),
        },
        "test",
        ArtifactDiagnostics {
            entries: Vec::new(),
        },
    );
    let mut node = diagram_node();
    node.artifact_id = Some(artifact_id);
    let artifacts = [artifact];
    let background = [30, 30, 30, 255];
    let factory = KucNodeFactory::new(&artifacts, STORYBOOK_MEDIA_MAX_WIDTH)
        .viewer_background(Some(background));

    let ui_node = factory.media_node(&node);
    let image_node = find_image_surface(&ui_node).kuc_expect("diagram image surface missing");

    assert_eq!(
        background.as_slice(),
        &image_node.props().image_surface.rgba[0..4]
    );
    assert!(
        image_node
            .props()
            .image_surface
            .fingerprint
            .contains(":background=1e1e1eff:renderer=")
    );
}

#[test]
fn export_surface_diagram_artifact_uses_reference_raster_scale() {
    let artifact_id = ArtifactId("diagram-svg".to_string());
    let artifact = ArtifactFactory::image_asset_with_id(
        artifact_id.clone(),
        ArtifactFormat::Svg,
        DocumentId("document".to_string()),
        SourceRevision("rev".to_string()),
        ArtifactBytes {
            bytes: br##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20">
<rect x="0" y="0" width="40" height="20" fill="#222222"/>
<path d="M2 18 L38 2" stroke="#ffffff" stroke-width="1"/>
</svg>"##
                .to_vec(),
        },
        "test",
        ArtifactDiagnostics {
            entries: Vec::new(),
        },
    );
    let mut node = diagram_node();
    node.artifact_id = Some(artifact_id);
    let artifacts = [artifact];
    let factory = KucNodeFactory::new(&artifacts, STORYBOOK_MEDIA_MAX_WIDTH).export_surface(true);

    let ui_node = factory.media_node(&node);
    let image_node = find_image_surface(&ui_node).kuc_expect("diagram image surface missing");

    assert_eq!(100, image_node.props().image_surface.content_scale);
}

#[test]
fn export_surface_diagram_uses_capped_logical_surface_size() {
    let node = diagram_node();
    let factory = KucNodeFactory::new(&[], STORYBOOK_MEDIA_MAX_WIDTH).export_surface(true);
    let mut surface = image_surface();
    surface.width = 1720;
    surface.height = 348;
    surface.display_width = 1040.0;
    surface.display_height = 220.0;
    surface.content_scale = 200;
    surface.rgba = [0, 0, 0, OPAQUE_ALPHA].repeat((1720 * 348) as usize);
    let ui_node = factory.image_surface_node(&node, &ArtifactId("artifact".to_string()), surface);

    assert_eq!(UiNodeKind::ImageSurface, ui_node.kind());
    assert_eq!(UiVisualRole::ExportMediaFrame, ui_node.props().visual_role);
    assert_eq!(860_000, ui_node.props().image_surface.display_width_milli);
    assert_eq!(181_923, ui_node.props().image_surface.display_height_milli);
}

#[test]
fn interactive_diagram_keeps_intrinsic_display_size_for_renderer_downscale() {
    let node = diagram_node();
    let factory = KucNodeFactory::new(&[], STORYBOOK_MEDIA_MAX_WIDTH);
    let mut surface = image_surface();
    surface.width = STORYBOOK_MEDIA_MAX_WIDTH.saturating_mul(4);
    surface.height = 160;
    surface.display_width = (STORYBOOK_MEDIA_MAX_WIDTH * 2) as f32;
    surface.display_height = 80.0;
    surface.content_scale = 200;
    surface.rgba = [0, 0, 0, OPAQUE_ALPHA].repeat((surface.width * surface.height) as usize);

    let ui_node = factory.image_surface_node(&node, &ArtifactId("artifact".to_string()), surface);

    assert_eq!(
        STORYBOOK_MEDIA_MAX_WIDTH * 2 * 1000,
        ui_node.props().image_surface.display_width_milli
    );
    assert_eq!(80_000, ui_node.props().image_surface.display_height_milli);
}

#[test]
fn image_surface_keeps_original_display_size() {
    let node = viewer_node(
        katana_document_viewer::ViewerNodeKind::Image,
        "image",
        "image",
    );
    let factory = KucNodeFactory::new(&[], STORYBOOK_MEDIA_MAX_WIDTH);
    let ui_node =
        factory.image_surface_node(&node, &ArtifactId("artifact".to_string()), image_surface());

    assert_eq!(2_000, ui_node.props().image_surface.display_width_milli);
    assert_eq!(1_000, ui_node.props().image_surface.display_height_milli);
}

fn find_image_surface(node: &UiNode) -> Option<&UiNode> {
    if node.kind() == UiNodeKind::ImageSurface {
        return Some(node);
    }
    node.children().iter().find_map(find_image_surface)
}
