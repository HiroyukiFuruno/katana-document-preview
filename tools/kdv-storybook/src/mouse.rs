use crate::layout::StorybookPreviewArea;
use crate::preview::PreviewScene;
use katana_document_viewer::{ViewerCommand, ViewerCommandFactory, ViewerMode};
use katana_ui_core::render_model::UiTextSpanAction;
pub(crate) use mouse_accordion::StorybookMouseAccordion;
pub(crate) use mouse_cursor::{StorybookHoverState, StorybookMouseCursor};
pub(crate) use mouse_document_point::DocumentPoint;
pub(crate) use mouse_host_action::{StorybookHostActionHits, StorybookHostActionRouter};
use mouse_media::StorybookMediaMouse;
use mouse_task::StorybookTaskMouse;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StorybookMouseButton {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StorybookPointer {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) button: StorybookMouseButton,
}

#[derive(Debug, Default)]
pub(crate) struct StorybookMouseState {
    left_down: bool,
    right_down: bool,
}

pub(crate) struct StorybookMouse;

impl StorybookPointer {
    pub(crate) fn new(x: f32, y: f32, button: StorybookMouseButton) -> Self {
        Self { x, y, button }
    }
}

impl StorybookMouseState {
    pub(crate) fn pressed(
        &mut self,
        left_down: bool,
        right_down: bool,
    ) -> Option<StorybookMouseButton> {
        let pressed = if left_down && !self.left_down {
            Some(StorybookMouseButton::Left)
        } else if right_down && !self.right_down {
            Some(StorybookMouseButton::Right)
        } else {
            None
        };
        self.left_down = left_down;
        self.right_down = right_down;
        pressed
    }
}

impl StorybookMouse {
    pub(crate) fn command_for_click(
        scene: &PreviewScene,
        scroll_y: f32,
        pointer: StorybookPointer,
        window_width: usize,
        window_height: usize,
    ) -> Option<ViewerCommand> {
        if pointer.button == StorybookMouseButton::Left
            && let Some(command) = Self::slideshow_viewport_command(
                scene,
                scroll_y,
                pointer,
                window_width,
                window_height,
            )
        {
            return Some(command);
        }
        let point = DocumentPoint::from_scene_pointer(
            scene,
            pointer,
            scroll_y,
            window_width,
            window_height,
        )?;
        let router = if scene.fullscreen_diagram_active() || scene.mode == ViewerMode::Document {
            StorybookHostActionRouter::for_window_with_scroll(
                scene,
                window_width,
                window_height,
                scroll_y,
            )
        } else {
            StorybookHostActionRouter::for_window(scene, window_width)
        };
        if let Some(command) = StorybookTaskMouse::command(scene, point, pointer.button, &router) {
            return Some(command);
        }
        if pointer.button != StorybookMouseButton::Left {
            return None;
        }
        if let Some(command) = Self::slideshow_command(point, &router) {
            return Some(command);
        }
        if let Some(command) = StorybookMediaMouse::command(scene, point, &router) {
            return Some(command);
        }
        Self::link_command(point, &router)
    }

    pub(crate) fn command_label(command: &ViewerCommand) -> &'static str {
        match command {
            ViewerCommand::Link(_) => "link",
            ViewerCommand::Task(_) => "task",
            ViewerCommand::Image(_) => "image",
            ViewerCommand::Diagram(command) if command.requires_host_propagation() => {
                "diagram:fullscreen"
            }
            ViewerCommand::Diagram(_) => "diagram",
            ViewerCommand::Search(_) => "search",
            ViewerCommand::Slideshow(_) => "slideshow",
            ViewerCommand::ScrollToHeading(_) => "toc",
            ViewerCommand::Host(_) => "host",
            ViewerCommand::Document(_) => "document",
        }
    }

    fn link_command(
        point: DocumentPoint,
        router: &StorybookHostActionRouter,
    ) -> Option<ViewerCommand> {
        router.resolved_hits_at(point).find_map(|hit| {
            let UiTextSpanAction::OpenLink { target } = hit.hit().action.text_span_action()? else {
                return None;
            };
            if target.starts_with('#')
                && let Some(anchor_target) = router.scene_target_for_internal_anchor(&target)
            {
                return Some(ViewerCommandFactory::scroll_to_target(
                    anchor_target.clone(),
                ));
            }
            Some(ViewerCommandFactory::open_link(
                hit.target().clone(),
                target,
            ))
        })
    }

    fn slideshow_command(
        point: DocumentPoint,
        router: &StorybookHostActionRouter,
    ) -> Option<ViewerCommand> {
        router.hits_at(point).find_map(|hit| {
            ViewerCommandFactory::slideshow_control_from_host_action(hit.action.action_id.as_str())
        })
    }

    fn slideshow_viewport_command(
        scene: &PreviewScene,
        scroll_y: f32,
        pointer: StorybookPointer,
        window_width: usize,
        window_height: usize,
    ) -> Option<ViewerCommand> {
        if scene.mode != ViewerMode::Slideshow {
            return None;
        }
        let area = StorybookPreviewArea::for_window(window_width, window_height, 0.0);
        let (x, y) = area.document_point(pointer.x, pointer.y)?;
        let hits = StorybookHostActionHits::viewport_hits_for_preview_width(
            scene,
            area.width,
            area.height,
            scroll_y,
        );
        katana_ui_core_storybook::UiTreeSurfaceHost::hits_at(hits.as_slice(), x, y)
            .into_iter()
            .find_map(|hit| {
                ViewerCommandFactory::slideshow_control_from_host_action(
                    hit.action.action_id.as_str(),
                )
            })
    }
}

#[path = "mouse_accordion.rs"]
mod mouse_accordion;
#[path = "mouse_cursor.rs"]
mod mouse_cursor;
#[path = "mouse_document_point.rs"]
mod mouse_document_point;
#[path = "mouse_host_action.rs"]
mod mouse_host_action;
#[path = "mouse_media.rs"]
mod mouse_media;
#[path = "mouse_task.rs"]
mod mouse_task;
#[path = "mouse_task_context_menu.rs"]
pub(crate) mod task_context_menu;

#[cfg(test)]
#[path = "mouse_code_tests.rs"]
mod code_tests;
#[cfg(test)]
#[path = "mouse_cursor_tests.rs"]
mod cursor_tests;
#[cfg(test)]
#[path = "mouse_hit_alignment_tests.rs"]
mod hit_alignment_tests;
#[cfg(test)]
#[path = "mouse_test_support.rs"]
pub(crate) mod mouse_test_support;
#[cfg(test)]
#[path = "mouse_state_tests.rs"]
mod state_tests;
#[cfg(test)]
#[path = "mouse_tests.rs"]
mod tests;
