use super::{OfficeWorkerError, SpreadsheetCellArtifact, SpreadsheetCoordinate};
use std::collections::{HashMap, VecDeque};

#[path = "spreadsheet_cell_cache_bytes.rs"]
mod bytes;
#[path = "spreadsheet_cell_cache_materialized.rs"]
mod materialized;

use bytes::cell_bytes;
use materialized::SpreadsheetMaterializedResponse;

const MAX_CACHED_CELLS: usize = 8_192;
const MAX_CACHED_BYTES: usize = 4 * 1024 * 1024;
type CacheKey = (usize, SpreadsheetCoordinate);

pub(super) struct SpreadsheetCellCache {
    cells: HashMap<CacheKey, (SpreadsheetCellArtifact, usize)>,
    order: VecDeque<CacheKey>,
    bytes: usize,
    max_cells: usize,
    max_bytes: usize,
}

impl SpreadsheetCellCache {
    pub(super) fn new() -> Self {
        Self {
            cells: HashMap::new(),
            order: VecDeque::new(),
            bytes: 0,
            max_cells: MAX_CACHED_CELLS,
            max_bytes: MAX_CACHED_BYTES,
        }
    }

    #[cfg(test)]
    pub(super) fn with_limits(max_cells: usize, max_bytes: usize) -> Self {
        Self {
            cells: HashMap::new(),
            order: VecDeque::new(),
            bytes: 0,
            max_cells,
            max_bytes,
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
        if bytes > self.max_bytes {
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

    pub(super) fn resolve_materialized(
        &mut self,
        sheet_index: usize,
        coordinates: &[SpreadsheetCoordinate],
        materialized: Vec<SpreadsheetCellArtifact>,
    ) -> Result<Vec<SpreadsheetCellArtifact>, OfficeWorkerError> {
        let cached = self.snapshot_requested_cells(sheet_index, coordinates);
        let fresh = SpreadsheetMaterializedResponse::from_cells(coordinates, materialized)?;
        self.cache_materialized_cells(sheet_index, coordinates, &fresh);
        fresh.resolve(coordinates, &cached)
    }

    fn snapshot_requested_cells(
        &mut self,
        sheet_index: usize,
        coordinates: &[SpreadsheetCoordinate],
    ) -> HashMap<SpreadsheetCoordinate, SpreadsheetCellArtifact> {
        let mut cached = HashMap::with_capacity(coordinates.len());
        for coordinate in coordinates {
            let key = (sheet_index, *coordinate);
            if let Some((cell, _)) = self.cells.get(&key).cloned() {
                self.touch(key);
                cached.insert(*coordinate, cell);
            }
        }
        cached
    }

    fn cache_materialized_cells(
        &mut self,
        sheet_index: usize,
        coordinates: &[SpreadsheetCoordinate],
        fresh: &SpreadsheetMaterializedResponse,
    ) {
        for coordinate in coordinates {
            if let Some(cell) = fresh.cell(*coordinate) {
                self.insert(sheet_index, cell.clone());
            }
        }
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
            && (self.cells.len() >= self.max_cells
                || self.bytes.saturating_add(incoming_bytes) > self.max_bytes)
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

#[cfg(test)]
#[path = "spreadsheet_cell_cache_bytes_tests.rs"]
mod byte_tests;
#[cfg(test)]
#[path = "spreadsheet_cell_cache_tests.rs"]
mod tests;
