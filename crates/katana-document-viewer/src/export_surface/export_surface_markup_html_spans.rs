use super::super::export_surface_markup_html_style::SurfaceHtmlStyle;
use super::{SurfaceHtmlMarkup, html_tag_end, quoted_attribute_value};
use crate::export_surface_span::{SurfaceTextSpan, SurfaceTextStyle};
use crate::export_surface_text::SurfaceTextParser;

#[derive(Clone)]
struct HtmlSpanContext {
    name: String,
    style: SurfaceTextStyle,
    link_target: Option<String>,
}

pub(super) fn html_spans(fragment: &str) -> Vec<SurfaceTextSpan> {
    let mut spans = Vec::new();
    let mut contexts = vec![HtmlSpanContext::root()];
    let mut cursor = 0;
    while let Some(relative_start) = fragment[cursor..].find('<') {
        let start = cursor + relative_start;
        push_text(&mut spans, &contexts, &fragment[cursor..start]);
        let tag_source = &fragment[start..];
        let Some(end) = html_tag_end(tag_source) else {
            push_text(&mut spans, &contexts, tag_source);
            return spans;
        };
        let tag = &tag_source[..=end];
        if let Some(parsed) = HtmlTag::parse(tag) {
            if parsed.closing {
                close_context(&mut contexts, &parsed.name);
            } else {
                open_context(&mut contexts, tag, parsed);
            }
        }
        cursor = start + end + 1;
    }
    push_text(&mut spans, &contexts, &fragment[cursor..]);
    spans
}

fn push_text(spans: &mut Vec<SurfaceTextSpan>, contexts: &[HtmlSpanContext], fragment: &str) {
    let text = SurfaceHtmlMarkup::normalize_text(&SurfaceTextParser::html_fragment_text(fragment));
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

fn open_context(contexts: &mut Vec<HtmlSpanContext>, tag: &str, parsed: HtmlTag) {
    let parent = contexts
        .last()
        .cloned()
        .unwrap_or_else(HtmlSpanContext::root);
    let mut style = parent.style;
    let mut link_target = parent.link_target.clone();
    if parsed.name == "a" {
        style = style.link();
        link_target = Some(quoted_attribute_value(tag, "href").unwrap_or_else(missing_link_target));
    }
    style = SurfaceHtmlStyle::apply(tag, style);
    if !parsed.self_closing && !is_void_tag(&parsed.name) {
        contexts.push(HtmlSpanContext {
            name: parsed.name,
            style,
            link_target,
        });
    }
}

fn close_context(contexts: &mut Vec<HtmlSpanContext>, name: &str) {
    if let Some(index) = contexts.iter().rposition(|context| context.name == name) {
        contexts.truncate(index);
    }
}

fn is_void_tag(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn missing_link_target() -> String {
    String::new()
}

struct HtmlTag {
    name: String,
    closing: bool,
    self_closing: bool,
}

impl HtmlTag {
    fn parse(tag: &str) -> Option<Self> {
        let body = tag.strip_prefix('<')?.strip_suffix('>')?.trim();
        if body.starts_with('!') || body.starts_with('?') {
            return None;
        }
        let (closing, body) = body
            .strip_prefix('/')
            .map_or((false, body), |body| (true, body.trim_start()));
        let name_end = body
            .find(|character: char| {
                !(character.is_ascii_alphanumeric() || character == '-' || character == ':')
            })
            .unwrap_or(body.len());
        if name_end == 0 {
            return None;
        }
        Some(Self {
            name: body[..name_end].to_ascii_lowercase(),
            closing,
            self_closing: body.trim_end().ends_with('/'),
        })
    }
}

impl HtmlSpanContext {
    fn root() -> Self {
        Self {
            name: String::new(),
            style: SurfaceTextStyle::default(),
            link_target: None,
        }
    }
}
