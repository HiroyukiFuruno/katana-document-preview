use super::SurfaceHtmlMarkup;
use crate::export_surface_span::SurfaceTextStyle;
use image::Rgba;

#[test]
fn html_spans_scope_nested_inline_styles_to_their_text_segments() {
    let spans = SurfaceHtmlMarkup::html_spans(
        r#"plain <strong>bold <span style="color: #ff0000">red</span></strong> plain"#,
    );

    assert_eq!(
        vec!["plain ", "bold ", "red", " plain"],
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
fn html_spans_preserve_spaces_across_inline_style_boundaries() {
    let spans = SurfaceHtmlMarkup::html_spans("<p>plain <strong>bold</strong> plain</p>");

    assert_eq!(
        "plain bold plain",
        spans
            .iter()
            .map(|span| span.text.as_str())
            .collect::<String>()
    );
    assert_eq!("plain ", spans[0].text);
    assert_eq!("bold", spans[1].text);
    assert_eq!(" plain", spans[2].text);
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

#[test]
fn html_spans_require_an_exact_href_attribute_name() {
    let spans = SurfaceHtmlMarkup::html_spans(r#"<a data-href="/docs">plain</a>"#);

    assert_eq!(1, spans.len());
    assert_eq!("plain", spans[0].text);
    assert_eq!(Some(""), spans[0].link_target.as_deref());
    assert!(spans[0].style.underline);
}

#[test]
fn html_spans_accept_single_quoted_href_attributes() {
    let spans = SurfaceHtmlMarkup::html_spans("<a href='/docs'>plain</a>");

    assert_eq!(1, spans.len());
    assert_eq!("plain", spans[0].text);
    assert_eq!(Some("/docs"), spans[0].link_target.as_deref());
}

#[test]
fn html_spans_keep_gt_inside_single_quoted_attributes() {
    let spans =
        SurfaceHtmlMarkup::html_spans("<span title='1>2' style='color: #ff0000'>plain</span>");

    assert_eq!(1, spans.len());
    assert_eq!("plain", spans[0].text);
    assert_eq!(
        SurfaceTextStyle::default().with_color(Rgba([255, 0, 0, 255])),
        spans[0].style
    );
}

#[test]
fn html_spans_preserve_br_boundaries() {
    let spans = SurfaceHtmlMarkup::html_spans("<p>first<br>second</p>");
    assert_eq!(
        "first\nsecond",
        spans
            .iter()
            .map(|span| span.text.as_str())
            .collect::<String>()
    );

    let self_closing = SurfaceHtmlMarkup::html_spans("<p>first<br/>second</p>");
    assert_eq!(
        "first\nsecond",
        self_closing
            .iter()
            .map(|span| span.text.as_str())
            .collect::<String>()
    );

    let linked = SurfaceHtmlMarkup::html_spans("<a href='/docs'>first<br>second</a>");
    assert_eq!(
        "first\nsecond",
        linked
            .iter()
            .map(|span| span.text.as_str())
            .collect::<String>()
    );
    assert_eq!(Some("/docs"), linked[1].link_target.as_deref());
    assert!(linked[1].style.underline);
}

#[test]
fn html_spans_preserve_boundaries_between_adjacent_blocks() {
    let spans =
        SurfaceHtmlMarkup::html_spans("<p><strong>one</strong></p><p><strong>two</strong></p>");

    assert_eq!(
        "one two",
        spans
            .iter()
            .map(|span| span.text.as_str())
            .collect::<String>()
    );
    assert!(spans[0].style.bold);
    assert_eq!(SurfaceTextStyle::default(), spans[1].style);
    assert!(spans[2].style.bold);
}

#[test]
fn html_attribute_values_handle_non_elements_boolean_and_unquoted_forms() {
    assert_eq!(None, super::attributes::attribute_value("</a>", "href"));
    assert_eq!(
        None,
        super::attributes::attribute_value(r#"<a ="/docs">"#, "href")
    );
    assert_eq!(
        Some("/docs".to_owned()),
        super::attributes::attribute_value(r#"<a disabled href="/docs">"#, "href")
    );
    assert_eq!(
        Some("42".to_owned()),
        super::attributes::attribute_value("<img width=42>", "width")
    );
    assert_eq!(
        Some("42".to_owned()),
        super::attributes::attribute_value("<img width=42 alt=preview>", "width")
    );
}

#[test]
fn html_spans_apply_inline_css_false_values_over_inherited_styles() {
    let spans = SurfaceHtmlMarkup::html_spans(
        r#"<strong><em><del><span style="font-weight: normal; font-style: normal; text-decoration: none">plain</span></del></em></strong>"#,
    );

    assert_eq!(1, spans.len());
    assert!(!spans[0].style.bold);
    assert!(!spans[0].style.italic);
    assert!(!spans[0].style.underline);
    assert!(!spans[0].style.strikethrough);
}
