#[path = "frame_surface_content_bounds.rs"]
mod content_bounds;
#[path = "frame_surface_foreground.rs"]
mod foreground;
#[path = "frame_surface_pixels.rs"]
mod pixels;
#[path = "frame_surface_row_alignment.rs"]
mod row_alignment;

use content_bounds::ContentBounds;
use foreground::ForegroundPreservation;
use pixels::SurfacePixels;
pub(super) use row_alignment::RowLossBand;
use std::collections::BTreeMap;

const PERCENT_SCALE: f64 = 100.0;
const BYTE_MAX: f64 = 255.0;
const RGB_CHANNELS: usize = 3;
const CONTENT_DISTANCE_THRESHOLD: u16 = 30;
const AVERAGE_SCORE_FLOOR_FOR_LAYOUT: u8 = 75;

pub(super) struct SurfaceParityScorer;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct SurfaceParityReport {
    pub(super) score: u8,
    pub(super) average_score: u8,
    pub(super) content_score: u8,
    pub(super) dimension_score: u8,
    pub(super) row_score: u8,
    pub(super) reference_to_candidate_row_score: u8,
    pub(super) candidate_to_reference_row_score: u8,
    pub(super) reference_row_loss_bands: Vec<RowLossBand>,
}

impl SurfaceParityScorer {
    pub(super) fn report(
        reference: &[u8],
        candidate: &[u8],
        width: usize,
        height: usize,
    ) -> SurfaceParityReport {
        let Ok((reference_rgb, candidate_rgb)) =
            SurfacePixels::rgba_pair_to_rgb(reference, candidate)
        else {
            return SurfaceParityReport::default();
        };
        Self::score_rgb(&reference_rgb, &candidate_rgb, width, height)
    }

    pub(super) fn report_with_dimensions(
        reference: &[u8],
        candidate: &[u8],
        reference_width: usize,
        reference_height: usize,
        candidate_width: usize,
        candidate_height: usize,
    ) -> SurfaceParityReport {
        if reference_width == candidate_width && reference_height == candidate_height {
            return Self::report(reference, candidate, reference_width, reference_height);
        }
        let width = reference_width.min(candidate_width);
        let height = reference_height.min(candidate_height);
        let reference_crop = SurfacePixels::crop_rgba(reference, reference_width, width, height);
        let candidate_crop = SurfacePixels::crop_rgba(candidate, candidate_width, width, height);
        let mut report = Self::report(&reference_crop, &candidate_crop, width, height);
        report.dimension_score = Self::dimension_score(
            reference_width,
            reference_height,
            candidate_width,
            candidate_height,
        );
        report.score = report.score.min(report.dimension_score);
        report
    }

    fn score_rgb(
        reference: &[u8],
        candidate: &[u8],
        width: usize,
        height: usize,
    ) -> SurfaceParityReport {
        if reference.is_empty() || reference.len() != candidate.len() || width == 0 || height == 0 {
            return SurfaceParityReport::default();
        }
        let average_score = Self::average_score(reference, candidate);
        let content_score = Self::content_score(reference, candidate, width, height);
        let row_report = Self::row_report(reference, candidate, width, height);
        let score = if average_score < AVERAGE_SCORE_FLOOR_FOR_LAYOUT {
            average_score
        } else {
            content_score
        };
        SurfaceParityReport {
            score,
            average_score,
            content_score,
            dimension_score: 100,
            row_score: row_report.score,
            reference_to_candidate_row_score: row_report.reference_to_candidate_score,
            candidate_to_reference_row_score: row_report.candidate_to_reference_score,
            reference_row_loss_bands: row_report.reference_loss_bands,
        }
    }

    fn dimension_score(
        reference_width: usize,
        reference_height: usize,
        candidate_width: usize,
        candidate_height: usize,
    ) -> u8 {
        let width_delta = reference_width.abs_diff(candidate_width);
        let height_delta = reference_height.abs_diff(candidate_height);
        let scale = reference_width
            .max(reference_height)
            .max(candidate_width)
            .max(candidate_height)
            .max(1) as f64;
        (PERCENT_SCALE - (width_delta + height_delta) as f64 / scale * PERCENT_SCALE)
            .max(0.0)
            .round()
            .clamp(0.0, PERCENT_SCALE) as u8
    }

    fn average_score(reference: &[u8], candidate: &[u8]) -> u8 {
        let total_diff = reference
            .iter()
            .zip(candidate.iter())
            .map(|(left, right)| left.abs_diff(*right) as u64)
            .sum::<u64>();
        let max_diff = reference.len() as f64 * BYTE_MAX;
        (PERCENT_SCALE - total_diff as f64 / max_diff * PERCENT_SCALE).round() as u8
    }

    fn content_score(reference: &[u8], candidate: &[u8], width: usize, height: usize) -> u8 {
        let reference_background = Self::dominant_background(reference);
        let candidate_background = Self::dominant_background(candidate);
        let reference_bounds =
            ContentBounds::collect(reference, width, height, reference_background);
        let candidate_bounds =
            ContentBounds::collect(candidate, width, height, candidate_background);
        let bounds_score = Self::bounds_score(reference_bounds, candidate_bounds, width, height);
        let preservation_score = ForegroundPreservation::score(
            reference,
            candidate,
            width,
            height,
            reference_background,
            candidate_background,
        );
        let row_report = row_alignment::RowAlignment::report(
            reference,
            candidate,
            width,
            height,
            reference_background,
            candidate_background,
        );
        bounds_score.min(preservation_score).min(row_report.score)
    }

    fn row_report(
        reference: &[u8],
        candidate: &[u8],
        width: usize,
        height: usize,
    ) -> row_alignment::RowAlignmentReport {
        let reference_background = Self::dominant_background(reference);
        let candidate_background = Self::dominant_background(candidate);
        row_alignment::RowAlignment::report(
            reference,
            candidate,
            width,
            height,
            reference_background,
            candidate_background,
        )
    }

    fn dominant_background(rgb: &[u8]) -> [u8; RGB_CHANNELS] {
        let mut counts = BTreeMap::<[u8; RGB_CHANNELS], usize>::new();
        for pixel in rgb.as_chunks::<RGB_CHANNELS>().0 {
            let color = [pixel[0], pixel[1], pixel[2]];
            *counts.entry(color).or_insert(0) += 1;
        }
        counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(color, _)| color)
            .unwrap_or([0, 0, 0])
    }

    fn bounds_score(
        reference: Option<ContentBounds>,
        candidate: Option<ContentBounds>,
        width: usize,
        height: usize,
    ) -> u8 {
        match (reference, candidate) {
            (None, None) => 0,
            (Some(_), None) | (None, Some(_)) => 0,
            (Some(reference), Some(candidate)) => {
                let max_delta = reference.max_delta(candidate);
                let scale = width.max(height).max(1) as f64;
                (PERCENT_SCALE - max_delta as f64 / scale * PERCENT_SCALE)
                    .max(0.0)
                    .round()
                    .clamp(0.0, PERCENT_SCALE) as u8
            }
        }
    }
}

#[cfg(test)]
#[path = "frame_surface_similarity_tests.rs"]
mod tests;
