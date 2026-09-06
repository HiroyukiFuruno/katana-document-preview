use super::{ViewerNodeClassifier, html_link_target, html_style, html_tag};
use crate::export_surface_text::SurfaceTextParser as TextParser;
use crate::html_style::HtmlStyle;
use crate::{ViewerTextSpan, ViewerTextStyle};

pub(super) fn parse(raw: &str) -> Vec<ViewerTextSpan> {
    let mut parser = HtmlSpanParser::new();
    parser.consume(raw);
    parser.into_spans()
}

struct HtmlSpanParser {
    spans: Vec<ViewerTextSpan>,
    contexts: Vec<HtmlSpanContext>,
    block_boundary_pending: bool,
}

impl HtmlSpanParser {
    fn new() -> Self {
        Self {
            spans: Vec::new(),
            contexts: vec![HtmlSpanContext::default()],
            block_boundary_pending: false,
        }
    }

    fn consume(&mut self, raw: &str) {
        let mut cursor = 0;
        while let Some(tag_start) = html_tag::next_start(raw, cursor) {
            self.push_text(&raw[cursor..tag_start]);
            let Some(tag_end) = html_tag::end(raw, tag_start) else {
                trim_last_html_span(&mut self.spans);
                return;
            };
            self.apply_tag(&raw[tag_start..=tag_end]);
            cursor = tag_end + 1;
        }
        self.push_text(&raw[cursor..]);
    }

    fn apply_tag(&mut self, tag: &str) {
        match html_tag::parse(tag) {
            html_tag::HtmlTag::Opening { name, .. } if name == "br" => self.push_text("\n"),
            html_tag::HtmlTag::Opening { name, .. } if name == "img" => {
                self.push_text(&TextParser::html_fragment_text(tag));
            }
            html_tag::HtmlTag::Opening { name, .. } if html_tag::is_void_element(&name) => {
                self.queue_block_boundary(&name);
            }
            html_tag::HtmlTag::Opening { name, self_closing } => {
                self.queue_block_boundary(&name);
                self.push_context(tag, name, self_closing);
            }
            html_tag::HtmlTag::Closing { name } => {
                close_html_context(&name, &mut self.contexts);
                self.queue_block_boundary(&name);
            }
            html_tag::HtmlTag::Other => {}
        }
    }

    fn queue_block_boundary(&mut self, name: &str) {
        if HtmlStyle::is_block_element(name)
            && !self.spans.is_empty()
            && !html_spans_end_with_whitespace(&self.spans)
        {
            self.block_boundary_pending = true;
        }
    }

    fn push_context(&mut self, tag: &str, name: String, self_closing: bool) {
        if self_closing {
            return;
        }
        let Some(parent) = self.contexts.last() else {
            return;
        };
        let context = HtmlSpanContext {
            name,
            style: html_style(tag, parent.style),
            link_target: html_link_target(tag).or_else(|| parent.link_target.clone()),
        };
        self.contexts.push(context);
    }

    fn push_text(&mut self, raw: &str) {
        let Some(context) = self.contexts.last().cloned() else {
            return;
        };
        let text = TextParser::decode_basic_entities(raw);
        if text.is_empty() {
            return;
        }
        if self.block_boundary_pending && !text.chars().next().is_some_and(char::is_whitespace) {
            self.spans.push(ViewerTextSpan::plain(" "));
        }
        self.block_boundary_pending = false;
        if let Some(target) = context.link_target {
            self.spans.extend(ViewerNodeClassifier::linked_span(
                text,
                target,
                context.style,
            ));
        } else {
            self.spans
                .extend(ViewerNodeClassifier::styled_span(text, context.style));
        }
    }

    fn into_spans(self) -> Vec<ViewerTextSpan> {
        self.spans
    }
}

fn html_spans_end_with_whitespace(spans: &[ViewerTextSpan]) -> bool {
    spans
        .last()
        .and_then(|span| span.text.chars().next_back())
        .is_some_and(char::is_whitespace)
}

#[derive(Clone, Default)]
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

fn close_html_context(name: &str, contexts: &mut Vec<HtmlSpanContext>) {
    if let Some(index) = contexts.iter().rposition(|context| context.name == name) {
        contexts.truncate(index);
    }
}

#[cfg(test)]
mod tests {
    use super::{HtmlSpanParser, parse};

    #[test]
    fn empty_html_context_discards_text() {
        let mut parser = HtmlSpanParser {
            spans: Vec::new(),
            contexts: Vec::new(),
            block_boundary_pending: false,
        };
        parser.push_text("discarded");
        assert!(parser.spans.is_empty());
    }

    #[test]
    fn empty_html_context_does_not_accept_an_opening_tag() {
        let mut parser = HtmlSpanParser {
            spans: Vec::new(),
            contexts: Vec::new(),
            block_boundary_pending: false,
        };

        parser.push_context("<strong>", "strong".to_string(), false);

        assert!(parser.contexts.is_empty());
    }

    #[test]
    fn self_closing_and_other_tags_do_not_change_the_active_html_context() {
        let spans = parse("before<custom/>after<!-- ignored comment -->tail");
        let text = spans.into_iter().map(|span| span.text).collect::<String>();
        assert_eq!("beforeaftertail", text);
    }
}
