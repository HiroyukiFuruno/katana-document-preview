use super::{ViewerAssetLoadPriority, ViewerNodeKind, ViewerNodePlanner};
use crate::{
    DocumentSnapshotFactory, DocumentSource, KdvThemeSnapshot, SourceKind, SourceRevision,
    SourceUri, ViewerInput, ViewerInteractionConfig, ViewerMode, ViewerSearchState, ViewerViewport,
};
use katana_markdown_model::{KatanaMarkdownModel, MarkdownInput};

const TEST_VIEWPORT_WIDTH: f32 = 640.0;
const TEST_VIEWPORT_HEIGHT: f32 = 160.0;

#[test]
fn node_plan_keeps_markdown_blocks_as_viewer_nodes() -> Result<(), Box<dyn std::error::Error>> {
    let input = viewer_input(
        "# Title\n\nBody\n\n```rust\nfn main() {}\n```\n\n```mermaid\ngraph TD\nA-->B\n```",
    )?;

    let plan = ViewerNodePlanner::create(&input, 128.0);

    assert!(matches!(
        plan.nodes[0].kind,
        ViewerNodeKind::Heading { level: 1 }
    ));
    assert!(matches!(plan.nodes[1].kind, ViewerNodeKind::Paragraph));
    assert!(matches!(plan.nodes[2].kind, ViewerNodeKind::Code { .. }));
    assert!(matches!(plan.nodes[3].kind, ViewerNodeKind::Diagram { .. }));
    assert_eq!(plan.asset_requests.len(), 1);
    assert_eq!(
        plan.asset_requests[0].priority,
        ViewerAssetLoadPriority::Visible
    );
    Ok(())
}

#[test]
fn node_plan_marks_offscreen_diagram_as_lazy_near_viewport()
-> Result<(), Box<dyn std::error::Error>> {
    let input = viewer_input(
        "# Title\n\nBody\n\nBody\n\nBody\n\nBody\n\nBody\n\nBody\n\n```mermaid\ngraph TD\nA-->B\n```",
    )?;

    let plan = ViewerNodePlanner::create(&input, 0.0);

    assert_eq!(plan.visible_artifact_ids.len(), 0);
    assert_eq!(plan.near_viewport_artifact_ids.len(), 1);
    assert_eq!(
        plan.asset_requests[0].priority,
        ViewerAssetLoadPriority::NearViewport
    );
    Ok(())
}

#[test]
fn node_plan_displays_markdown_blocks_without_source_markup()
-> Result<(), Box<dyn std::error::Error>> {
    let input = viewer_input(
        "<p align=\"center\"><img src=\"assets/kdv-icon.png\" alt=\"kdv icon\"></p>\n\n\
         ```rust\nfn main() {}\n```\n\n\
         - first\n- second\n\n\
         | A | B |\n|---|---|\n| 1 | 2 |\n\n\
         > [!NOTE]\n> readable note",
    )?;

    let plan = ViewerNodePlanner::create(&input, 0.0);

    assert!(matches!(plan.nodes[0].kind, ViewerNodeKind::Html { .. }));
    assert_eq!("kdv icon", plan.nodes[0].text);
    assert!(matches!(plan.nodes[1].kind, ViewerNodeKind::Code { .. }));
    assert_eq!("fn main() {}", plan.nodes[1].text);
    assert!(matches!(plan.nodes[2].kind, ViewerNodeKind::List));
    assert_eq!("- first\n- second", plan.nodes[2].text);
    assert!(matches!(plan.nodes[3].kind, ViewerNodeKind::Table));
    assert!(plan.nodes[3].text.contains("A | B"));
    assert!(matches!(plan.nodes[4].kind, ViewerNodeKind::Alert { .. }));
    assert_eq!("NOTE: readable note", plan.nodes[4].text);
    Ok(())
}

#[test]
fn node_plan_preserves_rich_html_style_boundaries() -> Result<(), Box<dyn std::error::Error>> {
    let input = viewer_input("<p>plain <strong>bold</strong> plain</p>")?;

    let plan = ViewerNodePlanner::create(&input, 0.0);

    assert!(matches!(plan.nodes[0].kind, ViewerNodeKind::Html { .. }));
    assert_eq!(3, plan.nodes[0].spans.len());
    assert_eq!("plain ", plan.nodes[0].spans[0].text);
    assert!(!plan.nodes[0].spans[0].style.bold);
    assert_eq!("bold", plan.nodes[0].spans[1].text);
    assert!(plan.nodes[0].spans[1].style.bold);
    assert_eq!(" plain", plan.nodes[0].spans[2].text);
    assert!(!plan.nodes[0].spans[2].style.bold);
    Ok(())
}

#[test]
fn node_plan_preserves_rich_html_line_break_boundaries() -> Result<(), Box<dyn std::error::Error>> {
    let input = viewer_input("<p>first<br>second</p>")?;

    let plan = ViewerNodePlanner::create(&input, 0.0);

    assert!(matches!(plan.nodes[0].kind, ViewerNodeKind::Html { .. }));
    assert_ne!("firstsecond", plan.nodes[0].text);
    assert_eq!("first", plan.nodes[0].spans[0].text);
    assert_eq!("\n", plan.nodes[0].spans[1].text);
    assert_eq!("second", plan.nodes[0].spans[2].text);
    Ok(())
}

fn viewer_input(markdown: &str) -> Result<ViewerInput, Box<dyn std::error::Error>> {
    let document = KatanaMarkdownModel::parse(MarkdownInput::from_content(
        "node-plan.md",
        markdown.to_string(),
    ))?;
    let source = DocumentSource {
        uri: SourceUri("preview://node-plan.md".to_string()),
        kind: SourceKind::Markdown,
        revision: SourceRevision(document.fingerprint.value.clone()),
        content: markdown.to_string(),
    };
    Ok(ViewerInput {
        snapshot: DocumentSnapshotFactory::from_kmm(source, document),
        artifacts: Vec::new(),
        theme: KdvThemeSnapshot::default(),
        mode: ViewerMode::Document,
        interaction: ViewerInteractionConfig::default(),
        typography: crate::ViewerTypographyConfig::default(),
        viewport: ViewerViewport {
            width: TEST_VIEWPORT_WIDTH,
            height: TEST_VIEWPORT_HEIGHT,
        },
        search: ViewerSearchState::default(),
    })
}
