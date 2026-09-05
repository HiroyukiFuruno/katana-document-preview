use crate::canvas::{Canvas, CanvasBlitRequest, SurfaceArea};
use crate::catalog::StorybookFixture;
use crate::frame_status::StorybookFrameStatus;
use crate::frame_ui_surface::{
    render_ui_tree_for_theme_flag, render_ui_tree_with_theme, sidebar_area,
};
use crate::layout::{
    HEADER_HEIGHT, SIDEBAR_WIDTH, StorybookPreviewArea, preview_content_width, preview_status_y,
    preview_viewport_height, sidebar_content_height, sidebar_content_width,
};
use crate::mouse::task_context_menu::StorybookTaskContextMenu;
use crate::palette::StorybookPalette;
use crate::preview::PreviewScene;
use crate::sidebar::{StorybookSidebar, StorybookSidebarRequest, StorybookSidebarScroll};
use crate::sidebar_settings_state::StorybookSettingsState;
use katana_document_viewer::{ViewerInteractionConfig, ViewerTypographyConfig};
use katana_ui_core::molecule::FileTreeState;
use katana_ui_core::render_model::{UiNode, UiNodeId, UiNodeKind};
use katana_ui_core::theme::ThemeSnapshot;
#[cfg(test)]
use std::path::PathBuf;
#[cfg(test)]
use std::sync::{Mutex, OnceLock};

#[cfg(test)]
const MAX_TEST_SIDEBAR_CACHE_ENTRIES: usize = 8;
const SCROLL_REDRAW_OVERSCAN: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreviewScrollRedraw {
    pub area_x: usize,
    pub area_y: usize,
    pub area_width: usize,
    pub content_height: usize,
    pub logical_delta_y: isize,
    pub band_y: usize,
    pub band_height: usize,
}

pub struct FrameRenderRequest<'a> {
    pub width: usize,
    pub height: usize,
    pub fixtures: &'a [StorybookFixture],
    pub selected_index: usize,
    pub scene: Option<&'a PreviewScene>,
    pub scroll_y: f32,
    pub sidebar_scroll: StorybookSidebarScroll,
    pub file_tree_state: FileTreeState,
    pub settings_state: &'a StorybookSettingsState,
    pub dark: bool,
    pub interaction: &'a ViewerInteractionConfig,
    pub typography: ViewerTypographyConfig,
    pub last_command_label: &'a str,
    pub task_context_menu: Option<&'a StorybookTaskContextMenu>,
    pub hovered_node_id: Option<&'a str>,
    pub hovered_action_node_id: Option<&'a UiNodeId>,
    pub animation_phase: u16,
}

pub struct StorybookFrameRenderer;

impl StorybookFrameRenderer {
    pub fn prewarm() {
        static PREWARMED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
        PREWARMED.get_or_init(|| {
            let fixtures = [StorybookFixture {
                label: "prewarm.md".to_string(),
                path: std::path::PathBuf::new(),
            }];
            let settings_state = StorybookSettingsState::default();
            let interaction = ViewerInteractionConfig::default();
            let request = FrameRenderRequest {
                width: 320,
                height: 180,
                fixtures: &fixtures,
                selected_index: 0,
                scene: None,
                scroll_y: 0.0,
                sidebar_scroll: StorybookSidebarScroll::default(),
                file_tree_state: FileTreeState::default(),
                settings_state: &settings_state,
                dark: true,
                interaction: &interaction,
                typography: ViewerTypographyConfig::default(),
                last_command_label: "none",
                task_context_menu: None,
                hovered_node_id: None,
                hovered_action_node_id: None,
                animation_phase: 0,
            };
            let sidebar = Self::render_sidebar(&request);
            let _ = Self::render_with_sidebar(request, &sidebar);
        });
    }

    pub fn prewarm_theme(theme: &ThemeSnapshot) {
        let mut canvas = Canvas::new(1, 1, 0);
        let root = UiNode::new(UiNodeKind::Stack, "theme-prewarm");
        render_ui_tree_with_theme(
            &mut canvas,
            &root,
            SurfaceArea {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
                scroll_y: 0.0,
            },
            theme,
        );
    }

    #[cfg(test)]
    pub fn render(request: FrameRenderRequest<'_>) -> Canvas {
        let sidebar = cached_test_sidebar(&request);
        Self::render_with_sidebar(request, &sidebar)
    }

    pub fn render_sidebar(request: &FrameRenderRequest<'_>) -> Canvas {
        Self::render_sidebar_scaled(request, 1.0)
    }

