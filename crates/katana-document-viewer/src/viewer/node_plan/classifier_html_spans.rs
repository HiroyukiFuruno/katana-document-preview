use super::super::super::types::{ViewerTextSpan, ViewerTextStyle};
use super::ViewerNodeClassifier;
use crate::export_surface_text::SurfaceTextParser as TextParser;
use crate::html_style::HtmlStyleProperties;

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
        let spans = Self::html_link_spans(raw);
        if spans.is_empty() {
            return vec![ViewerTextSpan::plain(fallback)];
        }
        spans
    }

    fn html_link_spans(raw: &str) -> Vec<ViewerTextSpan> {
        let lower = raw.to_ascii_lowercase();
        let mut cursor = 0;
        let mut spans = Vec::new();
        while let Some(segment) = next_html_link_segment(raw, &lower, cursor) {
            Self::push_html_plain(&raw[cursor..segment.link_start], &mut spans);
            Self::push_html_link(&segment, raw, &mut spans);
            cursor = segment.next_cursor;
        }
        Self::push_html_plain(&raw[cursor..], &mut spans);
        spans
    }

    fn push_html_plain(raw: &str, spans: &mut Vec<ViewerTextSpan>) {
        let text = TextParser::html_fragment_text(raw);
        if text.is_empty() {
            return;
        }
        spans.extend(Self::plain_span(
            &text,
            html_style(raw, ViewerTextStyle::default()),
        ));
    }

    fn push_html_link(segment: &HtmlLinkSegment<'_>, raw: &str, spans: &mut Vec<ViewerTextSpan>) {
        let target = html_link_target(segment.tag);
        let text = TextParser::html_fragment_text(segment.body);
        let style = html_style(segment.tag, html_style(raw, ViewerTextStyle::default()));
        if let Some(target) = target {
            spans.extend(Self::linked_span(text, target, style));
        } else {
            spans.extend(Self::plain_span(&text, style));
        }
    }
}

struct HtmlLinkSegment<'a> {
    link_start: usize,
    next_cursor: usize,
    tag: &'a str,
    body: &'a str,
}

fn next_html_link_segment<'a>(
    raw: &'a str,
    lower: &str,
    cursor: usize,
) -> Option<HtmlLinkSegment<'a>> {
    let link_start = cursor + lower[cursor..].find("<a ")?;
    let tag_end = link_start + raw[link_start..].find('>')?;
    let body_start = tag_end + 1;
    let close_start = body_start + lower[body_start..].find("</a>")?;
    Some(HtmlLinkSegment {
        link_start,
        next_cursor: close_start + "</a>".len(),
        tag: &raw[link_start..=tag_end],
        body: &raw[body_start..close_start],
    })
}

fn html_link_target(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let href_index = lower.find("href")?;
    let after_href = &html[href_index + "href".len()..];
    let equals_index = after_href.find('=')?;
    let value = after_href[equals_index + 1..].trim_start();
    let quote = value.chars().next()?;
    if quote == '"' || quote == '\'' {
        let target = &value[quote.len_utf8()..];
        let end = target.find(quote)?;
        return Some(target[..end].to_string());
    }
    let end = value
        .find(|character: char| character.is_whitespace() || character == '>')
        .unwrap_or(value.len());
    Some(value[..end].trim_matches('/').to_string())
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
    style
}
