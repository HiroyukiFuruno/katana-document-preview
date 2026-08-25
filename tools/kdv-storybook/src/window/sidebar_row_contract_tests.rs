use super::super::StorybookWindow;
use super::interaction_matrix_support::storybook_with_catalog;
use crate::layout::{
    SIDEBAR_CONTENT_INSET, sidebar_content_height, sidebar_content_width, sidebar_content_x,
};
use crate::mouse::mouse_test_support::{WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::mouse::{StorybookMouseButton, StorybookPointer};
use crate::sidebar::{StorybookSidebar, StorybookSidebarRequest};
use crate::sidebar_test_support::StorybookFileTreeItemPointRequest;
use katana_ui_core::molecule::TreeView;
use katana_ui_core::render_model::{UiCursor, UiNode, UiNodeKind, UiTreeNodeKind};
use katana_ui_core_storybook::StorybookPresentation;

const SIDEBAR_TREE_HOVER_BACKGROUND: u32 = 0x243041;
const SIDEBAR_TREE_SELECTION: u32 = 0x264f78;

#[test]
fn rendered_file_tree_row_center_selects_same_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = storybook_with_catalog()?;
    let selected = storybook.catalog.fixtures[storybook.selected_index]
        .label
        .clone();
    let row = visible_tree_row(&storybook, UiTreeNodeKind::File, Some(&selected))?;

    assert_eq!(
        UiCursor::Pointer,
        storybook.cursor_for_canvas_point(tree_row_x(), row.center_y, WINDOW_WIDTH, WINDOW_HEIGHT)
    );
    assert!(storybook.update_sidebar_tree_hover_for_canvas_point(
        tree_row_x(),
        row.center_y,
        WINDOW_WIDTH,
        WINDOW_HEIGHT
    ));
    assert_eq!(
        Some(row.id.as_str()),
        storybook.file_tree_state.hovered_item_id()
    );
    assert!(storybook.apply_canvas_click(
        StorybookPointer::new(tree_row_x(), row.center_y, StorybookMouseButton::Left),
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    )?);
    assert_eq!(
        row.id,
        storybook.catalog.fixtures[storybook.selected_index].label
    );
    assert_eq!("select-file", storybook.last_command_label);
    Ok(())
}

#[test]
fn rendered_file_tree_row_center_toggles_same_directory_after_scroll()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = storybook_with_catalog()?;
    storybook.sidebar_scroll.tree_y = TreeView::row_height();
    let row = visible_tree_row(&storybook, UiTreeNodeKind::Directory, None)?;

    assert_eq!(
        UiCursor::Pointer,
        storybook.cursor_for_canvas_point(tree_row_x(), row.center_y, WINDOW_WIDTH, WINDOW_HEIGHT)
    );
    assert!(storybook.update_sidebar_tree_hover_for_canvas_point(
        tree_row_x(),
        row.center_y,
        WINDOW_WIDTH,
        WINDOW_HEIGHT
    ));
    assert_eq!(
        Some(row.id.as_str()),
        storybook.file_tree_state.hovered_item_id()
    );
    assert!(storybook.apply_canvas_click(
        StorybookPointer::new(tree_row_x(), row.center_y, StorybookMouseButton::Left),
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    )?);
    assert!(storybook.file_tree_state.is_collapsed(&row.id));
    assert_eq!("toggle-directory", storybook.last_command_label);
    Ok(())
}

