use super::{SurfaceTextPainter, rendering};
use crate::export_surface_span::SurfaceTextSpan;
use image::Rgba;
use katana_ui_core::render_model::{UiTextSpan, UiTextSpanStyle};
use katana_ui_core::theme::{FontFamily, FontToken};
use katana_ui_core_text_raster::{PlatformTextRaster, PlatformTextRasterRequest};

const FONT_LINE_HEIGHT_MULTIPLIER: f32 = 1.45;
impl SurfaceTextPainter {
    pub(super) fn rasterize_text(
        &mut self,
        text: &str,
        size: f32,
        max_width: f32,
        color: Rgba<u8>,
    ) -> Option<PlatformTextRaster> {
        let mut request = PlatformTextRasterRequest::from_text(
            text,
            font_token(FontFamily::Proportional, size),
            color.0,
        );
        request.max_width_px = Some(max_width);
        self.rasterizer.rasterize(&request).ok()
    }

    pub(super) fn rasterize_spans(
        &mut self,
        spans: &[SurfaceTextSpan],
        size: f32,
        max_width: f32,
        color: Rgba<u8>,
    ) -> Option<PlatformTextRaster> {
        self.rasterizer
            .rasterize(&span_request(spans, size, max_width, color))
            .ok()
    }

    pub(super) fn span_visual_ranges(
        &mut self,
        spans: &[SurfaceTextSpan],
        size: f32,
        max_width: f32,
        color: Rgba<u8>,
    ) -> (
        Option<PlatformTextRaster>,
        Vec<Option<rendering::SpanVisualRange>>,
    ) {
        let raster = self.rasterize_spans(spans, size, max_width, color);
        let ranges = match &raster {
            Some(raster) => span_ranges(raster, spans, size),
            None => vec![None; spans.len()],
        };
        (raster, ranges)
    }
}
pub(super) fn raster_width(raster: &PlatformTextRaster) -> u32 {
    raster
        .grapheme_bounds
        .iter()
        .map(|bounds| bounds.x + bounds.width)
        .fold(0.0, f32::max)
        .ceil() as u32
}
fn span_request(
    spans: &[SurfaceTextSpan],
    size: f32,
    max_width: f32,
    color: Rgba<u8>,
) -> PlatformTextRasterRequest {
    PlatformTextRasterRequest {
        spans: spans
            .iter()
            .map(|span| ui_span(span, size, color))
            .collect(),
        font: font_token(font_family(spans), size),
        fallback_color_rgba: color.0,
        line_height_px: size * FONT_LINE_HEIGHT_MULTIPLIER,
        max_width_px: Some(max_width),
        scale_factor: 1.0,
    }
}

fn ui_span(span: &SurfaceTextSpan, size: f32, color: Rgba<u8>) -> UiTextSpan {
    UiTextSpan {
        text: span.layout_text(size),
        style: ui_style(span, color),
        link_target: match &span.link_target {
            Some(target) => target.clone(),
            None => String::new(),
        },
    }
}

fn font_family(spans: &[SurfaceTextSpan]) -> FontFamily {
    if spans
        .iter()
        .all(|span| span.style.monospace || span.style.inline_code)
    {
        FontFamily::Monospace
    } else {
        FontFamily::Proportional
    }
}

fn font_token(family: FontFamily, size: f32) -> FontToken {
    FontToken {
        name: "kdv-export".to_string(),
        family,
        size,
        weight: 400,
    }
}

fn ui_style(span: &SurfaceTextSpan, color: Rgba<u8>) -> UiTextSpanStyle {
    let style = span.style;
    UiTextSpanStyle {
        bold: style.bold,
        italic: style.italic,
        monospace: style.monospace || style.inline_code,
        underline: style.underline,
        strikethrough: style.strikethrough,
        highlight: style.highlight,
        inline_code: style.inline_code,
        emoji: style.emoji,
        color_rgba: text_color(span, color),
        ..UiTextSpanStyle::default()
    }
}

fn text_color(span: &SurfaceTextSpan, fallback: Rgba<u8>) -> [u8; 4] {
    if span.inline_image.is_some() {
        [0; 4]
    } else {
        span.style.color.unwrap_or(fallback).0
    }
}

fn span_ranges(
    raster: &PlatformTextRaster,
    spans: &[SurfaceTextSpan],
    size: f32,
) -> Vec<Option<rendering::SpanVisualRange>> {
    let mut byte_start = 0;
    spans
        .iter()
        .map(|span| {
            let byte_end = byte_start + span.layout_text(size).len();
            let range = span_range(raster, byte_start, byte_end);
            byte_start = byte_end;
            range
        })
        .collect()
}

fn span_range(
    raster: &PlatformTextRaster,
    byte_start: usize,
    byte_end: usize,
) -> Option<rendering::SpanVisualRange> {
    raster
        .grapheme_bounds
        .iter()
        .filter(|bounds| hit_is_inside(raster, bounds, byte_start, byte_end))
        .fold(None, extend_range)
}

fn hit_is_inside(
    raster: &PlatformTextRaster,
    bounds: &katana_ui_core_text_raster::PlatformTextGraphemeBounds,
    byte_start: usize,
    byte_end: usize,
) -> bool {
    raster
        .hit_test(
            bounds.x + bounds.width / 2.0,
            bounds.y + bounds.height / 2.0,
        )
        .is_some_and(|hit| hit.byte_start >= byte_start && hit.byte_end <= byte_end)
}

fn extend_range(
    range: Option<rendering::SpanVisualRange>,
    bounds: &katana_ui_core_text_raster::PlatformTextGraphemeBounds,
) -> Option<rendering::SpanVisualRange> {
    Some(match range {
        Some(range) => range.extend(bounds.x, bounds.x + bounds.width),
        None => rendering::SpanVisualRange::new(bounds.x, bounds.x + bounds.width),
    })
}
