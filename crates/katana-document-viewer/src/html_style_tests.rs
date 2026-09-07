use super::{HtmlStyle, HtmlStyleProperties};

const BLOCK_ELEMENT_NAMES: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "caption",
    "colgroup",
    "dd",
    "details",
    "dialog",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hgroup",
    "hr",
    "li",
    "main",
    "menu",
    "nav",
    "ol",
    "p",
    "pre",
    "search",
    "section",
    "summary",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "tr",
    "ul",
];

#[test]
fn recognizes_every_supported_html_block_element() {
    assert!(
        BLOCK_ELEMENT_NAMES
            .iter()
            .all(|name| HtmlStyle::is_block_element(name))
    );
    assert!(!HtmlStyle::is_block_element("span"));
}

#[test]
fn parses_tag_styles_and_all_supported_declaration_effects() {
    let properties = HtmlStyleProperties::from_fragment(
        r#"<strong><em><u><mark><del><code style="color: #abc; font-weight: 600; font-style: italic; text-decoration: underline line-through; background-color: yellow; font-family: monospace">styled</code>"#,
    );

    assert!(properties.bold);
    assert!(properties.italic);
    assert!(properties.underline);
    assert!(properties.strikethrough);
    assert!(properties.highlight);
    assert!(properties.inline_code);
    assert_eq!(Some([170, 187, 204, 255]), properties.color_rgba);
}

#[test]
fn accepts_named_and_rgb_colors_and_rejects_invalid_colors() {
    assert_named_colors();
    assert_eq!(
        Some([1, 2, 3, 255]),
        HtmlStyleProperties::from_fragment(r#"<span style="color: rgb(1, 2, 3)">x</span>"#)
            .color_rgba
    );
    assert_eq!(
        None,
        HtmlStyleProperties::from_fragment(r#"<span style="color: rgb(1, 2)">x</span>"#).color_rgba
    );
    assert_eq!(
        None,
        HtmlStyleProperties::from_fragment(
            r#"<span style="color: #12; font-weight: 500">x</span>"#
        )
        .color_rgba
    );
    assert!(!HtmlStyleProperties::from_fragment(r#"<span style="font-weight: 500">x</span>"#).bold);
    assert!(
        !HtmlStyleProperties::from_fragment(r#"<span style="font-weight: unknown">x</span>"#).bold
    );
}

fn assert_named_colors() {
    for (name, expected) in [
        ("black", [0, 0, 0, 255]),
        ("white", [255, 255, 255, 255]),
        ("red", [255, 0, 0, 255]),
        ("green", [0, 128, 0, 255]),
        ("blue", [0, 0, 255, 255]),
        ("gray", [128, 128, 128, 255]),
        ("grey", [128, 128, 128, 255]),
    ] {
        assert_eq!(
            Some(expected),
            HtmlStyleProperties::from_fragment(&format!(r#"<span style="color: {name}">x</span>"#))
                .color_rgba
        );
    }
}

#[test]
fn handles_unquoted_and_malformed_style_attributes() {
    assert_eq!(
        Some([255, 0, 0, 255]),
        HtmlStyleProperties::from_fragment("<div style=color:red>").color_rgba
    );
    assert_eq!(
        HtmlStyleProperties::default(),
        HtmlStyleProperties::from_fragment("<div style>")
    );
    assert_eq!(
        HtmlStyleProperties::default(),
        HtmlStyleProperties::from_fragment(r#"<div style="color: red>"#)
    );
    assert!(
        !HtmlStyleProperties::from_fragment(
            r#"<div style="not-a-declaration; background: transparent">x</div>"#
        )
        .highlight
    );
}

#[test]
fn ignores_style_substrings_in_metadata_attributes_and_values() {
    assert_eq!(
        None,
        HtmlStyleProperties::from_fragment(r#"<span data-style="color: red">plain</span>"#)
            .color_rgba
    );
    assert_eq!(
        None,
        HtmlStyleProperties::from_fragment(r#"<span title="style=color:red">plain</span>"#)
            .color_rgba
    );
}

#[test]
fn tag_name_prefixes_do_not_create_unrelated_text_decoration() {
    let properties = HtmlStyleProperties::from_fragment(
        "<strong>bold</strong><span>plain</span><section>section</section>",
    );
    assert!(properties.bold);
    assert!(!properties.strikethrough);
    assert!(!HtmlStyleProperties::from_fragment("<span>plain</span>").strikethrough);
    assert!(!HtmlStyleProperties::from_fragment("<section>plain</section>").strikethrough);
    assert!(!HtmlStyleProperties::from_fragment(r#"<span title="<s>">plain</span>"#).strikethrough);
    assert!(HtmlStyleProperties::from_fragment("<s>struck</s>").strikethrough);
}

#[test]
fn css_boolean_declarations_use_the_last_supported_value() {
    let properties = HtmlStyleProperties::from_fragment(
        r#"<span style="font-weight: bold; font-weight: normal; font-style: italic; font-style: normal; text-decoration: underline; text-decoration: none">x</span>"#,
    );

    assert_eq!(Some(false), properties.bold_override);
    assert_eq!(Some(false), properties.italic_override);
    assert_eq!(Some(false), properties.underline_override);
    assert_eq!(Some(false), properties.strikethrough_override);

    let properties = HtmlStyleProperties::from_fragment(
        r#"<span style="font-weight: normal; font-weight: bold; font-style: normal; font-style: italic; text-decoration: none; text-decoration: underline line-through">x</span>"#,
    );
    assert_eq!(Some(true), properties.bold_override);
    assert_eq!(Some(true), properties.italic_override);
    assert_eq!(Some(true), properties.underline_override);
    assert_eq!(Some(true), properties.strikethrough_override);
}
