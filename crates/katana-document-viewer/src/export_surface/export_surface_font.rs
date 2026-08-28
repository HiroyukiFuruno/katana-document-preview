mod export_surface_font_adapter;
mod export_surface_font_paint;
mod export_surface_font_rendering;
use self::export_surface_font_adapter::raster_width;
use self::export_surface_font_rendering as rendering;
use crate::export_surface_span::SurfaceTextSpan;
use image::Rgba;
use katana_ui_core_text_raster::{PlatformTextRasterConfig, PlatformTextRasterizer};
use std::cell::RefCell;

const DEFAULT_HIGHLIGHT_BACKGROUND: Rgba<u8> = Rgba([255, 235, 59, 255]);
const DEFAULT_INLINE_CODE_BACKGROUND: Rgba<u8> = Rgba([239, 242, 246, 255]);

thread_local! {
    static CACHED_TEXT_PAINTER: RefCell<SurfaceTextPainter> =
        RefCell::new(SurfaceTextPainter::from_system_fonts());
}

#[derive(Clone, Copy)]
pub(crate) struct SurfaceTextBackgroundPalette {
    pub(crate) highlight: Rgba<u8>,
    pub(crate) inline_code: Rgba<u8>,
}

impl Default for SurfaceTextBackgroundPalette {
    fn default() -> Self {
        Self {
            highlight: DEFAULT_HIGHLIGHT_BACKGROUND,
            inline_code: DEFAULT_INLINE_CODE_BACKGROUND,
        }
    }
}

pub(crate) struct SurfaceTextLayout {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) size: f32,
    pub(crate) color: Rgba<u8>,
    pub(crate) max_width: Option<f32>,
}

pub(crate) struct SurfaceSpansLayout {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) size: f32,
    pub(crate) color: Rgba<u8>,
    pub(crate) backgrounds: SurfaceTextBackgroundPalette,
}

pub(crate) struct SurfaceTextPainter {
    rasterizer: PlatformTextRasterizer,
}

impl SurfaceTextPainter {
    pub(crate) fn from_system_fonts() -> Self {
        Self {
            rasterizer: PlatformTextRasterizer::new(PlatformTextRasterConfig::default()),
        }
    }

    pub(crate) fn with_system_fonts<T>(render: impl FnOnce(&mut SurfaceTextPainter) -> T) -> T {
        CACHED_TEXT_PAINTER.with(|cell| render(&mut cell.borrow_mut()))
    }

    pub(crate) fn measure_spans_width(
        &mut self,
        spans: &[SurfaceTextSpan],
        size: f32,
        max_width: f32,
    ) -> u32 {
        match self.rasterize_spans(spans, size, max_width, Rgba([0, 0, 0, 255])) {
            Some(raster) => raster_width(&raster),
            None => 0,
        }
    }
}

#[cfg(test)]
mod export_surface_font_emoji_tests;
#[cfg(test)]
mod export_surface_font_test_cases;
#[cfg(test)]
mod export_surface_font_test_support;
