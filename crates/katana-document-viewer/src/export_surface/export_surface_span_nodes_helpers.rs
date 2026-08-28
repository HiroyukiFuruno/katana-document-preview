use crate::emoji_text::EmojiTextSegments;
use crate::export_surface_line::BODY_FONT_SIZE;
use crate::export_surface_math::SurfaceMathText;
use crate::export_surface_svg::SurfaceSvgRasterizer;
use crate::export_surface_text::SurfaceTextParser;
use crate::render_runtime::{KrrMathMode, KrrRenderRuntimeAdapter};
use crate::theme::KdvThemeSnapshot;
use katana_markdown_model::KmmNode;

use super::{INLINE_MATH_MAX_WIDTH, SurfaceTextSpan, SurfaceTextStyle};

pub(super) fn append_unknown_node(
    spans: &mut Vec<SurfaceTextSpan>,
    node: &KmmNode,
    style: SurfaceTextStyle,
    theme: &KdvThemeSnapshot,
) {
    if !node.children.is_empty() {
        for child in &node.children {
            super::SurfaceInlineSpans::append_node_without_fallback(spans, child, style, theme);
        }
        return;
    }
    push(
        spans,
        SurfaceTextParser::decode_basic_entities(&node.source.raw.text),
        style,
    )
}

pub(super) fn append_inline_html(
    spans: &mut Vec<SurfaceTextSpan>,
    html: &str,
    style: SurfaceTextStyle,
) {
    push(
        spans,
        SurfaceTextParser::html_fragment_text(html),
        html_style(html, style),
    )
}

pub(super) fn append_style_node(
    spans: &mut Vec<SurfaceTextSpan>,
    node: &KmmNode,
    text: &str,
    style: SurfaceTextStyle,
    theme: &KdvThemeSnapshot,
) {
    if node.children.is_empty() {
        push(spans, text, style);
        return;
    }
    for child in &node.children {
        super::SurfaceInlineSpans::append_node_without_fallback(spans, child, style, theme);
    }
}

pub(super) fn append_link(
    spans: &mut Vec<SurfaceTextSpan>,
    text: impl Into<String>,
    target: impl Into<String>,
    style: SurfaceTextStyle,
) {
    let text = text.into();
    if !text.is_empty() {
        spans.push(SurfaceTextSpan::linked(text, target, style.link()));
    }
}

pub(super) fn append_inline_math(
    spans: &mut Vec<SurfaceTextSpan>,
    expression: &str,
    style: SurfaceTextStyle,
    theme: &KdvThemeSnapshot,
) {
    let output = KrrRenderRuntimeAdapter::render_math_tex_with_theme(
        expression,
        KrrMathMode::Inline,
        Some(theme.krr_math_theme()),
    );
    if let Some(image) = output.svg_payload().and_then(|svg| {
        SurfaceSvgRasterizer::rasterize_with_root_font_size(
            svg,
            INLINE_MATH_MAX_WIDTH,
            Some(BODY_FONT_SIZE),
        )
    }) {
        spans.push(SurfaceTextSpan::inline_image(
            "math-svg:inline",
            image,
            style,
        ));
        return;
    }
    push(spans, SurfaceMathText::render(expression), style);
}

pub(super) fn push_plain(spans: &mut Vec<SurfaceTextSpan>, text: &str, style: SurfaceTextStyle) {
    push(spans, SurfaceTextParser::decode_basic_entities(text), style)
}

pub(super) fn push(
    spans: &mut Vec<SurfaceTextSpan>,
    text: impl Into<String>,
    style: SurfaceTextStyle,
) {
    let text = text.into();
    if !text.is_empty() {
        for segment in EmojiTextSegments::split(&text) {
            let segment_style = if segment.emoji { style.emoji() } else { style };
            push_styled_segment(spans, segment.text.as_str(), segment_style);
        }
    }
}

fn push_styled_segment(spans: &mut Vec<SurfaceTextSpan>, text: &str, style: SurfaceTextStyle) {
    if let Some(previous) = spans.last_mut() {
        let same_text_run = previous.style == style
            && previous.link_target.is_none()
            && previous.inline_image.is_none();
        if same_text_run {
            previous.text.push_str(text);
            return;
        }
    }
    spans.push(SurfaceTextSpan::styled(text, style));
}

fn html_style(html: &str, style: SurfaceTextStyle) -> SurfaceTextStyle {
    let lower = html.to_ascii_lowercase();
    if lower.contains("<code") {
        return style.inline_code();
    }
    if lower.contains("<strong") || lower.contains("<b") {
        return style.bold();
    }
    if lower.contains("<em") || lower.contains("<i") {
        return style.italic();
    }
    if lower.contains("<u") {
        return style.underline();
    }
    if lower.contains("<mark") {
        return style.highlight();
    }
    if lower.contains("<s") || lower.contains("<del") {
        return style.strikethrough();
    }
    style
}

#[cfg(test)]
#[path = "export_surface_span_nodes_helpers_tests.rs"]
mod tests;
