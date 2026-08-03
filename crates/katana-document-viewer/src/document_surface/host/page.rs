use super::{DocumentSurfaceHost, DocumentSurfaceHostOutput};
use crate::{DocumentSurfaceCommand, DocumentSurfaceFrame, DocumentViewport};
use katana_ui_core::render_model::UiImageSurfaceProps;

pub(super) fn show(
    host: &mut DocumentSurfaceHost,
    ui: &mut egui::Ui,
    frame: &DocumentSurfaceFrame,
    surface_id: u64,
) -> DocumentSurfaceHostOutput {
    let props = &frame.node().props().image_surface;
    let viewport = ui.available_size();
    let mut output = DocumentSurfaceHostOutput::default();
    output.push(DocumentSurfaceCommand::Resize(DocumentViewport::new(
        viewport.x.max(1.0) as u32,
        viewport.y.max(1.0) as u32,
    )));
    update_texture(host, ui, props);
    if let Some(texture) = &host.texture {
        paint_page(ui, texture, props, surface_id);
    }
    output
}

fn update_texture(host: &mut DocumentSurfaceHost, ui: &egui::Ui, props: &UiImageSurfaceProps) {
    if host.texture_fingerprint.as_deref() != Some(&props.fingerprint) {
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [props.width as usize, props.height as usize],
            &props.rgba,
        );
        host.texture = Some(ui.ctx().load_texture(
            format!("kdv-document:{}", props.fingerprint),
            image,
            egui::TextureOptions::LINEAR,
        ));
        host.texture_fingerprint = Some(props.fingerprint.clone());
    }
}

fn paint_page(
    ui: &mut egui::Ui,
    texture: &egui::TextureHandle,
    props: &UiImageSurfaceProps,
    surface_id: u64,
) {
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .id_salt(("kdv_document_page", surface_id))
        .show(ui, |ui| {
            paint_centered_image(ui, texture, display_size(props));
        });
}

fn display_size(props: &UiImageSurfaceProps) -> egui::Vec2 {
    egui::vec2(
        props.display_width_milli as f32 / 1_000.0,
        props.display_height_milli as f32 / 1_000.0,
    )
}

fn paint_centered_image(ui: &mut egui::Ui, texture: &egui::TextureHandle, size: egui::Vec2) {
    let available = ui.available_width();
    ui.horizontal(|ui| {
        if size.x < available {
            ui.add_space((available - size.x) * 0.5);
        }
        let response = ui.add(
            egui::Image::new(texture)
                .fit_to_exact_size(size)
                .sense(egui::Sense::click()),
        );
        let color = ui.visuals().widgets.noninteractive.bg_stroke.color;
        ui.painter().rect_stroke(
            response.rect,
            0.0,
            egui::Stroke::new(1.0, color),
            egui::StrokeKind::Inside,
        );
    });
}

#[cfg(test)]
#[path = "page_tests.rs"]
mod tests;
