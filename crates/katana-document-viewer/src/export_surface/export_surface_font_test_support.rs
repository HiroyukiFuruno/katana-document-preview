use super::Rgba;
use image::RgbaImage;

const HALF_WIDTH_SPACE_FACTOR: f32 = 0.35;
const HALF_WIDTH_PUNCTUATION_FACTOR: f32 = 0.43;
const HALF_WIDTH_TEXT_FACTOR: f32 = 0.54;
const HALF_WIDTH_MATH_FACTOR: f32 = 0.65;
const DEFAULT_WIDTH_FACTOR: f32 = 0.92;

pub(super) fn estimated_text_width(text: &str, size: f32) -> u32 {
    text.chars()
        .map(|character| character_width_factor(character) * size)
        .sum::<f32>()
        .ceil() as u32
}

fn character_width_factor(character: char) -> f32 {
    if character.is_ascii_whitespace() {
        return HALF_WIDTH_SPACE_FACTOR;
    }
    if character.is_ascii_punctuation() {
        return HALF_WIDTH_PUNCTUATION_FACTOR;
    }
    if character.is_ascii() {
        return HALF_WIDTH_TEXT_FACTOR;
    }
    if is_half_width_math_symbol(character) {
        return HALF_WIDTH_MATH_FACTOR;
    }
    DEFAULT_WIDTH_FACTOR
}

pub(super) fn is_half_width_math_symbol(character: char) -> bool {
    HALF_WIDTH_MATH_SYMBOLS.contains(&character)
}

const HALF_WIDTH_MATH_SYMBOLS: &[char] = &[
    'α', 'β', 'γ', 'δ', '∑', '∫', '√', '∞', '⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹', 'ⁿ',
    'ˣ', '₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉', 'ₖ',
];

pub(super) fn actual_span_x_range(
    spans: &[super::SurfaceTextSpan],
    span_index: usize,
    size: f32,
) -> Option<(u32, u32)> {
    super::SurfaceTextPainter::from_system_fonts().span_x_range(spans, span_index, size)
}

impl super::SurfaceTextPainter {
    pub(crate) fn span_x_range(
        &mut self,
        spans: &[super::SurfaceTextSpan],
        span_index: usize,
        size: f32,
    ) -> Option<(u32, u32)> {
        let (_, ranges) = self.span_visual_ranges(spans, size, 2048.0, Rgba([36, 41, 47, 255]));
        ranges
            .get(span_index)
            .and_then(|range| *range)
            .map(|range| (range.start_x, range.end_x()))
    }
}

pub(super) fn painted_x_range(image: &RgbaImage, color: Rgba<u8>) -> Option<(u32, u32)> {
    let mut min_x = None;
    let mut max_x = None;
    for (x, _, pixel) in image.enumerate_pixels() {
        if *pixel != color {
            continue;
        }
        min_x = Some(min_x.map_or(x, |current: u32| current.min(x)));
        max_x = Some(max_x.map_or(x, |current: u32| current.max(x)));
    }
    Some((min_x?, max_x?))
}

#[cfg(test)]
#[path = "export_surface_font_test_support_tests.rs"]
mod tests;
