use super::{PdfPageCache, PdfRenderedPage, PdfViewerLimits};
use crate::ViewerImageSurface;

fn page(index: usize, byte_count: usize) -> PdfRenderedPage {
    PdfRenderedPage {
        page_index: index,
        scale: 1.0,
        surface: ViewerImageSurface {
            fingerprint: format!("page-{index}"),
            width: 1,
            height: 1,
            display_width: 1.0,
            display_height: 1.0,
            content_scale: 1,
            rgba: vec![0; byte_count],
        },
    }
}

#[test]
fn cache_hits_touch_entries_and_both_limits_evict() {
    let mut cache = PdfPageCache::new();
    let mut limits = PdfViewerLimits::strict();
    limits.max_cached_pages = 2;
    limits.max_cached_bytes = 8;

    assert_eq!(None, cache.get((0, 1.0_f32.to_bits())));
    cache.insert((0, 1.0_f32.to_bits()), page(0, 4), limits);
    cache.insert((0, 1.0_f32.to_bits()), page(0, 4), limits);
    cache.insert((1, 1.0_f32.to_bits()), page(1, 4), limits);
    assert_eq!(Some(page(0, 4)), cache.get((0, 1.0_f32.to_bits())));

    cache.insert((2, 1.0_f32.to_bits()), page(2, 4), limits);
    assert_eq!(None, cache.get((1, 1.0_f32.to_bits())));
    assert_eq!(2, cache.page_count());
    assert_eq!(8, cache.byte_count());

    let mut byte_limited = PdfPageCache::new();
    byte_limited.insert((0, 0), page(0, 6), limits);
    byte_limited.insert((1, 0), page(1, 6), limits);
    assert_eq!(1, byte_limited.page_count());
    assert_eq!(6, byte_limited.byte_count());
}

#[test]
fn cache_rejects_disabled_and_oversized_entries() {
    let mut cache = PdfPageCache::new();
    let mut limits = PdfViewerLimits::strict();
    limits.max_cached_pages = 0;
    cache.insert((0, 0), page(0, 4), limits);
    assert_eq!(0, cache.page_count());

    limits.max_cached_pages = 1;
    limits.max_cached_bytes = 3;
    cache.insert((0, 0), page(0, 4), limits);
    assert_eq!(0, cache.page_count());
    assert_eq!(0, cache.byte_count());
}
