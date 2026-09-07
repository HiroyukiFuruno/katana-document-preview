use super::super::{ViewerNodeClassifier, ViewerNodeKind};
use super::test_support::node;
use crate::ViewerTextSpan;
use katana_markdown_model::{HtmlBlockRole, InlineHtmlNode, KmmNodeKind};

#[path = "classifier_html_spans_void_tests.rs"]
mod void_tests;

#[path = "classifier_html_spans_block_boundary_tests.rs"]
mod block_boundary_tests;

#[test]
fn html_block_spans_falls_back_to_plain_text_without_links() {
    let spans = node_spans_from_html("<p>Hello</p>");

    assert_eq!(1, spans.len());
    assert_eq!("Hello", spans[0].text);
    assert_eq!(ViewerTextSpan::plain("Hello").style, spans[0].style);
}

#[test]
fn centered_html_plain_text_does_not_insert_viewport_independent_line_breaks() {
    let spans = node_spans_from_centered_html(
        "<p align=\"center\">A fast, lightweight Markdown workspace for macOS — built with Rust and egui.</p>",
    );

    assert_eq!(1, spans.len());
    assert_eq!(
        "A fast, lightweight Markdown workspace for macOS — built with Rust and egui.",
        spans[0].text
    );
}

#[test]
fn html_block_spans_uses_fallback_when_html_has_no_visible_text() {
    let spans = node_spans_from_html("");

    assert_eq!(1, spans.len());
    assert_eq!("", spans[0].text);
}

#[test]
fn html_block_spans_stops_unclosed_anchor_tag_without_tag_end() {
    let spans = node_spans_from_html("prefix <a href=\"/broken\"");

    assert_eq!(1, spans.len());
    assert_eq!("prefix", spans[0].text);
}

#[test]
fn html_block_spans_keeps_anchor_body_plain_when_target_missing() {
    let spans = node_spans_from_html("<a class=\"button\">plain</a>");

    assert_eq!(1, spans.len());
    assert_eq!("plain", spans[0].text);
    assert!(spans[0].link_target.is_empty());
}

#[test]
fn html_block_spans_handles_unclosed_link_without_crash() {
    let spans = node_spans_from_html("<a href=\"/broken\">broken");

    assert_eq!(1, spans.len());
    assert_eq!("broken", spans[0].text);
}

#[test]
fn inline_html_spans_parses_quoted_link_target() {
    let spans = inline_node_spans("<a href=\"/docs/index.md\">Docs</a>");

    assert_eq!(1, spans.len());
    assert_eq!("Docs", spans[0].text);
    assert_eq!("/docs/index.md", spans[0].link_target);
    assert!(spans[0].style.underline);
}

#[test]
fn inline_html_spans_preserves_unquoted_link_targets() {
    let spans = inline_node_spans("<a href=/raw/path>raw</a>");

    assert_eq!("raw", spans[0].text);
    assert_eq!("/raw/path", spans[0].link_target);
    assert!(spans[0].style.underline);

    let protocol_relative = inline_node_spans("<a href=//cdn.example/x>cdn</a>");
    assert_eq!("cdn", protocol_relative[0].text);
    assert_eq!("//cdn.example/x", protocol_relative[0].link_target);
}

#[test]
fn html_spans_require_an_anchor_and_an_exact_href_attribute() {
    for raw in [
        r#"<span data-href="/spoof">plain</span>"#,
        r#"<a data-href="/spoof">plain</a>"#,
        r#"<a hrefx="/spoof">plain</a>"#,
        r#"<a-data href="/spoof">plain</a-data>"#,
    ] {
        let spans = inline_node_spans(raw);
        assert_eq!(1, spans.len(), "{raw}");
        assert_eq!("plain", spans[0].text, "{raw}");
        assert!(spans[0].link_target.is_empty(), "{raw}");
        assert!(!spans[0].style.underline, "{raw}");
    }

    let spans = node_spans_from_html(r#"<span data-href="/spoof">plain</span>"#);
    assert_eq!(1, spans.len());
    assert!(spans[0].link_target.is_empty());
}

#[test]
fn inline_html_spans_keeps_anchor_text_as_plain_when_target_missing() {
    let spans = inline_node_spans("<a>plain</a>");

    assert_eq!(1, spans.len());
    assert_eq!("plain", spans[0].text);
    assert!(spans[0].link_target.is_empty());
    assert!(!spans[0].style.underline);
}

#[test]
fn inline_html_spans_supports_style_markers() {
    let code = inline_node_spans("<code>code</code>");
    let bold = inline_node_spans("<strong>bold</strong>");
    let strike = inline_node_spans("<s>old</s>");
    let mark = inline_node_spans("<mark>hot</mark>");

    assert!(code[0].style.inline_code);
    assert!(bold[0].style.bold);
    assert!(strike[0].style.strikethrough);
    assert!(mark[0].style.highlight);
    assert!(inline_node_spans("<u>note</u>")[0].style.underline);
    assert!(inline_node_spans("<em>em</em>")[0].style.italic);
}

#[test]
fn inline_html_spans_allow_font_style_normal_to_clear_inherited_italics() {
    let spans = inline_node_spans(r#"<em><span style="font-style: normal">plain</span></em>"#);

    assert_eq!(1, spans.len());
    assert!(!spans[0].style.italic);
}

#[test]
fn html_block_spans_applies_inline_css_style() {
    let spans = node_spans_from_html(r#"<p style="color: #ff0000; font-weight: bold">Red</p>"#);

    assert_eq!("Red", spans[0].text);
    assert_eq!([255, 0, 0, 255], spans[0].style.color_rgba);
    assert!(spans[0].style.bold);
}

#[test]
fn inline_html_spans_skips_unclosed_anchor_tag_as_plain() {
    let spans = inline_node_spans("<a href=\"/broken\">broken");

    assert_eq!(1, spans.len());
    assert_eq!("broken", spans[0].text);
}

#[test]
fn inline_html_spans_without_link_target_keeps_plain_text() {
    let spans = inline_node_spans("<span>plain</span>");

    assert_eq!(1, spans.len());
    assert_eq!("plain", spans[0].text);
}

pub(super) fn node_spans_from_html(raw: &str) -> Vec<ViewerTextSpan> {
    node_spans_from_html_role(raw, crate::ViewerHtmlRole::Generic)
}

fn node_spans_from_centered_html(raw: &str) -> Vec<ViewerTextSpan> {
    node_spans_from_html_role(raw, crate::ViewerHtmlRole::Centered)
}

fn node_spans_from_html_role(raw: &str, role: crate::ViewerHtmlRole) -> Vec<ViewerTextSpan> {
    let block_role = match role {
        crate::ViewerHtmlRole::Centered => HtmlBlockRole::Centered,
        _ => HtmlBlockRole::Generic,
    };
    let current = node(KmmNodeKind::HtmlBlock(block_role), raw, Vec::new());
    ViewerNodeClassifier::node_spans(&current, &ViewerNodeKind::Html { role })
}

fn inline_node_spans(raw: &str) -> Vec<ViewerTextSpan> {
    let current = node(
        KmmNodeKind::Paragraph,
        raw,
        vec![node(
            KmmNodeKind::InlineHtml(InlineHtmlNode {
                html: raw.to_string(),
            }),
            raw,
            Vec::new(),
        )],
    );
    ViewerNodeClassifier::node_spans(&current, &ViewerNodeKind::Paragraph)
}
