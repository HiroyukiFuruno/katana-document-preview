use super::ViewerImageSurface;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock};

const MAX_CACHED_SURFACES: usize = 64;

pub(super) struct ViewerImageSurfaceCache;

impl ViewerImageSurfaceCache {
    pub(super) fn get(fingerprint: &str) -> Option<ViewerImageSurface> {
        cache_lock().get(fingerprint).cloned()
    }

    pub(super) fn put(surface: ViewerImageSurface) -> ViewerImageSurface {
        let mut cache = cache_lock();
        if cache.len() >= MAX_CACHED_SURFACES
            && let Some(first_key) = cache.keys().next().cloned()
        {
            cache.remove(&first_key);
        }
        cache.insert(surface.fingerprint.clone(), surface.clone());
        surface
    }

    #[cfg(test)]
    pub(super) fn clear_for_tests() {
        cache_lock().clear();
    }

    #[cfg(test)]
    pub(super) fn len_for_tests() -> usize {
        cache_lock().len()
    }

    #[cfg(test)]
    pub(super) fn contains_for_tests(fingerprint: &str) -> bool {
        cache_lock().contains_key(fingerprint)
    }
}

fn cache() -> &'static Mutex<HashMap<String, ViewerImageSurface>> {
    static CACHE: OnceLock<Mutex<HashMap<String, ViewerImageSurface>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cache_lock() -> MutexGuard<'static, HashMap<String, ViewerImageSurface>> {
    match cache().lock() {
        Ok(cache) => cache,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
#[path = "image_surface_cache_private_tests.rs"]
mod private_tests;
