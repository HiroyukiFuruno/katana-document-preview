use super::{ViewerImageSurfaceError, ViewerImageSurfaceFactory};
use crate::{
    ArtifactBytes, ArtifactDiagnostics, ArtifactFactory, ArtifactFormat, ArtifactId, DocumentId,
    SourceRevision,
};
use image::ImageEncoder;
use image::codecs::png::PngEncoder;

const RED_PIXEL_RGBA: [u8; 4] = [u8::MAX, 0, 0, u8::MAX];

#[test]
fn diagram_display_scale_keeps_katana_intrinsic_viewer_size() {
    assert_eq!(
        1.0,
        super::VIEWER_DIAGRAM_DISPLAY_SCALE,
        "diagram display size preserves the KatanA intrinsic size while avoiding a hard 640px cap"
    );
}

#[test]
fn svg_artifact_rasterizes_to_ui_independent_rgba_surface() -> Result<(), Box<dyn std::error::Error>>
{
    let artifact = image_artifact(
        ArtifactId("doc:diagram:Svg".to_string()),
        ArtifactFormat::Svg,
        svg().as_bytes().to_vec(),
    );

    let surface = ViewerImageSurfaceFactory::from_artifact(&artifact, 200)?;

    assert!(surface.fingerprint.starts_with("doc:diagram:Svg:bytes="));
    assert_svg_surface_contract(&surface);
    Ok(())
}

fn assert_svg_surface_contract(surface: &super::ViewerImageSurface) {
    assert!(surface.fingerprint.contains(":scale=200:"));
    assert!(
        surface
            .fingerprint
            .contains(":renderer=image-surface-resvg-usvg-system-embedded-epaint-fonts-v3")
    );
    assert_eq!(80, surface.width);
    assert_eq!(40, surface.height);
    assert_eq!(40.0, surface.display_width);
    assert_eq!(20.0, surface.display_height);
    assert_eq!(200, surface.content_scale);
    assert_eq!(40, surface.logical_width());
    assert_eq!(20, surface.logical_height());
    assert_eq!(80 * 40 * 4, surface.rgba.len());
}

#[test]
fn invalid_svg_returns_error_instead_of_fallback_surface() {
    let result = ViewerImageSurfaceFactory::from_svg_str("bad", "<svg><rect>", 200);

    assert_eq!(Err(ViewerImageSurfaceError::InvalidSvg), result);
}

#[test]
fn generated_svg_str_uses_high_density_surface_with_original_logical_size()
-> Result<(), Box<dyn std::error::Error>> {
    let surface = ViewerImageSurfaceFactory::from_svg_str("generated", svg(), 200)?;

    assert_eq!(80, surface.width);
    assert_eq!(40, surface.height);
    assert_eq!(40.0, surface.display_width);
    assert_eq!(20.0, surface.display_height);
    assert_eq!(200, surface.content_scale);
    assert_eq!(40, surface.logical_width());
    assert_eq!(20, surface.logical_height());
    Ok(())
}

#[test]
fn wide_svg_str_keeps_katana_retina_surface_before_display_scaling()
-> Result<(), Box<dyn std::error::Error>> {
    let surface = ViewerImageSurfaceFactory::from_svg_str("wide", wide_svg(), 50)?;

    assert_eq!(400, surface.width);
    assert_eq!(160, surface.height);
    assert_eq!(200, surface.content_scale);
    assert_eq!(200, surface.logical_width());
    assert_eq!(80, surface.logical_height());
    Ok(())
}

#[test]
fn diagram_svg_artifact_keeps_katana_display_size_with_retina_raster_density()
-> Result<(), Box<dyn std::error::Error>> {
    let artifact = image_artifact(
        ArtifactId("doc:diagram:Svg".to_string()),
        ArtifactFormat::Svg,
        svg().as_bytes().to_vec(),
    );

    let surface = ViewerImageSurfaceFactory::from_diagram_artifact(&artifact, 200)?;
    let effective_scale = surface.width as f32 / surface.display_width;

    assert_near(40.0, surface.display_width);
    assert_near(20.0, surface.display_height);
    assert!(surface.content_scale >= 200);
    assert!(
        effective_scale >= 2.0,
        "diagram SVG must remain retina at fixed KatanA viewer size: physical={} displayed={}",
        surface.width,
        surface.display_width
    );
    Ok(())
}

