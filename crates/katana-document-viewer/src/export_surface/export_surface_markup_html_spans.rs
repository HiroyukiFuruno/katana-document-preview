use super::super::export_surface_markup_html_style::SurfaceHtmlStyle;
use super::attributes::attribute_value;
use super::html_tag_end;
use super::spans_output::{
    HtmlSpanContext, push_line_break, push_text, trim_final_boundary_whitespace,
};
use crate::export_surface_span::SurfaceTextSpan;

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
            trim_final_boundary_whitespace(&mut spans);
            return spans;
        };
        let tag = &tag_source[..=end];
        if let Some(parsed) = HtmlTag::parse(tag) {
            apply_html_tag(&mut spans, &mut contexts, tag, parsed);
        }
        cursor = start + end + 1;
    }
    push_text(&mut spans, &contexts, &fragment[cursor..]);
    trim_final_boundary_whitespace(&mut spans);
    spans
}

fn apply_html_tag(
    spans: &mut Vec<SurfaceTextSpan>,
    contexts: &mut Vec<HtmlSpanContext>,
    tag: &str,
    parsed: HtmlTag,
) {
    if parsed.closing {
        close_context(contexts, &parsed.name);
        return;
    }
    if parsed.name == "br" {
        push_line_break(spans, contexts);
        return;
    }
    open_context(contexts, tag, parsed);
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
        let empty_link_target = String::new();
        link_target = Some(attribute_value(tag, "href").unwrap_or(empty_link_target));
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
