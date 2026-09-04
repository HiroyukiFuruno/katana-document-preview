pub(super) fn style_values(fragment: &str) -> Vec<String> {
    let lower = fragment.to_ascii_lowercase();
    let mut cursor = 0;
    let mut quote = None;
    let mut values = Vec::new();
    while let Some(character) = lower[cursor..].chars().next() {
        if let Some(delimiter) = quote {
            if character == delimiter {
                quote = None;
            }
            cursor += character.len_utf8();
            continue;
        }
        if character == '"' || character == '\'' {
            quote = Some(character);
            cursor += character.len_utf8();
            continue;
        }
        if let Some((value, next_cursor)) = style_attribute_at(fragment, &lower, cursor) {
            values.push(value);
            cursor = next_cursor;
            continue;
        }
        cursor += character.len_utf8();
    }
    values
}

fn style_attribute_at(fragment: &str, lower: &str, start: usize) -> Option<(String, usize)> {
    if !lower[start..].starts_with("style") || !attribute_name_starts_after_whitespace(lower, start)
    {
        return None;
    }
    let value_start = style_value_start(lower, start + "style".len())?;
    attribute_value(fragment, value_start)
}

fn attribute_name_starts_after_whitespace(fragment: &str, start: usize) -> bool {
    start == 0
        || fragment[..start]
            .chars()
            .next_back()
            .is_some_and(|character| character.is_ascii_whitespace())
}

fn style_value_start(fragment: &str, name_end: usize) -> Option<usize> {
    let suffix = fragment[name_end..].trim_start_matches(char::is_whitespace);
    let skipped = fragment[name_end..].len() - suffix.len();
    suffix.starts_with('=').then_some(name_end + skipped + 1)
}

fn attribute_value(fragment: &str, start: usize) -> Option<(String, usize)> {
    let value = fragment[start..].trim_start();
    let skipped = fragment[start..].len() - value.len();
    let start = start + skipped;
    let quote = value.chars().next()?;
    if quote == '"' || quote == '\'' {
        let body = &value[quote.len_utf8()..];
        let end = body.find(quote)?;
        return Some((
            body[..end].to_string(),
            start + quote.len_utf8() + end + quote.len_utf8(),
        ));
    }
    let end = value
        .find(|character: char| character.is_whitespace() || character == '>')
        .unwrap_or(value.len());
    Some((value[..end].to_string(), start + end))
}