#[test]
fn diagram_svg_artifact_fits_wide_diagram_to_katana_preview_width_without_losing_retina_density()
-> Result<(), Box<dyn std::error::Error>> {
    let artifact = image_artifact(
        ArtifactId("doc:diagram:wide-svg".to_string()),
        ArtifactFormat::Svg,
        wide_svg().as_bytes().to_vec(),
    );

    let surface = ViewerImageSurfaceFactory::from_diagram_artifact(&artifact, 50)?;

    assert_eq!(400, surface.width);
    assert_eq!(160, surface.height);
    assert_near(50.0, surface.display_width);
    assert_near(20.0, surface.display_height);
    assert_eq!(200, surface.content_scale);
    assert_eq!(50, surface.logical_width());
    assert_eq!(20, surface.logical_height());
    Ok(())
}

#[test]
fn diagram_svg_artifact_keeps_fixed_display_scale_when_scaled_width_fits_preview()
-> Result<(), Box<dyn std::error::Error>> {
    let artifact = image_artifact(
        ArtifactId("doc:diagram:slightly-wide-svg".to_string()),
        ArtifactFormat::Svg,
        wide_svg().as_bytes().to_vec(),
    );

    let surface = ViewerImageSurfaceFactory::from_diagram_artifact(&artifact, 190)?;

    assert_eq!(400, surface.width);
    assert_eq!(160, surface.height);
    assert_near(190.0, surface.display_width);
    assert_near(76.0, surface.display_height);
    Ok(())
}

#[test]
fn fullscreen_diagram_svg_artifact_keeps_katana_logical_size_with_retina_surface()
-> Result<(), Box<dyn std::error::Error>> {
    let artifact = image_artifact(
        ArtifactId("doc:diagram:fullscreen-svg".to_string()),
        ArtifactFormat::Svg,
        wide_svg().as_bytes().to_vec(),
    );

    let normal = ViewerImageSurfaceFactory::from_diagram_artifact(&artifact, 190)?;
    let fullscreen = ViewerImageSurfaceFactory::from_fullscreen_diagram_artifact(&artifact, 190)?;

    assert_near(190.0, normal.display_width);
    assert_near(200.0, fullscreen.display_width);
    assert_near(80.0, fullscreen.display_height);
    assert!(
        fullscreen.content_scale >= 200,
        "fullscreen SVG must keep retina-ready physical pixels without rewriting the KatanA logical display size"
    );
    Ok(())
}

#[test]
fn diagram_svg_artifact_caps_large_viewport_display_width_to_katana_reference_pane_width()
-> Result<(), Box<dyn std::error::Error>> {
    let artifact = image_artifact(
        ArtifactId("doc:diagram:large-svg".to_string()),
        ArtifactFormat::Svg,
        large_svg().as_bytes().to_vec(),
    );

    let surface = ViewerImageSurfaceFactory::from_diagram_artifact(&artifact, 3000)?;

    assert_near(
        super::VIEWER_DIAGRAM_DISPLAY_MAX_WIDTH as f32,
        surface.display_width,
    );
    assert_near(948.0, surface.display_height);
    Ok(())
}

#[test]
fn diagram_svg_artifact_aligns_physical_width_to_retina_preview_target()
-> Result<(), Box<dyn std::error::Error>> {
    let artifact = image_artifact(
        ArtifactId("doc:diagram:odd-width".to_string()),
        ArtifactFormat::Svg,
        odd_width_svg().as_bytes().to_vec(),
    );

    let surface = ViewerImageSurfaceFactory::from_diagram_artifact(&artifact, 200)?;
    let retina_target_width = (surface.display_width.round() as u32).saturating_mul(2);

    assert!(
        surface.width >= retina_target_width,
        "diagram SVG should not be resampled again by the 2x Storybook canvas"
    );
    Ok(())
}

#[test]
fn diagram_svg_artifact_preserves_transparent_base_and_antialias_alpha()
-> Result<(), Box<dyn std::error::Error>> {
    let artifact = image_artifact(
        ArtifactId("doc:diagram:transparent-edge".to_string()),
        ArtifactFormat::Svg,
        antialias_edge_svg().as_bytes().to_vec(),
    );

    let surface = ViewerImageSurfaceFactory::from_diagram_artifact(&artifact, 200)?;
    let pixels = surface.rgba.as_chunks::<4>().0;
    let alpha_values: Vec<u8> = pixels.iter().map(|pixel| pixel[3]).collect();

    assert!(
        alpha_values.contains(&0),
        "diagram SVG must keep KatanA-style transparent base"
    );
    assert!(
        alpha_values.contains(&u8::MAX),
        "diagram SVG must keep opaque stroke/body pixels"
    );
    assert!(
        alpha_values
            .iter()
            .any(|alpha| (1..u8::MAX).contains(alpha)),
        "diagram SVG antialias edge alpha must survive raster/cache path"
    );
    Ok(())
}