#[test]
fn rendered_file_tree_directory_hover_paints_kuc_row_background()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = storybook_with_catalog()?;
    let row = visible_tree_row(&storybook, UiTreeNodeKind::Directory, None)?;

    let normal = storybook.render_canvas(WINDOW_WIDTH, WINDOW_HEIGHT);
    assert_eq!(
        UiCursor::Pointer,
        storybook.cursor_for_canvas_point(tree_row_x(), row.center_y, WINDOW_WIDTH, WINDOW_HEIGHT)
    );
    assert!(storybook.update_sidebar_tree_hover_for_canvas_point(
        tree_row_x(),
        row.center_y,
        WINDOW_WIDTH,
        WINDOW_HEIGHT
    ));
    assert_eq!(
        Some(row.id.as_str()),
        storybook.file_tree_state.hovered_item_id()
    );
    let hovered = storybook.render_canvas(WINDOW_WIDTH, WINDOW_HEIGHT);

    let normal_count = color_count(&normal, SIDEBAR_TREE_HOVER_BACKGROUND);
    let hovered_count = color_count(&hovered, SIDEBAR_TREE_HOVER_BACKGROUND);
    assert!(
        hovered_count > normal_count,
        "directory hover must paint KUC TreeView row background: normal={normal_count} hovered={hovered_count}"
    );
    assert!(
        pixel_diff_count(&normal, &hovered) > 64,
        "directory hover must visibly change rendered sidebar pixels"
    );
    Ok(())
}

#[test]
fn rendered_file_tree_directory_click_collapses_visible_rows_and_pixels()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = storybook_with_catalog()?;
    let row = visible_tree_row(&storybook, UiTreeNodeKind::Directory, None)?;
    let before_visible_rows = visible_tree_node_count(&storybook)?;
    let before = storybook.render_canvas(WINDOW_WIDTH, WINDOW_HEIGHT);

    assert!(storybook.apply_canvas_click(
        StorybookPointer::new(tree_row_x(), row.center_y, StorybookMouseButton::Left),
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    )?);
    assert!(storybook.file_tree_state.is_collapsed(&row.id));
    assert_eq!("toggle-directory", storybook.last_command_label);

    let after_visible_rows = visible_tree_node_count(&storybook)?;
    assert!(
        after_visible_rows < before_visible_rows,
        "collapsing directory must reduce visible KUC TreeView rows: before={before_visible_rows} after={after_visible_rows}"
    );
    let after = storybook.render_canvas(WINDOW_WIDTH, WINDOW_HEIGHT);
    assert!(
        pixel_diff_count(&before, &after) > 64,
        "directory collapse must visibly change rendered sidebar pixels"
    );
    Ok(())
}

#[test]
fn scrolled_file_tree_hover_then_click_matrix_selects_visible_files()
-> Result<(), Box<dyn std::error::Error>> {
    let mut storybook = storybook_with_catalog()?;
    storybook.sidebar_scroll.tree_y = TreeView::row_height().saturating_mul(4);
    let target_ids = visible_fixture_ids(&storybook, 3)?;

    assert!(
        target_ids.len() >= 3,
        "scrolled FileTree must expose multiple visible file actions: {target_ids:?}"
    );
    for target_id in target_ids {
        let (x, y) = file_tree_item_canvas_point(&storybook, &target_id)?;
        assert_eq!(
            UiCursor::Pointer,
            storybook.cursor_for_canvas_point(x, y, WINDOW_WIDTH, WINDOW_HEIGHT),
            "visible scrolled FileTree row must expose pointer cursor: {target_id}"
        );
        assert!(storybook.update_sidebar_tree_hover_for_canvas_point(
            x,
            y,
            WINDOW_WIDTH,
            WINDOW_HEIGHT
        ));
        assert_eq!(
            Some(target_id.as_str()),
            storybook.file_tree_state.hovered_item_id(),
            "hover must resolve to the same KUC FileTree item before click"
        );
        assert!(storybook.apply_canvas_click(
            StorybookPointer::new(x, y, StorybookMouseButton::Left),
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        )?);
        assert_eq!(
            target_id, storybook.catalog.fixtures[storybook.selected_index].label,
            "click must select the same fixture returned by KUC host action"
        );
        assert_eq!("select-file", storybook.last_command_label);
    }
    Ok(())
}

#[test]
fn presented_file_tree_selection_pixels_click_the_same_rows_at_retina_scale()
-> Result<(), Box<dyn std::error::Error>> {
    assert_presented_selection_pixels_click_same_rows(2.0, WINDOW_WIDTH, WINDOW_HEIGHT, 0)
}

#[test]
fn presented_file_tree_selection_pixels_click_the_same_rows_after_resize_and_scroll()
-> Result<(), Box<dyn std::error::Error>> {
    assert_presented_selection_pixels_click_same_rows(
        2.0,
        WINDOW_WIDTH + 377,
        WINDOW_HEIGHT + 211,
        TreeView::row_height().saturating_mul(3),
    )
}

