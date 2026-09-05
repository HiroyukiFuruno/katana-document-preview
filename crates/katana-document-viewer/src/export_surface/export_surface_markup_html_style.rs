use crate::export_surface_span::SurfaceTextStyle;
use crate::html_style::HtmlStyleProperties;
use image::Rgba;

pub(crate) struct SurfaceHtmlStyle;

impl SurfaceHtmlStyle {
    pub(crate) fn apply(fragment: &str, style: SurfaceTextStyle) -> SurfaceTextStyle {
        let properties = HtmlStyleProperties::from_fragment(fragment);
        let mut style = style;
        if properties.inline_code {
            style = style.inline_code();
        }
        if properties.bold {
            style = style.bold();
        }
        if properties.italic {
            style = style.italic();
        }
        if properties.underline {
            style = style.underline();
        }
        if properties.highlight {
            style = style.highlight();
        }
        if properties.strikethrough {
            style = style.strikethrough();
        }
        if let Some([red, green, blue, alpha]) = properties.color_rgba {
            style = style.with_color(Rgba([red, green, blue, alpha]));
        }
        apply_boolean_overrides(style, properties)
    }
}

fn apply_boolean_overrides(
    mut style: SurfaceTextStyle,
    properties: HtmlStyleProperties,
) -> SurfaceTextStyle {
    if let Some(bold) = properties.bold_override {
        style.bold = bold;
    }
    if let Some(italic) = properties.italic_override {
        style.italic = italic;
    }
    if let Some(underline) = properties.underline_override {
        style.underline = underline;
    }
    if let Some(strikethrough) = properties.strikethrough_override {
        style.strikethrough = strikethrough;
    }
    style
}

#[cfg(test)]
#[path = "export_surface_markup_html_style_tests.rs"]
mod tests;
