pub(crate) struct DirectHtmlCssAttrs;

impl DirectHtmlCssAttrs {
    pub(crate) fn html_tag_end(fragment: &str) -> Option<usize> {
        let mut quote = None;
        for (index, character) in fragment.char_indices() {
            match (character, quote) {
                ('"' | '\'', None) => quote = Some(character),
                (current, Some(expected)) if current == expected => quote = None,
                ('>', None) => return Some(index),
                _ => {}
            }
        }
        None
    }

    pub(crate) fn tag_name(tag: &str) -> Option<String> {
        let tag = tag.trim_start().strip_prefix('<')?;
        if tag.starts_with('/') || tag.starts_with('!') || tag.starts_with('?') {
            return None;
        }
        let name = tag
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric())
            .collect::<String>();
        (!name.is_empty()).then(|| name.to_ascii_lowercase())
    }

    pub(crate) fn class_list(tag: &str) -> Vec<String> {
        match Self::attribute_value(tag, "class") {
            Some(value) => value.split_whitespace().map(str::to_string).collect(),
            None => Vec::new(),
        }
    }

    pub(crate) fn attribute_value(tag: &str, name: &str) -> Option<String> {
        let lower = tag.to_ascii_lowercase();
        let name = name.to_ascii_lowercase();
        attribute_value_range(tag, &lower, &name).map(|(_, value, _)| value)
    }

    pub(crate) fn style_attribute_range(tag: &str) -> Option<(std::ops::Range<usize>, String)> {
        let lower = tag.to_ascii_lowercase();
        let (start, value, value_range) = attribute_value_range(tag, &lower, "style")?;
        Some((start..value_range.end, value))
    }
}

fn attribute_value_range(
    tag: &str,
    lower: &str,
    name: &str,
) -> Option<(usize, String, std::ops::Range<usize>)> {
    let mut cursor = 0;
    let mut quote = None;
    while let Some(character) = lower[cursor..].chars().next() {
        if quoted_character(&mut quote, character) {
            cursor += character.len_utf8();
            continue;
        }
        if character == '>' {
            return None;
        }
        if let Some(value_start) = attribute_value_start(lower, cursor, name) {
            let (value, range) = quoted_attribute_value_at(tag, value_start)?;
            return Some((cursor, value, range));
        }
        cursor += character.len_utf8();
    }
    None
}

fn quoted_character(quote: &mut Option<char>, character: char) -> bool {
    if let Some(delimiter) = *quote {
        if character == delimiter {
            *quote = None;
        }
        return true;
    }
    if character == '"' || character == '\'' {
        *quote = Some(character);
        return true;
    }
    false
}

fn attribute_value_start(tag: &str, start: usize, name: &str) -> Option<usize> {
    if !tag[start..].starts_with(name) || !attribute_starts_after_whitespace(tag, start) {
        return None;
    }
    let name_end = start + name.len();
    let suffix = tag[name_end..].trim_start_matches(char::is_whitespace);
    let skipped = tag[name_end..].len() - suffix.len();
    suffix.starts_with('=').then_some(name_end + skipped + 1)
}

fn attribute_starts_after_whitespace(tag: &str, start: usize) -> bool {
    tag[..start]
        .chars()
        .next_back()
        .is_some_and(|character| character.is_ascii_whitespace())
}

fn quoted_attribute_value_at(tag: &str, start: usize) -> Option<(String, std::ops::Range<usize>)> {
    let value = tag[start..].trim_start();
    let skipped = tag[start..].len() - value.len();
    let value_start = start + skipped;
    let quote = value.chars().next()?;
    if quote == '"' || quote == '\'' {
        let body_start = value_start + quote.len_utf8();
        let body = &tag[body_start..];
        let end = body.find(quote)?;
        let body_end = body_start + end;
        return Some((
            tag[body_start..body_end].to_string(),
            value_start..body_end + quote.len_utf8(),
        ));
    }
    let end = value
        .find(|character: char| character.is_whitespace() || character == '>')
        .unwrap_or(value.len());
    Some((value[..end].to_string(), value_start..value_start + end))
}

#[cfg(test)]
#[path = "direct_html_css_attrs_tests.rs"]
mod tests;