    pub fn render_sidebar_scaled(request: &FrameRenderRequest<'_>, scale: f32) -> Canvas {
        let palette = StorybookPalette::new(request.dark);
        let mut canvas = Canvas::new_scaled(
            SIDEBAR_WIDTH,
            request.height,
            scale,
            palette.sidebar_background(),
        );
        Self::draw_sidebar(&mut canvas, request, palette);
        canvas
    }

    pub fn render_with_sidebar(request: FrameRenderRequest<'_>, sidebar: &Canvas) -> Canvas {
        let palette = StorybookPalette::new(request.dark);
        let mut canvas = Canvas::new(request.width, request.height, palette.background());
        canvas.blit_canvas(
            sidebar,
            CanvasBlitRequest {
                dest_x: 0,
                dest_y: 0,
                width: sidebar.width(),
                height: sidebar.height(),
                source_y: 0,
            },
        );
        Self::draw_header(&mut canvas, &request, palette);
        Self::draw_preview(&mut canvas, &request, palette);
        Self::draw_context_menu(&mut canvas, &request);
        canvas
    }

    #[cfg(test)]
    pub fn render_scaled(request: FrameRenderRequest<'_>, scale: f32) -> Canvas {
        let sidebar = Self::render_sidebar_scaled(&request, scale);
        Self::render_scaled_with_sidebar(request, &sidebar, scale)
    }

    pub fn render_scaled_with_sidebar(
        request: FrameRenderRequest<'_>,
        sidebar: &Canvas,
        scale: f32,
    ) -> Canvas {
        if scale <= 1.0 {
            return Self::render_with_sidebar(request, sidebar);
        }
        let palette = StorybookPalette::new(request.dark);
        let mut canvas =
            Canvas::new_scaled(request.width, request.height, scale, palette.background());
        canvas.blit_canvas(
            sidebar,
            CanvasBlitRequest {
                dest_x: 0,
                dest_y: 0,
                width: sidebar.width(),
                height: sidebar.height(),
                source_y: 0,
            },
        );
        Self::draw_header(&mut canvas, &request, palette);
        Self::draw_preview_direct(&mut canvas, &request, palette);
        Self::draw_context_menu(&mut canvas, &request);
        canvas
    }

    pub fn redraw_preview(canvas: &mut Canvas, request: &FrameRenderRequest<'_>) {
        let palette = StorybookPalette::new(request.dark);
        Self::draw_preview_direct(canvas, request, palette);
    }

    #[cfg(test)]
    pub fn redraw_preview_scroll_delta(
        canvas: &mut Canvas,
        request: &FrameRenderRequest<'_>,
        previous_scroll_y: f32,
    ) -> bool {
        Self::redraw_preview_scroll_delta_with_result(canvas, request, previous_scroll_y).is_some()
    }

    pub fn redraw_preview_scroll_delta_with_result(
        canvas: &mut Canvas,
        request: &FrameRenderRequest<'_>,
        previous_scroll_y: f32,
    ) -> Option<PreviewScrollRedraw> {
        Self::redraw_preview_scroll_delta_with_mode(canvas, request, previous_scroll_y, true)
    }

