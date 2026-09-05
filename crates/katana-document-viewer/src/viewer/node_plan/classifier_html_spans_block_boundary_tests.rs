use super::node_spans_from_html;
use crate::ViewerTextSpan;

#[test]
fn html_block_spans_preserve_boundaries_between_adjacent_blocks() {
    let spans = node_spans_from_html("<p><strong>one</strong></p><p><strong>two</strong></p>");

    assert_eq!(
        "one two",
        spans
            .iter()
            .map(|span| span.text.as_str())
            .collect::<String>()
    );
    assert!(spans[0].style.bold);
    assert_eq!(ViewerTextSpan::plain(" ").style, spans[1].style);
    assert!(spans[2].style.bold);
}