#[test]
fn diagram_svg_artifact_with_background_matches_katana_texture_composite()
-> Result<(), Box<dyn std::error::Error>> {
    let artifact = image_artifact(
        ArtifactId("doc:diagram:transparent-base".to_string()),
        ArtifactFormat::Svg,
        transparent_base_svg().as_bytes().to_vec(),
    );
    let background = [30, 30, 30, 255];

    let surface = ViewerImageSurfaceFactory::from_diagram_artifact_with_background(
        &artifact, 200, background,
    )?;

    assert!(
        surface
            .fingerprint
            .contains(":background=1e1e1eff:renderer=")
    );
    assert_eq!(
        background,
        surface.rgba[0..4],
        "transparent SVG pixels should be composited over viewer background before texture resize"
    );
    assert!(
        surface
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| pixel[3] == u8::MAX),
        "KatanA texture path composites diagram rgba to opaque preview background"
    );
    Ok(())
}

#[test]
fn export_surface_diagram_artifact_uses_export_raster_extent()
-> Result<(), Box<dyn std::error::Error>> {
    let artifact = image_artifact(
        ArtifactId("doc:diagram:wide-svg".to_string()),
        ArtifactFormat::Svg,
        wide_svg().as_bytes().to_vec(),
    );

    let surface = ViewerImageSurfaceFactory::from_export_surface_diagram_artifact(&artifact, 50)?;

    assert_eq!(50, surface.width);
    assert_eq!(20, surface.height);
    assert_eq!(50.0, surface.display_width);
    assert_eq!(20.0, surface.display_height);
    assert_eq!(100, surface.content_scale);
    assert_eq!(50, surface.logical_width());
    assert_eq!(20, surface.logical_height());
    Ok(())
}

#[test]
fn large_svg_surface_matches_katana_texture_side_limit() -> Result<(), Box<dyn std::error::Error>> {
    let surface = ViewerImageSurfaceFactory::from_svg_str("large", large_svg(), 1600)?;

    assert_eq!(2048, surface.width);
    assert_eq!(1536, surface.height);
    assert_eq!(1600.0, surface.display_width);
    assert_eq!(1200.0, surface.display_height);
    assert_eq!(128, surface.content_scale);
    assert_eq!(1600, surface.logical_width());
    assert_eq!(1200, surface.logical_height());
    Ok(())
}

#[test]
fn oversized_svg_surface_caps_at_katana_raster_edge() -> Result<(), Box<dyn std::error::Error>> {
    let surface = ViewerImageSurfaceFactory::from_svg_str("oversized", oversized_svg(), 5000)?;

    assert_eq!(2048, surface.width);
    assert_eq!(1024, surface.height);
    assert_eq!(5000.0, surface.display_width);
    assert_eq!(2500.0, surface.display_height);
    assert_eq!(41, surface.content_scale);
    assert_eq!(5000, surface.logical_width());
    assert_eq!(2500, surface.logical_height());
    Ok(())
}

#[test]
fn math_svg_artifact_uses_export_body_root_font_size() -> Result<(), Box<dyn std::error::Error>> {
    let artifact = image_artifact(
        ArtifactId("doc:math:Svg".to_string()),
        ArtifactFormat::Svg,
        math_svg().as_bytes().to_vec(),
    );

    let default_surface = ViewerImageSurfaceFactory::from_artifact(&artifact, 1000)?;
    let math_surface = ViewerImageSurfaceFactory::from_math_artifact(&artifact)?;

    assert!(math_surface.width > default_surface.width);
    assert!(math_surface.height > default_surface.height);
    Ok(())
}

