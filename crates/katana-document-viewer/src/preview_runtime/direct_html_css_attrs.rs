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
        let start = lower.find(name)? + name.len();
        let equals = lower[start..].find('=')? + start;
        quoted_attribute_value_at(tag, equals + 1).map(|(value, _)| value)
    }

    pub(crate) fn style_attribute_range(tag: &str) -> Option<(std::ops::Range<usize>, String)> {
        let lower = tag.to_ascii_lowercase();
        let style_start = lower.find("style")?;
        let equals = lower[style_start + "style".len()..].find('=')? + style_start + "style".len();
        let value_start = equals + 1;
        let (value, value_range) = quoted_attribute_value_at(tag, value_start)?;
        Some((style_start..value_range.end, value))
    }
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
