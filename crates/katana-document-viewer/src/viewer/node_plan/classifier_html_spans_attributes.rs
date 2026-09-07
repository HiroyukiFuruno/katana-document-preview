pub(super) fn is_exact_anchor_tag(tag: &str) -> bool {
    let lower = tag.trim_start().to_ascii_lowercase();
    let Some(after_name) = lower.strip_prefix("<a") else {
        return false;
    };
    after_name.chars().next().is_some_and(|character| {
        character.is_ascii_whitespace() || character == '>' || character == '/'
    })
}

pub(super) fn html_attribute_value(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let mut cursor = 0;
    let mut quote = None;
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
        if character == '>' {
            return None;
        }
        if attribute_value_starts_at(&lower, cursor, name) {
            return html_attribute_value_at(tag, cursor + name.len());
        }
        cursor += character.len_utf8();
    }
    None
}

fn attribute_value_starts_at(tag: &str, start: usize, name: &str) -> bool {
    if !tag[start..].starts_with(name)
        || !tag[..start]
            .chars()
            .next_back()
            .is_some_and(|character| character.is_ascii_whitespace())
    {
        return false;
    }
    tag[start + name.len()..]
        .trim_start_matches(char::is_whitespace)
        .starts_with('=')
}

fn html_attribute_value_at(tag: &str, name_end: usize) -> Option<String> {
    let after_name = tag[name_end..].trim_start_matches(char::is_whitespace);
    let value = after_name.strip_prefix('=')?.trim_start();
    let quote = value.chars().next()?;
    if quote == '"' || quote == '\'' {
        let target = &value[quote.len_utf8()..];
        let end = target.find(quote)?;
        return Some(target[..end].to_string());
    }
    let end = value
        .find(|character: char| character.is_whitespace() || character == '>')
        .unwrap_or(value.len());
    Some(value[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::{html_attribute_value, is_exact_anchor_tag};

    #[test]
    fn anchor_helper_rejects_non_anchor_prefixes() {
        assert!(!is_exact_anchor_tag(" <span>"));
    }

    #[test]
    fn attribute_helper_returns_none_after_scanning_without_a_match() {
        assert_eq!(None, html_attribute_value(r#"<a class="button""#, "href"));
    }
}
