use super::DirectHtmlCssAttrs;

#[test]
fn parses_quoted_tags_and_ignores_gt_inside_quotes() {
    assert_eq!(
        Some(12),
        DirectHtmlCssAttrs::html_tag_end(r#"<p title=">">"#)
    );
    assert_eq!(
        Some("p".to_string()),
        DirectHtmlCssAttrs::tag_name("  <P class='paragraph'>")
    );
    assert_eq!(None, DirectHtmlCssAttrs::tag_name("</p>"));
    assert_eq!(None, DirectHtmlCssAttrs::tag_name("<!doctype html>"));
    assert_eq!(None, DirectHtmlCssAttrs::tag_name("<?xml version='1.0'>"));
    assert_eq!(None, DirectHtmlCssAttrs::tag_name("< >"));
    assert_eq!(
        None,
        DirectHtmlCssAttrs::html_tag_end("<p title=\"unterminated>")
    );
}

#[test]
fn parses_quoted_and_unquoted_attribute_values() {
    assert_eq!(
        Some("hero".to_string()),
        DirectHtmlCssAttrs::attribute_value("<p id=hero>", "id")
    );
    assert_eq!(
        Some("hero".to_string()),
        DirectHtmlCssAttrs::attribute_value(r#"<p id='hero'>"#, "id")
    );
    assert_eq!(
        Some("hero".to_string()),
        DirectHtmlCssAttrs::attribute_value(r#"<p id="hero">"#, "id")
    );
    assert_eq!(None, DirectHtmlCssAttrs::attribute_value("<p id>", "id"));
    assert_eq!(
        None,
        DirectHtmlCssAttrs::attribute_value(r#"<p id="unterminated>"#, "id")
    );
    assert_eq!(
        Some((3..15, "hero".to_string())),
        DirectHtmlCssAttrs::style_attribute_range(r#"<p style='hero'>"#)
    );
    assert_eq!(None, DirectHtmlCssAttrs::style_attribute_range("<p style>"));
}

#[test]
fn requires_complete_attribute_names_outside_quoted_metadata() {
    assert_eq!(
        None,
        DirectHtmlCssAttrs::attribute_value(r#"<p data-id="hero">"#, "id")
    );
    assert!(DirectHtmlCssAttrs::class_list(r#"<p data-class="note">"#).is_empty());
    assert_eq!(
        None,
        DirectHtmlCssAttrs::style_attribute_range(r#"<p data-style="color: red">"#)
    );
    assert_eq!(
        None,
        DirectHtmlCssAttrs::attribute_value(r#"<p title="id=hero class=note">"#, "id")
    );
    assert_eq!(
        Some((3..17, "hero".to_string())),
        DirectHtmlCssAttrs::style_attribute_range(r#"<p style = 'hero'>"#)
    );
    assert_eq!(
        None,
        DirectHtmlCssAttrs::attribute_value("<p data-id=hero", "id")
    );
}
