use super::SurfaceHtmlMarkup;
use crate::export_surface_span::SurfaceTextStyle;
use image::Rgba;

#[test]
fn html_spans_scope_nested_inline_styles_to_their_text_segments() {
    let spans = SurfaceHtmlMarkup::html_spans(
        r#"plain <strong>bold <span style="color: #ff0000">red</span></strong> plain"#,
    );

    assert_eq!(
        vec!["plain", "bold", "red", "plain"],
        spans
            .iter()
            .map(|span| span.text.as_str())
            .collect::<Vec<_>>()
    );
    assert_eq!(SurfaceTextStyle::default(), spans[0].style);
    assert_eq!(SurfaceTextStyle::default().bold(), spans[1].style);
    assert_eq!(
        SurfaceTextStyle::default()
            .bold()
            .with_color(Rgba([255, 0, 0, 255])),
        spans[2].style
    );
    assert_eq!(SurfaceTextStyle::default(), spans[3].style);
}

#[test]
fn html_spans_ignore_non_element_and_incomplete_markup_without_style_leakage() {
    let spans = SurfaceHtmlMarkup::html_spans("before<!--comment-->after<>tail");
    assert_eq!(
        vec!["before", "after", "tail"],
        spans
            .iter()
            .map(|span| span.text.as_str())
            .collect::<Vec<_>>()
    );

    let incomplete = SurfaceHtmlMarkup::html_spans("before<strong");
    assert_eq!("before", incomplete[0].text);
}

#[test]
fn html_spans_preserve_an_empty_target_for_anchor_without_href() {
    let spans = SurfaceHtmlMarkup::html_spans("<a>plain</a>");

    assert_eq!(1, spans.len());
    assert_eq!("plain", spans[0].text);
    assert_eq!(Some(""), spans[0].link_target.as_deref());
    assert!(spans[0].style.underline);
}
