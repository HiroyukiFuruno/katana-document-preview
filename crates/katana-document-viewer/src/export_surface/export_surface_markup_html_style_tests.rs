use super::SurfaceHtmlStyle;
use crate::export_surface_span::SurfaceTextStyle;
use image::Rgba;

#[test]
fn applies_all_supported_html_properties_to_surface_style() {
    let style = SurfaceHtmlStyle::apply(
        r#"<code><strong><em><u><mark><del style="color: #abc; text-decoration: underline line-through; background-color: yellow; font-family: monospace">styled</del></mark></u></em></strong></code>"#,
        SurfaceTextStyle::default(),
    );

    assert!(style.bold);
    assert!(style.italic);
    assert!(style.underline);
    assert!(style.strikethrough);
    assert!(style.highlight);
    assert!(style.inline_code);
    assert_eq!(Some(Rgba([170, 187, 204, 255])), style.color);
}

#[test]
fn css_declarations_can_clear_inherited_surface_styles() {
    let style = SurfaceHtmlStyle::apply(
        r#"<strong><span style="font-weight: normal; text-decoration: none">plain</span></strong>"#,
        SurfaceTextStyle::default(),
    );

    assert!(!style.bold);
    assert!(!style.underline);
    assert!(!style.strikethrough);
}
