use katana_document_viewer::{
    ArtifactId, DiagramViewportState, PreviewSurfaceImage, ViewerMode, ViewerRect, ViewerTarget,
    ViewerTypographyConfig,
};
use katana_document_viewer::{ViewerNodeKind, ViewerNodePlan, ViewerSearchTarget};
use katana_ui_core::render_model::UiTree;
use katana_ui_core::theme::ThemeSnapshot;
use katana_ui_core_storybook::{
    UiTreeHitRect, UiTreeHostActionHit, UiTreeNodeHit, UiTreeRenderArea, UiTreeSurfaceHost,
};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

thread_local! {
    static DARK_TARGET_HOST: RefCell<UiTreeSurfaceHost> =
        RefCell::new(UiTreeSurfaceHost::new(ThemeSnapshot::dark()));
    static LIGHT_TARGET_HOST: RefCell<UiTreeSurfaceHost> =
        RefCell::new(UiTreeSurfaceHost::new(ThemeSnapshot::light()));
    static THEME_TARGET_HOSTS: RefCell<Vec<ThemeTargetHostCache>> = const { RefCell::new(Vec::new()) };
}

#[derive(Debug, Clone)]
pub struct PreviewScene {
    pub document_id: String,
    pub tree: UiTree,
    pub theme: ThemeSnapshot,
    pub host_action_cache: PreviewSceneHostActionCache,
    pub node_count: usize,
    pub mode: ViewerMode,
    pub typography: ViewerTypographyConfig,
    pub asset_request_count: usize,
    pub asset_request_key: String,
    pub loaded_asset_count: usize,
    pub failed_asset_count: usize,
    pub image_surface_count: usize,
    pub surface: Option<PreviewSurfaceImage>,
    pub content_height: f32,
    pub scroll_redraw_sensitive_rects: Vec<ViewerRect>,
    pub scroll_redraw_diagram_boundary_rects: Vec<ViewerRect>,
    pub slideshow_current_page: usize,
    pub slideshow_max_page: usize,
    pub diagram_viewports: BTreeMap<String, DiagramViewportState>,
    pub diagram_node_ids: BTreeSet<String>,
    pub search_targets: Vec<ViewerSearchTarget>,
    pub targets: Vec<ViewerTarget>,
    pub target_lookup: BTreeMap<String, usize>,
    pub internal_anchor_lookup: BTreeMap<String, usize>,
    pub warnings: Vec<String>,
}

impl PreviewScene {
    pub(crate) fn fullscreen_diagram_active(&self) -> bool {
        self.diagram_viewports
            .values()
            .any(|state| state.fullscreen_open)
    }

    pub(crate) fn needs_full_preview_redraw_for_scroll(
        &self,
        previous_scroll_y: f32,
        current_scroll_y: f32,
        viewport_height: usize,
    ) -> bool {
        if self.scroll_redraw_sensitive_rects.is_empty() {
            return false;
        }
        let top = previous_scroll_y.min(current_scroll_y).max(0.0);
        let bottom = previous_scroll_y.max(current_scroll_y).max(0.0) + viewport_height as f32;
        self.scroll_redraw_sensitive_rects
            .iter()
            .any(|rect| rect.y < bottom && rect.y + rect.height > top)
    }

