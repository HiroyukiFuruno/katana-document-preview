use super::RGB_CHANNELS;

const RGBA_CHANNELS: usize = 4;
const ALPHA_INDEX: usize = 3;
const OPAQUE_ALPHA: u8 = 255;

pub(super) type RgbSurfacePair = (Vec<u8>, Vec<u8>);

pub(super) struct SurfacePixels;

impl SurfacePixels {
    pub(super) fn rgba_pair_to_rgb(
        reference: &[u8],
        candidate: &[u8],
    ) -> Result<RgbSurfacePair, ()> {
        if reference.len() != candidate.len()
            || !reference.len().is_multiple_of(RGBA_CHANNELS)
            || !candidate.len().is_multiple_of(RGBA_CHANNELS)
        {
            return Err(());
        }
        let reference_background = Self::surface_background_rgb(reference);
        let candidate_background = Self::surface_background_rgb(candidate);
        let reference_composite_background = candidate_background
            .or(reference_background)
            .unwrap_or([0, 0, 0]);
        let candidate_composite_background = reference_background
            .or(candidate_background)
            .unwrap_or([0, 0, 0]);
        let mut reference_rgb = Vec::with_capacity(reference.len() / RGBA_CHANNELS * RGB_CHANNELS);
        let mut candidate_rgb = Vec::with_capacity(candidate.len() / RGBA_CHANNELS * RGB_CHANNELS);
        for (reference_pixel, candidate_pixel) in reference
            .as_chunks::<RGBA_CHANNELS>()
            .0
            .iter()
            .zip(candidate.as_chunks::<RGBA_CHANNELS>().0)
        {
            Self::push_composited_rgb(
                &mut reference_rgb,
                reference_pixel,
                reference_composite_background,
            );
            Self::push_composited_rgb(
                &mut candidate_rgb,
                candidate_pixel,
                candidate_composite_background,
            );
        }
        Ok((reference_rgb, candidate_rgb))
    }

    fn surface_background_rgb(rgba: &[u8]) -> Option<[u8; RGB_CHANNELS]> {
        let first = rgba.first_chunk::<RGBA_CHANNELS>()?;
        (first[ALPHA_INDEX] > 0).then(|| [first[0], first[1], first[2]])
    }

    fn push_composited_rgb(rgb: &mut Vec<u8>, pixel: &[u8], background: [u8; RGB_CHANNELS]) {
        let alpha = pixel[ALPHA_INDEX];
        if alpha == OPAQUE_ALPHA {
            rgb.extend_from_slice(&pixel[..RGB_CHANNELS]);
            return;
        }
        for (channel, background_channel) in pixel[..RGB_CHANNELS].iter().zip(background.iter()) {
            rgb.push(Self::composite_channel(
                *channel,
                *background_channel,
                alpha,
            ));
        }
    }

    fn composite_channel(foreground: u8, background: u8, alpha: u8) -> u8 {
        let foreground = u32::from(foreground);
        let background = u32::from(background);
        let alpha = u32::from(alpha);
        ((foreground * alpha + background * (u32::from(OPAQUE_ALPHA) - alpha) + 127)
            / u32::from(OPAQUE_ALPHA)) as u8
    }

    pub(super) fn crop_rgba(
        rgba: &[u8],
        source_width: usize,
        width: usize,
        height: usize,
    ) -> Vec<u8> {
        if width == 0 || height == 0 || source_width == 0 {
            return Vec::new();
        }
        let mut cropped = Vec::with_capacity(width * height * RGBA_CHANNELS);
        for row in 0..height {
            let start = row * source_width * RGBA_CHANNELS;
            let end = start + width * RGBA_CHANNELS;
            let Some(slice) = rgba.get(start..end) else {
                return Vec::new();
            };
            cropped.extend_from_slice(slice);
        }
        cropped
    }
}
