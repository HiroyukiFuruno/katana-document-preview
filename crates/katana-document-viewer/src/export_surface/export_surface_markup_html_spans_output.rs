use super::SurfaceHtmlMarkup;
use crate::export_surface_span::{SurfaceTextSpan, SurfaceTextStyle};
use crate::export_surface_text::SurfaceTextParser;

#[derive(Clone)]
pub(super) struct HtmlSpanContext {
    pub(super) name: String,
    pub(super) style: SurfaceTextStyle,
    pub(super) link_target: Option<String>,
}

impl HtmlSpanContext {
    pub(super) fn root() -> Self {
        Self {
            name: String::new(),
            style: SurfaceTextStyle::default(),
            link_target: None,
        }
    }
}

pub(super) fn push_text(
    spans: &mut Vec<SurfaceTextSpan>,
    contexts: &[HtmlSpanContext],
    fragment: &str,
) {
    let mut text = normalize_span_text(fragment);
    if spans.is_empty() || spans.last().is_some_and(|span| span.text.ends_with(' ')) {
        text = text.trim_start_matches(' ').to_string();
    }
    if text.is_empty() {
        return;
    }
    let context = contexts
        .last()
        .cloned()
        .unwrap_or_else(HtmlSpanContext::root);
    if let Some(link_target) = &context.link_target {
        spans.push(SurfaceTextSpan::linked(
            text,
            link_target.clone(),
            context.style,
        ));
    } else {
        spans.push(SurfaceTextSpan::styled(text, context.style));
    }
}

pub(super) fn push_line_break(spans: &mut Vec<SurfaceTextSpan>, contexts: &[HtmlSpanContext]) {
    let context = contexts
        .last()
        .cloned()
        .unwrap_or_else(HtmlSpanContext::root);
    if let Some(link_target) = &context.link_target {
        spans.push(SurfaceTextSpan::linked(
            "\n",
            link_target.clone(),
            context.style,
        ));
    } else {
        spans.push(SurfaceTextSpan::styled("\n", context.style));
    }
}

pub(super) fn trim_final_boundary_whitespace(spans: &mut Vec<SurfaceTextSpan>) {
    while let Some(last) = spans.last_mut() {
        if !last.text.ends_with(' ') {
            return;
        }
        last.text.pop();
        if !last.text.is_empty() {
            return;
        }
        spans.pop();
    }
}

fn normalize_span_text(fragment: &str) -> String {
    let source = if fragment.contains('<') {
        SurfaceTextParser::html_fragment_text(fragment)
    } else {
        SurfaceTextParser::decode_basic_entities(fragment)
    };
    let leading_whitespace = source
        .chars()
        .next()
        .is_some_and(|character| character.is_whitespace());
    let trailing_whitespace = source
        .chars()
        .next_back()
        .is_some_and(|character| character.is_whitespace());
    let normalized = SurfaceHtmlMarkup::normalize_text(&source);
    if normalized.is_empty() {
        if leading_whitespace || trailing_whitespace {
            return String::from(" ");
        }
        return String::new();
    }
    let prefix = if leading_whitespace { " " } else { "" };
    let suffix = if trailing_whitespace { " " } else { "" };
    format!("{prefix}{normalized}{suffix}")
}