fn assert_presented_selection_pixels_click_same_rows(
    scale: f32,
    width: usize,
    height: usize,
    tree_scroll_y: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let catalog = storybook_with_catalog()?.catalog.clone();
    let candidate_indices = (0..catalog.fixtures.len()).take(12).collect::<Vec<_>>();
    let mut checked = 0;

    for target_index in candidate_indices {
        let target_id = catalog.fixtures[target_index].label.clone();
        let mut rendered = storybook_with_catalog()?;
        rendered.selected_index = target_index;
        rendered.sidebar_scroll.tree_y = tree_scroll_y;
        let scaled = rendered.render_canvas_scaled(width, height, scale);
        let presented = StorybookPresentation::present_frame_for_window(
            &scaled,
            width,
            height,
            scaled.pixels().first().copied().unwrap_or_default(),
        );
        let Some((x, y)) = selected_row_pixel_center(&presented, height) else {
            continue;
        };

        let mut clicked = storybook_with_catalog()?;
        clicked.sidebar_scroll.tree_y = tree_scroll_y;
        if clicked.catalog.fixtures[clicked.selected_index].label == target_id {
            clicked.selected_index = (target_index + 1) % clicked.catalog.fixtures.len();
        }
        assert!(
            clicked.update_sidebar_tree_hover_for_canvas_point(x, y, width, height),
            "presented FileTree selection pixel must hover a row: target={target_id} point=({x},{y})"
        );
        assert_eq!(
            Some(target_id.as_str()),
            clicked.file_tree_state.hovered_item_id(),
            "presented FileTree selection pixel must hover the same item"
        );
        assert!(clicked.apply_canvas_click(
            StorybookPointer::new(x, y, StorybookMouseButton::Left),
            width,
            height,
        )?);
        assert_eq!(
            target_id, clicked.catalog.fixtures[clicked.selected_index].label,
            "presented FileTree selection pixel must click the same item"
        );
        checked += 1;
        if checked == 3 {
            break;
        }
    }

    assert_eq!(
        3, checked,
        "three visible FileTree rows must be independently checked"
    );
    Ok(())
}

fn selected_row_pixel_center(
    canvas: &crate::canvas::Canvas,
    window_height: usize,
) -> Option<(f32, f32)> {
    let top = SIDEBAR_CONTENT_INSET;
    let bottom = SIDEBAR_CONTENT_INSET + sidebar_content_height(window_height) / 2;
    let row_width = sidebar_content_width().saturating_sub(8);
    let selected_y = (top..bottom.min(canvas.height())).find(|y| {
        (sidebar_content_x()..(sidebar_content_x() + row_width).min(canvas.width()))
            .filter(|x| canvas.pixels()[y * canvas.width() + x] == SIDEBAR_TREE_SELECTION)
            .count()
            >= row_width.saturating_sub(2)
    })?;
    let row_height = TreeView::row_height() as usize;
    Some((
        sidebar_content_x() as f32 + 24.0,
        selected_y.saturating_add(row_height / 2) as f32,
    ))
}

fn visible_tree_row(
    storybook: &StorybookWindow,
    kind: UiTreeNodeKind,
    excluded_id: Option<&str>,
) -> Result<RenderedTreeRow, Box<dyn std::error::Error>> {
    let sidebar = StorybookSidebar::render(StorybookSidebarRequest {
        fixtures: &storybook.catalog.fixtures,
        selected_index: storybook.selected_index,
        scene: storybook.scene.as_ref(),
        dark: storybook.dark,
        interaction: &storybook.interaction,
        typography: storybook.typography,
        file_tree_state: storybook.file_tree_state.clone(),
        settings_state: &storybook.settings_state,
        width: sidebar_content_width(),
        height: sidebar_content_height(WINDOW_HEIGHT),
        preview_width: WINDOW_WIDTH,
        preview_height: WINDOW_HEIGHT,
        scroll: storybook.sidebar_scroll,
    });
    let tree = find_tree_view(sidebar.root()).ok_or("KUC TreeView missing")?;
    let label_rows = usize::from(!tree.props().label.trim().is_empty());
    for (index, node) in tree.props().tree.nodes.iter().enumerate() {
        if node.kind != kind || excluded_id.is_some_and(|id| id == node.id) {
            continue;
        }
        let center_y = row_center_y(index + label_rows, storybook.sidebar_scroll.tree_y);
        if center_y >= SIDEBAR_CONTENT_INSET as f32
            && center_y < (SIDEBAR_CONTENT_INSET + sidebar_content_height(WINDOW_HEIGHT) / 2) as f32
        {
            return Ok(RenderedTreeRow {
                id: node.id.clone(),
                center_y,
            });
        }
    }
    Err("visible KUC tree row missing".into())
}

