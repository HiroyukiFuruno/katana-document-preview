use std::collections::BTreeMap;

use super::{PERCENT_SCALE, RGB_CHANNELS, ZERO_SCORE};

const COLOR_BUCKET_WIDTH: u8 = 16;
const CONTENT_DISTANCE_THRESHOLD: u16 = 64;
const PRESERVED_DISTANCE_TOLERANCE: u16 = 96;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SurfaceColorBucket {
    red_sum: u64,
    green_sum: u64,
    blue_sum: u64,
    count: u64,
}

impl SurfaceColorBucket {
    fn new(pixel: &[u8]) -> Self {
        Self {
            red_sum: pixel[0] as u64,
            green_sum: pixel[1] as u64,
            blue_sum: pixel[2] as u64,
            count: 1,
        }
    }

    fn add(&mut self, pixel: &[u8]) {
        self.red_sum += pixel[0] as u64;
        self.green_sum += pixel[1] as u64;
        self.blue_sum += pixel[2] as u64;
        self.count += 1;
    }

    fn color(&self) -> [u8; RGB_CHANNELS] {
        [
            (self.red_sum / self.count) as u8,
            (self.green_sum / self.count) as u8,
            (self.blue_sum / self.count) as u8,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SurfaceContentStats {
    content_pixels: u64,
    preserved_pixels: u64,
}

impl SurfaceContentStats {
    fn score(&self) -> u8 {
        if self.content_pixels == 0 {
            return 100;
        }
        let ratio = self.preserved_pixels as f64 / self.content_pixels as f64;
        (ratio * PERCENT_SCALE).round() as u8
    }
}

pub(super) struct SurfaceContentScorer;

impl SurfaceContentScorer {
    pub(super) fn score(reference: &[u8], candidate: &[u8]) -> u8 {
        if reference.is_empty() || reference.len() != candidate.len() {
            return ZERO_SCORE;
        }
        let reference_score = Self::one_direction_score(reference, candidate);
        let candidate_score = Self::one_direction_score(candidate, reference);
        reference_score.min(candidate_score)
    }

    fn one_direction_score(reference: &[u8], candidate: &[u8]) -> u8 {
        let background = Self::background_color(reference);
        let stats = Self::content_stats(reference, candidate, background);
        stats.score()
    }

    fn background_color(rgb: &[u8]) -> [u8; RGB_CHANNELS] {
        let mut buckets = BTreeMap::new();
        for pixel in rgb.as_chunks::<RGB_CHANNELS>().0 {
            Self::append_bucket(&mut buckets, pixel);
        }
        let Some(bucket) = buckets.values().max_by_key(|bucket| bucket.count) else {
            return [0, 0, 0];
        };
        bucket.color()
    }

    fn append_bucket(buckets: &mut BTreeMap<[u8; RGB_CHANNELS], SurfaceColorBucket>, pixel: &[u8]) {
        let key = [
            pixel[0] / COLOR_BUCKET_WIDTH,
            pixel[1] / COLOR_BUCKET_WIDTH,
            pixel[2] / COLOR_BUCKET_WIDTH,
        ];
        match buckets.get_mut(&key) {
            Some(bucket) => bucket.add(pixel),
            None => {
                buckets.insert(key, SurfaceColorBucket::new(pixel));
            }
        }
    }

    fn content_stats(
        reference: &[u8],
        candidate: &[u8],
        background: [u8; RGB_CHANNELS],
    ) -> SurfaceContentStats {
        let mut stats = SurfaceContentStats {
            content_pixels: 0,
            preserved_pixels: 0,
        };
        for (reference_pixel, candidate_pixel) in reference
            .as_chunks::<RGB_CHANNELS>()
            .0
            .iter()
            .zip(candidate.as_chunks::<RGB_CHANNELS>().0)
        {
            Self::record_content_pixel(&mut stats, reference_pixel, candidate_pixel, background);
        }
        stats
    }

    fn record_content_pixel(
        stats: &mut SurfaceContentStats,
        reference_pixel: &[u8],
        candidate_pixel: &[u8],
        background: [u8; RGB_CHANNELS],
    ) {
        if Self::pixel_distance(reference_pixel, &background) < CONTENT_DISTANCE_THRESHOLD {
            return;
        }
        stats.content_pixels += 1;
        if Self::pixel_distance(reference_pixel, candidate_pixel) <= PRESERVED_DISTANCE_TOLERANCE {
            stats.preserved_pixels += 1;
        }
    }

    fn pixel_distance(left: &[u8], right: &[u8]) -> u16 {
        left[0].abs_diff(right[0]) as u16
            + left[1].abs_diff(right[1]) as u16
            + left[2].abs_diff(right[2]) as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_zero_when_sizes_mismatch() {
        assert_eq!(SurfaceContentScorer::score(&[1, 2, 3], &[1, 2, 3, 4]), 0);
    }

    #[test]
    fn returns_zero_for_empty_reference() {
        assert_eq!(SurfaceContentScorer::score(&[], &[]), 0);
    }

    #[test]
    fn empty_pixels_use_black_as_the_background_fallback() {
        assert_eq!([0, 0, 0], SurfaceContentScorer::background_color(&[]));
    }

    #[test]
    fn returns_perfect_score_when_reference_and_candidate_match() {
        let reference = vec![255, 0, 0, 0, 255, 0];
        let candidate = reference.clone();

        assert_eq!(SurfaceContentScorer::score(&reference, &candidate), 100);
    }

    #[test]
    fn returns_symmetric_partial_match_score() {
        let reference = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, // background
            200, 0, 0, 200, 0, 0, // content
        ];
        let candidate = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, // background
            180, 0, 0, // preserved content
            0, 0, 0, // dropped content
        ];

        assert_eq!(SurfaceContentScorer::score(&reference, &candidate), 50);
    }

    #[test]
    fn treats_dominant_color_as_background_if_no_content() {
        let reference = vec![0, 0, 0, 0, 0, 0, 5, 5, 5];
        let candidate = vec![255, 0, 0, 255, 0, 0, 255, 0, 0];

        assert_eq!(SurfaceContentScorer::score(&reference, &candidate), 100);
    }
}
