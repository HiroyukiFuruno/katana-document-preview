use super::{DirectHtmlCss, merge_style_attribute};
use crate::preview_runtime::direct_html_css_attrs::DirectHtmlCssAttrs;

#[test]
fn applies_body_and_class_rules_to_html_line() {
    let css = DirectHtmlCss::parse(
        r#"
        body { color: red; }
        .note { font-weight: bold; text-align: center; }
        "#,
    );

    let line = css.apply_to_line(r#"<p class="note">Visible</p>"#);

    assert!(line.contains(r#"style="color: red; font-weight: bold; text-align: center""#));
}

#[test]
fn applies_each_class_rule_to_every_element_in_a_minified_line() {
    let css = DirectHtmlCss::parse(".a { color: red; } .b { color: blue; }");

    let line = css.apply_to_line(r#"<p class="a">one</p><p class="b">two</p>"#);

    assert_eq!(
        r#"<p class="a" style="color: red">one</p><p class="b" style="color: blue">two</p>"#,
        line
    );
}

#[test]
fn keeps_inline_style_after_stylesheet_declarations() {
    let css = DirectHtmlCss::parse("p { color: red; font-weight: bold; }");

    let line = css.apply_to_line(r#"<p style="color: blue">Visible</p>"#);

    assert!(line.contains(r#"style="color: red; font-weight: bold; color: blue""#));
}

#[test]
fn escapes_quoted_css_values_in_generated_style_attributes() {
    let css = DirectHtmlCss::parse(r#"p { font-family: "Fira Code"; }"#);

    assert_eq!(
        r#"<p style="font-family: &quot;Fira Code&quot;">Visible</p>"#,
        css.apply_to_line("<p>Visible</p>")
    );
    assert_eq!(
        r#"<p style="font-family: &quot;Fira Code&quot;; font-family: &quot;Existing&quot;">Visible</p>"#,
        css.apply_to_line(r#"<p style='font-family: "Existing"'>Visible</p>"#)
    );
}

#[test]
fn applies_id_and_supported_properties_and_skips_complex_selectors() {
    let css = DirectHtmlCss::parse(concat!(
        "#hero { color: green; font-weight: 700; font-style: italic; ",
        "text-align: right; text-decoration: underline; background: yellow; ",
        "background-color: white; font-family: monospace; unsupported: ignored; }\n",
        ".hero, .other { color: red; }\n",
        "p span { color: red; }"
    ));

    let line = css.apply_to_line(r#"<p id="hero">Visible</p>"#);

    assert!(line.contains(
        r#"style="color: green; font-weight: 700; font-style: italic; text-align: right; text-decoration: underline; background: yellow; background-color: white; font-family: monospace"#
    ));
    assert_eq!("plain", css.apply_to_line("plain"));
    assert_eq!("<!doctype html>", css.apply_to_line("<!doctype html>"));
    assert_eq!(
        r#"<p class="other">No match</p>"#,
        css.apply_to_line(r#"<p class="other">No match</p>"#)
    );
}

#[test]
fn applies_matching_rules_in_css_specificity_order() {
    let css = DirectHtmlCss::parse("#hero { color: red; } p { color: blue; }");

    let line = css.apply_to_line(r#"<p id="hero">Visible</p>"#);

    assert!(line.contains(r#"style="color: blue; color: red""#));
}

#[test]
fn matches_class_and_id_values_case_sensitively_but_tag_names_case_insensitively() {
    let css = DirectHtmlCss::parse(
        ".Note { color: red; } #Hero { font-weight: bold; } DIV { text-align: center; }",
    );

    let mismatch = css.apply_to_line(r#"<div class="note" id="hero">Visible</div>"#);
    assert!(mismatch.contains("text-align: center"));
    assert!(!mismatch.contains("color: red"));
    assert!(!mismatch.contains("font-weight: bold"));

    let exact = css.apply_to_line(r#"<div class="Note" id="Hero">Visible</div>"#);
    assert!(exact.contains("text-align: center"));
    assert!(exact.contains("color: red"));
    assert!(exact.contains("font-weight: bold"));
}

#[test]
fn ignores_data_attributes_when_matching_and_merging_css_rules() {
    let css = DirectHtmlCss::parse(
        ".note { color: red; } #hero { font-weight: bold; } p { color: blue; }",
    );

    let line = css.apply_to_line(
        r#"<p data-class="note" data-id="hero" data-style="color: red">Visible</p>"#,
    );

    assert!(line.contains(r#"data-style="color: red" style="color: blue""#));
    assert!(!line.contains("font-weight: bold"));
}

#[test]
fn handles_empty_existing_style_self_closing_tags_and_malformed_tags() {
    let css = DirectHtmlCss::parse("p { color: red; }");

    assert_eq!(
        r#"<p style="color: red"></p>"#,
        css.apply_to_line(r#"<p style=""></p>"#)
    );
    assert_eq!(r#"<p style="color: red"/>"#, css.apply_to_line("<p/>"));
    assert_eq!(
        "text",
        merge_style_attribute("text", &["color: red".to_string()])
    );
    assert_eq!(
        r#"<p style="color: red">text</p>"#,
        css.apply_to_line("<p>text</p>")
    );
}

#[test]
fn exercises_css_attribute_helpers_and_unquoted_values() {
    assert_eq!(
        vec!["hero".to_string(), "wide".to_string()],
        DirectHtmlCssAttrs::class_list(r#"<p class="hero wide">"#)
    );
    assert!(DirectHtmlCssAttrs::class_list("<p>").is_empty());
    assert_eq!(
        Some("hero".to_string()),
        DirectHtmlCssAttrs::attribute_value("<p id=hero>", "id")
    );
    assert_eq!(
        Some((3..15, "hero".to_string())),
        DirectHtmlCssAttrs::style_attribute_range(r#"<p style='hero'>"#)
    );
    assert_eq!(None, DirectHtmlCssAttrs::html_tag_end("<p title=\">"));
}
