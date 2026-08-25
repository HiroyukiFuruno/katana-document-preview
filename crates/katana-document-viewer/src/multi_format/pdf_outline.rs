use crate::PdfOutlineItem;
use hayro::hayro_syntax::Pdf;
use hayro::hayro_syntax::object::{Array, Dict, ObjectIdentifier, String as PdfString};
use std::collections::{HashMap, HashSet};

const MAX_OUTLINE_ITEMS: usize = 10_000;
const MAX_OUTLINE_DEPTH: usize = 64;

pub(super) struct PdfOutlineBuilder;

impl PdfOutlineBuilder {
    pub(super) fn build(pdf: &Pdf) -> Vec<PdfOutlineItem> {
        let page_indices = pdf
            .pages()
            .iter()
            .enumerate()
            .filter_map(|(index, page)| page.raw().obj_id().map(|id| (id, index)))
            .collect::<HashMap<_, _>>();
        let first = pdf
            .xref()
            .get::<Dict<'_>>(pdf.xref().root_id())
            .and_then(|catalog| catalog.get::<Dict<'_>>(b"Outlines"))
            .and_then(|outlines| outlines.get::<Dict<'_>>(b"First"));
        let mut items = Vec::new();
        let mut visited = HashSet::new();
        walk_siblings(first, 1, &page_indices, &mut visited, &mut items);
        items
    }
}

fn walk_siblings<'a>(
    mut current: Option<Dict<'a>>,
    level: usize,
    page_indices: &HashMap<ObjectIdentifier, usize>,
    visited: &mut HashSet<ObjectIdentifier>,
    items: &mut Vec<PdfOutlineItem>,
) {
    while let Some(item) = current {
        if items.len() >= MAX_OUTLINE_ITEMS || level > MAX_OUTLINE_DEPTH {
            return;
        }
        if item.obj_id().is_some_and(|id| !visited.insert(id)) {
            return;
        }
        append_outline_item(&item, level, page_indices, items);
        walk_siblings(
            item.get::<Dict<'_>>(b"First"),
            level.saturating_add(1),
            page_indices,
            visited,
            items,
        );
        current = item.get::<Dict<'_>>(b"Next");
    }
}

fn append_outline_item(
    item: &Dict<'_>,
    level: usize,
    page_indices: &HashMap<ObjectIdentifier, usize>,
    items: &mut Vec<PdfOutlineItem>,
) {
    if let Some(title) = item.get::<PdfString<'_>>(b"Title").map(decode_title)
        && !title.is_empty()
    {
        items.push(PdfOutlineItem {
            title,
            level,
            page_index: destination_page(item, page_indices),
        });
    }
}

fn destination_page(
    item: &Dict<'_>,
    page_indices: &HashMap<ObjectIdentifier, usize>,
) -> Option<usize> {
    let destination = item.get::<Array<'_>>(b"Dest").or_else(|| {
        item.get::<Dict<'_>>(b"A")
            .and_then(|action| action.get::<Array<'_>>(b"D"))
    })?;
    let page = destination.raw_iter().next()?.as_obj_ref()?;
    page_indices.get(&page.into()).copied()
}

fn decode_title(value: PdfString<'_>) -> String {
    let bytes = value.as_bytes();
    if let Some(utf16) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        let units = utf16
            .chunks_exact(2)
            .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
            .collect::<Vec<_>>();
        String::from_utf16_lossy(&units)
    } else {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

#[cfg(test)]
#[path = "pdf_outline_tests.rs"]
mod tests;
