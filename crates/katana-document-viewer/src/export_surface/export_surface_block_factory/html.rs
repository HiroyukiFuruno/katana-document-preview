use crate::export_surface_text::SurfaceTextParser;
use crate::forge::BuildGraph;
use crate::html_sanitizer::HtmlFragmentNormalizer;
use crate::theme::KdvThemeSnapshot;
use katana_markdown_model::{HtmlBlockRole, KmmNode};

use super::super::markup::SurfaceHtmlMarkup;
use super::super::{SurfaceBadgeRowBlock, SurfaceBlock};
use super::SurfaceBlockFactory;
use crate::export_surface_helpers::{BODY_MAX_CHARS, WrappedText};
use crate::export_surface_line::SurfaceLine;
use crate::export_surface_span::SurfaceTextSpan;

impl SurfaceBlockFactory {
    pub(super) fn append_html(
        blocks: &mut Vec<SurfaceBlock>,
        graph: &BuildGraph,
        node: &KmmNode,
        role: &HtmlBlockRole,
        quote_depth: u32,
        list_depth: u32,
        theme: &KdvThemeSnapshot,
    ) {
        let fragment = HtmlFragmentNormalizer::normalize(&node.source.raw.text);
        if Self::append_special_html(blocks, graph, &fragment, quote_depth, list_depth, theme) {
            return;
        }
        if matches!(role, HtmlBlockRole::BadgeRow) {
            Self::append_badge_row(blocks, &fragment);
            return;
        }
        Self::append_textual_html(blocks, role, &fragment, quote_depth, list_depth);
    }

    fn append_textual_html(
        blocks: &mut Vec<SurfaceBlock>,
        role: &HtmlBlockRole,
        fragment: &str,
        quote_depth: u32,
        list_depth: u32,
    ) {
        let text = Self::normalized_html_text(fragment);
        if Self::append_heading_html(blocks, fragment, text.clone()) {
            return;
        }
        if Self::is_centered_html(role, fragment) {
            Self::append_centered_html(blocks, fragment, text);
            return;
        }
        if SurfaceHtmlMarkup::has_right_alignment(fragment) {
            Self::append_right_html(blocks, fragment);
            return;
        }
        if Self::append_rich_html(blocks, fragment, quote_depth, list_depth) {
            return;
        }
        Self::append_wrapped(blocks, text, quote_depth, list_depth);
    }

    fn append_heading_html(blocks: &mut Vec<SurfaceBlock>, fragment: &str, text: String) -> bool {
        if Self::heading_level(fragment).is_none() {
            return false;
        }
        let spans = Self::html_heading_spans(fragment, text);
        let line = if SurfaceHtmlMarkup::has_right_alignment(fragment) {
            SurfaceLine::right_spans(spans)
        } else if SurfaceHtmlMarkup::has_center_alignment(fragment) {
            SurfaceLine::centered_spans(spans)
        } else {
            SurfaceLine::body_spans(spans, 0)
        };
        blocks.push(SurfaceBlock::Line(line));
        true
    }

    fn html_heading_spans(fragment: &str, text: String) -> Vec<SurfaceTextSpan> {
        let spans = SurfaceHtmlMarkup::html_spans(fragment);
        if !spans.is_empty() {
            return spans;
        }
        vec![SurfaceTextSpan::plain(text)]
    }

    fn heading_level(fragment: &str) -> Option<u8> {
        let normalized = fragment.trim_start().to_ascii_lowercase();
        for level in 1..=6 {
            if normalized.starts_with(&format!("<h{level}")) {
                return Some(level);
            }
        }
        None
    }

    fn append_special_html(
        blocks: &mut Vec<SurfaceBlock>,
        graph: &BuildGraph,
        fragment: &str,
        quote_depth: u32,
        list_depth: u32,
        theme: &KdvThemeSnapshot,
    ) -> bool {
        if Self::append_details(blocks, graph, fragment, quote_depth, list_depth, theme) {
            return true;
        }
        if let Some(local_image) = Self::local_html_image_block(graph, fragment) {
            blocks.push(SurfaceBlock::Image(local_image));
            return true;
        }
        if let Some(data_image) = Self::data_html_image_block(fragment) {
            blocks.push(SurfaceBlock::Image(data_image));
            return true;
        }
        false
    }

    fn append_badge_row(blocks: &mut Vec<SurfaceBlock>, fragment: &str) {
        blocks.push(SurfaceBlock::BadgeRow(SurfaceBadgeRowBlock::new(
            SurfaceHtmlMarkup::badge_row_badges(fragment),
        )));
    }

    fn normalized_html_text(fragment: &str) -> String {
        SurfaceHtmlMarkup::normalize_text(&SurfaceTextParser::html_fragment_text(fragment))
    }

    fn is_centered_html(role: &HtmlBlockRole, fragment: &str) -> bool {
        matches!(role, HtmlBlockRole::Centered) || SurfaceHtmlMarkup::has_center_alignment(fragment)
    }

    fn append_centered_html(blocks: &mut Vec<SurfaceBlock>, fragment: &str, text: String) {
        let spans = SurfaceHtmlMarkup::centered_html_spans(fragment);
        if !spans.is_empty() && text.chars().count() <= BODY_MAX_CHARS {
            blocks.push(SurfaceBlock::Line(SurfaceLine::centered_spans(spans)));
            return;
        }
        for chunk in WrappedText::new(&text, BODY_MAX_CHARS) {
            blocks.push(SurfaceBlock::Line(SurfaceLine::body_centered(chunk)));
        }
    }

    fn append_right_html(blocks: &mut Vec<SurfaceBlock>, fragment: &str) {
        let spans = SurfaceHtmlMarkup::html_spans(fragment);
        if !spans.is_empty() {
            blocks.push(SurfaceBlock::Line(SurfaceLine::right_spans(spans)));
        }
    }

    fn append_rich_html(
        blocks: &mut Vec<SurfaceBlock>,
        fragment: &str,
        quote_depth: u32,
        list_depth: u32,
    ) -> bool {
        let spans = SurfaceHtmlMarkup::html_spans(fragment);
        if !Self::has_rich_span(&spans) {
            return false;
        }
        for line_spans in super::text::SurfaceInlineLineWrapper::wrap(
            spans,
            Self::line_width(quote_depth, list_depth),
        ) {
            Self::append_rich_line_spans(blocks, line_spans, quote_depth, list_depth);
        }
        true
    }

    fn has_rich_span(spans: &[SurfaceTextSpan]) -> bool {
        spans.iter().any(|span| !span.is_plain())
    }
}

#[path = "html_image_block.rs"]
mod html_image_block;

#[cfg(test)]
#[path = "html_image_tests.rs"]
mod image_tests;

#[cfg(test)]
#[path = "html_link_tests.rs"]
mod link_tests;

#[cfg(test)]
#[path = "html_tests.rs"]
mod tests;