    fn redraw_preview_scroll_delta_with_mode(
        canvas: &mut Canvas,
        request: &FrameRenderRequest<'_>,
        previous_scroll_y: f32,
        scroll_source_canvas: bool,
    ) -> Option<PreviewScrollRedraw> {
        if request.hovered_node_id.is_some()
            || request.hovered_action_node_id.is_some()
            || request.task_context_menu.is_some()
        {
            return None;
        }
        let scene = request.scene?;
        if scene.fullscreen_diagram_active() {
            return None;
        }
        if scene.mode != katana_document_viewer::ViewerMode::Document {
            return None;
        }
        let area = preview_area(request);
        let content_height = area.height;
        let previous = previous_scroll_y.round().max(0.0) as isize;
        let current = request.scroll_y.round().max(0.0) as isize;
        let delta = current.saturating_sub(previous);
        if delta == 0 {
            return Some(PreviewScrollRedraw {
                area_x: area.x,
                area_y: area.y,
                area_width: area.width,
                content_height,
                logical_delta_y: 0,
                band_y: 0,
                band_height: 0,
            });
        }
        if scene.needs_full_preview_redraw_for_scroll(
            previous_scroll_y,
            request.scroll_y,
            content_height,
        ) {
            return None;
        }
        if delta < 0 {
            return None;
        }
        let absolute_delta = delta.unsigned_abs();
        if absolute_delta == 0 || absolute_delta >= content_height {
            return None;
        }
        if scroll_source_canvas
            && !canvas.scroll_rect_vertically(area.x, area.y, area.width, content_height, -delta)
        {
            return None;
        }
        let palette = StorybookPalette::new(request.dark);
        let (band_y, band_height, band_scroll_y) = if delta > 0 {
            let default_band_y =
                content_height.saturating_sub(absolute_delta + SCROLL_REDRAW_OVERSCAN);
            let band_y = scene.scroll_redraw_band_y_for_downward_scroll(
                request.scroll_y,
                content_height,
                default_band_y,
            );
            (
                band_y,
                content_height.saturating_sub(band_y),
                request.scroll_y + band_y as f32,
            )
        } else {
            (0, absolute_delta, request.scroll_y)
        };
        canvas.fill_rect(
            area.x,
            area.y.saturating_add(band_y),
            area.width,
            band_height,
            palette.preview_background(),
        );
        Self::render_preview_content_direct(
            canvas,
            scene,
            SurfaceArea {
                x: area.x,
                y: area.y.saturating_add(band_y),
                width: area.width,
                height: area.height,
                scroll_y: band_scroll_y,
            },
            None,
            None,
            0,
        );
        Self::restore_below_preview_after_scroll_delta(canvas, request, scene, area, palette);
        Some(PreviewScrollRedraw {
            area_x: area.x,
            area_y: area.y,
            area_width: area.width,
            content_height,
            logical_delta_y: -delta,
            band_y,
            band_height,
        })
    }

    fn restore_below_preview_after_scroll_delta(
        canvas: &mut Canvas,
        request: &FrameRenderRequest<'_>,
        scene: &PreviewScene,
        area: SurfaceArea,
        palette: StorybookPalette,
    ) {
        let below_preview_y = area.y.saturating_add(area.height);
        if below_preview_y < request.height {
            canvas.fill_rect(
                area.x,
                below_preview_y,
                area.width,
                request.height.saturating_sub(below_preview_y),
                palette.background(),
            );
        }
        StorybookFrameStatus::draw(
            canvas,
            scene,
            area.x,
            preview_status_y(request.height),
            palette,
            request.last_command_label,
        );
    }

    fn draw_sidebar(
        canvas: &mut Canvas,
        request: &FrameRenderRequest<'_>,
        palette: StorybookPalette,
    ) {
        canvas.fill_rect(
            0,
            0,
            SIDEBAR_WIDTH,
            request.height,
            palette.sidebar_background(),
        );
        let tree = StorybookSidebar::render(StorybookSidebarRequest {
            fixtures: request.fixtures,
            selected_index: request.selected_index,
            scene: request.scene,
            dark: request.dark,
            interaction: request.interaction,
            typography: request.typography,
            file_tree_state: request.file_tree_state.clone(),
            settings_state: request.settings_state,
            width: sidebar_content_width(),
            height: sidebar_content_height(request.height),
            preview_width: preview_content_width(request.width),
            preview_height: preview_viewport_height(request.height),
            scroll: request.sidebar_scroll,
        });
        render_ui_tree_for_theme_flag(canvas, tree.root(), sidebar_area(request), request.dark);
    }

    fn draw_header(
        canvas: &mut Canvas,
        request: &FrameRenderRequest<'_>,
        palette: StorybookPalette,
    ) {
        canvas.fill_rect(
            SIDEBAR_WIDTH,
            0,
            request.width.saturating_sub(SIDEBAR_WIDTH),
            HEADER_HEIGHT,
            palette.header(),
        );
        let fixture = &request.fixtures[request.selected_index];
        let header_label = Self::fit_header_label(
            Self::header_document_label(fixture, request.scene),
            preview_content_width(request.width),
        );
        canvas.draw_text(SIDEBAR_WIDTH + 16, 14, &header_label, palette.text());
        if preview_content_width(request.width) < 420 {
            return;
        }
        let theme = if request.dark { "dark" } else { "light" };
        canvas.draw_text(
            request.width.saturating_sub(240),
            14,
            &format!(
                "{theme} {}",
                StorybookFrameStatus::viewer_mode_label(request.scene)
            ),
            palette.text(),
        );
    }

