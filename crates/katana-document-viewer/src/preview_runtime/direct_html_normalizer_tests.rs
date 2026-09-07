use super::DirectHtmlNormalizer;

#[test]
fn direct_html_is_split_into_top_level_blocks() {
    let normalized = DirectHtmlNormalizer::normalize(&source());

    assert!(!normalized.contains("<main>"));
    assert!(normalized.contains("<h1 align=\"center\">Title</h1>\n\n"));
    assert!(normalized.contains("<details open><summary>Details</summary><p>Body</p></details>"));
}

#[test]
fn direct_html_skips_head_style_and_script_as_body_text() {
    let normalized = DirectHtmlNormalizer::normalize(&document_source());

    assert!(!normalized.contains("Hidden metadata"));
    assert!(!normalized.contains("body { color: red; }"));
    assert!(!normalized.contains("window.bad = true"));
    assert!(normalized.contains("Visible"));
    assert!(normalized.contains("color: red"));
    assert!(normalized.contains("font-weight: bold"));
}

#[test]
fn direct_html_normalizes_a_table_as_one_block() {
    let normalized = DirectHtmlNormalizer::normalize(
        "<table>\n<tr><th>Feature</th><td>Status</td></tr>\n</table>",
    );

    assert_eq!("| Feature | Status |\n| --- | --- |", normalized);
}

#[test]
fn direct_html_malformed_style_blocks_fail_closed() {
    assert_eq!("<style", DirectHtmlNormalizer::normalize("<style"));
    assert_eq!(
        "",
        DirectHtmlNormalizer::normalize("<style>body { color: red; }")
    );
}

fn source() -> String {
    [
        "<main>",
        "<h1 align=\"center\">Title</h1>",
        "<details open>",
        "<summary>Details</summary>",
        "<p>Body</p>",
        "</details>",
        "</main>",
    ]
    .join("\n")
}

fn document_source() -> String {
    [
        "<!doctype html>",
        "<html>",
        "<head>",
        "<title>Hidden metadata</title>",
        "<style>",
        "body { color: red; }",
        ".important { font-weight: bold; }",
        "</style>",
        "</head>",
        "<body>",
        r#"<p class="important">Visible</p>"#,
        "<script>",
        "window.bad = true;",
        "</script>",
        "</body>",
        "</html>",
    ]
    .join("\n")
}
