use super::{RGB_CHANNELS, RGBA_CHANNELS};

pub(super) struct SurfaceRgbConverter;

impl SurfaceRgbConverter {
    pub(super) fn rgba_to_rgb(rgba: &[u8]) -> Result<Vec<u8>, String> {
        if !rgba.len().is_multiple_of(RGBA_CHANNELS) {
            return Err("rgba byte length is not divisible by 4".to_string());
        }
        let mut rgb = Vec::with_capacity(rgba.len() / RGBA_CHANNELS * RGB_CHANNELS);
        for pixel in rgba.as_chunks::<RGBA_CHANNELS>().0 {
            rgb.extend_from_slice(&pixel[..RGB_CHANNELS]);
        }
        Ok(rgb)
    }
}
