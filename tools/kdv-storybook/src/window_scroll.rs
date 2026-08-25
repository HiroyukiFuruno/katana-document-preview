use super::StorybookWindow;
use crate::layout::{
    SIDEBAR_WIDTH, preview_content_width, preview_viewport_height, sidebar_content_contains,
    sidebar_content_height, sidebar_content_local_y,
};
use crate::scroll::StorybookScroll;
use crate::sidebar::{StorybookSidebar, StorybookSidebarBoundsRequest, StorybookSidebarPaneBounds};
use katana_document_viewer::ViewerVector;
use minifb::{Key, Window};

impl StorybookWindow {
    pub(super) fn apply_scroll(&mut self, window: &Window) -> bool {
        let Some((delta_x, delta_y)) = window.get_scroll_wheel() else {
            return false;
        };
        if delta_x == 0.0 && delta_y == 0.0 {
            return false;
        }
        let (_, height) = self.current_canvas_size(window);
        if let Some((pointer_x, pointer_y)) = self.current_canvas_mouse_position(window) {
            if sidebar_content_contains(pointer_x, pointer_y, height) {
                return self.apply_sidebar_scroll(delta_y, pointer_y, height);
            }
            if pointer_x.is_finite() && pointer_x < SIDEBAR_WIDTH as f32 {
                return false;
            }
        }
        if fullscreen_zoom_modifier_down(window)
            && self.apply_fullscreen_diagram_trackpad_zoom(trackpad_zoom_multiplier(delta_y))
        {
            return true;
        }
        if self.apply_fullscreen_diagram_smooth_scroll(ViewerVector {
            x: -delta_x * 48.0,
            y: -delta_y * 48.0,
        }) {
            return true;
        }
        if fullscreen_zoom_modifier_down(window)
            && self.apply_document_diagram_trackpad_zoom_at(
                self.current_canvas_mouse_position(window),
                self.current_canvas_size(window).0,
                height,
                trackpad_zoom_multiplier(delta_y),
            )
        {
            return true;
        }
        self.apply_preview_scroll(delta_y, height)
    }

    pub(super) fn apply_preview_scroll(&mut self, delta_y: f32, height: usize) -> bool {
        let content_height = self
            .scene
            .as_ref()
            .map_or(0.0, |scene| scene.content_height);
        let viewport_height = preview_viewport_height(height) as f32;
        let max_scroll = StorybookScroll::max_offset(content_height, viewport_height);
        let requested_delta =
            StorybookScroll::wheel_delta_pixels(delta_y) + self.preview_scroll_pixel_residual;
        if requested_delta.abs() < 1.0 {
            self.preview_scroll_pixel_residual = requested_delta;
            return false;
        }
        let pixel_delta = requested_delta.trunc();
        let next = (self.scroll_y + pixel_delta).clamp(0.0, max_scroll);
        let actual_delta = next - self.scroll_y;
        if actual_delta.abs() <= f32::EPSILON {
            self.preview_scroll_pixel_residual = 0.0;
            return false;
        }
        self.scroll_y = next;
        self.preview_scroll_pixel_residual = requested_delta - actual_delta;
        true
    }

    pub(super) fn apply_sidebar_scroll(
        &mut self,
        delta_y: f32,
        pointer_y: f32,
        height: usize,
    ) -> bool {
        let Some(local_y) = sidebar_content_local_y(pointer_y, height) else {
            return false;
        };
        let sidebar_height = sidebar_content_height(height);
        let (tree, settings) = self.sidebar_scroll_bounds(sidebar_height);
        if (local_y as f32) < tree.viewport_height {
            let (value, changed) = StorybookScroll::apply(
                tree.offset_y,
                delta_y,
                tree.content_height,
                tree.viewport_height,
            );
            self.sidebar_scroll.tree_y = value.round() as u32;
            if changed {
                self.invalidate_sidebar_scroll_caches();
            }
            return changed;
        }
        let (value, changed) = StorybookScroll::apply(
            settings.offset_y,
            delta_y,
            settings.content_height,
            settings.viewport_height,
        );
        self.sidebar_scroll.settings_y = value.round() as u32;
        if changed {
            self.invalidate_sidebar_scroll_caches();
        }
        changed
    }

    fn invalidate_sidebar_scroll_caches(&mut self) {
        self.frame_cache = None;
        self.sidebar_interaction_cache = None;
        self.sidebar_interaction_surface_cache = None;
    }

    fn sidebar_scroll_bounds(
        &self,
        sidebar_height: usize,
    ) -> (StorybookSidebarPaneBounds, StorybookSidebarPaneBounds) {
        StorybookSidebar::scroll_bounds(StorybookSidebarBoundsRequest {
            fixtures: &self.catalog.fixtures,
            selected_index: self.selected_index,
            scene: self.scene.as_ref(),
            dark: self.dark,
            interaction: &self.interaction,
            typography: self.typography,
            file_tree_state: &self.file_tree_state,
            settings_state: &self.settings_state,
            height: sidebar_height,
            preview_width: preview_content_width(self.frame_size.map_or(SIDEBAR_WIDTH, |it| it.0)),
            preview_height: preview_viewport_height(sidebar_height),
            scroll: self.sidebar_scroll,
        })
    }
}

fn fullscreen_zoom_modifier_down(window: &Window) -> bool {
    window.is_key_down(Key::LeftCtrl)
        || window.is_key_down(Key::RightCtrl)
        || window.is_key_down(Key::LeftSuper)
        || window.is_key_down(Key::RightSuper)
}

fn trackpad_zoom_multiplier(delta_y: f32) -> f32 {
    if delta_y > 0.0 { 1.1 } else { 0.9 }
}
