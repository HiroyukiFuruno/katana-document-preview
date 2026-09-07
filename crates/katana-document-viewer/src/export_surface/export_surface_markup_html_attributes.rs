use crate::export_surface_text::SurfaceTextParser;

pub(super) fn attribute_value(tag: &str, name: &str) -> Option<String> {
    let body = tag_body(tag)?;
    let value = attribute_value_in_body(body, name)?;
    Some(SurfaceTextParser::decode_basic_entities(value))
}

fn tag_body(tag: &str) -> Option<&str> {
    let tag_end = super::html_tag_end(tag)?;
    if !tag.starts_with('<') || tag.get(1..tag_end)?.starts_with('/') {
        return None;
    }
    tag.get(1..tag_end)
}

fn attribute_value_in_body<'a>(body: &'a str, name: &str) -> Option<&'a str> {
    let mut cursor = tag_name_end(body);
    while let Some((attribute_name, value, next_cursor)) = next_attribute(body, cursor) {
        if attribute_name.eq_ignore_ascii_case(name)
            && let Some(value) = value
        {
            return Some(value);
        }
        cursor = next_cursor;
    }
    None
}

fn tag_name_end(body: &str) -> usize {
    body.find(|character: char| character.is_ascii_whitespace() || character == '/')
        .unwrap_or(body.len())
}

fn next_attribute(body: &str, cursor: usize) -> Option<(&str, Option<&str>, usize)> {
    let cursor = skip_ascii_whitespace(body, cursor);
    if body.as_bytes().get(cursor).is_none_or(|byte| *byte == b'/') {
        return None;
    }
    let name_end = attribute_name_end(body, cursor);
    if cursor == name_end {
        return None;
    }
    let name = body.get(cursor..name_end)?;
    let after_name = skip_ascii_whitespace(body, name_end);
    if body.as_bytes().get(after_name) != Some(&b'=') {
        return Some((name, None, after_name));
    }
    let value_start = skip_ascii_whitespace(body, after_name + 1);
    let (value, next_cursor) = attribute_value_at(body, value_start)?;
    Some((name, Some(value), next_cursor))
}

fn attribute_name_end(body: &str, mut cursor: usize) -> usize {
    while body
        .as_bytes()
        .get(cursor)
        .is_some_and(|byte| !is_attribute_name_delimiter(*byte))
    {
        cursor += 1;
    }
    cursor
}

fn skip_ascii_whitespace(fragment: &str, mut cursor: usize) -> usize {
    while fragment
        .as_bytes()
        .get(cursor)
        .is_some_and(u8::is_ascii_whitespace)
    {
        cursor += 1;
    }
    cursor
}

fn is_attribute_name_delimiter(byte: u8) -> bool {
    byte.is_ascii_whitespace() || byte == b'=' || byte == b'/'
}

fn attribute_value_at(fragment: &str, cursor: usize) -> Option<(&str, usize)> {
    let quote = *fragment.as_bytes().get(cursor)?;
    if quote == b'"' || quote == b'\'' {
        let value_start = cursor + 1;
        let value_end = value_start + fragment[value_start..].find(quote as char)?;
        return Some((&fragment[value_start..value_end], value_end + 1));
    }
    let mut value_end = cursor;
    while let Some(byte) = fragment.as_bytes().get(value_end) {
        if byte.is_ascii_whitespace() {
            break;
        }
        value_end += 1;
    }
    Some((&fragment[cursor..value_end], value_end))
}
