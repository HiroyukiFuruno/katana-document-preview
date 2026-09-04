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

fn tag_boundary(remaining: &str) -> bool {
    remaining.chars().next().is_none_or(|character| {
        character == '>' || character == '/' || character.is_ascii_whitespace()
    })
}
