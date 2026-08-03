use super::{PdfPageRenderRequest, PdfViewerLimits};

#[test]
fn pdf_request_and_strict_limits_are_stable() {
    assert_eq!(
        PdfPageRenderRequest {
            page_index: 7,
            scale: 1.25,
        },
        PdfPageRenderRequest::new(7, 1.25)
    );

    assert_eq!(
        PdfViewerLimits {
            max_source_bytes: 256 * 1024 * 1024,
            max_pages: 10_000,
            max_render_dimension: 8_192,
            max_render_pixels: 16_777_216,
            max_cached_pages: 4,
            max_cached_bytes: 128 * 1024 * 1024,
        },
        PdfViewerLimits::strict()
    );
}
