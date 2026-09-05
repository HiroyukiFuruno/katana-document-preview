use katana_document_viewer::{
    ViewerDiagramKind, ViewerHtmlAlignment, ViewerHtmlRole, ViewerNode, ViewerNodeKind,
};

const DOCUMENT_BODY_FONT_ROLE: &str = "document-body";
pub(crate) const DOCUMENT_EXPORT_BODY_FONT_ROLE: &str = "document-export-body";
pub(crate) const CODE_FONT_ROLE: &str = "document-code";

pub(crate) struct KucNodeLabels;

impl KucNodeLabels {
    pub(crate) fn media_label(node: &ViewerNode) -> String {
        match &node.kind {
            ViewerNodeKind::Diagram { kind } => format!("diagram:{kind:?}"),
            ViewerNodeKind::Image => "image".to_string(),
            ViewerNodeKind::Math => "math".to_string(),
            _ => "media".to_string(),
        }
    }

    pub(crate) fn rendering_label(node: &ViewerNode) -> String {
        match &node.kind {
            ViewerNodeKind::Diagram { kind } => {
                format!("Rendering {}...", Self::diagram_label(kind))
            }
            ViewerNodeKind::Image => "Loading image...".to_string(),
            ViewerNodeKind::Math => "Rendering Math...".to_string(),
            _ => "Loading media...".to_string(),
        }
    }

    pub(crate) fn label(node: &ViewerNode) -> String {
        match &node.kind {
            ViewerNodeKind::Rule => String::from("-----"),
            ViewerNodeKind::Table => node.text.clone(),
            _ => node.text.clone(),
        }
    }

    pub(crate) fn font_role(kind: &ViewerNodeKind) -> &'static str {
        match kind {
            ViewerNodeKind::Code { .. } => CODE_FONT_ROLE,
            _ => DOCUMENT_BODY_FONT_ROLE,
        }
    }

    pub(crate) fn export_surface_font_role(kind: &ViewerNodeKind) -> &'static str {
        match kind {
            ViewerNodeKind::Code { .. } => CODE_FONT_ROLE,
            _ => DOCUMENT_EXPORT_BODY_FONT_ROLE,
        }
    }

    pub(crate) fn text_role(kind: &ViewerNodeKind) -> &'static str {
        match kind {
            ViewerNodeKind::Heading { level: 1 } => "heading",
            ViewerNodeKind::Heading { level: 2 } => "heading-2",
            ViewerNodeKind::Heading { .. } => "heading-3",
            ViewerNodeKind::Code { .. } => "code",
            ViewerNodeKind::Html {
                role: ViewerHtmlRole::Heading { alignment, .. },
            } => Self::html_heading_text_role(*alignment),
            ViewerNodeKind::Html {
                role: ViewerHtmlRole::Centered,
            } => "html-centered-preview",
            ViewerNodeKind::Html {
                role: ViewerHtmlRole::Right,
            } => "html-right-preview",
            ViewerNodeKind::Html {
                role: ViewerHtmlRole::Left,
            } => "html-left-preview",
            ViewerNodeKind::Html {
                role: ViewerHtmlRole::Accordion,
            } => "html-accordion-preview",
            ViewerNodeKind::Html { .. } => "html-block-preview",
            ViewerNodeKind::Paragraph => "paragraph",
            ViewerNodeKind::Table => "table",
            ViewerNodeKind::List => "list",
            ViewerNodeKind::Alert { .. } => "alert",
            ViewerNodeKind::BlockQuote => "blockquote",
            ViewerNodeKind::FootnoteDefinition { .. } => "footnote",
            _ => "body",
        }
    }

    pub(crate) fn export_surface_text_role(kind: &ViewerNodeKind) -> &'static str {
        match kind {
            ViewerNodeKind::Heading { level: 1 } => "heading-export",
            ViewerNodeKind::Heading { level: 2 } => "heading-2-export",
            ViewerNodeKind::Heading { .. } => "heading-3-export",
            ViewerNodeKind::Html {
                role: ViewerHtmlRole::Heading { alignment, .. },
            } => Self::html_alignment_text_role(*alignment),
            ViewerNodeKind::Html {
                role: ViewerHtmlRole::Centered,
            } => "html-centered",
            ViewerNodeKind::Html {
                role: ViewerHtmlRole::Right,
            } => "html-right",
            ViewerNodeKind::Html {
                role: ViewerHtmlRole::Left,
            } => "html-left",
            ViewerNodeKind::Html {
                role: ViewerHtmlRole::Accordion,
            } => "html-accordion",
            ViewerNodeKind::Html { .. } => "html-block",
            // export の既存 wrap 契約は維持し、KatanA preview だけ body 幅補正を避ける。
            ViewerNodeKind::Paragraph => "body",
            _ => Self::text_role(kind),
        }
    }

    fn html_heading_text_role(alignment: ViewerHtmlAlignment) -> &'static str {
        match alignment {
            ViewerHtmlAlignment::Center => "heading-html-centered",
            ViewerHtmlAlignment::Right => "heading-html-right",
            ViewerHtmlAlignment::Left => "heading-html-left",
        }
    }

    fn html_alignment_text_role(alignment: ViewerHtmlAlignment) -> &'static str {
        match alignment {
            ViewerHtmlAlignment::Center => "html-centered",
            ViewerHtmlAlignment::Right => "html-right",
            ViewerHtmlAlignment::Left => "html-left",
        }
    }

    fn diagram_label(kind: &ViewerDiagramKind) -> &'static str {
        match kind {
            ViewerDiagramKind::Mermaid => "Mermaid",
            ViewerDiagramKind::DrawIo => "Draw.io",
            ViewerDiagramKind::PlantUml => "PlantUML",
        }
    }
}
