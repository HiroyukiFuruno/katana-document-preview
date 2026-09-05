pub(super) enum HtmlTag {
    Opening { name: String, self_closing: bool },
    Closing { name: String },
    Other,
}

pub(super) fn is_void_element(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

pub(super) fn next_start(raw: &str, cursor: usize) -> Option<usize> {
    raw[cursor..].find('<').map(|index| cursor + index)
}

pub(super) fn end(raw: &str, start: usize) -> Option<usize> {
    let mut quote = None;
    for (offset, character) in raw[start..].char_indices() {
        match (quote, character) {
            (None, '\'' | '"') => quote = Some(character),
            (Some(active), character) if character == active => quote = None,
            (None, '>') => return Some(start + offset),
            _ => {}
        }
    }
    None
}

pub(super) fn parse(tag: &str) -> HtmlTag {
    let Some(body) = tag
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
        .map(str::trim)
    else {
        return HtmlTag::Other;
    };
    let (closing, body) = match body.strip_prefix('/') {
        Some(body) => (true, body.trim_start()),
        None => (false, body),
    };
    let name = body
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    if name.is_empty() || body.starts_with('!') || body.starts_with('?') {
        return HtmlTag::Other;
    }
    if closing {
        return HtmlTag::Closing { name };
    }
    HtmlTag::Opening {
        name,
        self_closing: body.trim_end().ends_with('/'),
    }
}

#[cfg(test)]
mod tests {
    use super::{HtmlTag, parse};

    #[test]
    fn malformed_non_tag_is_other() {
        assert!(matches!(parse("span"), HtmlTag::Other));
    }
}
