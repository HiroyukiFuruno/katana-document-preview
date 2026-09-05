use crate::artifact::ArtifactFormat;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const VIEWER_DIAGRAM_DISPLAY_SCALE: f32 = 1.0;
pub const VIEWER_DIAGRAM_DISPLAY_MAX_WIDTH: u32 = 1264;

#[path = "image_surface_cache.rs"]
mod image_surface_cache;
#[path = "image_surface_factory.rs"]
mod image_surface_factory;
#[path = "image_surface_svg_factory.rs"]
mod image_surface_svg_factory;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewerImageSurface {
    pub fingerprint: String,
    pub width: u32,
    pub height: u32,
    pub display_width: f32,
    pub display_height: f32,
    pub content_scale: u32,
    pub rgba: Vec<u8>,
}

impl ViewerImageSurface {
    #[must_use]
    pub fn logical_width(&self) -> u32 {
        if self.display_width.is_finite() && self.display_width > 0.0 {
            return self.display_width.ceil() as u32;
        }
        logical_extent(self.width, self.content_scale)
    }

    #[must_use]
    pub fn logical_height(&self) -> u32 {
        if self.display_height.is_finite() && self.display_height > 0.0 {
            return self.display_height.ceil() as u32;
        }
        logical_extent(self.height, self.content_scale)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ViewerImageSurfaceError {
    #[error("unsupported viewer image format: {0:?}")]
    UnsupportedFormat(ArtifactFormat),
    #[error("SVG artifact is not UTF-8")]
    InvalidSvgEncoding,
    #[error("SVG artifact cannot be rasterized")]
    InvalidSvg,
    #[error("image artifact cannot be decoded: {0}")]
    InvalidRaster(String),
}

pub struct ViewerImageSurfaceFactory;

fn logical_extent(physical_extent: u32, content_scale: u32) -> u32 {
    let scale = u64::from(content_scale.max(1));
    ((u64::from(physical_extent) * 100).div_ceil(scale) as u32).max(1)
}

#[cfg(test)]
#[path = "image_surface_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "image_surface_cache_tests.rs"]
mod cache_tests;