    fn fit_header_label(label: String, available_width: usize) -> String {
        const APPROX_HEADER_CHAR_WIDTH: usize = 8;
        const HEADER_SIDE_PADDING: usize = 32;
        let max_chars = available_width
            .saturating_sub(HEADER_SIDE_PADDING)
            .checked_div(APPROX_HEADER_CHAR_WIDTH)
            .unwrap_or(0);
        if max_chars == 0 || label.chars().count() <= max_chars {
            return label;
        }
        let keep = max_chars.saturating_sub(1).max(1);
        let mut truncated = label.chars().take(keep).collect::<String>();
        truncated.push('…');
        truncated
    }

    fn header_document_label(fixture: &StorybookFixture, scene: Option<&PreviewScene>) -> String {
        if !fixture.label.trim().is_empty() {
            return fixture.label.clone();
        }
        scene
            .map(|scene| scene.document_id.clone())
            .unwrap_or_else(|| "document".to_string())
    }

    fn draw_preview(
        canvas: &mut Canvas,
        request: &FrameRenderRequest<'_>,
        palette: StorybookPalette,
    ) {
        if let Some(scene) = request.scene
            && scene.fullscreen_diagram_active()
        {
            let area = fullscreen_area(request);
            canvas.fill_rect(
                0,
                0,
                request.width,
                request.height,
                palette.preview_background(),
            );
            Self::render_preview_content_direct(
                canvas,
                scene,
                area,
                request.hovered_node_id,
                request.hovered_action_node_id,
                request.animation_phase,
            );
            return;
        }
        let area = preview_area(request);
        canvas.fill_rect(
            area.x,
            area.y,
            area.width,
            area.height,
            palette.preview_background(),
        );
        if let Some(scene) = request.scene {
            Self::render_preview_content_direct(
                canvas,
                scene,
                area,
                request.hovered_node_id,
                request.hovered_action_node_id,
                request.animation_phase,
            );
            StorybookFrameStatus::draw(
                canvas,
                scene,
                area.x,
                preview_status_y(request.height),
                palette,
                request.last_command_label,
            );
        }
    }

    fn draw_preview_direct(
        canvas: &mut Canvas,
        request: &FrameRenderRequest<'_>,
        palette: StorybookPalette,
    ) {
        if let Some(scene) = request.scene
            && scene.fullscreen_diagram_active()
        {
            let area = fullscreen_area(request);
            canvas.fill_rect(
                0,
                0,
                request.width,
                request.height,
                palette.preview_background(),
            );
            Self::render_preview_content_direct(
                canvas,
                scene,
                area,
                request.hovered_node_id,
                request.hovered_action_node_id,
                request.animation_phase,
            );
            return;
        }
        let area = preview_area(request);
        canvas.fill_rect(
            area.x,
            area.y,
            area.width,
            area.height,
            palette.preview_background(),
        );
        if let Some(scene) = request.scene {
            Self::render_preview_content_direct(
                canvas,
                scene,
                area,
                request.hovered_node_id,
                request.hovered_action_node_id,
                request.animation_phase,
            );
            StorybookFrameStatus::draw(
                canvas,
                scene,
                area.x,
                preview_status_y(request.height),
                palette,
                request.last_command_label,
            );
        }
    }

    fn render_preview_content_direct(
        canvas: &mut Canvas,
        scene: &PreviewScene,
        area: SurfaceArea,
        hovered_node_id: Option<&str>,
        hovered_action_node_id: Option<&UiNodeId>,
        animation_phase: u16,
    ) {
        let slideshow_scroll_y = Self::slideshow_scroll_y(scene, area.scroll_y);
        let hovered_node_id = hovered_node_id.map(UiNodeId::from);
        let staged_tree = if hovered_node_id.is_some()
            || hovered_action_node_id.is_some()
            || animation_phase != 0
            || slideshow_scroll_y > 0
        {
            let hover_surface_tree = scene
                .tree
                .with_hover_surface_for_node_id(hovered_node_id.as_ref());
            let hovered_tree = hover_surface_tree.with_hovered_node_id(hovered_action_node_id);
            let animated_tree = hovered_tree.with_animation_phase(animation_phase);
            Some(if slideshow_scroll_y > 0 {
                animated_tree.with_scroll_area_offset_y(slideshow_scroll_y)
            } else {
                animated_tree
            })
        } else {
            None
        };
        let render_root = staged_tree
            .as_ref()
            .map_or_else(|| scene.tree.root(), |tree| tree.root());
        render_ui_tree_with_theme(
            canvas,
            render_root,
            SurfaceArea {
                x: area.x,
                y: area.y,
                width: area.width,
                height: area.height,
                scroll_y: Self::render_scroll_delta(scene, area.scroll_y),
            },
            &scene.theme,
        );
    }

