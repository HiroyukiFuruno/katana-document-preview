use super::SurfaceBadge;
use crate::export_surface_span::SurfaceTextSpan;
use crate::export_surface_text::SurfaceTextParser;

#[path = "export_surface_markup_html_attributes.rs"]
pub(super) mod attributes;
#[path = "export_surface_markup_badge.rs"]
mod badge;
#[path = "export_surface_markup_html_spans.rs"]
mod spans;
#[path = "export_surface_markup_html_spans_output.rs"]
mod spans_output;

use attributes::attribute_value;

pub(crate) struct SurfaceHtmlMarkup;

impl SurfaceHtmlMarkup {
    pub(crate) fn normalize_text(text: &str) -> String {
        let decoded = SurfaceTextParser::decode_basic_entities(text);
        let compact = decoded.split_whitespace().collect::<Vec<_>>().join(" ");
        compact
            .replace(" | ", "|")
            .replace(" |", "|")
            .replace("| ", "|")
            .replace('|', " | ")
    }

    pub(crate) fn badge_row_badges(fragment: &str) -> Vec<SurfaceBadge> {
        let badges = Self::extract_img_refs(fragment)
            .into_iter()
            .filter_map(|image| badge::shields_badge(&image.src, image.link_target))
            .collect::<Vec<_>>();
        if !badges.is_empty() {
            return badges;
        }
        let alt_text = SurfaceTextParser::html_fragment_text(fragment);
        let normalized = Self::normalize_text(&alt_text);
        if normalized.is_empty() {
            Vec::new()
        } else {
            vec![SurfaceBadge::single(normalized)]
        }
    }

    pub(crate) fn extract_img_refs(fragment: &str) -> Vec<SurfaceHtmlImageRef> {
        let mut images = Vec::new();
        let mut rest = fragment;
        while let Some(img_start) = rest.find("<img") {
            let link_target = enclosing_link_target(&rest[..img_start]);
            let after_img = &rest[img_start..];
            let Some(img_end) = html_tag_end(after_img) else {
                break;
            };
            let tag = &after_img[..=img_end];
            if let Some(src) = attribute_value(tag, "src") {
                let alt = attribute_value(tag, "alt").unwrap_or_else(empty_attribute_value);
                images.push(SurfaceHtmlImageRef {
                    src,
                    alt,
                    width: attribute_value(tag, "width").and_then(|value| value.parse().ok()),
                    link_target,
                });
            }
            rest = &after_img[img_end + 1..];
        }
        images
    }

    pub(crate) fn has_center_alignment(fragment: &str) -> bool {
        let normalized = alignment_source(fragment);
        normalized.contains("align=\"center\"")
            || normalized.contains("align=center")
            || normalized.contains("text-align:center")
    }

    pub(crate) fn has_right_alignment(fragment: &str) -> bool {
        let normalized = alignment_source(fragment);
        normalized.contains("align=\"right\"")
            || normalized.contains("align=right")
            || normalized.contains("text-align:right")
    }

    pub(crate) fn starts_with_block_tag(fragment: &str) -> bool {
        let lower = fragment.trim_start().to_ascii_lowercase();
        [
            "<h1", "<h2", "<h3", "<h4", "<h5", "<h6", "<p", "<div", "<section", "<article",
        ]
        .iter()
        .any(|tag| lower.starts_with(tag))
    }

    pub(crate) fn html_spans(fragment: &str) -> Vec<SurfaceTextSpan> {
        spans::html_spans(fragment)
    }

    pub(crate) fn centered_html_spans(fragment: &str) -> Vec<SurfaceTextSpan> {
        Self::html_spans(fragment)
    }
}

pub(crate) struct SurfaceHtmlImageRef {
    pub(crate) src: String,
    pub(crate) alt: String,
    pub(crate) width: Option<u32>,
    pub(crate) link_target: Option<String>,
}

#[cfg(test)]
#[path = "export_surface_markup_html_spans_tests.rs"]
mod spans_tests;
#[cfg(test)]
#[path = "export_surface_markup_html_tests.rs"]
mod tests;

fn enclosing_link_target(prefix: &str) -> Option<String> {
    let link_start = prefix.rfind("<a")?;
    attribute_value(&prefix[link_start..], "href")
}

fn html_tag_end(fragment: &str) -> Option<usize> {
    let mut quote = None;
    for (index, character) in fragment.char_indices() {
        match quote {
            Some(delimiter) if character == delimiter => quote = None,
            Some(_) => {}
            None if character == '"' || character == '\'' => quote = Some(character),
            None if character == '>' => return Some(index),
            None => {}
        }
    }
    None
}

fn empty_attribute_value() -> String {
    String::new()
}

fn alignment_source(fragment: &str) -> String {
    fragment
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<String>()
        .replace('\'', "\"")
}
