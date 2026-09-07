pub(crate) struct DirectHtmlVisibility;

impl DirectHtmlVisibility {
    pub(crate) fn visible_lines(content: &str) -> Vec<String> {
        let visible_content = strip_hidden_blocks(content);
        let mut lines = Vec::new();
        let mut state = SkipState::None;
        for raw_line in visible_content.lines() {
            let line = raw_line.trim();
            match state {
                SkipState::Style => {
                    state = skip_until_close(line, "style");
                }
                SkipState::Script | SkipState::Head => {
                    state = skip_until_close(line, state.tag_name());
                }
                SkipState::None if starts_tag(line, "style") => {
                    state = skip_until_close(line, "style");
                }
                SkipState::None if starts_tag(line, "script") => {
                    state = skip_until_close(line, "script");
                }
                SkipState::None if starts_tag(line, "head") => {
                    state = skip_until_close(line, "head");
                }
                SkipState::None => lines.push(strip_structural_wrappers(line)),
            }
        }
        lines
    }
}

#[derive(Clone, Copy)]
enum SkipState {
    None,
    Head,
    Style,
    Script,
}

impl SkipState {
    fn tag_name(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Head => "head",
            Self::Style => "style",
            Self::Script => "script",
        }
    }
}

fn starts_tag(line: &str, tag: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.starts_with(&format!("<{tag}>")) || lower.starts_with(&format!("<{tag} "))
}

fn skip_until_close(line: &str, tag: &str) -> SkipState {
    if line.to_ascii_lowercase().contains(&format!("</{tag}>")) {
        return SkipState::None;
    }
    match tag {
        "head" => SkipState::Head,
        "style" => SkipState::Style,
        "script" => SkipState::Script,
        _ => SkipState::None,
    }
}

fn strip_structural_wrappers(line: &str) -> String {
    ["!doctype", "html", "body", "main"]
        .iter()
        .fold(line.to_string(), |current, tag| {
            remove_tag_occurrences(&current, tag)
        })
}

fn strip_hidden_blocks(content: &str) -> String {
    ["head", "style", "script"]
        .iter()
        .fold(content.to_string(), |current, tag| {
            remove_block(&current, tag)
        })
}

fn remove_block(content: &str, tag: &str) -> String {
    let lower = content.to_ascii_lowercase();
    let mut cursor = 0;
    let mut stripped = String::new();
    while let Some(relative_start) = lower[cursor..].find(&format!("<{tag}")) {
        let start = cursor + relative_start;
        let Some(open_end_relative) = lower[start..].find('>') else {
            break;
        };
        let body_start = start + open_end_relative + 1;
        let Some(close_relative) = lower[body_start..].find(&format!("</{tag}>")) else {
            break;
        };
        let end = body_start + close_relative + tag.len() + 3;
        stripped.push_str(&content[cursor..start]);
        cursor = end;
    }
    stripped.push_str(&content[cursor..]);
    stripped
}

fn remove_tag_occurrences(line: &str, tag: &str) -> String {
    let mut cursor = 0;
    let mut stripped = String::new();
    while let Some((start, end)) = next_tag_range(&line[cursor..], tag) {
        let absolute_start = cursor + start;
        let absolute_end = cursor + end;
        stripped.push_str(&line[cursor..absolute_start]);
        cursor = absolute_end;
    }
    stripped.push_str(&line[cursor..]);
    stripped
}

fn next_tag_range(fragment: &str, tag: &str) -> Option<(usize, usize)> {
    let lower = fragment.to_ascii_lowercase();
    let opening = lower.find(&format!("<{tag}"));
    let closing = lower.find(&format!("</{tag}"));
    let start = match (opening, closing) {
        (Some(open), Some(close)) => open.min(close),
        (Some(open), None) => open,
        (None, Some(close)) => close,
        (None, None) => return None,
    };
    tag_end(&fragment[start..]).map(|end| (start, start + end + 1))
}

fn tag_end(fragment: &str) -> Option<usize> {
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

#[cfg(test)]
#[path = "direct_html_visibility_tests.rs"]
mod tests;
