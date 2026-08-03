use crate::export_semantics::EvaluatedMarkdownFragment;
use crate::forge::BuildGraph;
use crate::theme::KdvThemeSnapshot;

use super::super::markup::{SurfaceDetailsParts, SurfaceHtmlMarkup};
use super::{SurfaceBlock, SurfaceBlockFactory};

impl SurfaceBlockFactory {
    pub(super) fn append_details(
        blocks: &mut Vec<SurfaceBlock>,
        graph: &BuildGraph,
        fragment: &str,
        quote_depth: u32,
        list_depth: u32,
        theme: &KdvThemeSnapshot,
    ) -> bool {
        let Some(parts) = SurfaceDetailsParts::parse(fragment) else {
            return false;
        };
        Self::append_details_summary(blocks, &parts, quote_depth, list_depth);
        Self::append_details_body(blocks, graph, &parts, quote_depth, list_depth, theme);
        true
    }

    fn append_details_summary(
        blocks: &mut Vec<SurfaceBlock>,
        parts: &SurfaceDetailsParts,
        quote_depth: u32,
        list_depth: u32,
    ) {
        Self::append_wrapped(
            blocks,
            SurfaceHtmlMarkup::normalize_text(parts.summary),
            quote_depth,
            list_depth,
        );
    }

    fn append_details_body(
        blocks: &mut Vec<SurfaceBlock>,
        graph: &BuildGraph,
        parts: &SurfaceDetailsParts,
        quote_depth: u32,
        list_depth: u32,
        theme: &KdvThemeSnapshot,
    ) {
        let fragment = EvaluatedMarkdownFragment::evaluate("surface-details.md", parts.body.trim());
        if !fragment.has_nodes() {
            Self::append_wrapped(
                blocks,
                SurfaceHtmlMarkup::normalize_text(parts.body),
                quote_depth,
                list_depth,
            );
            return;
        }
        let body_blocks =
            Self::details_body_blocks(graph, &fragment, quote_depth, list_depth, theme);
        blocks.extend(Self::non_blank_detail_blocks(body_blocks));
    }

    fn details_body_blocks(
        graph: &BuildGraph,
        fragment: &EvaluatedMarkdownFragment,
        quote_depth: u32,
        list_depth: u32,
        theme: &KdvThemeSnapshot,
    ) -> Vec<SurfaceBlock> {
        let mut blocks = Vec::new();
        for node in fragment.nodes() {
            Self::append_node(&mut blocks, graph, node, quote_depth, list_depth, theme);
        }
        blocks
    }

    fn non_blank_detail_blocks(blocks: Vec<SurfaceBlock>) -> Vec<SurfaceBlock> {
        let mut non_blank = Vec::with_capacity(blocks.len());
        for block in blocks {
            if !matches!(&block, SurfaceBlock::Line(line) if line.text.trim().is_empty()) {
                non_blank.push(block);
            }
        }
        non_blank
    }
}
