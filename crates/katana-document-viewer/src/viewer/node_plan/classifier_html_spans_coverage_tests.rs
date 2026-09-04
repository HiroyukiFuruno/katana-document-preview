use super::super::{ViewerNodeClassifier, ViewerNodeKind};
use super::test_support::node;
use crate::ViewerTextSpan;
use katana_markdown_model::{InlineHtmlNode, KmmNodeKind};

#[test]
fn inline_html_spans_applies_false_css_overrides_to_tag_styles() {
    let bold = inline_node_spans(r#"<strong style="font-weight: normal">not bold</strong>"#);
    let underline = inline_node_spans(r#"<u style="text-decoration: none">not underlined</u>"#);
    let strikethrough = inline_node_spans(r#"<s style="text-decoration: none">not struck</s>"#);

    assert!(!bold[0].style.bold);
    assert!(!underline[0].style.underline);
    assert!(!strikethrough[0].style.strikethrough);
}

#[test]
fn inline_html_spans_ignores_closing_tag_as_link_target() {
    let spans = inline_node_spans("</a>plain");

    assert_eq!(1, spans.len());
    assert_eq!("plain", spans[0].text);
    assert!(spans[0].link_target.is_empty());
}

#[test]
fn inline_html_spans_scans_anchor_attributes_until_tag_end_without_href() {
    let spans = inline_node_spans(r#"<a class="button" aria-label="plain">plain</a>"#);

    assert_eq!(1, spans.len());
    assert_eq!("plain", spans[0].text);
    assert!(spans[0].link_target.is_empty());
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
