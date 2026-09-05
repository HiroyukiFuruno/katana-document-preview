use super::{
    StorybookSidebar, StorybookSidebarBoundsRequest, StorybookSidebarRequest,
    StorybookSidebarScroll,
};
use crate::catalog::StorybookFixture;
use crate::preview::PreviewScene;
use crate::sidebar_settings_state::StorybookSettingsState;
use katana_document_viewer::{ViewerInteractionConfig, ViewerMode};
use katana_ui_core::atom::Text;
use katana_ui_core::molecule::SettingsListEvent;
use katana_ui_core::render_model::{UiNode, UiNodeKind, UiTree, UiTreeNodeKind};
use katana_ui_core::theme::ThemeSnapshot;
use std::path::PathBuf;

#[test]
fn sidebar_uses_kuc_file_tree_and_settings_list() {
    let fixtures = vec![
        fixture("direct/sample.md"),
        fixture("katana/sample_html.md"),
    ];
    let tree = render_sidebar(&fixtures, 1, None, StorybookSidebarScroll::default());
    let nodes = tree.root().children();
    assert_eq!(UiNodeKind::ScrollArea, nodes[0].kind());
    assert_eq!(UiNodeKind::TreeView, nodes[0].children()[0].kind());
    assert_eq!(UiNodeKind::ScrollArea, nodes[1].kind());
    assert_eq!(UiNodeKind::SettingsList, nodes[1].children()[0].kind());
    assert_eq!(450, nodes[0].props().scroll_area.viewport_height);
    assert_eq!(450, nodes[1].props().scroll_area.viewport_height);
    assert_eq!(
        "katana/sample_html.md",
        nodes[0].children()[0].props().tree.active_id
    );
    assert!(
        nodes[0].children()[0]
            .props()
            .tree
            .nodes
            .iter()
            .any(|node| {
                node.kind == UiTreeNodeKind::Directory && node.id == "katana" && node.depth == 0
            })
    );
}

#[test]
fn sidebar_file_tree_hides_redundant_direct_fixture_extensions() {
    let fixtures = vec![
        fixture("direct/html-alignment.htm"),
        fixture("direct/html-alignment.html"),
        fixture("direct/kdv-icon.bmp"),
        fixture("direct/kdv-icon.png"),
        fixture("direct/sample.drawio"),
        fixture("direct/sample.drowio"),
        fixture("direct/sample.mmd"),
        fixture("direct/sample.puml"),
        fixture("direct/sample.md"),
        fixture("direct/sample.markdown"),
    ];

    let tree = render_sidebar(&fixtures, 1, None, StorybookSidebarScroll::default());
    let tree_view = &tree.root().children()[0].children()[0];
    let labels = tree_view
        .props()
        .tree
        .nodes
        .iter()
        .map(|node| node.label.as_str())
        .collect::<Vec<_>>();

    assert!(labels.contains(&"html"));
    assert!(labels.contains(&"image"));
    assert!(labels.contains(&"diagram"));
    assert!(!labels.contains(&"markdown"));
    assert!(labels.contains(&"html-alignment.html"));
    assert!(labels.contains(&"kdv-icon.png"));
    assert!(labels.contains(&"sample.drawio"));
    assert!(labels.contains(&"sample.mmd"));
    assert!(labels.contains(&"sample.puml"));
    assert!(!labels.contains(&"html-alignment.htm"));
    assert!(!labels.contains(&"kdv-icon.bmp"));
    assert!(!labels.contains(&"sample.drowio"));
    assert!(!labels.contains(&"sample.md"));
    assert!(!labels.contains(&"sample.markdown"));
}

#[test]
fn sidebar_groups_katana_aggregate_diagram_fixtures_as_diagrams() {
    let fixtures = vec![
        fixture("katana/sample.md"),
        fixture("katana/sample_basic.md"),
        fixture("katana/sample_html.md"),
        fixture("katana/sample_diagrams.md"),
        fixture("katana/sample_mermaid.md"),
    ];

    let tree = render_sidebar(&fixtures, 0, None, StorybookSidebarScroll::default());
    let tree_view = &tree.root().children()[0].children()[0];
    let nodes = &tree_view.props().tree.nodes;

    assert!(
        nodes
            .iter()
            .any(|node| { node.kind == UiTreeNodeKind::Directory && node.id == "katana/diagram" })
    );
    assert!(nodes.iter().any(|node| node.label == "sample_diagrams.md"));
    assert!(nodes.iter().any(|node| node.label == "sample_mermaid.md"));
}

