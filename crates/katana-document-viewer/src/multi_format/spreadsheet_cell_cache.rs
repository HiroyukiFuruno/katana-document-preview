use super::{OfficeWorkerError, SpreadsheetCellArtifact, SpreadsheetCoordinate};
use std::collections::{HashMap, VecDeque};

const MAX_CACHED_CELLS: usize = 8_192;
const MAX_CACHED_BYTES: usize = 4 * 1024 * 1024;
type CacheKey = (usize, SpreadsheetCoordinate);

pub(super) struct SpreadsheetCellCache {
    cells: HashMap<CacheKey, (SpreadsheetCellArtifact, usize)>,
    order: VecDeque<CacheKey>,
    bytes: usize,
}

impl SpreadsheetCellCache {
    pub(super) fn new() -> Self {
        Self {
            cells: HashMap::new(),
            order: VecDeque::new(),
            bytes: 0,
        }
    }

    pub(super) fn missing(
        &self,
        sheet_index: usize,
        coordinates: &[SpreadsheetCoordinate],
    ) -> Vec<SpreadsheetCoordinate> {
        coordinates
            .iter()
            .copied()
            .filter(|coordinate| !self.cells.contains_key(&(sheet_index, *coordinate)))
            .collect()
    }

    pub(super) fn insert(&mut self, sheet_index: usize, cell: SpreadsheetCellArtifact) {
        let key = (sheet_index, cell.coordinate);
        if self.cells.contains_key(&key) {
            self.remove(key);
        }
        let bytes = cell_bytes(&cell);
        if bytes > MAX_CACHED_BYTES {
            return;
        }
        self.evict_until_available(bytes);
        self.cells.insert(key, (cell, bytes));
        self.order.push_back(key);
        self.bytes = self.bytes.saturating_add(bytes);
        super::resource_metrics::SpreadsheetCacheMetrics::insert(bytes);
    }

    pub(super) fn resolve(
        &mut self,
        sheet_index: usize,
        coordinates: &[SpreadsheetCoordinate],
    ) -> Result<Vec<SpreadsheetCellArtifact>, OfficeWorkerError> {
        let mut resolved = Vec::with_capacity(coordinates.len());
        for coordinate in coordinates {
            let key = (sheet_index, *coordinate);
            let Some((cell, _)) = self.cells.get(&key).cloned() else {
                return Err(OfficeWorkerError::protocol(format!(
                    "spreadsheet cell ({}, {}) was not materialized",
                    coordinate.row, coordinate.column
                )));
            };
            self.touch(key);
            resolved.push(cell);
        }
        Ok(resolved)
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.cells.len()
    }

    #[cfg(test)]
    pub(super) const fn byte_count(&self) -> usize {
        self.bytes
    }

    fn touch(&mut self, key: CacheKey) {
        self.order.retain(|candidate| *candidate != key);
        self.order.push_back(key);
    }

    fn evict_until_available(&mut self, incoming_bytes: usize) {
        while !self.order.is_empty()
            && (self.cells.len() >= MAX_CACHED_CELLS
                || self.bytes.saturating_add(incoming_bytes) > MAX_CACHED_BYTES)
        {
            if let Some(key) = self.order.pop_front() {
                self.remove(key);
            }
        }
    }

    fn remove(&mut self, key: CacheKey) {
        if let Some((_, bytes)) = self.cells.remove(&key) {
            self.bytes = self.bytes.saturating_sub(bytes);
            super::resource_metrics::SpreadsheetCacheMetrics::remove(bytes);
        }
    }
}

impl Drop for SpreadsheetCellCache {
    fn drop(&mut self) {
        let bytes = self
            .cells
            .values()
            .map(|(_, bytes)| *bytes)
            .collect::<Vec<_>>();
        for bytes in bytes {
            super::resource_metrics::SpreadsheetCacheMetrics::remove(bytes);
        }
    }
}

fn cell_bytes(cell: &SpreadsheetCellArtifact) -> usize {
    std::mem::size_of::<SpreadsheetCellArtifact>()
        .saturating_add(cell.display_text.len())
        .saturating_add(cell.formula.as_ref().map_or(0, String::len))
        .saturating_add(cell.style.font_name.len())
        .saturating_add(cell.style.number_format.len())
}

#[cfg(test)]
#[path = "spreadsheet_cell_cache_tests.rs"]
mod tests;
