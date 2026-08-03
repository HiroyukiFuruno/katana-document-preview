use super::DocumentSurfaceHostOutput;
use super::grid_paint::{paint_grid, translated};
use crate::{
    DocumentGridCommand, DocumentGridNavigation, DocumentSurfaceCommand, DocumentSurfaceFrame,
    DocumentViewport,
};
use katana_ui_core::render_model::UiGridCell;

pub(super) fn show(ui: &mut egui::Ui, frame: &DocumentSurfaceFrame) -> DocumentSurfaceHostOutput {
    let size = ui.available_size().max(egui::vec2(1.0, 1.0));
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
    let props = &frame.node().props().grid;
    paint_grid(ui, rect, props);

    let mut output = DocumentSurfaceHostOutput::default();
    output.push(resize_command(rect));
    for command in [
        selection_command(ui, rect, &response, &props.cells),
        scroll_command(
            ui,
            &response,
            props.viewport.scroll_x,
            props.viewport.scroll_y,
        ),
        navigation_command(ui, &response),
    ]
    .into_iter()
    .flatten()
    {
        output.push(command);
    }
    output
}

fn resize_command(rect: egui::Rect) -> DocumentSurfaceCommand {
    DocumentSurfaceCommand::Resize(DocumentViewport::new(
        rect.width() as u32,
        rect.height() as u32,
    ))
}

fn selection_command(
    ui: &egui::Ui,
    viewport: egui::Rect,
    response: &egui::Response,
    cells: &[UiGridCell],
) -> Option<DocumentSurfaceCommand> {
    if !response.clicked() {
        return None;
    }
    response.request_focus();
    let pointer = response.interact_pointer_pos()?;
    let cell = hit_cell(viewport, pointer, cells)?;
    Some(DocumentSurfaceCommand::Grid(DocumentGridCommand::Select {
        row: cell.coordinate.row,
        column: cell.coordinate.column,
        extend: ui.input(|input| input.modifiers.shift),
    }))
}

fn scroll_command(
    ui: &egui::Ui,
    response: &egui::Response,
    scroll_x: u32,
    scroll_y: u32,
) -> Option<DocumentSurfaceCommand> {
    if !response.hovered() {
        return None;
    }
    let delta = ui.input(|input| input.smooth_scroll_delta);
    let drag = if response.dragged() {
        ui.input(|input| input.pointer.delta())
    } else {
        egui::Vec2::ZERO
    };
    let movement = delta + drag;
    (movement != egui::Vec2::ZERO).then(|| {
        DocumentSurfaceCommand::Grid(DocumentGridCommand::ScrollTo {
            x: offset(scroll_x, -movement.x),
            y: offset(scroll_y, -movement.y),
        })
    })
}

fn navigation_command(ui: &egui::Ui, response: &egui::Response) -> Option<DocumentSurfaceCommand> {
    let intent = response
        .has_focus()
        .then(|| navigation_intent(ui))
        .flatten()?;
    Some(DocumentSurfaceCommand::Grid(
        DocumentGridCommand::Navigate {
            intent,
            extend: ui.input(|input| input.modifiers.shift),
        },
    ))
}

fn hit_cell(
    viewport: egui::Rect,
    pointer: egui::Pos2,
    cells: &[UiGridCell],
) -> Option<&UiGridCell> {
    cells.iter().rev().find(|cell| {
        translated(viewport.min, cell.clipped_bounds)
            .intersect(viewport)
            .contains(pointer)
    })
}

fn offset(current: u32, delta: f32) -> u32 {
    if delta >= 0.0 {
        current.saturating_add(delta.round() as u32)
    } else {
        current.saturating_sub((-delta).round() as u32)
    }
}

fn navigation_intent(ui: &egui::Ui) -> Option<DocumentGridNavigation> {
    let mappings = [
        (egui::Key::ArrowLeft, DocumentGridNavigation::Left),
        (egui::Key::ArrowRight, DocumentGridNavigation::Right),
        (egui::Key::ArrowUp, DocumentGridNavigation::Up),
        (egui::Key::ArrowDown, DocumentGridNavigation::Down),
        (egui::Key::Home, DocumentGridNavigation::Home),
        (egui::Key::End, DocumentGridNavigation::End),
        (egui::Key::PageUp, DocumentGridNavigation::PageUp),
        (egui::Key::PageDown, DocumentGridNavigation::PageDown),
    ];
    ui.input(|input| {
        mappings
            .into_iter()
            .find_map(|(key, intent)| input.key_pressed(key).then_some(intent))
    })
}

#[cfg(test)]
#[path = "grid_tests.rs"]
mod tests;
