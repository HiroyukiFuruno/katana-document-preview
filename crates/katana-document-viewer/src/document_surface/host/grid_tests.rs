use super::*;
use crate::document_surface::host::test_support::*;

#[test]
fn grid_host_maps_click_scroll_drag_and_keyboard_input() {
    let mut harness = GridHarness::new();
    assert_click(&mut harness);
    assert_navigation(&mut harness);
    assert_wheel_scroll(&mut harness);
    assert_drag_scroll(&mut harness);
}

struct GridHarness {
    context: egui::Context,
    host: crate::DocumentSurfaceHost,
    frame: DocumentSurfaceFrame,
    pointer: egui::Pos2,
}

impl GridHarness {
    fn new() -> Self {
        Self {
            context: egui::Context::default(),
            host: crate::DocumentSurfaceHost::default(),
            frame: grid_frame(vec![grid_cell(0, 0, "A1")], true),
            pointer: egui::pos2(20.0, 20.0),
        }
    }

    fn run(&mut self, events: Vec<egui::Event>) -> DocumentSurfaceHostOutput {
        run_surface(
            &self.context,
            &mut self.host,
            &self.frame,
            raw_input(events),
        )
    }
}

fn assert_click(harness: &mut GridHarness) {
    let pointer = harness.pointer;
    let _ = harness.run(vec![egui::Event::PointerMoved(pointer)]);
    let _ = harness.run(vec![pointer_button(pointer, true)]);
    let clicked = harness.run(vec![pointer_button(pointer, false)]);
    assert!(has_grid_command(clicked.commands(), |command| {
        matches!(
            command,
            DocumentGridCommand::Select {
                row: 0,
                column: 0,
                ..
            }
        )
    }));
}

fn assert_navigation(harness: &mut GridHarness) {
    let navigated = harness.run(vec![key_event(egui::Key::PageDown)]);
    assert!(has_grid_command(navigated.commands(), |command| {
        matches!(
            command,
            DocumentGridCommand::Navigate {
                intent: DocumentGridNavigation::PageDown,
                ..
            }
        )
    }));
}

fn assert_wheel_scroll(harness: &mut GridHarness) {
    let pointer = harness.pointer;
    let scrolled = harness.run(vec![
        egui::Event::PointerMoved(pointer),
        egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(-4.0, -6.0),
            modifiers: egui::Modifiers::default(),
            phase: egui::TouchPhase::Move,
        },
    ]);
    assert!(has_grid_command(scrolled.commands(), |command| {
        matches!(command, DocumentGridCommand::ScrollTo { .. })
    }));
}

fn assert_drag_scroll(harness: &mut GridHarness) {
    let pointer = harness.pointer;
    let _ = harness.run(vec![pointer_button(pointer, true)]);
    let dragged = harness.run(vec![egui::Event::PointerMoved(egui::pos2(55.0, 55.0))]);
    assert!(has_grid_command(dragged.commands(), |command| {
        matches!(command, DocumentGridCommand::ScrollTo { .. })
    }));
}

fn has_grid_command(
    commands: &[DocumentSurfaceCommand],
    predicate: impl Fn(&DocumentGridCommand) -> bool,
) -> bool {
    commands.iter().any(|command| match command {
        DocumentSurfaceCommand::Grid(command) => predicate(command),
        DocumentSurfaceCommand::Resize(_) => false,
    })
}

#[test]
fn grid_geometry_and_offsets_cover_hits_misses_and_both_directions() {
    let cells = vec![grid_cell(0, 0, "A1")];
    let viewport = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(100.0, 60.0));

    assert!(hit_cell(viewport, egui::pos2(20.0, 20.0), &cells).is_some());
    assert!(hit_cell(viewport, egui::pos2(150.0, 150.0), &cells).is_none());
    assert_eq!(15, offset(10, 5.0));
    assert_eq!(5, offset(10, -5.0));
    assert_eq!(0, offset(2, -10.0));
}