    fn render_scroll_delta(scene: &PreviewScene, requested_scroll_y: f32) -> f32 {
        if scene.mode == katana_document_viewer::ViewerMode::Slideshow {
            return 0.0;
        }
        Self::remaining_scroll_delta(scene, requested_scroll_y)
    }

    fn slideshow_scroll_y(scene: &PreviewScene, requested_scroll_y: f32) -> u32 {
        if scene.mode != katana_document_viewer::ViewerMode::Slideshow {
            return 0;
        }
        requested_scroll_y.round().max(0.0) as u32
    }

    fn remaining_scroll_delta(scene: &PreviewScene, requested_scroll_y: f32) -> f32 {
        let tree_offset = scene.tree.root().props().scroll_area.offset_y as f32;
        (requested_scroll_y - tree_offset).max(0.0)
    }

    fn draw_context_menu(canvas: &mut Canvas, request: &FrameRenderRequest<'_>) {
        if let Some(menu) = request.task_context_menu {
            render_ui_tree_for_theme_flag(
                canvas,
                menu.node(),
                SurfaceArea {
                    x: 0,
                    y: 0,
                    width: request.width,
                    height: request.height,
                    scroll_y: 0.0,
                },
                request.dark,
            );
        }
    }
}

#[cfg(test)]
fn cached_test_sidebar(request: &FrameRenderRequest<'_>) -> Canvas {
    let key = FrameSidebarCacheKey::from_request(request);
    let cache = TEST_SIDEBAR_CACHE.get_or_init(|| Mutex::new(Vec::new()));
    let Ok(mut entries) = cache.lock() else {
        return StorybookFrameRenderer::render_sidebar(request);
    };
    if let Some(entry) = entries.iter().find(|entry| entry.key == key) {
        return entry.canvas.clone();
    }
    let canvas = StorybookFrameRenderer::render_sidebar(request);
    entries.retain(|entry| entry.key != key);
    entries.push(FrameSidebarCacheEntry {
        key,
        canvas: canvas.clone(),
    });
    while entries.len() > MAX_TEST_SIDEBAR_CACHE_ENTRIES {
        entries.remove(0);
    }
    canvas
}

#[cfg(test)]
static TEST_SIDEBAR_CACHE: OnceLock<Mutex<Vec<FrameSidebarCacheEntry>>> = OnceLock::new();

#[cfg(test)]
struct FrameSidebarCacheEntry {
    key: FrameSidebarCacheKey,
    canvas: Canvas,
}

#[cfg(test)]
#[derive(Clone, PartialEq, Eq)]
struct FrameSidebarCacheKey {
    width: usize,
    height: usize,
    fixtures: Vec<FrameSidebarFixtureKey>,
    selected_index: usize,
    sidebar_scroll: StorybookSidebarScroll,
    file_tree_state: FileTreeState,
    settings_state: StorybookSettingsState,
    dark: bool,
    interaction: ViewerInteractionConfig,
    typography: ViewerTypographyConfig,
    scene: Option<FrameSidebarSceneKey>,
}

#[cfg(test)]
impl FrameSidebarCacheKey {
    fn from_request(request: &FrameRenderRequest<'_>) -> Self {
        Self {
            width: request.width,
            height: request.height,
            fixtures: request
                .fixtures
                .iter()
                .map(FrameSidebarFixtureKey::from)
                .collect(),
            selected_index: request.selected_index,
            sidebar_scroll: request.sidebar_scroll,
            file_tree_state: request.file_tree_state.clone(),
            settings_state: request.settings_state.clone(),
            dark: request.dark,
            interaction: request.interaction.clone(),
            typography: request.typography,
            scene: request.scene.map(FrameSidebarSceneKey::from),
        }
    }
}

#[cfg(test)]
#[derive(Clone, PartialEq, Eq)]
struct FrameSidebarFixtureKey {
    label: String,
    path: PathBuf,
}

#[cfg(test)]
impl From<&StorybookFixture> for FrameSidebarFixtureKey {
    fn from(fixture: &StorybookFixture) -> Self {
        Self {
            label: fixture.label.clone(),
            path: fixture.path.clone(),
        }
    }
}

#[cfg(test)]
#[derive(Clone, PartialEq, Eq)]
struct FrameSidebarSceneKey {
    mode: katana_document_viewer::ViewerMode,
    slide_current: usize,
    slide_max: usize,
    node_count: usize,
    loaded_assets: usize,
    failed_assets: usize,
    image_surfaces: usize,
    surface: Option<FrameSidebarSurfaceKey>,
    typography: ViewerTypographyConfig,
}

