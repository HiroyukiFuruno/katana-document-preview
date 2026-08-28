use self::text_width::SpanTextWidthMeasurer;
use super::super::types::ViewerTextSpan;
use crate::viewer::settings_update::ViewerTypographyConfig;

const MIN_LINE_WRAP_WIDTH: u32 = 120;
const COMPACT_BODY_FONT_SIZE: f32 = 14.0;
const COMPACT_BODY_RASTER_SCALE: f32 = 1.25;

pub(super) struct SpanLineCounter {
    max_width: u32,
    font_size: f32,
    line_count: usize,
    current_width: u32,
    has_current_line: bool,
}

impl SpanLineCounter {
    pub(super) fn count(
        spans: &[ViewerTextSpan],
        max_width: u32,
        typography: ViewerTypographyConfig,
    ) -> usize {
        let mut counter = Self::new(max_width, typography);
        for span in spans {
            counter.push_span(span);
        }
        counter.finish()
    }

    fn new(max_width: u32, typography: ViewerTypographyConfig) -> Self {
        Self {
            max_width: max_width.max(MIN_LINE_WRAP_WIDTH),
            font_size: calibrated_document_font_size(typography),
            line_count: 0,
            current_width: 0,
            has_current_line: false,
        }
    }

    fn push_span(&mut self, span: &ViewerTextSpan) {
        let mut current = String::new();
        for character in span.text.chars() {
            if character == '\n' {
                if !current.is_empty() {
                    self.push_segment(span, &current);
                    current.clear();
                }
                self.start_new_line();
                continue;
            }
            current.push(character);
            if character.is_whitespace() {
                self.push_segment(span, &current);
                current.clear();
            }
        }
        if !current.is_empty() {
            self.push_segment(span, &current);
        }
    }

    fn push_segment(&mut self, span: &ViewerTextSpan, text: &str) {
        let segment_width = SpanTextWidthMeasurer::cached_width(span, text, self.font_size);
        if self.should_start_new_line(segment_width) {
            self.start_new_line();
        }
        if self.current_width == 0 && text.trim().is_empty() {
            return;
        }
        self.current_width += segment_width;
        self.has_current_line = true;
    }

    fn finish(mut self) -> usize {
        self.start_new_line();
        self.line_count.max(1)
    }

    fn should_start_new_line(&self, segment_width: u32) -> bool {
        self.current_width > 0 && self.current_width + segment_width > self.max_width
    }

    fn start_new_line(&mut self) {
        if !self.has_current_line {
            return;
        }
        self.line_count += 1;
        self.current_width = 0;
        self.has_current_line = false;
    }
}

fn calibrated_document_font_size(typography: ViewerTypographyConfig) -> f32 {
    let font_size = f32::from(typography.preview_font_size);
    if font_size <= COMPACT_BODY_FONT_SIZE {
        return font_size * COMPACT_BODY_RASTER_SCALE;
    }
    font_size
}

#[path = "builder_span_text_width.rs"]
mod text_width;

#[cfg(test)]
mod tests {
    use super::SpanLineCounter;
    use crate::{ViewerTextSpan, ViewerTypographyConfig};

    #[test]
    fn line_count_scales_with_preview_font_size() {
        let spans = vec![ViewerTextSpan::plain(
            "alpha beta gamma delta epsilon zeta eta theta",
        )];
        let small = SpanLineCounter::count(&spans, 180, typography(12));
        let large = SpanLineCounter::count(&spans, 180, typography(24));

        assert!(small < large, "small={small} large={large}");
    }

    #[test]
    fn line_count_honors_explicit_newlines_inside_spans() {
        let spans = vec![ViewerTextSpan::plain("first\nsecond\nthird")];

        let count = SpanLineCounter::count(&spans, 600, typography(14));

        assert_eq!(3, count);
    }

    #[test]
    fn katana_sample_centering_note_stays_single_line_at_preview_font_14() {
        let spans = vec![ViewerTextSpan::plain(
            "↑ \"English | 日本語\" should appear on the same line, centered.",
        )];

        let count = SpanLineCounter::count(&spans, 1248, typography(14));

        assert_eq!(1, count);
    }

    #[test]
    fn platform_measurement_keeps_japanese_and_zwj_emoji_on_the_same_line_when_it_fits() {
        let spans = vec![ViewerTextSpan::plain("日本語 ⭐️ 🧑‍💻 text")];

        let count = SpanLineCounter::count(&spans, 640, typography(20));

        assert_eq!(1, count);
    }

    fn typography(preview_font_size: u16) -> ViewerTypographyConfig {
        ViewerTypographyConfig { preview_font_size }
    }
}
