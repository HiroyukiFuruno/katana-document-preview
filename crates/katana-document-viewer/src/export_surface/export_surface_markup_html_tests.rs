use super::SurfaceHtmlMarkup;
use crate::export_surface_span::SurfaceTextStyle;
use image::Rgba;

#[test]
fn normalize_text_compacts_spacing_and_normalizes_pipe_token() {
    let text = "  left   right  ";
    let normalized = SurfaceHtmlMarkup::normalize_text(text);
    assert_eq!(normalized, "left right");

    let with_pipe = "left|right";
    assert_eq!(SurfaceHtmlMarkup::normalize_text(with_pipe), "left | right");
}

#[test]
fn badge_row_badges_prefers_shields_badges_from_img_refs() {
    let badges = SurfaceHtmlMarkup::badge_row_badges(
        "<img src=\"https://img.shields.io/badge/foo-bar-blue\" alt=\"alt\" />",
    );
    assert_eq!(badges.len(), 1);
    assert_eq!(badges[0].text(), "foo=bar");
}

#[test]
fn badge_row_badges_falls_back_to_normalized_text() {
    let badges = SurfaceHtmlMarkup::badge_row_badges("   just plain text   ");
    assert_eq!(badges.len(), 1);
    assert_eq!(badges[0].text(), "just plain text");
}

#[test]
fn extract_img_refs_tracks_width_and_link_targets() {
    let html = "<a href=\"/docs\"><img src=\"icon.png\" alt=\"Icon\" width=\"24\" /></a>";
    let refs = SurfaceHtmlMarkup::extract_img_refs(html);

    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].src, "icon.png");
    assert_eq!(refs[0].alt, "Icon");
    assert_eq!(refs[0].width, Some(24));
    assert_eq!(refs[0].link_target, Some("/docs".to_string()));
}

#[test]
fn extract_img_refs_ignores_gt_inside_quoted_attributes() {
    let html = "<img src=\"data:image/svg+xml,%3Csvg xmlns=%22<http://www.w3.org/2000/svg%22> width=%2216%22%3E\" alt=\"Icon\">";
    let refs = SurfaceHtmlMarkup::extract_img_refs(html);

    assert_eq!(refs.len(), 1);
    assert!(refs[0].src.contains("data:image/svg+xml"));
    assert_eq!(refs[0].alt, "Icon");
}

#[test]
fn has_center_alignment_detects_html_alignment_hints() {
    assert!(SurfaceHtmlMarkup::has_center_alignment(
        "<p align=\"center\">text</p>"
    ));
    assert!(SurfaceHtmlMarkup::has_center_alignment(
        "<p style=\"text-align: center\">text</p>"
    ));
    assert!(SurfaceHtmlMarkup::has_center_alignment(
        "<H2 ALIGN='CENTER'>text</H2>"
    ));
    assert!(SurfaceHtmlMarkup::has_center_alignment(
        "<p align=center>text</p>"
    ));
    assert!(!SurfaceHtmlMarkup::has_center_alignment("<p>text</p>"));
}

#[test]
fn has_right_alignment_detects_html_alignment_hints() {
    assert!(SurfaceHtmlMarkup::has_right_alignment(
        "<p style=\"text-align: right\">text</p>"
    ));
    assert!(SurfaceHtmlMarkup::has_right_alignment(
        "<P ALIGN='RIGHT'>text</P>"
    ));
    assert!(!SurfaceHtmlMarkup::has_right_alignment("<p>text</p>"));
}

#[test]
fn starts_with_block_tag_detects_paragraph_html_blocks() {
    assert!(SurfaceHtmlMarkup::starts_with_block_tag(
        "<H2 ALIGN='CENTER'>text</H2>"
    ));
    assert!(SurfaceHtmlMarkup::starts_with_block_tag(
        "  <section>text</section>"
    ));
    assert!(!SurfaceHtmlMarkup::starts_with_block_tag(
        "<span>inline</span>"
    ));
}

#[test]
fn centered_html_spans_builds_mixed_text_and_link_spans() {
    let spans =
        SurfaceHtmlMarkup::centered_html_spans("A <a href=\"https://example.com\">go</a> B");
    assert_eq!(spans.len(), 3);
    assert_eq!(spans[0].text, "A ");
    assert_eq!(spans[0].link_target, None);
    assert_eq!(spans[0].style, SurfaceTextStyle::default());
    assert_eq!(spans[1].text, "go");
    assert_eq!(
        spans[1].link_target,
        Some("https://example.com".to_string())
    );
    assert_eq!(spans[1].style, SurfaceTextStyle::default().link());
    assert_eq!(spans[2].text, " B");
    assert_eq!(spans[2].link_target, None);
    assert_eq!(spans[2].style, SurfaceTextStyle::default());
}

#[test]
fn html_spans_apply_inline_css_style() {
    let spans =
        SurfaceHtmlMarkup::html_spans(r#"<p style="color: #ff0000; font-weight: bold">Red</p>"#);

    assert_eq!(1, spans.len());
    assert_eq!("Red", spans[0].text);
    assert_eq!(
        SurfaceTextStyle::default()
            .bold()
            .with_color(Rgba([255, 0, 0, 255])),
        spans[0].style
    );
}

#[test]
fn badge_row_badges_returns_empty_for_blank_fragment() {
    let badges = SurfaceHtmlMarkup::badge_row_badges(" \n\t ");

    assert!(badges.is_empty());
}

#[test]
fn extract_img_refs_stops_on_unclosed_img_and_defaults_empty_alt() {
    let refs = SurfaceHtmlMarkup::extract_img_refs("<img src=\"icon.png\" alt=\"open\"");
    assert!(refs.is_empty());

    let complete_refs =
        SurfaceHtmlMarkup::extract_img_refs("<img src=\"icon.png\" width=\"wide\">");
    assert_eq!(complete_refs.len(), 1);
    assert_eq!(complete_refs[0].alt, "");
    assert_eq!(complete_refs[0].width, None);
}

#[test]
fn extract_img_refs_ignores_img_without_src() {
    let refs = SurfaceHtmlMarkup::extract_img_refs("<img alt=\"Icon\">");

    assert!(refs.is_empty());
}

#[test]
fn centered_html_spans_stops_on_unclosed_or_empty_link() {
    let unclosed = SurfaceHtmlMarkup::centered_html_spans("before <a href=\"/x\">missing");
    let empty = SurfaceHtmlMarkup::centered_html_spans("before <a href=\"/x\"> </a> after");

    assert_eq!(unclosed.len(), 2);
    assert_eq!(unclosed[0].text, "before ");
    assert_eq!(unclosed[1].text, "missing");
    assert_eq!(unclosed[1].link_target, Some("/x".to_string()));
    assert_eq!(empty.len(), 2);
    assert_eq!(empty[0].text, "before ");
    assert_eq!(empty[1].text, "after");
}
