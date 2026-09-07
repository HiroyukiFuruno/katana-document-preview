use super::{PdfRenderedPage, PdfViewerLimits};
use std::collections::{HashMap, VecDeque};

pub(super) type PdfRenderCacheKey = (usize, u32);

pub(super) struct PdfPageCache {
    pages: HashMap<PdfRenderCacheKey, PdfRenderedPage>,
    order: VecDeque<PdfRenderCacheKey>,
    bytes: usize,
}

impl PdfPageCache {
    pub(super) fn new() -> Self {
        Self {
            pages: HashMap::new(),
            order: VecDeque::new(),
            bytes: 0,
        }
    }

    pub(super) fn get(&mut self, key: PdfRenderCacheKey) -> Option<PdfRenderedPage> {
        let rendered = self.pages.get(&key).cloned()?;
        self.touch(key);
        Some(rendered)
    }

    pub(super) fn insert(
        &mut self,
        key: PdfRenderCacheKey,
        rendered: PdfRenderedPage,
        limits: PdfViewerLimits,
    ) {
        let page_bytes = rendered.surface.rgba.len();
        if limits.max_cached_pages == 0 || page_bytes > limits.max_cached_bytes {
            return;
        }
        if self.pages.contains_key(&key) {
            self.order.retain(|cached| *cached != key);
            self.evict(key);
        }
        self.evict_until_available(page_bytes, limits);
        self.bytes = self.bytes.saturating_add(page_bytes);
        super::resource_metrics::CacheMetrics::insert(page_bytes);
        self.order.push_back(key);
        self.pages.insert(key, rendered);
    }

    pub(super) fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub(super) const fn byte_count(&self) -> usize {
        self.bytes
    }

    fn touch(&mut self, key: PdfRenderCacheKey) {
        self.order.retain(|cached| *cached != key);
        self.order.push_back(key);
    }

    fn evict_until_available(&mut self, page_bytes: usize, limits: PdfViewerLimits) {
        while self.requires_eviction(page_bytes, limits) {
            if let Some(key) = self.order.pop_front() {
                self.evict(key);
            }
        }
    }

    fn requires_eviction(&self, page_bytes: usize, limits: PdfViewerLimits) -> bool {
        !self.order.is_empty()
            && (self.pages.len() >= limits.max_cached_pages
                || self.bytes.saturating_add(page_bytes) > limits.max_cached_bytes)
    }

    fn evict(&mut self, key: PdfRenderCacheKey) {
        if let Some(rendered) = self.pages.remove(&key) {
            let bytes = rendered.surface.rgba.len();
            self.bytes = self.bytes.saturating_sub(bytes);
            super::resource_metrics::CacheMetrics::remove(bytes);
        }
    }
}

impl Drop for PdfPageCache {
    fn drop(&mut self) {
        let bytes = self
            .pages
            .values()
            .map(|rendered| rendered.surface.rgba.len())
            .collect::<Vec<_>>();
        for bytes in bytes {
            super::resource_metrics::CacheMetrics::remove(bytes);
        }
    }
}

#[cfg(test)]
#[path = "pdf_render_cache_tests.rs"]
mod tests;
