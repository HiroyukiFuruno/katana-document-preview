use super::{OfficeDocumentFormat, ViewerCapabilities, ViewerDiagnostic, ViewerSourceIdentity};
use crate::ViewerImageSurface;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfPageRotation {
    None,
    Clockwise90,
    Clockwise180,
    Clockwise270,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfPageArtifact {
    pub index: usize,
    pub width: f32,
    pub height: f32,
    pub rotation: PdfPageRotation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfDocumentArtifact {
    pub identity: ViewerSourceIdentity,
    pub mime: String,
    pub page_count: usize,
    pub pages: Vec<PdfPageArtifact>,
    pub capabilities: ViewerCapabilities,
    pub diagnostics: Vec<ViewerDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PdfPageRenderRequest {
    pub page_index: usize,
    pub scale: f32,
}

impl PdfPageRenderRequest {
    #[must_use]
    pub const fn new(page_index: usize, scale: f32) -> Self {
        Self { page_index, scale }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfRenderedPage {
    pub page_index: usize,
    pub scale: f32,
    pub surface: ViewerImageSurface,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfResourceLimitKind {
    SourceBytes,
    PageCount,
    RenderDimension,
    RenderPixels,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PdfViewerLimits {
    pub max_source_bytes: usize,
    pub max_pages: usize,
    pub max_render_dimension: u32,
    pub max_render_pixels: u64,
    pub max_cached_pages: usize,
    pub max_cached_bytes: usize,
}

impl PdfViewerLimits {
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            max_source_bytes: 256 * 1024 * 1024,
            max_pages: 10_000,
            max_render_dimension: 8_192,
            max_render_pixels: 16_777_216,
            max_cached_pages: 4,
            max_cached_bytes: 128 * 1024 * 1024,
        }
    }
}

#[cfg(test)]
#[path = "artifact_tests.rs"]
mod tests;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfficeStaticItemArtifact {
    pub index: usize,
    pub width: f32,
    pub height: f32,
    pub rotation: PdfPageRotation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfficeStaticDocumentArtifact {
    pub identity: ViewerSourceIdentity,
    pub format: OfficeDocumentFormat,
    pub mime: String,
    pub item_count: usize,
    pub items: Vec<OfficeStaticItemArtifact>,
    pub capabilities: ViewerCapabilities,
    pub diagnostics: Vec<ViewerDiagnostic>,
}
