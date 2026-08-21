use super::{DecodedRgbSurface, RGB_CHANNELS};

pub(super) struct PdfPageGapNormalizer;

impl PdfPageGapNormalizer {
    pub(super) fn normalize(candidate: &DecodedRgbSurface, target_height: u32) -> Option<Vec<u8>> {
        let extra_rows = candidate.height.checked_sub(target_height)? as usize;
        let row_bytes = candidate.width as usize * RGB_CHANNELS;
        let mut skipped = vec![false; candidate.height as usize];
        Self::mark_rows(candidate, row_bytes, extra_rows, &mut skipped)?;
        Some(Self::copy_rows(candidate, row_bytes, &skipped))
    }

    fn mark_rows(
        candidate: &DecodedRgbSurface,
        row_bytes: usize,
        mut remaining: usize,
        skipped: &mut [bool],
    ) -> Option<()> {
        for run in Self::blank_runs(candidate, row_bytes) {
            if remaining == 0 {
                return Some(());
            }
            let count = run.len.min(remaining);
            let start = run.end - count;
            for skip in skipped.iter_mut().take(run.end).skip(start) {
                *skip = true;
            }
            remaining -= count;
        }
        (remaining == 0).then_some(())
    }

    fn blank_runs(candidate: &DecodedRgbSurface, row_bytes: usize) -> Vec<BlankRowRun> {
        let background = Self::background(candidate, row_bytes);
        let mut runs = Self::collect_blank_runs(candidate, row_bytes, background);
        runs.sort_by_key(|run| std::cmp::Reverse(run.len));
        runs
    }

    fn collect_blank_runs(
        candidate: &DecodedRgbSurface,
        row_bytes: usize,
        background: [u8; RGB_CHANNELS],
    ) -> Vec<BlankRowRun> {
        let mut runs = Vec::new();
        let mut start = None;
        for row in 0..candidate.height as usize {
            if Self::is_blank_row(candidate, row_bytes, row, background) {
                start.get_or_insert(row);
                continue;
            }
            Self::push_run(&mut runs, start.take(), row);
        }
        Self::push_run(&mut runs, start, candidate.height as usize);
        runs
    }

    fn push_run(runs: &mut Vec<BlankRowRun>, start: Option<usize>, end: usize) {
        if let Some(start) = start {
            runs.push(BlankRowRun {
                end,
                len: end - start,
            });
        }
    }

    fn is_blank_row(
        candidate: &DecodedRgbSurface,
        row_bytes: usize,
        row: usize,
        background: [u8; RGB_CHANNELS],
    ) -> bool {
        let start = row * row_bytes;
        let end = start + row_bytes;
        candidate.rgb[start..end]
            .as_chunks::<RGB_CHANNELS>()
            .0
            .iter()
            .all(|pixel| pixel == &background)
    }

    fn background(candidate: &DecodedRgbSurface, row_bytes: usize) -> [u8; RGB_CHANNELS] {
        let mut counts = Vec::new();
        for row in 0..candidate.height as usize {
            if let Some(color) = Self::uniform_row_color(candidate, row_bytes, row) {
                Self::increment_color_count(&mut counts, color);
            }
        }
        counts.into_iter().max_by_key(|entry| entry.count).map_or(
            [candidate.rgb[0], candidate.rgb[1], candidate.rgb[2]],
            |entry| entry.color,
        )
    }

    fn uniform_row_color(
        candidate: &DecodedRgbSurface,
        row_bytes: usize,
        row: usize,
    ) -> Option<[u8; RGB_CHANNELS]> {
        let start = row * row_bytes;
        let end = start + row_bytes;
        let color = [
            candidate.rgb[start],
            candidate.rgb[start + 1],
            candidate.rgb[start + 2],
        ];
        candidate.rgb[start..end]
            .as_chunks::<RGB_CHANNELS>()
            .0
            .iter()
            .all(|pixel| pixel == &color)
            .then_some(color)
    }

    fn increment_color_count(counts: &mut Vec<BlankColorCount>, color: [u8; RGB_CHANNELS]) {
        if let Some(entry) = counts.iter_mut().find(|entry| entry.color == color) {
            entry.count += 1;
            return;
        }
        counts.push(BlankColorCount { color, count: 1 });
    }

    fn copy_rows(candidate: &DecodedRgbSurface, row_bytes: usize, skipped: &[bool]) -> Vec<u8> {
        let mut rgb = Vec::with_capacity(candidate.rgb.len());
        for (row, skip) in skipped.iter().enumerate() {
            if *skip {
                continue;
            }
            let start = row * row_bytes;
            rgb.extend_from_slice(&candidate.rgb[start..start + row_bytes]);
        }
        rgb
    }
}

struct BlankRowRun {
    end: usize,
    len: usize,
}

struct BlankColorCount {
    color: [u8; RGB_CHANNELS],
    count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_stops_after_requested_extra_rows_are_removed() {
        let candidate = DecodedRgbSurface {
            width: 1,
            height: 3,
            rgb: vec![0, 0, 0, 1, 2, 3, 0, 0, 0],
        };

        let length = PdfPageGapNormalizer::normalize(&candidate, 2).map(|rgb| rgb.len());

        assert_eq!(Some(RGB_CHANNELS * 2), length);
    }
}
