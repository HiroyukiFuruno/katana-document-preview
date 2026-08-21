use image::{Rgba, RgbaImage};

use super::SurfaceHelpers;

impl SurfaceHelpers {
    pub(crate) fn surface_block_height(block_heights: impl Iterator<Item = u32>) -> u32 {
        let content_height = Self::block_stack_height(block_heights) + super::PAGE_PADDING * 2;
        content_height.max(super::SURFACE_MIN_HEIGHT)
    }

    pub(crate) fn block_stack_height(block_heights: impl Iterator<Item = u32>) -> u32 {
        block_heights.sum()
    }

    pub(crate) fn parse_color(value: &str) -> Rgba<u8> {
        let value = value.trim_start_matches('#');
        if value.len() != HEX_COLOR_TEXT_LENGTH {
            return DEFAULT_WHITE;
        }
        let red = u8::from_str_radix(&value[COLOR_RED_START..COLOR_GREEN_START], HEX_RADIX)
            .unwrap_or(DEFAULT_CHANNEL);
        let green = u8::from_str_radix(&value[COLOR_GREEN_START..COLOR_BLUE_START], HEX_RADIX)
            .unwrap_or(DEFAULT_CHANNEL);
        let blue = u8::from_str_radix(&value[COLOR_BLUE_START..HEX_COLOR_TEXT_LENGTH], HEX_RADIX)
            .unwrap_or(DEFAULT_CHANNEL);
        Rgba([red, green, blue, MAX_ALPHA_U8])
    }

    pub(crate) fn draw_quote_bars(
        image: &mut RgbaImage,
        depth: u32,
        y: u32,
        height: u32,
        color: Rgba<u8>,
    ) {
        for index in 0..depth {
            let x = super::PAGE_PADDING + index * super::QUOTE_INDENT;
            for dy in 0..height {
                for dx in 0..SUPER_SMALL_ICON_SPAN {
                    Self::put_pixel_if_inside(image, x + dx, y + dy, color);
                }
            }
        }
    }

    pub(crate) fn fill_rect(
        image: &mut RgbaImage,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        color: Rgba<u8>,
    ) {
        let Some(rect) = ClippedRect::from_image(image, x, y, width, height) else {
            return;
        };
        let row_width = image.width() as usize;
        let channels = color.0;
        let buffer = image.as_mut();
        for row in rect.y..rect.bottom {
            let start = (row * row_width + rect.x) * RGBA_CHANNEL_COUNT;
            let end = start + rect.width * RGBA_CHANNEL_COUNT;
            for pixel in buffer[start..end].as_chunks_mut::<RGBA_CHANNEL_COUNT>().0 {
                pixel.copy_from_slice(&channels);
            }
        }
    }

    pub(crate) fn stroke_rect(
        image: &mut RgbaImage,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        color: Rgba<u8>,
    ) {
        if width == 0 || height == 0 {
            return;
        }
        for dx in 0..width {
            Self::put_pixel_if_inside(image, x + dx, y, color);
            Self::put_pixel_if_inside(image, x + dx, y + height - 1, color);
        }
        for dy in 0..height {
            Self::put_pixel_if_inside(image, x, y + dy, color);
            Self::put_pixel_if_inside(image, x + width - 1, y + dy, color);
        }
    }

    pub(crate) fn paste_rgba(image: &mut RgbaImage, source: &RgbaImage, x: u32, y: u32) {
        for source_y in 0..source.height() {
            for source_x in 0..source.width() {
                let target_x = x + source_x;
                let target_y = y + source_y;
                if target_x >= image.width() || target_y >= image.height() {
                    continue;
                }
                let foreground = source.get_pixel(source_x, source_y);
                let alpha = foreground[ALPHA_CHANNEL_INDEX] as u16;
                if alpha == 0 {
                    continue;
                }
                let background = image.get_pixel(target_x, target_y);
                let inverse_alpha = MAX_ALPHA_U16 - alpha;
                let red = Self::blend_channel(foreground[0], background[0], alpha, inverse_alpha);
                let green = Self::blend_channel(foreground[1], background[1], alpha, inverse_alpha);
                let blue = Self::blend_channel(foreground[2], background[2], alpha, inverse_alpha);
                image.put_pixel(target_x, target_y, Rgba([red, green, blue, MAX_ALPHA_U8]));
            }
        }
    }

    pub(crate) fn paste_rgba_resized(
        image: &mut RgbaImage,
        source: &RgbaImage,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) {
        if width == 0 || height == 0 {
            return;
        }
        if source.width() == width && source.height() == height {
            Self::paste_rgba(image, source, x, y);
            return;
        }
        let resized =
            image::imageops::resize(source, width, height, image::imageops::FilterType::Lanczos3);
        Self::paste_rgba(image, &resized, x, y);
    }

    fn blend_channel(foreground: u8, background: u8, alpha: u16, inverse_alpha: u16) -> u8 {
        let blended = foreground as u16 * alpha + background as u16 * inverse_alpha;
        (blended / MAX_ALPHA_U16) as u8
    }

    fn put_pixel_if_inside(image: &mut RgbaImage, x: u32, y: u32, color: Rgba<u8>) {
        if x < image.width() && y < image.height() {
            image.put_pixel(x, y, color);
        }
    }
}

#[cfg(test)]
#[path = "export_surface_helpers_canvas_tests.rs"]
mod tests;

const DEFAULT_WHITE: image::Rgba<u8> = Rgba([255, 255, 255, 255]);
const DEFAULT_CHANNEL: u8 = 255;
const MAX_ALPHA_U8: u8 = 255;
const MAX_ALPHA_U16: u16 = 255;
const HEX_COLOR_TEXT_LENGTH: usize = 6;
const HEX_RADIX: u32 = 16;
const COLOR_RED_START: usize = 0;
const COLOR_GREEN_START: usize = 2;
const COLOR_BLUE_START: usize = 4;
const ALPHA_CHANNEL_INDEX: usize = 3;
const SUPER_SMALL_ICON_SPAN: u32 = 4;
const RGBA_CHANNEL_COUNT: usize = 4;

struct ClippedRect {
    x: usize,
    y: usize,
    width: usize,
    bottom: usize,
}

impl ClippedRect {
    fn from_image(image: &RgbaImage, x: u32, y: u32, width: u32, height: u32) -> Option<Self> {
        let end_x = x.saturating_add(width).min(image.width());
        let end_y = y.saturating_add(height).min(image.height());
        if x >= end_x || y >= end_y {
            return None;
        }
        Some(Self {
            x: x as usize,
            y: y as usize,
            width: (end_x - x) as usize,
            bottom: end_y as usize,
        })
    }
}
