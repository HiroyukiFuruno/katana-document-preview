use super::super::super::types::{ViewerTextSpan, ViewerTextStyle};
use super::ViewerNodeClassifier;
use crate::export_surface_text::SurfaceTextParser as TextParser;
use crate::html_style::HtmlStyleProperties;

#[path = "classifier_html_spans_attributes.rs"]
mod html_attributes;
#[path = "classifier_html_spans_tag.rs"]
mod html_tag;
#[path = "classifier_html_spans_rich.rs"]
mod rich;

impl ViewerNodeClassifier {
    pub(super) fn inline_html_spans(html: &str, style: ViewerTextStyle) -> Vec<ViewerTextSpan> {
        let text = TextParser::html_fragment_text(html);
        let html_style = html_style(html, style);
        if let Some(target) = html_link_target(html) {
            return Self::linked_span(text, target, html_style);
        }
        Self::styled_span(text, html_style)
    }

    pub(super) fn html_block_spans(raw: &str, fallback: String) -> Vec<ViewerTextSpan> {
        let spans = rich::parse(raw);
        if spans.is_empty() {
            return vec![ViewerTextSpan::plain(fallback)];
        }
        spans
    }
}

fn html_link_target(html: &str) -> Option<String> {
    let html = html.trim_start();
    let tag_end = html_tag::end(html, 0)?;
    let tag = &html[..=tag_end];
    let html_tag::HtmlTag::Opening { name, .. } = html_tag::parse(tag) else {
        return None;
    };
    if name != "a" || !html_attributes::is_exact_anchor_tag(tag) {
        return None;
    }
    html_attributes::html_attribute_value(tag, "href")
}

fn html_style(html: &str, style: ViewerTextStyle) -> ViewerTextStyle {
    let properties = HtmlStyleProperties::from_fragment(html);
    let mut style = style;
    if properties.inline_code {
        style = style.inline_code();
    }
    if properties.bold {
        style = style.bold();
    }
    if properties.italic {
        style = style.italic();
    }
    if properties.underline {
        style = style.underline();
    }
    if properties.highlight {
        style = style.highlight();
    }
    if properties.strikethrough {
        style = style.strikethrough();
    }
    if let Some(color) = properties.color_rgba {
        style = style.color_rgba(color);
    }
    apply_boolean_overrides(style, properties)
}

fn apply_boolean_overrides(
    mut style: ViewerTextStyle,
    properties: HtmlStyleProperties,
) -> ViewerTextStyle {
    if let Some(bold) = properties.bold_override {
        style.bold = bold;
    }
    if let Some(italic) = properties.italic_override {
        style.italic = italic;
    }
    if let Some(underline) = properties.underline_override {
        style.underline = underline;
    }
    if let Some(strikethrough) = properties.strikethrough_override {
        style.strikethrough = strikethrough;
    }
    style
}
