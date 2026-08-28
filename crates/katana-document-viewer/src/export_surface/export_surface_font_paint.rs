use super::{SurfaceSpansLayout, SurfaceTextLayout, SurfaceTextPainter, rendering};
use crate::export_surface_span::SurfaceTextSpan;
#[cfg(test)]
use image::Rgba;
use image::RgbaImage;

impl SurfaceTextPainter {
    pub(crate) fn draw_text(
        &mut self,
        image: &mut RgbaImage,
        text: &str,
        layout: SurfaceTextLayout,
    ) {
        let max_width = layout
            .max_width
            .unwrap_or_else(|| image.width().saturating_sub(layout.x) as f32);
        if let Some(raster) = self.rasterize_text(text, layout.size, max_width, layout.color) {
            draw_raster(image, &raster, layout.x, layout.y);
        }
    }

    #[cfg(test)]
    pub(crate) fn draw_spans(
        &mut self,
        image: &mut RgbaImage,
        spans: &[SurfaceTextSpan],
        x: u32,
        y: u32,
        size: f32,
        color: Rgba<u8>,
    ) {
        self.draw_spans_with_backgrounds(
            image,
            spans,
            SurfaceSpansLayout {
                x,
                y,
                size,
                color,
                backgrounds: super::SurfaceTextBackgroundPalette::default(),
            },
        );
    }

    pub(crate) fn draw_spans_with_backgrounds(
        &mut self,
        image: &mut RgbaImage,
        spans: &[SurfaceTextSpan],
        layout: SurfaceSpansLayout,
    ) {
        let max_width = image.width().saturating_sub(layout.x) as f32;
        let (raster, ranges) = self.span_visual_ranges(spans, layout.size, max_width, layout.color);
        rendering::draw_span_backgrounds(
            image,
            spans,
            &ranges,
            layout.x,
            layout.y,
            layout.size,
            layout.backgrounds,
        );
        if let Some(raster) = raster {
            draw_raster(image, &raster, layout.x, layout.y);
        }
        rendering::draw_inline_images(image, spans, &ranges, layout.x, layout.y, layout.size);
        rendering::draw_span_decorations(image, spans, &ranges, layout.x, layout.y, layout.size);
    }
}

fn draw_raster(
    image: &mut RgbaImage,
    raster: &katana_ui_core_text_raster::PlatformTextRaster,
    x: u32,
    y: u32,
) {
    for (index, pixel) in raster.rgba_pixels.iter().enumerate() {
        if pixel[3] == 0 {
            continue;
        }
        let pixel_x = x.saturating_add((index % raster.width) as u32);
        let pixel_y = y.saturating_add((index / raster.width) as u32);
        rendering::SurfacePixelBlender::blend(
            image,
            pixel_x as i32,
            pixel_y as i32,
            image::Rgba(*pixel),
        );
    }
}
