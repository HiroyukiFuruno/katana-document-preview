use super::super::super::types::{ViewerTextSpan, ViewerTextStyle};
use katana_ui_core::render_model::{UiTextSpan, UiTextSpanStyle};
use katana_ui_core::theme::{FontFamily, FontToken};
use katana_ui_core_text_raster::{
    PlatformTextRasterConfig, PlatformTextRasterRequest, PlatformTextRasterizer,
};
use std::cell::RefCell;

const LINE_HEIGHT_RATIO: f32 = 1.45;
const TEXT_BUFFER_WIDTH: f32 = 4096.0;
const COMPACT_WHITESPACE_WIDTH_FACTOR: f32 = 0.30;
const PRESERVED_WHITESPACE_WIDTH_FACTOR: f32 = 0.58;

thread_local! {
    static TEXT_RASTERIZER: RefCell<PlatformTextRasterizer> =
        RefCell::new(PlatformTextRasterizer::new(PlatformTextRasterConfig::default()));
}

pub(super) struct SpanTextWidthMeasurer;

impl SpanTextWidthMeasurer {
    pub(super) fn cached_width(span: &ViewerTextSpan, text: &str, font_size: f32) -> u32 {
        if text.is_empty() {
            return 0;
        }
        let mut width = 0u32;
        let mut segment = String::new();
        for character in text.chars() {
            if character.is_whitespace() && character != '\n' {
                width = width.saturating_add(Self::segment_width(&segment, span.style, font_size));
                segment.clear();
                width =
                    width.saturating_add(whitespace_width(font_size, preserves_whitespace(span)));
                continue;
            }
            segment.push(character);
        }
        width
            .saturating_add(Self::segment_width(&segment, span.style, font_size))
            .max(1)
    }

    fn segment_width(text: &str, style: ViewerTextStyle, font_size: f32) -> u32 {
        let text = text.trim_end_matches(char::is_whitespace);
        if text.is_empty() {
            return 0;
        }
        let request = raster_request(text, style, font_size);
        TEXT_RASTERIZER.with(
            |rasterizer| match rasterizer.borrow_mut().rasterize(&request) {
                Ok(raster) => raster_width(&raster),
                Err(_) => 1,
            },
        )
    }
}

fn raster_request(text: &str, style: ViewerTextStyle, font_size: f32) -> PlatformTextRasterRequest {
    PlatformTextRasterRequest {
        spans: vec![UiTextSpan {
            text: text.to_string(),
            style: ui_style(style),
            link_target: String::new(),
        }],
        font: font_token(style, font_size),
        fallback_color_rgba: [255, 255, 255, 255],
        line_height_px: font_size * LINE_HEIGHT_RATIO,
        max_width_px: Some(TEXT_BUFFER_WIDTH),
        scale_factor: 1.0,
    }
}

fn raster_width(raster: &katana_ui_core_text_raster::PlatformTextRaster) -> u32 {
    raster
        .grapheme_bounds
        .iter()
        .map(|bounds| bounds.x + bounds.width)
        .fold(0.0, f32::max)
        .floor() as u32
}

fn ui_style(style: ViewerTextStyle) -> UiTextSpanStyle {
    UiTextSpanStyle {
        bold: style.bold,
        italic: style.italic,
        monospace: style.monospace || style.inline_code || style.inline_math,
        inline_code: style.inline_code,
        inline_math: style.inline_math,
        emoji: style.emoji,
        ..UiTextSpanStyle::default()
    }
}

fn font_token(style: ViewerTextStyle, font_size: f32) -> FontToken {
    FontToken {
        name: "kdv-document".to_string(),
        family: if style.monospace || style.inline_code || style.inline_math {
            FontFamily::Monospace
        } else {
            FontFamily::Proportional
        },
        size: font_size,
        weight: if style.bold { 700 } else { 400 },
    }
}

fn whitespace_width(font_size: f32, preserve_whitespace: bool) -> u32 {
    let factor = if preserve_whitespace {
        PRESERVED_WHITESPACE_WIDTH_FACTOR
    } else {
        COMPACT_WHITESPACE_WIDTH_FACTOR
    };
    (font_size * factor).ceil() as u32
}

fn preserves_whitespace(span: &ViewerTextSpan) -> bool {
    span.style.monospace || span.style.inline_code
}