#[cfg(test)]
impl From<&PreviewScene> for FrameSidebarSceneKey {
    fn from(scene: &PreviewScene) -> Self {
        Self {
            mode: scene.mode.clone(),
            slide_current: scene.slideshow_current_page,
            slide_max: scene.slideshow_max_page,
            node_count: scene.node_count,
            loaded_assets: scene.loaded_asset_count,
            failed_assets: scene.failed_asset_count,
            image_surfaces: scene.image_surface_count,
            surface: scene.surface.as_ref().map(FrameSidebarSurfaceKey::from),
            typography: scene.typography,
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq)]
struct FrameSidebarSurfaceKey {
    width: u32,
    height: u32,
}

#[cfg(test)]
impl From<&katana_document_viewer::PreviewSurfaceImage> for FrameSidebarSurfaceKey {
    fn from(surface: &katana_document_viewer::PreviewSurfaceImage) -> Self {
        Self {
            width: surface.width,
            height: surface.height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn sidebar_cache_key_ignores_preview_only_frame_state() {
        let fixtures = fixtures();
        let settings = StorybookSettingsState::default();
        let interaction = ViewerInteractionConfig::default();
        let first = request(&fixtures, &settings, &interaction, 0, 0.0, "none");
        let second = request(&fixtures, &settings, &interaction, 0, 240.0, "copied");

        assert!(
            FrameSidebarCacheKey::from_request(&first)
                == FrameSidebarCacheKey::from_request(&second)
        );
    }

    #[test]
    fn sidebar_cache_key_keeps_selected_fixture_identity() {
        let fixtures = fixtures();
        let settings = StorybookSettingsState::default();
        let interaction = ViewerInteractionConfig::default();
        let first = request(&fixtures, &settings, &interaction, 0, 0.0, "none");
        let second = request(&fixtures, &settings, &interaction, 1, 0.0, "none");

        assert!(
            FrameSidebarCacheKey::from_request(&first)
                != FrameSidebarCacheKey::from_request(&second)
        );
    }

    fn request<'a>(
        fixtures: &'a [StorybookFixture],
        settings: &'a StorybookSettingsState,
        interaction: &'a ViewerInteractionConfig,
        selected_index: usize,
        scroll_y: f32,
        last_command_label: &'a str,
    ) -> FrameRenderRequest<'a> {
        FrameRenderRequest {
            width: SIDEBAR_WIDTH + 640,
            height: 720,
            fixtures,
            selected_index,
            scene: None,
            scroll_y,
            sidebar_scroll: StorybookSidebarScroll::default(),
            file_tree_state: FileTreeState::default(),
            settings_state: settings,
            dark: false,
            interaction,
            typography: ViewerTypographyConfig::default(),
            last_command_label,
            task_context_menu: None,
            hovered_node_id: None,
            hovered_action_node_id: None,
            animation_phase: 0,
        }
    }

    fn fixtures() -> Vec<StorybookFixture> {
        vec![
            StorybookFixture {
                label: "first.md".to_string(),
                path: PathBuf::from("first.md"),
            },
            StorybookFixture {
                label: "second.md".to_string(),
                path: PathBuf::from("second.md"),
            },
        ]
    }
}

fn preview_area(request: &FrameRenderRequest<'_>) -> SurfaceArea {
    let area = StorybookPreviewArea::for_window(request.width, request.height, request.scroll_y);
    SurfaceArea {
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height,
        scroll_y: area.scroll_y,
    }
}

fn fullscreen_area(request: &FrameRenderRequest<'_>) -> SurfaceArea {
    SurfaceArea {
        x: 0,
        y: 0,
        width: request.width,
        height: request.height,
        scroll_y: 0.0,
    }
}

#[cfg(test)]
mod header_tests {
    use super::StorybookFrameRenderer;

    #[test]
    fn narrow_header_label_is_truncated_inside_preview_width() {
        let label =
            StorybookFrameRenderer::fit_header_label("katana/sample_basic.md".to_string(), 284);

        assert_eq!("katana/sample_basic.md", label);

        let long = StorybookFrameRenderer::fit_header_label(
            "katana/very/deep/path/with/a/long/document-name.md".to_string(),
            284,
        );

        assert!(long.ends_with('…'));
        assert!(long.chars().count() <= 31);
    }
}

#[cfg(test)]
#[path = "frame_test_modules.rs"]
mod test_modules;
