use super::node_spans_from_html;
use crate::ViewerTextSpan;

#[test]
fn html_block_spans_does_not_leak_style_after_standard_void_elements() {
    for tag in [
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
        "source", "track", "wbr",
    ] {
        let spans = node_spans_from_html(&format!(r#"<{tag} style="color: #ff0000">after"#));

        let after_style = spans
            .iter()
            .find(|span| span.text.contains("after"))
            .map(|span| span.style);
        assert_eq!(
            Some(ViewerTextSpan::plain("after").style),
            after_style,
            "<{tag}> must not leak its style into sibling text"
        );
    }
}
