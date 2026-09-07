use super::{diagram_display_scale, fit_diagram_display_size};
use crate::viewer::ViewerImageSurface;

#[test]
fn diagram_display_scale_caps_intrinsic_and_wide_surfaces() {
    assert_eq!(1.0, diagram_display_scale(100.0, 200));
    assert_eq!(0.5, diagram_display_scale(400.0, 200));
}

#[test]
fn fit_diagram_display_size_keeps_invalid_dimensions_unchanged() {
    let surface = ViewerImageSurface {
        fingerprint: "invalid-display".to_string(),
        width: 1,
        height: 1,
        display_width: f32::NAN,
        display_height: 1.0,
        content_scale: 100,
        rgba: vec![0; 4],
    };

    let fitted = fit_diagram_display_size(surface.clone(), 200);
    assert_eq!(surface.fingerprint, fitted.fingerprint);
    assert!(fitted.display_width.is_nan());
    assert_eq!(surface.display_height, fitted.display_height);
}
