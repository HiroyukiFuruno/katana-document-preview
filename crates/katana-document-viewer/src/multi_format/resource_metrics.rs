use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};

static LIVE_DOCUMENT_SESSIONS: AtomicUsize = AtomicUsize::new(0);
static LIVE_SPREADSHEET_WORKERS: AtomicUsize = AtomicUsize::new(0);
static LIVE_WORKER_WORKSPACES: AtomicUsize = AtomicUsize::new(0);
static RETAINED_ARTIFACT_BYTES: AtomicUsize = AtomicUsize::new(0);
static CACHED_PAGE_COUNT: AtomicUsize = AtomicUsize::new(0);
static CACHED_PAGE_BYTES: AtomicUsize = AtomicUsize::new(0);
static CACHED_SPREADSHEET_CELL_COUNT: AtomicUsize = AtomicUsize::new(0);
static CACHED_SPREADSHEET_CELL_BYTES: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentResourceSnapshot {
    pub live_document_sessions: usize,
    pub live_spreadsheet_workers: usize,
    pub live_worker_workspaces: usize,
    pub retained_artifact_bytes: usize,
    pub cached_page_count: usize,
    pub cached_page_bytes: usize,
    pub cached_spreadsheet_cell_count: usize,
    pub cached_spreadsheet_cell_bytes: usize,
}

impl DocumentResourceSnapshot {
    #[must_use]
    pub fn capture() -> Self {
        Self {
            live_document_sessions: LIVE_DOCUMENT_SESSIONS.load(Ordering::Relaxed),
            live_spreadsheet_workers: LIVE_SPREADSHEET_WORKERS.load(Ordering::Relaxed),
            live_worker_workspaces: LIVE_WORKER_WORKSPACES.load(Ordering::Relaxed),
            retained_artifact_bytes: RETAINED_ARTIFACT_BYTES.load(Ordering::Relaxed),
            cached_page_count: CACHED_PAGE_COUNT.load(Ordering::Relaxed),
            cached_page_bytes: CACHED_PAGE_BYTES.load(Ordering::Relaxed),
            cached_spreadsheet_cell_count: CACHED_SPREADSHEET_CELL_COUNT.load(Ordering::Relaxed),
            cached_spreadsheet_cell_bytes: CACHED_SPREADSHEET_CELL_BYTES.load(Ordering::Relaxed),
        }
    }
}

pub(crate) struct DocumentSessionLease;

impl DocumentSessionLease {
    pub(crate) fn acquire() -> Self {
        LIVE_DOCUMENT_SESSIONS.fetch_add(1, Ordering::Relaxed);
        Self
    }
}

impl Drop for DocumentSessionLease {
    fn drop(&mut self) {
        LIVE_DOCUMENT_SESSIONS.fetch_sub(1, Ordering::Relaxed);
    }
}

pub(crate) struct SpreadsheetWorkerLease;

impl SpreadsheetWorkerLease {
    pub(crate) fn acquire() -> Self {
        LIVE_SPREADSHEET_WORKERS.fetch_add(1, Ordering::Relaxed);
        LIVE_WORKER_WORKSPACES.fetch_add(1, Ordering::Relaxed);
        Self
    }
}

impl Drop for SpreadsheetWorkerLease {
    fn drop(&mut self) {
        LIVE_SPREADSHEET_WORKERS.fetch_sub(1, Ordering::Relaxed);
        LIVE_WORKER_WORKSPACES.fetch_sub(1, Ordering::Relaxed);
    }
}

pub(crate) struct ArtifactByteLease {
    bytes: usize,
}

impl ArtifactByteLease {
    pub(crate) fn acquire(bytes: usize) -> Self {
        RETAINED_ARTIFACT_BYTES.fetch_add(bytes, Ordering::Relaxed);
        Self { bytes }
    }
}

impl Drop for ArtifactByteLease {
    fn drop(&mut self) {
        RETAINED_ARTIFACT_BYTES.fetch_sub(self.bytes, Ordering::Relaxed);
    }
}

pub(crate) struct CacheMetrics;

impl CacheMetrics {
    pub(crate) fn insert(bytes: usize) {
        CACHED_PAGE_COUNT.fetch_add(1, Ordering::Relaxed);
        CACHED_PAGE_BYTES.fetch_add(bytes, Ordering::Relaxed);
    }

    pub(crate) fn remove(bytes: usize) {
        CACHED_PAGE_COUNT.fetch_sub(1, Ordering::Relaxed);
        CACHED_PAGE_BYTES.fetch_sub(bytes, Ordering::Relaxed);
    }
}

pub(crate) struct SpreadsheetCacheMetrics;

impl SpreadsheetCacheMetrics {
    pub(crate) fn insert(bytes: usize) {
        CACHED_SPREADSHEET_CELL_COUNT.fetch_add(1, Ordering::Relaxed);
        CACHED_SPREADSHEET_CELL_BYTES.fetch_add(bytes, Ordering::Relaxed);
    }

    pub(crate) fn remove(bytes: usize) {
        CACHED_SPREADSHEET_CELL_COUNT.fetch_sub(1, Ordering::Relaxed);
        CACHED_SPREADSHEET_CELL_BYTES.fetch_sub(bytes, Ordering::Relaxed);
    }
}