    pub(crate) fn scroll_redraw_band_y_for_downward_scroll(
        &self,
        current_scroll_y: f32,
        viewport_height: usize,
        default_band_y: usize,
    ) -> usize {
        scroll_redraw_band_y_for_diagram_boundaries(
            &self.scroll_redraw_diagram_boundary_rects,
            current_scroll_y,
            viewport_height,
            default_band_y,
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct PreviewSceneHostActionCache {
    entries: Arc<Mutex<Vec<PreviewSceneHostActionHits>>>,
    node_entries: Arc<Mutex<Vec<PreviewSceneNodeHits>>>,
}

impl PreviewSceneHostActionCache {
    pub fn hits_or_insert(
        &self,
        width: usize,
        create: impl FnOnce() -> Vec<UiTreeHostActionHit>,
    ) -> Arc<Vec<UiTreeHostActionHit>> {
        let mut entries = match self.entries.lock() {
            Ok(entries) => entries,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(entry) = entries.iter().find(|entry| entry.width == width) {
            return Arc::clone(&entry.hits);
        }
        let hits = Arc::new(create());
        entries.push(PreviewSceneHostActionHits {
            width,
            hits: Arc::clone(&hits),
        });
        hits
    }

    pub fn node_hits_or_insert(
        &self,
        width: usize,
        create: impl FnOnce() -> Vec<UiTreeNodeHit>,
    ) -> Arc<Vec<UiTreeNodeHit>> {
        let mut entries = match self.node_entries.lock() {
            Ok(entries) => entries,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(entry) = entries.iter().find(|entry| entry.width == width) {
            return Arc::clone(&entry.hits);
        }
        let hits = Arc::new(create());
        entries.push(PreviewSceneNodeHits {
            width,
            hits: Arc::clone(&hits),
        });
        hits
    }
}

#[derive(Debug, Clone)]
struct PreviewSceneHostActionHits {
    width: usize,
    hits: Arc<Vec<UiTreeHostActionHit>>,
}

#[derive(Debug, Clone)]
struct PreviewSceneNodeHits {
    width: usize,
    hits: Arc<Vec<UiTreeNodeHit>>,
}

impl PreviewScene {
    pub fn target_for_node_id(&self, node_id: &str) -> Option<&ViewerTarget> {
        self.target_lookup
            .get(node_id)
            .and_then(|index| self.targets.get(*index))
    }

    pub fn target_for_internal_anchor(&self, anchor: &str) -> Option<&ViewerTarget> {
        let key = anchor.strip_prefix('#').unwrap_or(anchor);
        self.internal_anchor_lookup
            .get(key)
            .and_then(|index| self.targets.get(*index))
    }
}

pub fn viewer_targets(
    plan: &ViewerNodePlan,
    tree: &UiTree,
    theme: &ThemeSnapshot,
    width: f32,
    height: f32,
) -> Vec<ViewerTarget> {
    let rendered_hits = rendered_node_hits(tree, theme, width, height);
    let rendered_rects = rendered_node_rects(&rendered_hits);
    let semantic_rects = rendered_semantic_rects(&rendered_hits);
    let mut targets = plan
        .nodes
        .iter()
        .map(|node| ViewerTarget {
            node_id: node.node_id.clone(),
            source: node.source.clone(),
            artifact_id: node
                .artifact_id
                .clone()
                .unwrap_or_else(|| ArtifactId(format!("node:{}", node.node_id.0))),
            rect: semantic_rects
                .get(node.node_id.0.as_str())
                .or_else(|| rendered_rects.get(node.node_id.0.as_str()))
                .copied()
                .unwrap_or(node.rect),
        })
        .collect::<Vec<_>>();
    append_semantic_alias_targets(&mut targets, &rendered_hits);
    targets
}

pub fn scroll_redraw_sensitive_rects(plan: &ViewerNodePlan) -> Vec<ViewerRect> {
    plan.nodes
        .iter()
        .filter(|node| {
            matches!(node.kind, ViewerNodeKind::Table)
                || (matches!(
                    node.kind,
                    ViewerNodeKind::Paragraph
                        | ViewerNodeKind::List
                        | ViewerNodeKind::Html { .. }
                        | ViewerNodeKind::Alert { .. }
                ) && node.rect.height >= 96.0)
        })
        .map(|node| node.rect)
        .collect()
}

pub(crate) fn scroll_redraw_diagram_boundary_rects(
    plan: &ViewerNodePlan,
    targets: &[ViewerTarget],
) -> Vec<ViewerRect> {
    plan.nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| matches!(node.kind, ViewerNodeKind::Diagram { .. }))
        .filter_map(|(index, _)| {
            let diagram = targets.get(index)?.rect;
            let boundary = targets[..index]
                .iter()
                .rev()
                .find(|target| target.rect.height > 0.0 && target.rect.y <= diagram.y)
                .map_or(diagram, |target| target.rect);
            Some(ViewerRect {
                x: diagram.x,
                y: boundary.y,
                width: diagram.width,
                height: (diagram.y + diagram.height - boundary.y).max(diagram.height),
            })
        })
        .collect()
}

fn scroll_redraw_band_y_for_diagram_boundaries(
    boundaries: &[ViewerRect],
    current_scroll_y: f32,
    viewport_height: usize,
    default_band_y: usize,
) -> usize {
    let current_scroll_y = current_scroll_y.round().max(0.0);
    let band_top = current_scroll_y + default_band_y as f32;
    let band_bottom = current_scroll_y + viewport_height as f32;
    boundaries
        .iter()
        .filter(|rect| rect.y < band_bottom && rect.y + rect.height > band_top)
        .filter_map(|rect| {
            let local_y = rect.y - current_scroll_y;
            (local_y >= 0.0 && local_y.is_finite()).then_some(local_y.floor() as usize)
        })
        .fold(default_band_y, usize::min)
}

fn rendered_node_hits(
    tree: &UiTree,
    theme: &ThemeSnapshot,
    width: f32,
    height: f32,
) -> Vec<UiTreeNodeHit> {
    document_node_hits_with_cached_host(
        tree.root(),
        UiTreeRenderArea {
            x: 0,
            y: 0,
            width: width.ceil().max(1.0) as usize,
            height: height.ceil().max(1.0) as usize,
            scroll_y: 0.0,
        },
        theme,
    )
}

fn document_node_hits_with_cached_host(
    root: &katana_ui_core::render_model::UiNode,
    area: UiTreeRenderArea,
    theme: &ThemeSnapshot,
) -> Vec<UiTreeNodeHit> {
    if theme.eq(&ThemeSnapshot::dark()) {
        return DARK_TARGET_HOST.with(|host| host.borrow().document_node_hits(root, area));
    }
    if theme.eq(&ThemeSnapshot::light()) {
        return LIGHT_TARGET_HOST.with(|host| host.borrow().document_node_hits(root, area));
    }
    THEME_TARGET_HOSTS.with(|hosts| {
        let mut hosts = hosts.borrow_mut();
        let index = target_host_index(&mut hosts, theme);
        hosts[index].host.document_node_hits(root, area)
    })
}

fn target_host_index(hosts: &mut Vec<ThemeTargetHostCache>, theme: &ThemeSnapshot) -> usize {
    if let Some(index) = hosts.iter().position(|cached| cached.theme.eq(theme)) {
        return index;
    }
    hosts.push(ThemeTargetHostCache {
        theme: theme.clone(),
        host: UiTreeSurfaceHost::new(theme.clone()),
    });
    hosts.len() - 1
}

struct ThemeTargetHostCache {
    theme: ThemeSnapshot,
    host: UiTreeSurfaceHost,
}

fn rendered_node_rects(hits: &[UiTreeNodeHit]) -> BTreeMap<String, ViewerRect> {
    hits.iter()
        .map(|hit| (hit.node_id.as_str().to_string(), viewer_rect(hit.rect)))
        .collect()
}

fn rendered_semantic_rects(hits: &[UiTreeNodeHit]) -> BTreeMap<String, ViewerRect> {
    let mut rects = BTreeMap::<String, ViewerRect>::new();
    for hit in hits {
        let Some(semantic_node_id) = &hit.semantic_node_id else {
            continue;
        };
        let rect = viewer_rect(hit.rect);
        rects
            .entry(semantic_node_id.as_str().to_string())
            .and_modify(|current| *current = union_rect(*current, rect))
            .or_insert(rect);
    }
    rects
}

fn append_semantic_alias_targets(targets: &mut Vec<ViewerTarget>, hits: &[UiTreeNodeHit]) {
    let mut by_node_id = targets
        .iter()
        .cloned()
        .map(|target| (target.node_id.0.clone(), target))
        .collect::<BTreeMap<_, _>>();
    for hit in hits {
        let Some(semantic_node_id) = &hit.semantic_node_id else {
            continue;
        };
        let semantic_node_id = semantic_node_id.as_str();
        let hit_node_id = hit.node_id.as_str();
        if semantic_node_id == hit_node_id || by_node_id.contains_key(hit_node_id) {
            continue;
        }
        let Some(source_target) = by_node_id.get(semantic_node_id) else {
            continue;
        };
        let mut target = source_target.clone();
        target.node_id.0 = hit_node_id.to_string();
        target.rect = viewer_rect(hit.rect);
        by_node_id.insert(hit_node_id.to_string(), target.clone());
        targets.push(target);
    }
}

fn viewer_rect(rect: UiTreeHitRect) -> ViewerRect {
    ViewerRect {
        x: rect.x as f32,
        y: rect.y as f32,
        width: rect.width as f32,
        height: rect.height as f32,
    }
}

fn union_rect(left: ViewerRect, right: ViewerRect) -> ViewerRect {
    let min_x = left.x.min(right.x);
    let min_y = left.y.min(right.y);
    let max_x = (left.x + left.width).max(right.x + right.width);
    let max_y = (left.y + left.height).max(right.y + right.height);
    ViewerRect {
        x: min_x,
        y: min_y,
        width: max_x - min_x,
        height: max_y - min_y,
    }
}

pub fn viewer_target_lookup(targets: &[ViewerTarget]) -> BTreeMap<String, usize> {
    targets
        .iter()
        .enumerate()
        .map(|(index, target)| (target.node_id.0.clone(), index))
        .collect()
}

pub fn viewer_internal_anchor_lookup(
    plan: &ViewerNodePlan,
    targets: &[ViewerTarget],
) -> BTreeMap<String, usize> {
    let target_indexes = targets
        .iter()
        .enumerate()
        .map(|(index, target)| (target.node_id.0.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let mut lookup = BTreeMap::new();
    for node in &plan.nodes {
        let Some(index) = target_indexes.get(node.node_id.0.as_str()).copied() else {
            continue;
        };
        if let ViewerNodeKind::FootnoteDefinition { label } = &node.kind {
            lookup.insert(format!("fn-{label}"), index);
        }
        for span in &node.spans {
            if let Some(label) = span.link_target.strip_prefix("#fn-") {
                lookup.entry(format!("fnref-{label}")).or_insert(index);
            }
        }
    }
    lookup
}

#[cfg(test)]
mod tests {
    use super::scroll_redraw_band_y_for_diagram_boundaries;
    use katana_document_viewer::ViewerRect;

    #[test]
    fn diagram_boundary_extends_redraw_band_at_boundary_offsets() {
        let current_scroll_y = 8_343.0;
        let viewport_height = 600;
        let default_band_y = 520;
        for offset in [-1.0, 0.0, 1.0] {
            let boundary = ViewerRect {
                x: 0.0,
                y: current_scroll_y + default_band_y as f32 - 58.0 + offset,
                width: 240.0,
                height: 178.0,
            };
            assert_eq!(
                (default_band_y as f32 - 58.0 + offset).floor() as usize,
                scroll_redraw_band_y_for_diagram_boundaries(
                    &[boundary],
                    current_scroll_y,
                    viewport_height,
                    default_band_y,
                )
            );
        }
    }

    #[test]
    fn diagram_boundary_outside_redraw_band_keeps_default_start() {
        let boundary = ViewerRect {
            x: 0.0,
            y: 8_343.0 + 640.0,
            width: 240.0,
            height: 120.0,
        };
        assert_eq!(
            520,
            scroll_redraw_band_y_for_diagram_boundaries(&[boundary], 8_343.0, 600, 520)
        );
    }
}
