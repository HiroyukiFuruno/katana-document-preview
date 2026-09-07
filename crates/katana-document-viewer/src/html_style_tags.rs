pub(super) fn contains_opening_tag(fragment: &str, tag: &str) -> bool {
    let mut quote = None;
    for (index, character) in fragment.char_indices() {
        if let Some(delimiter) = quote {
            if character == delimiter {
                quote = None;
            }
            continue;
        }
        match character {
            '"' | '\'' => quote = Some(character),
            '<' => {
                let remaining = &fragment[index + 1..];
                if !remaining.starts_with('/')
                    && remaining.strip_prefix(tag).is_some_and(tag_boundary)
                {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

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

pub(super) fn is_block_element(name: &str) -> bool {
    BLOCK_ELEMENT_NAMES.contains(&name)
}

fn tag_boundary(remaining: &str) -> bool {
    remaining.chars().next().is_none_or(|character| {
        character == '>' || character == '/' || character.is_ascii_whitespace()
    })
}
