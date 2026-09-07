use super::{STORYBOOK_SEARCH_QUERY, SearchKeyPress, StorybookSearchKeys};
use crate::preview::PreviewScene;
use katana_document_viewer::{
    ByteRange, KmmNodeId, LineColumn, LineColumnRange, RawSnippet, SourceSpan, ViewerMode,
    ViewerRect, ViewerSearchEngine, ViewerSearchMatch, ViewerSearchMatchId, ViewerSearchState,
    ViewerSearchTarget, ViewerTextRange,
};
use katana_ui_core::atom::Text;
use katana_ui_core::render_model::UiTree;
use katana_ui_core::theme::ThemeSnapshot;

#[test]
fn toggles_storybook_search_query() {
    let mut search = ViewerSearchState::default();

    assert!(StorybookSearchKeys::toggle_search(&mut search));

    assert_eq!(STORYBOOK_SEARCH_QUERY, search.query);

    assert!(StorybookSearchKeys::toggle_search(&mut search));

    assert!(search.query.is_empty());
}

#[test]
fn next_search_hit_updates_scroll_and_current_index() {
    let mut search = ViewerSearchEngine::state(STORYBOOK_SEARCH_QUERY, Vec::new(), None);
    let scene = scene_with_targets([120.0, 360.0]);
    let mut scroll_y = 0.0;

    let changed = StorybookSearchKeys::apply_pressed(
        SearchKeyPress::Next,
        &mut search,
        Some(&scene),
        &mut scroll_y,
        400.0,
    );

    assert!(changed);
    assert!((scroll_y - 120.0).abs() < f32::EPSILON);
    assert_eq!(search.current_index, Some(0));
    assert_eq!(search.matches.len(), 2);
}

#[test]
fn previous_search_hit_wraps_to_last_target() {
    let mut search = ViewerSearchEngine::state(STORYBOOK_SEARCH_QUERY, Vec::new(), None);
    let scene = scene_with_targets([120.0, 360.0]);
    let mut scroll_y = 0.0;

    let changed = StorybookSearchKeys::apply_pressed(
        SearchKeyPress::Previous,
        &mut search,
        Some(&scene),
        &mut scroll_y,
        400.0,
    );

    assert!(changed);
    assert!((scroll_y - 360.0).abs() < f32::EPSILON);
    assert_eq!(search.current_index, Some(1));
}

#[test]
fn search_jump_scroll_is_clamped_to_viewport_bounds() {
    let mut search = ViewerSearchEngine::state(STORYBOOK_SEARCH_QUERY, Vec::new(), None);
    let scene = scene_with_targets([760.0]);
    let mut scroll_y = 0.0;

    let changed = StorybookSearchKeys::apply_pressed(
        SearchKeyPress::Next,
        &mut search,
        Some(&scene),
        &mut scroll_y,
        400.0,
    );

    assert!(changed);
    assert!((scroll_y - 760.0).abs() < f32::EPSILON);
    assert_eq!(search.current_index, Some(0));
}

fn scene_with_targets<const N: usize>(ys: [f32; N]) -> PreviewScene {
    PreviewScene {
        document_id: "test.md".to_string(),
        tree: UiTree::new(Text::new("scene")),
        theme: ThemeSnapshot::light(),
        host_action_cache: Default::default(),
        node_count: 0,
        mode: ViewerMode::Document,
        typography: Default::default(),
        asset_request_count: 0,
        asset_request_key: String::new(),
        loaded_asset_count: 0,
        failed_asset_count: 0,
        image_surface_count: 0,
        surface: None,
        content_height: 800.0,
        scroll_redraw_sensitive_rects: Vec::new(),
        scroll_redraw_diagram_boundary_rects: Vec::new(),
        slideshow_current_page: 0,
        slideshow_max_page: 0,
        diagram_viewports: Default::default(),
        diagram_node_ids: Default::default(),
        search_targets: ys
            .into_iter()
            .enumerate()
            .map(|(index, y)| test_target(index, y))
            .collect(),
        targets: Vec::new(),
        target_lookup: Default::default(),
        internal_anchor_lookup: Default::default(),
        warnings: Vec::new(),
    }
}

fn test_target(index: usize, y: f32) -> ViewerSearchTarget {
    ViewerSearchTarget {
        index,
        matched: ViewerSearchMatch {
            id: ViewerSearchMatchId(format!("hit-{index}")),
            node_id: KmmNodeId(format!("node-{index}")),
            source: source_span(STORYBOOK_SEARCH_QUERY),
            range: ViewerTextRange { start: 0, end: 6 },
            text: STORYBOOK_SEARCH_QUERY.to_string(),
            artifact_id: None,
        },
        rect: ViewerRect {
            x: 0.0,
            y,
            width: 120.0,
            height: 32.0,
        },
    }
}

fn source_span(text: &str) -> SourceSpan {
    SourceSpan {
        byte_range: ByteRange {
            start: 0,
            end: text.len(),
        },
        line_column_range: LineColumnRange {
            start: LineColumn { line: 1, column: 1 },
            end: LineColumn {
                line: 1,
                column: text.len() + 1,
            },
        },
        raw: RawSnippet {
            text: text.to_string(),
        },
    }
}