#[test]
fn sidebar_file_tree_passes_katana_reference_file_icon_kinds_to_kuc_nodes() {
    let fixtures = vec![
        fixture("katana/sample.md"),
        fixture("direct/kdv-icon.png"),
        fixture("katana/drawio/basic/05-edge-variants.drawio"),
    ];

    let tree = render_sidebar(&fixtures, 0, None, StorybookSidebarScroll::default());
    let tree_view = &tree.root().children()[0].children()[0];
    let nodes = &tree_view.props().tree.nodes;

    assert!(nodes.iter().any(|node| node.label == "sample.md"
        && node.kind == UiTreeNodeKind::File
        && node.icon == "markdown"));
    assert!(nodes.iter().any(|node| node.label == "kdv-icon.png"
        && node.kind == UiTreeNodeKind::File
        && node.icon == "image"));
    assert!(
        nodes
            .iter()
            .any(|node| node.label == "05-edge-variants.drawio"
                && node.kind == UiTreeNodeKind::File
                && node.icon == "document")
    );
}

#[test]
fn sidebar_file_tree_keeps_hidden_direct_fixture_when_selected() {
    let fixtures = vec![
        fixture("direct/html-alignment.htm"),
        fixture("direct/html-alignment.html"),
        fixture("katana/sample.md"),
    ];

    let tree = render_sidebar(&fixtures, 1, None, StorybookSidebarScroll::default());
    let tree_view = &tree.root().children()[0].children()[0];
    let labels = tree_view
        .props()
        .tree
        .nodes
        .iter()
        .map(|node| node.label.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        "direct/html-alignment.html",
        tree_view.props().tree.active_id
    );
    assert!(labels.contains(&"html-alignment.html"));
}

#[test]
fn sidebar_state_shows_mode_and_human_slide_index() {
    let fixtures = vec![fixture("direct/sample.md")];
    let scene = PreviewScene {
        document_id: "direct/sample.md".to_string(),
        tree: UiTree::new(Text::new("scene")),
        theme: ThemeSnapshot::dark(),
        host_action_cache: Default::default(),
        node_count: 8,
        mode: ViewerMode::Slideshow,
        typography: Default::default(),
        asset_request_count: 0,
        asset_request_key: String::new(),
        loaded_asset_count: 2,
        failed_asset_count: 0,
        image_surface_count: 1,
        surface: None,
        content_height: 1200.0,
        scroll_redraw_sensitive_rects: Vec::new(),
        scroll_redraw_diagram_boundary_rects: Vec::new(),
        slideshow_current_page: 1,
        slideshow_max_page: 4,
        diagram_viewports: Default::default(),
        diagram_node_ids: Default::default(),
        search_targets: Vec::new(),
        targets: Vec::new(),
        target_lookup: Default::default(),
        internal_anchor_lookup: Default::default(),
        warnings: Vec::new(),
    };

    let tree = render_sidebar(
        &fixtures,
        0,
        Some(&scene),
        StorybookSidebarScroll::default(),
    );

    assert!(contains_select_value(tree.root(), "slideshow"));
    assert!(contains_input_value(tree.root(), "2/5"));
}

#[test]
fn sidebar_settings_viewport_reports_preview_viewport_not_sidebar_layout() {
    let fixtures = vec![fixture("direct/sample.md")];

    let tree = render_sidebar(&fixtures, 0, None, StorybookSidebarScroll::default());

    assert!(contains_input_value(tree.root(), "900x868"));
    assert!(!contains_input_value(tree.root(), "300x450"));
}

