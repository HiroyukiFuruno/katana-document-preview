use super::KucNodeFactory;
use super::media_fixture::*;
use super::media_frame_tests_support::find_image_surface;
use crate::test_assert::KucTestExpect;
use katana_document_viewer::{ArtifactId, ViewerImageSurface, ViewerInteractionConfig, ViewerRect};
use katana_ui_core::render_model::{UiDimension, UiVisualRole};

#[test]
fn viewer_row_height_does_not_expand_diagram_image_container() {
    let factory =
        KucNodeFactory::new(&[], DIAGRAM_MEDIA_MAX_WIDTH).interaction(ViewerInteractionConfig {
            hover_highlight_enabled: false,
            selection_enabled: false,
            image_controls_enabled: false,
            diagram_controls_enabled: false,
            code_controls_enabled: false,
        });
    let mut node = diagram_node();
    node.rect = ViewerRect {
        x: 0.0,
        y: 0.0,
        width: 860.0,
        height: 487.0,
    };
    let surface = ViewerImageSurface {
        fingerprint: "katana-flowchart".to_string(),
        width: 640,
        height: 890,
        display_width: 320.0,
        display_height: 445.0,
        content_scale: 200,
        rgba: [0, 0, 0, OPAQUE_ALPHA].repeat(640 * 890),
    };

    let media = factory.image_surface_node(&node, &ArtifactId("artifact".to_string()), surface);
    let media = factory.media_with_controls(&node, media);
    let row = factory.node_with_viewer_height(media, &node);
    let image = find_image_surface(&row).kuc_expect("diagram image surface");

    assert_eq!(
        UiDimension::Px(487),
        row.props().common.height,
        "viewer row keeps KDV layout height for scroll/anchor spacing"
    );
    assert_eq!(
        UiDimension::Px(445),
        image.props().common.height,
        "image container must stay at the KatanA media height and not include trailing viewer row gap"
    );
}

#[test]
fn viewer_row_height_expands_to_unclipped_diagram_media_frame() {
    let factory =
        KucNodeFactory::new(&[], DIAGRAM_MEDIA_MAX_WIDTH).interaction(ViewerInteractionConfig {
            hover_highlight_enabled: false,
            selection_enabled: false,
            image_controls_enabled: false,
            diagram_controls_enabled: false,
            code_controls_enabled: false,
        });
    let mut node = diagram_node();
    node.rect = ViewerRect {
        x: 0.0,
        y: 0.0,
        width: 860.0,
        height: 412.0,
    };
    let surface = ViewerImageSurface {
        fingerprint: "katana-flowchart-native".to_string(),
        width: 640,
        height: 890,
        display_width: 320.0,
        display_height: 445.0,
        content_scale: 200,
        rgba: [0, 0, 0, OPAQUE_ALPHA].repeat(640 * 890),
    };

    let media = factory.image_surface_node(&node, &ArtifactId("artifact".to_string()), surface);
    let media = factory.media_with_controls(&node, media);
    let row = factory.node_with_viewer_height(media, &node);

    assert_eq!(
        UiDimension::Px(445),
        row.props().common.height,
        "a native-size diagram media frame must not overlap the following viewer row"
    );
}

#[test]
fn diagram_media_row_wrapper_uses_full_katana_row_width_even_without_extra_height() {
    let factory = KucNodeFactory::new(&[], KATANA_VIEWER_ROW_MAX_WIDTH).interaction(
        ViewerInteractionConfig {
            hover_highlight_enabled: false,
            selection_enabled: false,
            image_controls_enabled: false,
            diagram_controls_enabled: false,
            code_controls_enabled: false,
        },
    );
    let mut node = diagram_node();
    node.rect = ViewerRect {
        x: 0.0,
        y: 0.0,
        width: KATANA_VIEWER_ROW_MAX_WIDTH as f32,
        height: 267.0,
    };
    let surface = ViewerImageSurface {
        fingerprint: "katana-state".to_string(),
        width: 1162,
        height: 574,
        display_width: 538.0,
        display_height: 267.0,
        content_scale: 200,
        rgba: [0, 0, 0, OPAQUE_ALPHA].repeat(1162 * 574),
    };

    let media = factory.image_surface_node(&node, &ArtifactId("artifact".to_string()), surface);
    let media = factory.media_with_controls(&node, media);
    let row = factory.node_with_viewer_height(media, &node);
    let image = find_image_surface(&row).kuc_expect("diagram image surface");

    assert_eq!(
        UiDimension::Px(KATANA_VIEWER_ROW_MAX_WIDTH as u16),
        row.props().common.width,
        "KatanA diagram rows allocate the full available width even when the image height exactly matches the row height"
    );
    assert_eq!(
        UiVisualRole::MediaFrame,
        row.props().visual_role,
        "the full-width row owns the media frame background"
    );
    assert_eq!(
        UiDimension::Auto,
        image.props().common.width,
        "the full-width row must not upscale the image body"
    );
}

