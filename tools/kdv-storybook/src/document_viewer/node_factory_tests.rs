use super::KucNodeFactory;
use super::node_factory_tests_support::{has_style_class, viewer_node};
use katana_document_viewer::{
    ViewerInteractionConfig, ViewerNodeKind, ViewerTextSpan, ViewerTextStyle,
};
use katana_ui_core::render_model::{UiDimension, UiNodeKind, UiTextWrapMode, UiVisualRole};

#[test]
fn text_node_preserves_viewer_inline_spans() {
    let factory = KucNodeFactory::new(&[], 120);
    let mut node = viewer_node(ViewerNodeKind::Paragraph, "bold link");
    node.spans = vec![
        ViewerTextSpan::styled("bold", ViewerTextStyle::default().bold()),
        ViewerTextSpan::linked(
            "link",
            "https://example.com",
            ViewerTextStyle::default().link(),
        ),
    ];

    let ui_node = factory.viewer_node(&node);

    assert!(ui_node.props().text.spans[0].style.bold);
    assert_eq!(
        "https://example.com",
        ui_node.props().text.spans[1].link_target
    );
    assert!(ui_node.props().text.spans[1].style.underline);
}

#[test]
fn paragraph_text_uses_kuc_wrap_contract() {
    let factory = KucNodeFactory::new(&[], 120);
    let node = viewer_node(ViewerNodeKind::Paragraph, "long body line");

    let ui_node = factory.viewer_node(&node);

    assert_eq!(UiTextWrapMode::Wrap, ui_node.props().text.wrap);
}

#[test]
fn heading_node_preserves_viewer_inline_code_spans() {
    let factory = KucNodeFactory::new(&[], 120);
    let mut node = viewer_node(
        ViewerNodeKind::Heading { level: 3 },
        r#"1.1 `<h1 align="center">`"#,
    );
    node.spans = vec![
        ViewerTextSpan::plain("1.1 "),
        ViewerTextSpan::styled(
            r#"<h1 align="center">"#,
            ViewerTextStyle::default().inline_code(),
        ),
    ];

    let ui_node = factory.viewer_node(&node);

    assert_eq!("heading-3", ui_node.props().text.role);
    assert_eq!(2, ui_node.props().text.spans.len());
    assert!(ui_node.props().text.spans[1].style.inline_code);
    assert!(!ui_node.props().text.spans[1].text.contains('`'));
}

#[test]
fn viewer_node_height_comes_from_viewer_rect() {
    let factory = KucNodeFactory::new(&[], 120);
    let node = viewer_node(ViewerNodeKind::Paragraph, "Body");

    let ui_node = factory.viewer_node(&node);

    assert_eq!(UiDimension::Px(32), ui_node.props().common.height);
}

#[test]
fn viewer_text_node_exposes_stable_id_for_host_hover() {
    let factory = KucNodeFactory::new(&[], 120);
    let node = viewer_node(ViewerNodeKind::Paragraph, "Body");

    let ui_node = factory.viewer_node(&node);

    assert_eq!(node.node_id.0, ui_node.id().as_str());
    assert_eq!(node.node_id.0, ui_node.props().state_id.as_str());
}

#[test]
fn hover_surface_preserves_viewer_node_geometry_and_semantic_id() {
    let mut node = viewer_node(ViewerNodeKind::Paragraph, "Body");
    node.rect.width = 184.0;
    node.rect.height = 36.0;
    let factory = KucNodeFactory::new(&[], 240)
        .interaction(ViewerInteractionConfig {
            hover_highlight_enabled: true,
            selection_enabled: false,
            image_controls_enabled: false,
            diagram_controls_enabled: false,
            code_controls_enabled: false,
        })
        .hovered_node_id(Some(node.node_id.0.as_str()));

    let ui_node = factory.viewer_node(&node);

    assert_eq!(UiVisualRole::HoverSurface, ui_node.props().visual_role);
    assert_eq!(
        UiDimension::Px(240),
        ui_node.props().common.width,
        "hover/click surface must use the full Markdown row width, not the intrinsic text rect"
    );
    assert_eq!(UiDimension::Px(36), ui_node.props().common.height);
    assert_eq!(node.node_id.0, ui_node.props().common.semantic_node_id);
    assert_eq!(node.node_id.0, ui_node.children()[0].id().as_str());
}

#[test]
fn text_node_preserves_current_search_highlight_style() {
    let factory = KucNodeFactory::new(&[], 120);
    let mut node = viewer_node(ViewerNodeKind::Paragraph, "alpha");
    node.spans = vec![ViewerTextSpan::styled(
        "alpha",
        ViewerTextStyle::default().current_highlight(),
    )];

    let ui_node = factory.viewer_node(&node);

    assert!(ui_node.props().text.spans[0].style.highlight);
    assert!(ui_node.props().text.spans[0].style.current_highlight);
}

#[test]
fn inline_code_only_paragraph_uses_code_font_for_width_parity() {
    let factory = KucNodeFactory::new(&[], 120);
    let mut node = viewer_node(ViewerNodeKind::Paragraph, "long inline code");
    node.spans = vec![ViewerTextSpan::styled(
        "long inline code",
        ViewerTextStyle::default().inline_code(),
    )];

    let ui_node = factory.viewer_node(&node);

    assert_eq!("document-code", ui_node.props().font_role);
    assert_eq!("paragraph", ui_node.props().text.role);
}

#[test]
fn document_heading_uses_plain_heading_role_and_code_uses_common_props() {
    let factory = KucNodeFactory::new(&[], 120);
    let mut heading_node = viewer_node(ViewerNodeKind::Heading { level: 2 }, "Title With Space");
    heading_node.spans = vec![ViewerTextSpan::plain("Title With Space".to_string())];
    let heading = factory.viewer_node(&heading_node);
    let code_factory = KucNodeFactory::new(&[], 120).interaction(ViewerInteractionConfig {
        hover_highlight_enabled: false,
        selection_enabled: false,
        image_controls_enabled: false,
        diagram_controls_enabled: false,
        code_controls_enabled: true,
    });
    let code = code_factory.viewer_node(&viewer_node(
        ViewerNodeKind::Code {
            language: Some("rust".to_string()),
        },
        "fn main() {}",
    ));

    assert!(!has_style_class(&heading, "kdv-document-heading"));
    assert!(!heading.props().common.border.visible);
    assert_eq!("Title With Space", heading.props().text.spans[0].text);
    assert_eq!("Title With Space", heading.props().label);
    assert_eq!(UiNodeKind::Stack, code.kind());
    assert!(!has_style_class(&code.children()[0], "kdv-document-code"));
    assert!(code.children()[0].props().common.border.visible);
    assert_eq!(
        UiDimension::Px(24),
        code.children()[0].props().common.padding.left
    );
}
