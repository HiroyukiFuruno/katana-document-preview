use super::{SurfaceTextLayout, SurfaceTextPainter};
use crate::export_surface_span::{SurfaceTextSpan, SurfaceTextStyle};
use image::{Rgba, RgbaImage};

#[test]
fn emoji_characters_are_not_rendered_as_blank_advance() -> Result<(), Box<dyn std::error::Error>> {
    let mut painter = SurfaceTextPainter::from_system_fonts();
    let background = Rgba([255, 255, 255, 255]);
    let mut image = RgbaImage::from_pixel(96, 64, background);

    painter.draw_text(
        &mut image,
        "🌍",
        SurfaceTextLayout {
            x: 8,
            y: 8,
            size: 32.0,
            color: Rgba([0, 0, 0, 255]),
            max_width: None,
        },
    );

    assert!(image.pixels().any(|pixel| *pixel != background));
    Ok(())
}

#[test]
fn emoji_span_preserves_color_pixels() {
    let mut painter = SurfaceTextPainter::from_system_fonts();
    let background = Rgba([16, 16, 16, 255]);
    let mut image = RgbaImage::from_pixel(120, 96, background);
    let spans = vec![SurfaceTextSpan::styled(
        "🔥",
        SurfaceTextStyle::default().emoji(),
    )];

    painter.draw_spans(&mut image, &spans, 16, 12, 64.0, Rgba([245, 245, 245, 255]));

    let chromatic_pixels = image
        .pixels()
        .filter(|pixel| **pixel != background)
        .filter(|pixel| is_chromatic(**pixel))
        .count();
    assert!(chromatic_pixels > 32);
}

#[test]
fn issue_14_emoji_sequence_paints_visible_pixels() {
    let mut painter = SurfaceTextPainter::from_system_fonts();
    let background = Rgba([255, 255, 255, 255]);
    let mut image = RgbaImage::from_pixel(720, 120, background);
    let spans = vec![SurfaceTextSpan::styled(
        "🧪 ✨ ✅ ⚠️ 🛠️ 🧑‍💻",
        SurfaceTextStyle::default().emoji(),
    )];

    painter.draw_spans(&mut image, &spans, 12, 16, 32.0, Rgba([36, 36, 36, 255]));

    let painted_pixels = image.pixels().filter(|pixel| **pixel != background).count();
    assert!(
        painted_pixels > 64,
        "issue #14 emoji sequence must not render as missing blank glyphs"
    );
}

#[test]
fn star_variation_sequence_paints_visible_pixels() {
    let mut painter = SurfaceTextPainter::from_system_fonts();
    let background = Rgba([255, 255, 255, 255]);
    let mut image = RgbaImage::from_pixel(160, 96, background);
    let spans = vec![SurfaceTextSpan::styled(
        "⭐️",
        SurfaceTextStyle::default().emoji(),
    )];

    painter.draw_spans(&mut image, &spans, 16, 12, 48.0, Rgba([36, 36, 36, 255]));

    let painted_pixels = image.pixels().filter(|pixel| **pixel != background).count();
    assert!(
        painted_pixels > 32,
        "star variation emoji must not render as missing blank glyphs"
    );
}

#[test]
fn platform_raster_keeps_japanese_and_zwj_grapheme_ranges_hittable()
-> Result<(), Box<dyn std::error::Error>> {
    let mut painter = SurfaceTextPainter::from_system_fonts();
    let spans = vec![
        SurfaceTextSpan::plain("日本語 "),
        SurfaceTextSpan::styled("🧑‍💻", SurfaceTextStyle::default().emoji()),
    ];

    let raster = painter
        .rasterize_spans(&spans, 32.0, 640.0, Rgba([36, 41, 47, 255]))
        .ok_or("platform text raster must accept KDV spans")?;
    let zwj_start = "日本語 ".len();
    let zwj_end = zwj_start + "🧑‍💻".len();
    let zwj_bounds = raster
        .grapheme_bounds
        .iter()
        .find(|bounds| bounds.byte_start == zwj_start && bounds.byte_end == zwj_end)
        .ok_or("ZWJ emoji must remain one grapheme range")?;
    let hit = raster
        .hit_test(
            zwj_bounds.x + zwj_bounds.width / 2.0,
            zwj_bounds.y + zwj_bounds.height / 2.0,
        )
        .ok_or("ZWJ emoji must be hit-testable")?;

    assert_eq!((hit.byte_start, hit.byte_end), (zwj_start, zwj_end));
    assert!(raster.width > 0 && raster.height > 0);
    assert!(!raster.rgba_pixels.is_empty());
    Ok(())
}

fn is_chromatic(pixel: Rgba<u8>) -> bool {
    pixel[0] != pixel[1] || pixel[1] != pixel[2]
}