#[test]
fn fullscreen_diagram_row_wrapper_preserves_viewport_height() {
    let mut viewports = viewport_states();
    viewports
        .get_mut("diagram")
        .kuc_expect("diagram viewport fixture")
        .fullscreen_open = true;
    let factory = KucNodeFactory::new(&[], KATANA_VIEWER_ROW_MAX_WIDTH)
        .interaction(ViewerInteractionConfig {
            hover_highlight_enabled: false,
            selection_enabled: false,
            image_controls_enabled: false,
            diagram_controls_enabled: true,
            code_controls_enabled: false,
        })
        .diagram_viewports(&viewports)
        .fullscreen_viewport_size(640, 600);
    let mut node = diagram_node();
    node.rect = ViewerRect {
        x: 0.0,
        y: 0.0,
        width: KATANA_VIEWER_ROW_MAX_WIDTH as f32,
        height: 267.0,
    };

    let media =
        factory.image_surface_node(&node, &ArtifactId("artifact".to_string()), image_surface());
    let media = factory.media_with_controls(&node, media);
    let row = factory.node_with_viewer_height(media, &node);

    assert_eq!(
        UiDimension::Px(640),
        row.props().common.width,
        "fullscreen diagram row must use the current viewer width instead of the document max media width"
    );
    assert_eq!(
        UiDimension::Px(600),
        row.props().common.height,
        "fullscreen diagram row must keep the viewport-height media frame instead of reverting to the document node height"
    );
}

#[test]
fn export_surface_diagram_row_wrapper_preserves_export_frame_height() {
    let factory = KucNodeFactory::new(&[], DIAGRAM_MEDIA_MAX_WIDTH)
        .export_surface(true)
        .interaction(ViewerInteractionConfig {
            hover_highlight_enabled: false,
            selection_enabled: false,
            image_controls_enabled: false,
            diagram_controls_enabled: false,
            code_controls_enabled: false,
        });
    let mut node = diagram_node();
    node.rect = ViewerRect {
        x: 0.0,
        y: 0.0,
        width: 860.0,
        height: 65.0,
    };
    let surface = ViewerImageSurface {
        fingerprint: "export-graph".to_string(),
        width: 860,
        height: 65,
        display_width: 860.0,
        display_height: 65.0,
        content_scale: 100,
        rgba: [0, 0, 0, OPAQUE_ALPHA].repeat(860 * 65),
    };

    let media = factory.image_surface_node(&node, &ArtifactId("artifact".to_string()), surface);
    let media = factory.media_with_controls(&node, media);
    let row = factory.node_with_viewer_height(media, &node);

    assert_eq!(
        UiDimension::Px(101),
        row.props().common.height,
        "export surface diagram rows must advance by the KUC export frame height so following content does not overlap the media bottom margin"
    );
}

#[test]
fn diagram_media_row_wrapper_preserves_katana_image_origin_for_tall_viewer_row() {
    let factory =
        KucNodeFactory::new(&[], DIAGRAM_MEDIA_MAX_WIDTH).interaction(ViewerInteractionConfig {
            hover_highlight_enabled: false,
            selection_enabled: false,
            image_controls_enabled: false,
            diagram_controls_enabled: false,
            code_controls_enabled: false,
        });
    let mut node = diagram_node();
    node.rect = ViewerRect {
        x: 0.0,
        y: 0.0,
        width: 860.0,
        height: 487.0,
    };
    let surface = ViewerImageSurface {
        fingerprint: "katana-flowchart".to_string(),
        width: 640,
        height: 890,
        display_width: 320.0,
        display_height: 445.0,
        content_scale: 200,
        rgba: [0, 0, 0, OPAQUE_ALPHA].repeat(640 * 890),
    };

    let media = factory.image_surface_node(&node, &ArtifactId("artifact".to_string()), surface);
    let media = factory.media_with_controls(&node, media);
    let row = factory.node_with_viewer_height(media, &node);
    let media_child = row.children().first().kuc_expect("media row child");

    assert_eq!(
        UiDimension::Px(0),
        media_child.props().common.margin.top,
        "KatanA preview pins full-size diagram media to the row origin instead of adding a synthetic top inset"
    );
}
