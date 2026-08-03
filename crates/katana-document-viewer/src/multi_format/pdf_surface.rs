use super::{PdfDocumentArtifact, PdfPageRenderRequest, PdfViewerError};
use crate::ViewerImageSurface;
use image::ImageFormat;

pub(crate) struct PdfSurfaceDecoder;

impl PdfSurfaceDecoder {
    pub(crate) fn decode(
        artifact: &PdfDocumentArtifact,
        request: PdfPageRenderRequest,
        png: &[u8],
    ) -> Result<ViewerImageSurface, PdfViewerError> {
        let mut rgba = image::load_from_memory_with_format(png, ImageFormat::Png)
            .map_err(|_| PdfViewerError::RenderDecode)?
            .into_rgba8();
        for pixel in rgba.pixels_mut() {
            flatten_pixel(pixel.0.as_mut_slice());
        }
        Ok(ViewerImageSurface {
            fingerprint: format!(
                "pdf:{}:{}:{}",
                artifact.identity.revision,
                request.page_index,
                request.scale.to_bits()
            ),
            width: rgba.width(),
            height: rgba.height(),
            display_width: rgba.width() as f32,
            display_height: rgba.height() as f32,
            content_scale: 100,
            rgba: rgba.into_raw(),
        })
    }
}

fn flatten_pixel(pixel: &mut [u8]) {
    let alpha = u16::from(pixel[3]);
    for channel in &mut pixel[..3] {
        let value = u16::from(*channel);
        *channel = ((value * alpha + 255 * (255 - alpha) + 127) / 255) as u8;
    }
    pixel[3] = 255;
}
