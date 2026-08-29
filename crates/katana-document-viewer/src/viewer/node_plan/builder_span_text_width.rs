use super::super::super::types::{ViewerTextSpan, ViewerTextStyle};
use cosmic_text::{
    Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, Style, SwashCache, Weight, Wrap,
};

#[cfg(target_os = "macos")]
const APPLE_COLOR_EMOJI_FONT_FAMILY: &str = "Apple Color Emoji";
#[cfg(target_os = "macos")]
const MACOS_PROPORTIONAL_FONT_FAMILY: &str = "ヒラギノ角ゴシック";
#[cfg(target_os = "macos")]
const MACOS_MONOSPACE_FONT_FAMILY: &str = "Menlo";

const TEXT_BUFFER_WIDTH: f32 = 4096.0;
const TEXT_SUPERSAMPLE_SCALE: f32 = 2.0;
const LINE_HEIGHT_RATIO: f32 = 1.45;
const BUFFER_HEIGHT_SCALE: f32 = 1.8;
const REGULAR_WEIGHT: u16 = 400;
const BOLD_WEIGHT: u16 = 700;
const COMPACT_WHITESPACE_WIDTH_FACTOR: f32 = 0.30;
const PRESERVED_WHITESPACE_WIDTH_FACTOR: f32 = 0.58;

pub(super) struct SpanTextWidthMeasurer {
    font_system: FontSystem,
    swash_cache: SwashCache,
}

impl SpanTextWidthMeasurer {
    fn new() -> Self {
        Self {
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
        }
    }

    pub(super) fn cached_width(span: &ViewerTextSpan, text: &str, font_size: f32) -> u32 {
        cache::cached_width(span, text, font_size)
    }

    fn width(&mut self, span: &ViewerTextSpan, text: &str, font_size: f32) -> u32 {
        if text.is_empty() {
            return 0;
        }
        let mut width = 0u32;
        let mut segment = String::new();
        for character in text.chars() {
            if character.is_whitespace() && character != '\n' {
                width = width.saturating_add(self.segment_width(&segment, span.style, font_size));
                segment.clear();
                width =
                    width.saturating_add(whitespace_width(font_size, preserves_whitespace(span)));
                continue;
            }
            segment.push(character);
        }
        width
            .saturating_add(self.segment_width(&segment, span.style, font_size))
            .max(1)
    }

    fn segment_width(&mut self, text: &str, style: ViewerTextStyle, font_size: f32) -> u32 {
        if text.is_empty() {
            return 0;
        }
        let metrics = Metrics::new(
            font_size * TEXT_SUPERSAMPLE_SCALE,
            font_size * LINE_HEIGHT_RATIO * TEXT_SUPERSAMPLE_SCALE,
        );
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        let mut buffer = buffer.borrow_with(&mut self.font_system);
        buffer.set_wrap(Wrap::None);
        buffer.set_size(
            Some(TEXT_BUFFER_WIDTH * TEXT_SUPERSAMPLE_SCALE),
            Some(font_size * BUFFER_HEIGHT_SCALE * TEXT_SUPERSAMPLE_SCALE),
        );
        buffer.set_text(text, &attrs_for_style(style), Shaping::Advanced, None);
        buffer.shape_until_scroll(false);
        measured_raster_width(&mut buffer, &mut self.swash_cache).max(1)
    }
}

fn measured_raster_width(
    buffer: &mut cosmic_text::BorrowedWithFontSystem<'_, Buffer>,
    swash_cache: &mut SwashCache,
) -> u32 {
    let mut max_width = 0.0f32;
    buffer.draw(
        swash_cache,
        Color::rgba(255, 255, 255, 255),
        |left, _, _, _, color| {
            if color.a() == 0 {
                return;
            }
            let x = (left as f32 / TEXT_SUPERSAMPLE_SCALE).floor().max(0.0);
            max_width = max_width.max(x + 1.0);
        },
    );
    max_width.ceil().max(0.0) as u32
}

fn attrs_for_style(style: ViewerTextStyle) -> Attrs<'static> {
    let mut attrs = Attrs::new()
        .family(family_for_style(style))
        .weight(weight_for_style(style));
    if style.italic {
        attrs = attrs.style(Style::Italic);
    }
    attrs
}

fn weight_for_style(style: ViewerTextStyle) -> Weight {
    if style.bold {
        return Weight(BOLD_WEIGHT);
    }
    Weight(REGULAR_WEIGHT)
}

fn family_for_style(style: ViewerTextStyle) -> Family<'static> {
    if style.emoji {
        return os_emoji_font_family();
    }
    if style.monospace || style.inline_code || style.inline_math {
        return os_monospace_font_family();
    }
    os_proportional_font_family()
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

#[cfg(target_os = "macos")]
fn os_emoji_font_family() -> Family<'static> {
    Family::Name(APPLE_COLOR_EMOJI_FONT_FAMILY)
}

#[cfg(not(target_os = "macos"))]
fn os_emoji_font_family() -> Family<'static> {
    Family::SansSerif
}

#[cfg(target_os = "macos")]
fn os_proportional_font_family() -> Family<'static> {
    Family::Name(MACOS_PROPORTIONAL_FONT_FAMILY)
}

#[cfg(not(target_os = "macos"))]
fn os_proportional_font_family() -> Family<'static> {
    Family::SansSerif
}

#[cfg(target_os = "macos")]
fn os_monospace_font_family() -> Family<'static> {
    Family::Name(MACOS_MONOSPACE_FONT_FAMILY)
}

#[cfg(not(target_os = "macos"))]
fn os_monospace_font_family() -> Family<'static> {
    Family::Monospace
}

#[path = "builder_span_text_width_cache.rs"]
mod cache;

#[cfg(test)]
#[path = "builder_span_text_width_tests.rs"]
mod tests;
