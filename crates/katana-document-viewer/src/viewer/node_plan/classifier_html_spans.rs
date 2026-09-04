use super::super::super::types::{ViewerTextSpan, ViewerTextStyle};
use super::ViewerNodeClassifier;
use crate::export_surface_text::SurfaceTextParser as TextParser;
use crate::html_style::HtmlStyleProperties;

#[path = "classifier_html_spans_tag.rs"]
mod html_tag;

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
        let spans = Self::rich_html_spans(raw);
        if spans.is_empty() {
            return vec![ViewerTextSpan::plain(fallback)];
        }
        spans
    }

    fn rich_html_spans(raw: &str) -> Vec<ViewerTextSpan> {
        let mut cursor = 0;
        let mut spans = Vec::new();
        let mut contexts = vec![HtmlSpanContext::default()];
        while let Some(tag_start) = html_tag::next_start(raw, cursor) {
            Self::push_html_text(&raw[cursor..tag_start], contexts.last(), &mut spans);
            let Some(tag_end) = html_tag::end(raw, tag_start) else {
                trim_last_html_span(&mut spans);
                return spans;
            };
            let tag = &raw[tag_start..=tag_end];
            Self::apply_html_tag(tag, &mut contexts, &mut spans);
            cursor = tag_end + 1;
        }
        Self::push_html_text(&raw[cursor..], contexts.last(), &mut spans);
        spans
    }

    fn apply_html_tag(
        tag: &str,
        contexts: &mut Vec<HtmlSpanContext>,
        spans: &mut Vec<ViewerTextSpan>,
    ) {
        match html_tag::parse(tag) {
            html_tag::HtmlTag::Opening { name, .. } if name == "br" => {
                Self::push_html_text("\n", contexts.last(), spans);
            }
            html_tag::HtmlTag::Opening { name, .. } if name == "img" => {
                Self::push_html_text(&TextParser::html_fragment_text(tag), contexts.last(), spans)
            }
            html_tag::HtmlTag::Opening { name, self_closing } => {
                Self::push_html_context(tag, name, self_closing, contexts);
            }
            html_tag::HtmlTag::Closing { name } => close_html_context(name, contexts),
            html_tag::HtmlTag::Other => {}
        }
    }

    fn push_html_context(
        tag: &str,
        name: String,
        self_closing: bool,
        contexts: &mut Vec<HtmlSpanContext>,
    ) {
        if let (false, Some(parent)) = (self_closing, contexts.last()) {
            contexts.push(HtmlSpanContext {
                name,
                style: html_style(tag, parent.style),
                link_target: html_link_target(tag).or_else(|| parent.link_target.clone()),
            });
        }
    }

    fn push_html_text(
        raw: &str,
        context: Option<&HtmlSpanContext>,
        spans: &mut Vec<ViewerTextSpan>,
    ) {
        let Some(context) = context else {
            return;
        };
        let text = TextParser::decode_basic_entities(raw);
        if let Some(target) = &context.link_target {
            spans.extend(Self::linked_span(text, target.clone(), context.style));
        } else {
            spans.extend(Self::styled_span(text, context.style));
        }
    }
}

#[derive(Default)]
struct HtmlSpanContext {
    name: String,
    style: ViewerTextStyle,
    link_target: Option<String>,
}

fn trim_last_html_span(spans: &mut [ViewerTextSpan]) {
    if let Some(span) = spans.last_mut() {
        span.text = span.text.trim_end().to_string();
    }
}

fn close_html_context(name: String, contexts: &mut Vec<HtmlSpanContext>) {
    if let Some(index) = contexts.iter().rposition(|context| context.name == name) {
        contexts.truncate(index);
    }
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

#[cfg(test)]
mod html_span_tests {
    use super::ViewerNodeClassifier;

    #[test]
    fn empty_html_context_discards_text() {
        let mut spans = Vec::new();
        ViewerNodeClassifier::push_html_text("discarded", None, &mut spans);
        assert!(spans.is_empty());
    }

    #[test]
    fn self_closing_and_other_tags_do_not_change_the_active_html_context() {
        let spans = ViewerNodeClassifier::rich_html_spans(
            "before<custom/>after<!-- ignored comment -->tail",
        );
        let text = spans.into_iter().map(|span| span.text).collect::<String>();
        assert_eq!("beforeaftertail", text);
    }
}
