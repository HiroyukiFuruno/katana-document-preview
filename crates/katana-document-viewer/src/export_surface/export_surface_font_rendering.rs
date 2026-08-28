#[path = "export_surface_font_rendering_pixels.rs"]
mod pixels;
#[path = "export_surface_font_rendering_shapes.rs"]
mod shapes;

pub(super) use pixels::SurfacePixelBlender;

#[derive(Clone, Copy, Debug)]
pub(super) struct SpanVisualRange {
    pub(super) start_x: u32,
    end_x: u32,
}

impl SpanVisualRange {
    pub(super) fn new(start_x: f32, end_x: f32) -> Self {
        Self {
            start_x: start_x.floor().max(0.0) as u32,
            end_x: end_x.ceil().max(0.0) as u32,
        }
    }

    pub(super) fn width(self) -> u32 {
        self.end_x.saturating_sub(self.start_x).max(1)
    }

    #[cfg(test)]
    pub(super) fn end_x(self) -> u32 {
        self.end_x
    }

    pub(super) fn extend(self, start_x: f32, end_x: f32) -> Self {
        Self {
            start_x: (self.start_x as f32).min(start_x) as u32,
            end_x: (self.end_x as f32).max(end_x) as u32,
        }
    }
}

pub(super) fn draw_span_backgrounds(
    image: &mut image::RgbaImage,
    spans: &[crate::export_surface_span::SurfaceTextSpan],
    ranges: &[Option<SpanVisualRange>],
    x: u32,
    y: u32,
    size: f32,
    palette: crate::export_surface_font::SurfaceTextBackgroundPalette,
) {
    shapes::draw_span_backgrounds(image, spans, ranges, x, y, size, palette);
}

pub(super) fn draw_inline_images(
    image: &mut image::RgbaImage,
    spans: &[crate::export_surface_span::SurfaceTextSpan],
    ranges: &[Option<SpanVisualRange>],
    x: u32,
    y: u32,
    size: f32,
) {
    shapes::draw_inline_images(image, spans, ranges, x, y, size);
}

pub(super) fn draw_span_decorations(
    image: &mut image::RgbaImage,
    spans: &[crate::export_surface_span::SurfaceTextSpan],
    ranges: &[Option<SpanVisualRange>],
    x: u32,
    y: u32,
    size: f32,
) {
    shapes::draw_span_decorations(image, spans, ranges, x, y, size);
}
