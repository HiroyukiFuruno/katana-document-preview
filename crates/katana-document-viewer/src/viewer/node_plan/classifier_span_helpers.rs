use super::super::super::types::{ViewerTextSpan, ViewerTextStyle};
use super::ViewerNodeClassifier;
use crate::emoji_text::EmojiTextSegments;
use crate::export_surface_text::SurfaceTextParser as TextParser;
use katana_markdown_model::{FootnoteDefinitionNode, ListItemNode, ListNode};

impl ViewerNodeClassifier {
    pub(super) fn footnote_reference_span(
        label: &str,
        style: ViewerTextStyle,
    ) -> Vec<ViewerTextSpan> {
        Self::linked_span(Self::footnote_text(label), format!("#fn-{label}"), style)
    }

    pub(super) fn footnote_definition_spans(
        definition: &FootnoteDefinitionNode,
    ) -> Vec<ViewerTextSpan> {
        vec![
            ViewerTextSpan::plain(format!("{}. ", definition.label)),
            ViewerTextSpan::plain(definition.text.clone()),
            ViewerTextSpan::plain(" "),
            ViewerTextSpan::linked(
                "↩",
                format!("#fnref-{}", definition.label),
                ViewerTextStyle::default().link(),
            ),
        ]
    }

    pub(super) fn list_spans(list: &ListNode) -> Vec<ViewerTextSpan> {
        let mut spans = Vec::new();
        for (index, item) in list.items.iter().enumerate() {
            if index > 0 {
                spans.push(ViewerTextSpan::plain("\n"));
            }
            spans.extend(Self::list_item_spans(item, list.ordered));
        }
        spans
    }

    fn list_item_spans(item: &ListItemNode, ordered: bool) -> Vec<ViewerTextSpan> {
        let mut spans = vec![ViewerTextSpan::plain(Self::list_marker(item, ordered))];
        let body = Self::inline_nodes_spans(&item.body, ViewerTextStyle::default());
        if !body.is_empty() {
            spans.push(ViewerTextSpan::plain(" "));
            spans.extend(body);
        }
        spans
    }

    pub(super) fn plain_span(text: &str, style: ViewerTextStyle) -> Vec<ViewerTextSpan> {
        Self::styled_span(TextParser::decode_basic_entities(text), style)
    }

    pub(super) fn styled_span(text: String, style: ViewerTextStyle) -> Vec<ViewerTextSpan> {
        if text.is_empty() {
            return Vec::new();
        }
        EmojiTextSegments::split(&text)
            .into_iter()
            .map(|segment| {
                let segment_style = if segment.emoji { style.emoji() } else { style };
                ViewerTextSpan::styled(segment.text.as_str(), segment_style)
            })
            .collect()
    }

    pub(super) fn linked_span(
        text: String,
        target: String,
        style: ViewerTextStyle,
    ) -> Vec<ViewerTextSpan> {
        if text.is_empty() {
            return Vec::new();
        }
        Self::styled_span(text, style.link())
            .into_iter()
            .map(|mut span| {
                span.link_target = target.clone();
                span
            })
            .collect()
    }
}