#[test]
fn png_artifact_decodes_to_ui_independent_rgba_surface() -> Result<(), Box<dyn std::error::Error>> {
    let artifact = image_artifact(
        ArtifactId("doc:image:Png".to_string()),
        ArtifactFormat::Png,
        encode_png()?,
    );

    let surface = ViewerImageSurfaceFactory::from_artifact(&artifact, 200)?;

    assert!(surface.fingerprint.starts_with("doc:image:Png:bytes="));
    assert!(surface.fingerprint.contains(":scale=100:"));
    assert_eq!(1, surface.width);
    assert_eq!(1, surface.height);
    assert_eq!(100, surface.content_scale);
    assert_eq!(vec![255, 0, 0, 255], surface.rgba);
    Ok(())
}

#[test]
fn direct_raster_fixture_formats_decode_to_ui_independent_rgba_surface()
-> Result<(), Box<dyn std::error::Error>> {
    for (extension, format) in [
        ("bmp", ArtifactFormat::Bmp),
        ("gif", ArtifactFormat::Gif),
        ("jpeg", ArtifactFormat::Jpeg),
        ("jpg", ArtifactFormat::Jpeg),
        ("png", ArtifactFormat::Png),
        ("webp", ArtifactFormat::Webp),
    ] {
        let artifact = image_artifact(
            ArtifactId(format!("doc:image:{extension}")),
            format,
            std::fs::read(direct_fixture(extension))?,
        );

        let surface = ViewerImageSurfaceFactory::from_artifact(&artifact, 512)?;

        assert_eq!(512, surface.width, "{extension}");
        assert_eq!(512, surface.height, "{extension}");
        assert_eq!(512 * 512 * 4, surface.rgba.len(), "{extension}");
    }
    Ok(())
}

#[test]
fn unsupported_image_format_returns_error() {
    let artifact = image_artifact(
        ArtifactId("doc:image:Html".to_string()),
        ArtifactFormat::Html,
        Vec::new(),
    );

    let result = ViewerImageSurfaceFactory::from_artifact(&artifact, 200);

    assert!(matches!(
        result,
        Err(ViewerImageSurfaceError::UnsupportedFormat(
            ArtifactFormat::Html
        ))
    ));
}

#[test]
fn invalid_png_returns_decode_error() {
    let artifact = image_artifact(
        ArtifactId("doc:image:Png".to_string()),
        ArtifactFormat::Png,
        b"not png".to_vec(),
    );

    let result = ViewerImageSurfaceFactory::from_artifact(&artifact, 200);

    assert!(matches!(
        result,
        Err(ViewerImageSurfaceError::InvalidRaster(_))
    ));
}

#[test]
fn logical_dimensions_fallback_when_display_size_is_not_finite() {
    let surface = super::ViewerImageSurface {
        fingerprint: "logical".to_string(),
        width: 5,
        height: 5,
        display_width: f32::NAN,
        display_height: f32::NEG_INFINITY,
        content_scale: 10,
        rgba: vec![0; 100],
    };

    assert_eq!(50, surface.logical_width());
    assert_eq!(50, surface.logical_height());
}

#[test]
fn non_svg_artifact_from_diagram_reports_unsupported_format() {
    let artifact = image_artifact(
        ArtifactId("doc:diagram:svg".to_string()),
        ArtifactFormat::RenderTree,
        b"hello".to_vec(),
    );

    assert_eq!(
        Err(ViewerImageSurfaceError::UnsupportedFormat(
            ArtifactFormat::RenderTree
        )),
        ViewerImageSurfaceFactory::from_diagram_artifact(&artifact, 200)
    );
}

#[test]
fn non_svg_fullscreen_diagram_artifact_uses_from_artifact_path() {
    let artifact = image_artifact(
        ArtifactId("doc:diagram:svg".to_string()),
        ArtifactFormat::RenderTree,
        b"hello".to_vec(),
    );

    assert!(matches!(
        ViewerImageSurfaceFactory::from_fullscreen_diagram_artifact(&artifact, 200),
        Err(ViewerImageSurfaceError::UnsupportedFormat(
            ArtifactFormat::RenderTree
        ))
    ));
}

#[test]
fn non_svg_export_surface_diagram_artifact_reports_unsupported_format() {
    let artifact = image_artifact(
        ArtifactId("doc:diagram:svg".to_string()),
        ArtifactFormat::RenderTree,
        b"hello".to_vec(),
    );

    assert!(matches!(
        ViewerImageSurfaceFactory::from_export_surface_diagram_artifact(&artifact, 200),
        Err(ViewerImageSurfaceError::UnsupportedFormat(
            ArtifactFormat::RenderTree
        ))
    ));
}

