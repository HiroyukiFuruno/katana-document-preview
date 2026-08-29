use super::{MarkdownSource, PreviewConfig, PreviewError, PreviewOutput, PreviewOutputFactory};
use crate::{DocumentKind, ViewerHtmlAlignment, ViewerHtmlRole, ViewerNodeKind};
use crate::{ViewerNodePlan, ViewerNodePlanner, ViewerViewport};
use std::path::PathBuf;

const CONTENT_HEIGHT: f32 = 120.0;

#[test]
fn direct_html_source_becomes_html_document_node() -> Result<(), PreviewError> {
    let output = output_for_html(centered_link_html())?;
    let plan = ViewerNodePlanner::create(&output.input, 0.0);

    assert_html_snapshot(&output);
    assert_centered_link_node(&plan);
    Ok(())
}

#[test]
fn direct_htm_source_becomes_html_document_node() -> Result<(), PreviewError> {
    let output = output_for_html_path("<p>HTM body</p>".to_string(), "fixture.htm")?;
    let plan = ViewerNodePlanner::create(&output.input, 0.0);

    assert_eq!(DocumentKind::Html, output.input.snapshot.kind);
    assert!(matches!(plan.nodes[0].kind, ViewerNodeKind::Html { .. }));
    assert_eq!("HTM body", plan.nodes[0].text);
    Ok(())
}

#[test]
fn direct_html_source_keeps_uppercase_heading_alignment() -> Result<(), PreviewError> {
    let output = output_for_html("<H2 ALIGN='CENTER'>Uppercase centered heading</H2>".to_string())?;
    let plan = ViewerNodePlanner::create(&output.input, 0.0);

    assert_eq!("Uppercase centered heading", plan.nodes[0].text);
    assert!(
        matches!(
            plan.nodes[0].kind,
            ViewerNodeKind::Html {
                role: ViewerHtmlRole::Heading {
                    level: 2,
                    alignment: ViewerHtmlAlignment::Center
                }
            }
        ),
        "{:#?}",
        plan.nodes
    );
    Ok(())
}

#[test]
fn direct_html_source_does_not_emit_structural_container_text() -> Result<(), PreviewError> {
    let output = output_for_html(structural_container_html())?;
    let plan = ViewerNodePlanner::create(&output.input, 0.0);

    assert_no_structural_text(&plan);
    assert_no_empty_paragraph(&plan);
    Ok(())
}

#[test]
fn direct_html_source_preserves_right_aligned_anchor_span() -> Result<(), PreviewError> {
    let output = output_for_html(structural_container_html())?;
    let plan = ViewerNodePlanner::create(&output.input, 0.0);

    let node = plan
        .nodes
        .iter()
        .find(|node| node.text == "Right aligned link")
        .ok_or_else(|| PreviewError::Render("right aligned link node missing".to_string()))?;

    assert!(matches!(
        node.kind,
        ViewerNodeKind::Html {
            role: ViewerHtmlRole::Right
        }
    ));
    assert_eq!("https://example.com", node.spans[0].link_target);
    Ok(())
}

#[test]
fn direct_html_source_keeps_details_as_accordion_node() -> Result<(), PreviewError> {
    let output = output_for_html(structural_container_html())?;
    let plan = ViewerNodePlanner::create(&output.input, 0.0);

    assert!(plan.nodes.iter().any(|node| matches!(
        node.kind,
        ViewerNodeKind::Html {
            role: ViewerHtmlRole::Accordion
        }
    )));
    assert!(
        !plan
            .nodes
            .iter()
            .any(|node| node.text.trim() == "</details>")
    );
    Ok(())
}

#[test]
fn direct_html_source_keeps_table_as_table_node() -> Result<(), PreviewError> {
    let output = output_for_html(structural_container_html())?;
    let plan = ViewerNodePlanner::create(&output.input, 0.0);

    assert!(
        plan.nodes
            .iter()
            .any(|node| matches!(node.kind, ViewerNodeKind::Table)),
        "{:#?}",
        plan.nodes
    );
    Ok(())
}

fn output_for_html(content: String) -> Result<PreviewOutput, PreviewError> {
    output_for_html_path(content, "fixture.html")
}

fn output_for_html_path(content: String, document_id: &str) -> Result<PreviewOutput, PreviewError> {
    PreviewOutputFactory::from_source(
        &MarkdownSource {
            content,
            document_id: Some(document_id.to_string()),
        },
        &PreviewConfig {
            viewport: ViewerViewport {
                width: 640.0,
                height: 480.0,
            },
            ..PreviewConfig::default()
        },
        CONTENT_HEIGHT,
    )
}

fn centered_link_html() -> String {
    r#"<p align="center"><a href="https://example.com">Centered link</a></p>"#.to_string()
}

fn structural_container_html() -> String {
    [
        "<main>",
        r#"  <h1 align="center">KatanA HTML Fixture</h1>"#,
        r#"  <p style="text-align: right"><a href="https://example.com">Right aligned link</a></p>"#,
        "  <table>",
        "    <tr><td>Feature</td><td>Status</td></tr>",
        "  </table>",
        "  <details open>",
        "    <summary>Details title</summary>",
        "    <p>Details body</p>",
        "  </details>",
        "</main>",
    ]
    .join("\n")
}

fn assert_html_snapshot(output: &PreviewOutput) {
    assert_eq!(DocumentKind::Html, output.input.snapshot.kind);
    assert_eq!(
        PathBuf::from("fixture.html"),
        output.input.snapshot.source_path
    );
}

fn assert_centered_link_node(plan: &ViewerNodePlan) {
    assert!(matches!(
        plan.nodes[0].kind,
        ViewerNodeKind::Html {
            role: ViewerHtmlRole::Centered
        }
    ));
    assert_eq!("Centered link", plan.nodes[0].text);
    assert_eq!("https://example.com", plan.nodes[0].spans[0].link_target);
}

fn assert_no_structural_text(plan: &ViewerNodePlan) {
    assert!(
        plan.nodes
            .iter()
            .all(|node| !node.text.contains("<main>") && !node.text.contains("</main>")),
        "{:#?}",
        plan.nodes
    );
}

fn assert_no_empty_paragraph(plan: &ViewerNodePlan) {
    assert!(
        plan.nodes.iter().all(|node| {
            !matches!(node.kind, ViewerNodeKind::Paragraph) || !node.text.trim().is_empty()
        }),
        "{:#?}",
        plan.nodes
    );
}

#[path = "direct_html_source_script_tests.rs"]
mod script_tests;
