use crate::export_surface_span::SurfaceTextSpan;
use image::RgbaImage;

use super::super::SpanVisualRange;
use super::super::pixels::SurfacePixelBlender;

const INLINE_IMAGE_OFFSET_SCALE: f32 = 0.22;

pub(super) fn draw_inline_images(
    image: &mut RgbaImage,
    spans: &[SurfaceTextSpan],
    ranges: &[Option<SpanVisualRange>],
    x: u32,
    y: u32,
    size: f32,
) {
    for (index, span) in spans.iter().enumerate() {
        InlineImagePainter::paint(image, span, ranges.get(index), x, y, size);
    }
}

struct InlineImagePainter;

impl InlineImagePainter {
    fn paint(
        image: &mut RgbaImage,
        span: &SurfaceTextSpan,
        range: Option<&Option<SpanVisualRange>>,
        x: u32,
        y: u32,
        size: f32,
    ) {
        let Some(inline_image) = &span.inline_image else {
            return;
        };
        let Some(range) = range.and_then(|range| *range) else {
            return;
        };
        let image_x = x.saturating_add(range.start_x);
        let image_y = Self::image_y(inline_image.height(), y, size);
        paste_inline_rgba(image, inline_image.image(), image_x, image_y);
    }

    fn image_y(image_height: u32, y: u32, size: f32) -> u32 {
        let baseline_adjustment = size * INLINE_IMAGE_OFFSET_SCALE;
        y.saturating_add(baseline_adjustment as u32)
            .saturating_sub(image_height.saturating_sub(size as u32) / 2)
    }
}

#[cfg(test)]
#[path = "export_surface_font_rendering_inline_images_tests.rs"]
mod tests;

fn paste_inline_rgba(target: &mut RgbaImage, source: &RgbaImage, x: u32, y: u32) {
    for (source_x, source_y, pixel) in source.enumerate_pixels() {
        SurfacePixelBlender::blend(
            target,
            x.saturating_add(source_x) as i32,
            y.saturating_add(source_y) as i32,
            *pixel,
        );
    }
}
