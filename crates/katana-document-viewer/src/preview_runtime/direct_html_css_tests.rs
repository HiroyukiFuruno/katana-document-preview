use super::DirectHtmlCss;

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
fn keeps_inline_style_after_stylesheet_declarations() {
    let css = DirectHtmlCss::parse("p { color: red; font-weight: bold; }");

    let line = css.apply_to_line(r#"<p style="color: blue">Visible</p>"#);

    assert!(line.contains(r#"style="color: red; font-weight: bold; color: blue""#));
}
