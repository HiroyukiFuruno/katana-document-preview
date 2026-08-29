use super::ViewerImageSurfaceFactory;
use super::image_surface_cache::ViewerImageSurfaceCache;
use std::sync::{Mutex, OnceLock};

const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10">
<rect x="0" y="0" width="20" height="10" fill="#222222"/>
</svg>"##;

#[test]
fn svg_surface_cache_reuses_same_fingerprint() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = cache_test_guard();
    ViewerImageSurfaceCache::clear_for_tests();

    let first = ViewerImageSurfaceFactory::from_svg_str("diagram-node", SVG, 120)?;
    let second = ViewerImageSurfaceFactory::from_svg_str("diagram-node", SVG, 120)?;

    assert_eq!(first, second);
    assert!(ViewerImageSurfaceCache::get(&first.fingerprint).is_some());
    assert!(ViewerImageSurfaceCache::contains_for_tests(
        &first.fingerprint
    ));
    Ok(())
}

#[test]
fn svg_surface_cache_keeps_scale_variants_separate() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = cache_test_guard();
    ViewerImageSurfaceCache::clear_for_tests();

    let standard = ViewerImageSurfaceFactory::from_svg_str("diagram-node", SVG, 120)?;
    let narrow = ViewerImageSurfaceFactory::from_svg_str("diagram-node", SVG, 10)?;

    assert_ne!(standard.fingerprint, narrow.fingerprint);
    assert!(ViewerImageSurfaceCache::get(&standard.fingerprint).is_some());
    assert!(ViewerImageSurfaceCache::get(&narrow.fingerprint).is_some());
    assert!(ViewerImageSurfaceCache::contains_for_tests(
        &standard.fingerprint
    ));
    assert!(ViewerImageSurfaceCache::contains_for_tests(
        &narrow.fingerprint
    ));
    Ok(())
}

#[test]
fn svg_surface_cache_limits_entries_and_displaces_overflow()
-> Result<(), Box<dyn std::error::Error>> {
    let _guard = cache_test_guard();
    ViewerImageSurfaceCache::clear_for_tests();

    let first = ViewerImageSurfaceFactory::from_svg_str("capacity", SVG, 120)?;
    let mut fingerprints = vec![first.fingerprint];
    for index in 0..66 {
        let max_width = 200 + index;
        let surface = ViewerImageSurfaceFactory::from_svg_str("capacity", SVG, max_width)?;
        fingerprints.push(surface.fingerprint);
    }

    assert!(ViewerImageSurfaceCache::len_for_tests() <= 64);
    assert!(
        fingerprints
            .iter()
            .filter(|fingerprint| ViewerImageSurfaceCache::get(fingerprint).is_none())
            .count()
            >= 3
    );
    Ok(())
}

fn cache_test_guard() -> std::sync::MutexGuard<'static, ()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    match GUARD.get_or_init(|| Mutex::new(())).lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
