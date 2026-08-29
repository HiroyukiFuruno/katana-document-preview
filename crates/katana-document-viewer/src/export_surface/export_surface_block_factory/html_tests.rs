#[path = "html_heading_and_details_tests.rs"]
mod html_heading_and_details_tests;
#[path = "html_wrapping_tests.rs"]
mod html_wrapping_tests;
#[path = "html_test_support.rs"]
mod support;

use crate::export_surface_helpers::BODY_MAX_CHARS;
use crate::export_surface_line::SurfaceLine;
use crate::theme::KdvThemeSnapshot;
use katana_markdown_model::HtmlBlockRole;

use super::SurfaceBlockFactory;
use support::{graph, node};

#[test]
fn generic_html_is_treated_as_wrapped_text() {
    let graph = graph();
    let mut blocks = Vec::new();
    let node = node("<div>hello</div>");
    SurfaceBlockFactory::append_html(
        &mut blocks,
        &graph,
        &node,
        &HtmlBlockRole::Generic,
        0,
        0,
        &KdvThemeSnapshot::katana_light(),
    );

    assert_eq!(blocks.len(), 1);
    assert!(!blocks[0].text_for_tests().is_empty());
}

#[test]
fn styled_html_is_rendered_as_rich_surface_span() {
    let graph = graph();
    let mut blocks = Vec::new();
    let node = node(r#"<p style="color: red; font-weight: bold">Red</p>"#);
    SurfaceBlockFactory::append_html(
        &mut blocks,
        &graph,
        &node,
        &HtmlBlockRole::Generic,
        0,
        0,
        &KdvThemeSnapshot::katana_light(),
    );

    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].text_for_tests(), "Red");
    assert_eq!(
        blocks[0].debug_for_tests(),
        "line:Red:Red:[\"bold\", \"color\"]"
    );
}

#[test]
fn centered_html_is_rendered_into_centered_line() {
    let graph = graph();
    let mut blocks = Vec::new();
    let node = node("<center>Center</center>");
    SurfaceBlockFactory::append_html(
        &mut blocks,
        &graph,
        &node,
        &HtmlBlockRole::Centered,
        0,
        0,
        &KdvThemeSnapshot::katana_light(),
    );

    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].text_for_tests(), "Center");
}

#[test]
fn html_heading_is_rendered_as_body_line_for_export_reference() {
    let graph = graph();
    let mut blocks = Vec::new();
    let node = node(r#"<h1 align="center">Centered Heading</h1>"#);
    SurfaceBlockFactory::append_html(
        &mut blocks,
        &graph,
        &node,
        &HtmlBlockRole::Generic,
        0,
        0,
        &KdvThemeSnapshot::katana_light(),
    );

    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].text_for_tests(), "Centered Heading");
    assert!(!blocks[0].is_heading());
    assert_eq!(
        blocks[0].height(),
        SurfaceLine::body("h1".to_string()).line_height()
    );
    assert!(blocks[0].debug_for_tests().contains("line:[\"centered\"]"));
}

#[test]
fn long_centered_html_falls_back_to_wrapped_centered_lines() {
    let graph = graph();
    let mut blocks = Vec::new();
    let long_text = "a".repeat(BODY_MAX_CHARS + 1);
    let source = format!("<center>{long_text}</center>");
    let node = node(&source);

    SurfaceBlockFactory::append_html(
        &mut blocks,
        &graph,
        &node,
        &HtmlBlockRole::Centered,
        0,
        0,
        &KdvThemeSnapshot::katana_light(),
    );

    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].text_for_tests(), "a".repeat(BODY_MAX_CHARS));
    assert_eq!(blocks[1].text_for_tests(), "a");
}

#[test]
fn badge_row_html_creates_badge_block() {
    let graph = graph();
    let mut blocks = Vec::new();
    let node = node("<div class=\"badge-row\">A</div>");
    SurfaceBlockFactory::append_html(
        &mut blocks,
        &graph,
        &node,
        &HtmlBlockRole::BadgeRow,
        0,
        0,
        &KdvThemeSnapshot::katana_light(),
    );

    assert_eq!(blocks.len(), 1);
    assert!(!blocks[0].text_for_tests().is_empty());
}

#[test]
fn details_html_is_split_into_summary_and_body_lines() {
    let graph = graph();
    let mut blocks = Vec::new();
    let node = node("<details><summary>Summary</summary><div>Body</div></details>");
    SurfaceBlockFactory::append_html(
        &mut blocks,
        &graph,
        &node,
        &HtmlBlockRole::Generic,
        0,
        0,
        &KdvThemeSnapshot::katana_light(),
    );

    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].text_for_tests(), "Summary");
    assert_eq!(blocks[1].text_for_tests(), "Body");
}

#[test]
fn details_html_falls_back_to_raw_body_when_markdown_nodes_are_empty() {
    let graph = graph();
    let mut blocks = Vec::new();
    let node = node("<details><summary>Summary</summary> </details>");
    SurfaceBlockFactory::append_html(
        &mut blocks,
        &graph,
        &node,
        &HtmlBlockRole::Generic,
        0,
        0,
        &KdvThemeSnapshot::katana_light(),
    );

    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].text_for_tests(), "Summary");
    assert_eq!(blocks[1].text_for_tests(), "");
}