#[test]
fn non_svg_math_artifact_reports_unsupported_format() {
    let artifact = image_artifact(
        ArtifactId("doc:math:RenderTree".to_string()),
        ArtifactFormat::RenderTree,
        b"math".to_vec(),
    );

    assert_eq!(
        Err(ViewerImageSurfaceError::UnsupportedFormat(
            ArtifactFormat::RenderTree
        )),
        ViewerImageSurfaceFactory::from_math_artifact(&artifact)
    );
}

#[test]
fn fullscreen_diagram_background_is_part_of_surface_identity()
-> Result<(), Box<dyn std::error::Error>> {
    let artifact = image_artifact(
        ArtifactId("doc:diagram:fullscreen-background".to_string()),
        ArtifactFormat::Svg,
        svg().as_bytes().to_vec(),
    );

    let surface = ViewerImageSurfaceFactory::from_fullscreen_diagram_artifact_with_background(
        &artifact,
        200,
        [1, 2, 3, 255],
    )?;

    assert!(surface.fingerprint.contains(":background=010203ff:"));
    Ok(())
}

#[test]
fn repeated_export_svg_and_raster_requests_reuse_cached_surfaces()
-> Result<(), Box<dyn std::error::Error>> {
    let svg_artifact = image_artifact(
        ArtifactId("doc:diagram:export-cache".to_string()),
        ArtifactFormat::Svg,
        svg().as_bytes().to_vec(),
    );
    let first_export =
        ViewerImageSurfaceFactory::from_export_surface_diagram_artifact(&svg_artifact, 200)?;
    let second_export =
        ViewerImageSurfaceFactory::from_export_surface_diagram_artifact(&svg_artifact, 200)?;
    assert_eq!(first_export, second_export);

    let raster_artifact = image_artifact(
        ArtifactId("doc:image:raster-cache".to_string()),
        ArtifactFormat::Png,
        encode_png()?,
    );
    let first_raster = ViewerImageSurfaceFactory::from_artifact(&raster_artifact, 200)?;
    let second_raster = ViewerImageSurfaceFactory::from_artifact(&raster_artifact, 400)?;
    assert_eq!(first_raster, second_raster);
    Ok(())
}

fn svg() -> &'static str {
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20" fill="red"/></svg>"#
}

fn wide_svg() -> &'static str {
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="80"><rect width="200" height="80" fill="red"/></svg>"#
}

fn odd_width_svg() -> &'static str {
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="41" height="20"><path d="M1 19 L40 1" stroke="white" stroke-width="1"/></svg>"#
}

fn antialias_edge_svg() -> &'static str {
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="41" height="20"><path d="M1 19 L40 1" stroke="white" stroke-width="1.25" stroke-linecap="round"/></svg>"#
}

fn transparent_base_svg() -> &'static str {
    r##"<svg xmlns="http://www.w3.org/2000/svg" width="2" height="1"><rect x="1" width="1" height="1" fill="#d4d4d4"/></svg>"##
}

fn large_svg() -> &'static str {
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="1200"><rect width="1600" height="1200" fill="red"/></svg>"#
}

fn oversized_svg() -> &'static str {
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="5000" height="2500"><rect width="5000" height="2500" fill="red"/></svg>"#
}

fn math_svg() -> &'static str {
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="8.704ex" height="1.912ex"><text x="0" y="10">E = mc^2</text></svg>"#
}

fn image_artifact(id: ArtifactId, format: ArtifactFormat, bytes: Vec<u8>) -> crate::Artifact {
    ArtifactFactory::image_asset_with_id(
        id,
        format,
        DocumentId("doc".to_string()),
        SourceRevision("rev".to_string()),
        ArtifactBytes { bytes },
        "test",
        ArtifactDiagnostics {
            entries: Vec::new(),
        },
    )
}

fn assert_near(expected: f32, actual: f32) {
    assert!(
        (expected - actual).abs() <= 0.01,
        "expected {expected}, actual {actual}"
    );
}

fn encode_png() -> Result<Vec<u8>, image::ImageError> {
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes).write_image(
        &RED_PIXEL_RGBA,
        1,
        1,
        image::ColorType::Rgba8.into(),
    )?;
    Ok(bytes)
}

fn direct_fixture(extension: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/direct")
        .join(format!("kdv-icon.{extension}"))
}
