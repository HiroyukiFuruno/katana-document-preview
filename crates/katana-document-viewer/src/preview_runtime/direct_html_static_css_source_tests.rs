use super::{MarkdownSource, PreviewConfig, PreviewError, PreviewOutput, PreviewOutputFactory};
use crate::{ViewerNodePlan, ViewerNodePlanner, ViewerViewport};

const CONTENT_HEIGHT: f32 = 120.0;

#[test]
fn direct_html_source_applies_static_css_without_metadata_body_text() -> Result<(), PreviewError> {
    let output = output_for_html(static_css_document_html())?;
    let plan = ViewerNodePlanner::create(&output.input, 0.0);

    assert_hidden_text_is_absent(&plan);
    assert_visible_node_has_css_style(&plan)?;
    Ok(())
}

fn output_for_html(content: String) -> Result<PreviewOutput, PreviewError> {
    PreviewOutputFactory::from_source(
        &MarkdownSource {
            content,
            document_id: Some("fixture.html".to_string()),
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

fn assert_hidden_text_is_absent(plan: &ViewerNodePlan) {
    for fragment in ["Hidden metadata", "body { color", "window.bad"] {
        assert!(
            !plan.nodes.iter().any(|node| node.text.contains(fragment)),
            "{:#?}",
            plan.nodes
        );
    }
}

fn assert_visible_node_has_css_style(plan: &ViewerNodePlan) -> Result<(), PreviewError> {
    let node = plan
        .nodes
        .iter()
        .find(|node| node.text == "Visible")
        .ok_or_else(|| PreviewError::Render("visible html node missing".to_string()))?;
    assert_eq!([255, 0, 0, 255], node.spans[0].style.color_rgba);
    assert!(node.spans[0].style.bold);
    Ok(())
}

fn static_css_document_html() -> String {
    [
        "<!doctype html>",
        "<html>",
        "<head>",
        "<title>Hidden metadata</title>",
        "<style>",
        "body { color: red; }",
        ".note { font-weight: bold; }",
        "</style>",
        "<script>window.bad = true;</script>",
        "</head>",
        "<body>",
        r#"<p class="note">Visible</p>"#,
        "</body>",
        "</html>",
    ]
    .join("\n")
}
