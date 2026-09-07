use crate::document_viewer::node_labels::KucNodeLabels;
use katana_document_viewer::{
    ViewerDiagramKind, ViewerHtmlRole, ViewerNode, ViewerNodeKind, ViewerRect,
};
use katana_markdown_model::{
    ByteRange, KmmNodeId, LineColumn, LineColumnRange, RawSnippet, SourceSpan,
};

const LABEL_FIXTURE_WIDTH: f32 = 120.0;
const LABEL_FIXTURE_HEIGHT: f32 = 32.0;

#[test]
fn media_label_handles_diagram_and_default_media_kinds() {
    let math = viewer_node(ViewerNodeKind::Math, "1+1");
    let drawio = viewer_node(
        ViewerNodeKind::Diagram {
            kind: ViewerDiagramKind::DrawIo,
        },
        "diagram",
    );

    assert_eq!("math", KucNodeLabels::media_label(&math));
    assert_eq!("diagram:DrawIo", KucNodeLabels::media_label(&drawio));
}

#[test]
fn rendering_label_default_branch_for_unsupported_nodes() {
    assert_eq!(
        "Rendering Math...",
        KucNodeLabels::rendering_label(&viewer_node(ViewerNodeKind::Math, "x"))
    );
    assert_eq!(
        "Loading media...",
        KucNodeLabels::rendering_label(&viewer_node(ViewerNodeKind::Paragraph, "p"))
    );
}

#[test]
fn label_rule_nodes_use_separator() {
    assert_eq!(
        "-----",
        KucNodeLabels::label(&viewer_node(ViewerNodeKind::Rule, ""))
    );
}

#[test]
fn table_label_uses_viewer_text_without_raw_separator_row() {
    let raw = "| Left | Center | Right |\n| :--- | :---: | ---: |\n| A | B | C |";
    let mut node = viewer_node(ViewerNodeKind::Table, "Left | Center | Right\nA | B | C");
    node.source = source(raw);

    assert_eq!(
        "Left | Center | Right\nA | B | C",
        KucNodeLabels::label(&node)
    );
}

#[test]
fn table_label_keeps_viewer_text_when_source_has_no_separator_row() {
    let mut node = viewer_node(ViewerNodeKind::Table, "Left | Center\nA | B");
    node.source = source("Left | Center\nA | B");

    assert_eq!("Left | Center\nA | B", KucNodeLabels::label(&node));
}

#[test]
fn text_roles_cover_html_and_blockquote_cases() {
    assert_eq!(
        "html-accordion-preview",
        KucNodeLabels::text_role(&ViewerNodeKind::Html {
            role: ViewerHtmlRole::Accordion,
        })
    );
    assert_eq!("list", KucNodeLabels::text_role(&ViewerNodeKind::List));
    assert_eq!(
        "blockquote",
        KucNodeLabels::text_role(&ViewerNodeKind::BlockQuote)
    );
    assert_eq!(
        "paragraph",
        KucNodeLabels::text_role(&ViewerNodeKind::Paragraph),
        "interactive paragraphs must use KatanA's unadjusted compact wrap role"
    );
}

#[test]
fn heading_text_roles_keep_level_specific_metrics() {
    assert_eq!(
        "heading",
        KucNodeLabels::text_role(&ViewerNodeKind::Heading { level: 1 })
    );
    assert_eq!(
        "heading-2",
        KucNodeLabels::text_role(&ViewerNodeKind::Heading { level: 2 })
    );
    assert_eq!(
        "heading-3",
        KucNodeLabels::text_role(&ViewerNodeKind::Heading { level: 3 })
    );
}

#[test]
fn export_surface_html_heading_uses_body_alignment_role() {
    assert_eq!(
        "html-centered",
        KucNodeLabels::export_surface_text_role(&ViewerNodeKind::Html {
            role: ViewerHtmlRole::Heading {
                level: 1,
                alignment: katana_document_viewer::ViewerHtmlAlignment::Center,
            },
        })
    );
}

#[test]
fn export_surface_body_uses_export_font_role() {
    assert_eq!(
        "document-export-body",
        KucNodeLabels::export_surface_font_role(&ViewerNodeKind::Paragraph)
    );
    assert_eq!(
        "body",
        KucNodeLabels::export_surface_text_role(&ViewerNodeKind::Paragraph),
        "the preview-only paragraph role must not alter the export contract"
    );
}

#[test]
fn export_surface_markdown_heading_uses_export_metric_role() {
    assert_eq!(
        "heading-3-export",
        KucNodeLabels::export_surface_text_role(&ViewerNodeKind::Heading { level: 3 })
    );
}

#[test]
fn table_uses_body_font_role_for_export_surface_parity() {
    assert_eq!(
        "document-body",
        KucNodeLabels::font_role(&ViewerNodeKind::Table)
    );
    assert_eq!(
        "document-code",
        KucNodeLabels::font_role(&ViewerNodeKind::Code {
            language: Some("rust".to_string()),
        })
    );
}

#[test]
fn diagram_label_for_drawio_and_plantuml() {
    assert_eq!(
        "Rendering Draw.io...",
        KucNodeLabels::rendering_label(&viewer_node(
            ViewerNodeKind::Diagram {
                kind: ViewerDiagramKind::DrawIo,
            },
            "drawio",
        ))
    );
    assert_eq!(
        "Rendering PlantUML...",
        KucNodeLabels::rendering_label(&viewer_node(
            ViewerNodeKind::Diagram {
                kind: ViewerDiagramKind::PlantUml,
            },
            "plantuml",
        ))
    );
}

fn viewer_node(kind: ViewerNodeKind, text: &str) -> ViewerNode {
    ViewerNode {
        node_id: KmmNodeId("node".to_string()),
        kind,
        source: source(text),
        text: text.to_string(),
        spans: Vec::new(),
        html_margin_left_px: 0,
        rule_line_offset_px: 0,
        rect: ViewerRect {
            x: 0.0,
            y: 0.0,
            width: LABEL_FIXTURE_WIDTH,
            height: LABEL_FIXTURE_HEIGHT,
        },
        artifact_id: None,
    }
}

fn source(raw: &str) -> SourceSpan {
    SourceSpan {
        byte_range: ByteRange {
            start: 0,
            end: raw.len(),
        },
        line_column_range: LineColumnRange {
            start: LineColumn { line: 1, column: 1 },
            end: LineColumn {
                line: 1,
                column: raw.len() + 1,
            },
        },
        raw: RawSnippet {
            text: raw.to_string(),
        },
    }
}