fn visible_tree_node_count(
    storybook: &StorybookWindow,
) -> Result<usize, Box<dyn std::error::Error>> {
    let sidebar = StorybookSidebar::render(StorybookSidebarRequest {
        fixtures: &storybook.catalog.fixtures,
        selected_index: storybook.selected_index,
        scene: storybook.scene.as_ref(),
        dark: storybook.dark,
        interaction: &storybook.interaction,
        typography: storybook.typography,
        file_tree_state: storybook.file_tree_state.clone(),
        settings_state: &storybook.settings_state,
        width: sidebar_content_width(),
        height: sidebar_content_height(WINDOW_HEIGHT),
        preview_width: WINDOW_WIDTH,
        preview_height: WINDOW_HEIGHT,
        scroll: storybook.sidebar_scroll,
    });
    let tree = find_tree_view(sidebar.root()).ok_or("KUC TreeView missing")?;
    Ok(tree.props().tree.nodes.len())
}

fn visible_fixture_ids(
    storybook: &StorybookWindow,
    limit: usize,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let selected = storybook.catalog.fixtures[storybook.selected_index]
        .label
        .as_str();
    let visible = storybook
        .catalog
        .fixtures
        .iter()
        .filter(|fixture| fixture.label != selected)
        .filter_map(|fixture| {
            file_tree_item_canvas_point(storybook, &fixture.label)
                .ok()
                .map(|_| fixture.label.clone())
        })
        .take(limit)
        .collect::<Vec<_>>();
    Ok(visible)
}

fn file_tree_item_canvas_point(
    storybook: &StorybookWindow,
    item_id: &str,
) -> Result<(f32, f32), Box<dyn std::error::Error>> {
    let point =
        StorybookSidebar::fixture_canvas_point_for_item_id(StorybookFileTreeItemPointRequest {
            fixtures: &storybook.catalog.fixtures,
            selected_index: storybook.selected_index,
            state: &storybook.file_tree_state,
            item_id,
            height: sidebar_content_height(WINDOW_HEIGHT),
            scroll: storybook.sidebar_scroll,
        })
        .ok_or_else(|| format!("visible FileTree action missing: {item_id}"))?;
    Ok((point.x, point.y))
}

fn row_center_y(row_index: usize, scroll_y: u32) -> f32 {
    let row_height = TreeView::row_height() as f32;
    SIDEBAR_CONTENT_INSET as f32 + row_index as f32 * row_height + row_height / 2.0
        - scroll_y as f32
}

fn tree_row_x() -> f32 {
    sidebar_content_x() as f32 + 24.0
}

fn find_tree_view(node: &UiNode) -> Option<&UiNode> {
    if node.kind() == UiNodeKind::TreeView {
        return Some(node);
    }
    node.children().iter().find_map(find_tree_view)
}

fn color_count(canvas: &crate::canvas::Canvas, color: u32) -> usize {
    canvas
        .pixels()
        .iter()
        .filter(|pixel| **pixel == color)
        .count()
}

fn pixel_diff_count(left: &crate::canvas::Canvas, right: &crate::canvas::Canvas) -> usize {
    left.pixels()
        .iter()
        .zip(right.pixels().iter())
        .filter(|(left, right)| left != right)
        .count()
}

struct RenderedTreeRow {
    id: String,
    center_y: f32,
}
