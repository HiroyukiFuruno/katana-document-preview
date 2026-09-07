use super::super::export_surface_markup_html_style::SurfaceHtmlStyle;
use super::attributes::attribute_value;
use super::html_tag_end;
use super::spans_output::{
    HtmlSpanContext, push_line_break, push_text, trim_final_boundary_whitespace,
};
use crate::export_surface_span::{SurfaceTextSpan, SurfaceTextStyle};
use crate::html_style::HtmlStyle;

pub(super) fn html_spans(fragment: &str) -> Vec<SurfaceTextSpan> {
    let mut builder = HtmlSpanBuilder::new();
    builder.consume(fragment);
    builder.into_spans()
}

struct HtmlSpanBuilder {
    spans: Vec<SurfaceTextSpan>,
    contexts: Vec<HtmlSpanContext>,
    block_boundary_pending: bool,
}

impl HtmlSpanBuilder {
    fn new() -> Self {
        Self {
            spans: Vec::new(),
            contexts: vec![HtmlSpanContext::root()],
            block_boundary_pending: false,
        }
    }

    fn consume(&mut self, fragment: &str) {
        let mut cursor = 0;
        while let Some(relative_start) = fragment[cursor..].find('<') {
            let start = cursor + relative_start;
            self.push_fragment(&fragment[cursor..start]);
            let tag_source = &fragment[start..];
            let Some(end) = html_tag_end(tag_source) else {
                self.push_fragment(tag_source);
                return;
            };
            let tag = &tag_source[..=end];
            if let Some(parsed) = HtmlTag::parse(tag) {
                self.apply_tag(tag, parsed);
            }
            cursor = start + end + 1;
        }
        self.push_fragment(&fragment[cursor..]);
    }

    fn push_fragment(&mut self, fragment: &str) {
        if self.block_boundary_pending && !fragment.is_empty() {
            if !fragment.chars().next().is_some_and(char::is_whitespace) {
                self.spans
                    .push(SurfaceTextSpan::styled(" ", SurfaceTextStyle::default()));
            }
            self.block_boundary_pending = false;
        }
        push_text(&mut self.spans, &self.contexts, fragment);
    }

    fn apply_tag(&mut self, tag: &str, parsed: HtmlTag) {
        if parsed.closing {
            close_context(&mut self.contexts, &parsed.name);
            self.queue_block_boundary(&parsed.name);
            return;
        }
        if parsed.name == "br" {
            push_line_break(&mut self.spans, &self.contexts);
            self.block_boundary_pending = false;
            return;
        }
        self.queue_block_boundary(&parsed.name);
        open_context(&mut self.contexts, tag, parsed);
    }

    fn queue_block_boundary(&mut self, name: &str) {
        if HtmlStyle::is_block_element(name)
            && !self.spans.is_empty()
            && !spans_end_with_whitespace(&self.spans)
        {
            self.block_boundary_pending = true;
        }
    }

    fn into_spans(mut self) -> Vec<SurfaceTextSpan> {
        trim_final_boundary_whitespace(&mut self.spans);
        self.spans
    }
}

fn spans_end_with_whitespace(spans: &[SurfaceTextSpan]) -> bool {
    spans
        .last()
        .and_then(|span| span.text.chars().next_back())
        .is_some_and(char::is_whitespace)
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