#[test]
fn sidebar_settings_section_state_collapses_display_fields() {
    let fixtures = vec![fixture("direct/sample.md")];
    let mut settings_state = StorybookSettingsState::default();
    settings_state.apply_events(&[SettingsListEvent::SectionCollapsed {
        section_id: "display".to_string(),
        collapsed: true,
    }]);

    let tree = render_sidebar_with_settings_state(
        &fixtures,
        0,
        None,
        StorybookSidebarScroll::default(),
        &settings_state,
    );

    assert!(contains_label(tree.root(), "Display"));
    assert!(!contains_label(tree.root(), "Dark"));
    assert!(contains_label(tree.root(), "Interaction"));
}

#[test]
fn sidebar_scroll_bounds_match_rendered_scroll_areas() {
    let fixtures = vec![
        fixture("katana/sample.md"),
        fixture("katana/drawio/basic/01-empty-mxfile.drawio"),
        fixture("katana/drawio/basic/02-standalone-mxgraphmodel.drawio"),
        fixture("katana/drawio/basic/03-basic-flow.drawio"),
    ];
    let scroll = StorybookSidebarScroll {
        tree_y: 48,
        settings_y: 24,
    };
    let rendered = render_sidebar(&fixtures, 0, None, scroll);
    let (tree, settings) = StorybookSidebar::scroll_bounds(StorybookSidebarBoundsRequest {
        fixtures: &fixtures,
        selected_index: 0,
        scene: None,
        dark: true,
        interaction: &ViewerInteractionConfig::default(),
        typography: Default::default(),
        file_tree_state: &Default::default(),
        settings_state: &Default::default(),
        height: 900,
        preview_width: 900,
        preview_height: 868,
        scroll,
    });
    let rendered_tree = &rendered.root().children()[0].props().scroll_area;
    let rendered_settings = &rendered.root().children()[1].props().scroll_area;

    assert_eq!(rendered_tree.offset_y as f32, tree.offset_y);
    assert_eq!(rendered_tree.viewport_height as f32, tree.viewport_height);
    assert_eq!(rendered_tree.content_height as f32, tree.content_height);
    assert_eq!(rendered_settings.offset_y as f32, settings.offset_y);
    assert_eq!(
        rendered_settings.viewport_height as f32,
        settings.viewport_height
    );
    assert_eq!(
        rendered_settings.content_height as f32,
        settings.content_height
    );
}

fn contains_select_value(node: &UiNode, value: &str) -> bool {
    if node.kind() == UiNodeKind::SelectBox && node.props().interaction.value == value {
        return true;
    }
    node.children()
        .iter()
        .any(|child| contains_select_value(child, value))
}

fn contains_input_value(node: &UiNode, value: &str) -> bool {
    if node.kind() == UiNodeKind::Input && node.props().interaction.value == value {
        return true;
    }
    node.children()
        .iter()
        .any(|child| contains_input_value(child, value))
}

fn contains_label(node: &UiNode, value: &str) -> bool {
    if node.props().label == value {
        return true;
    }
    node.children()
        .iter()
        .any(|child| contains_label(child, value))
}

fn fixture(label: &str) -> StorybookFixture {
    StorybookFixture {
        label: label.to_string(),
        path: PathBuf::from(label),
    }
}

fn render_sidebar(
    fixtures: &[StorybookFixture],
    selected_index: usize,
    scene: Option<&PreviewScene>,
    scroll: StorybookSidebarScroll,
) -> UiTree {
    render_sidebar_with_settings_state(
        fixtures,
        selected_index,
        scene,
        scroll,
        &StorybookSettingsState::default(),
    )
}

fn render_sidebar_with_settings_state(
    fixtures: &[StorybookFixture],
    selected_index: usize,
    scene: Option<&PreviewScene>,
    scroll: StorybookSidebarScroll,
    settings_state: &StorybookSettingsState,
) -> UiTree {
    StorybookSidebar::render(StorybookSidebarRequest {
        fixtures,
        selected_index,
        scene,
        dark: true,
        interaction: &ViewerInteractionConfig::default(),
        typography: Default::default(),
        file_tree_state: Default::default(),
        settings_state,
        width: 300,
        height: 900,
        preview_width: 900,
        preview_height: 868,
        scroll,
    })
}
