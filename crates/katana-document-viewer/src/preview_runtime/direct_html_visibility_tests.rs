use super::{
    DirectHtmlVisibility, SkipState, next_tag_range, remove_block, skip_until_close, tag_end,
};

#[test]
fn visible_lines_remove_structural_wrappers_and_hidden_complete_blocks() {
    let lines = DirectHtmlVisibility::visible_lines(
        "<!doctype html>\n<html>\n<body>\n<main>\n<p>Visible</p>\n</main>\n</body>\n</html>\n<head><title>Hidden</title></head>\n<style>.hidden { color: red; }</style>\n<script>bad()</script>",
    );

    assert_eq!(
        vec!["", "", "", "", "<p>Visible</p>", "", "", "", "", ""],
        lines
    );
}

#[test]
fn incomplete_hidden_blocks_keep_the_state_until_end_of_input() {
    assert!(DirectHtmlVisibility::visible_lines("<style>\ncolor: red").is_empty());
    assert!(DirectHtmlVisibility::visible_lines("<script>\nwindow.bad = true").is_empty());
    assert!(DirectHtmlVisibility::visible_lines("<head>\n<meta>").is_empty());
}

#[test]
fn skip_states_and_tag_scanning_cover_all_variants() {
    assert_eq!(SkipState::None.tag_name(), "");
    assert_eq!(SkipState::Head.tag_name(), "head");
    assert_eq!(SkipState::Style.tag_name(), "style");
    assert_eq!(SkipState::Script.tag_name(), "script");
    assert!(matches!(
        skip_until_close("</head>", "head"),
        SkipState::None
    ));
    assert!(matches!(skip_until_close("body", "head"), SkipState::Head));
    assert!(matches!(
        skip_until_close("body", "style"),
        SkipState::Style
    ));
    assert!(matches!(
        skip_until_close("body", "script"),
        SkipState::Script
    ));
    assert!(matches!(
        skip_until_close("body", "unknown"),
        SkipState::None
    ));

    assert_tag_ranges();
}

fn assert_tag_ranges() {
    assert_eq!(Some((0, 3)), next_tag_range("<p>text", "p"));
    assert_eq!(Some((0, 3)), next_tag_range("<p>text</p>", "p"));
    assert_eq!(Some((0, 4)), next_tag_range("</p>x", "p"));
    assert_eq!(Some((1, 5)), next_tag_range("x</p>", "p"));
    assert_eq!(None, next_tag_range("text", "p"));
    assert_eq!(Some(12), tag_end(r#"<p title=">">"#));
    assert_eq!(None, tag_end("<p title=\"unterminated>"));
}

#[test]
fn malformed_hidden_blocks_fail_closed() {
    assert_eq!("<style", remove_block("<style", "style"));
    assert_eq!("<style>body", remove_block("<style>body", "style"));
}
