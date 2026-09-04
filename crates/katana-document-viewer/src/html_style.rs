#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct HtmlStyleProperties {
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    pub(crate) underline: bool,
    pub(crate) strikethrough: bool,
    pub(crate) highlight: bool,
    pub(crate) inline_code: bool,
    pub(crate) color_rgba: Option<[u8; 4]>,
}

impl HtmlStyleProperties {
    pub(crate) fn from_fragment(fragment: &str) -> Self {
        let mut properties = Self::from_tag_names(fragment);
        for style in style_attributes(fragment) {
            properties.apply_declarations(&style);
        }
        properties
    }

    fn from_tag_names(fragment: &str) -> Self {
        let lower = fragment.to_ascii_lowercase();
        Self {
            bold: tags::contains_opening_tag(&lower, "strong")
                || tags::contains_opening_tag(&lower, "b"),
            italic: tags::contains_opening_tag(&lower, "em")
                || tags::contains_opening_tag(&lower, "i"),
            underline: tags::contains_opening_tag(&lower, "u"),
            strikethrough: tags::contains_opening_tag(&lower, "s")
                || tags::contains_opening_tag(&lower, "del"),
            highlight: tags::contains_opening_tag(&lower, "mark"),
            inline_code: tags::contains_opening_tag(&lower, "code"),
            color_rgba: None,
        }
    }

    fn apply_declarations(&mut self, declarations: &str) {
        for declaration in declarations.split(';') {
            let Some((name, value)) = declaration.split_once(':') else {
                continue;
            };
            self.apply_declaration(name.trim(), value.trim());
        }
    }

    fn apply_declaration(&mut self, name: &str, value: &str) {
        let name = name.to_ascii_lowercase();
        let value = value.to_ascii_lowercase();
        match name.as_str() {
            "color" => self.color_rgba = parse_color(&value),
            "font-weight" if font_weight_is_bold(&value) => self.bold = true,
            "font-style" if value == "italic" => self.italic = true,
            "text-decoration" => {
                self.underline |= value.contains("underline");
                self.strikethrough |= value.contains("line-through");
            }
            "background" | "background-color" if value != "transparent" => self.highlight = true,
            "font-family" if value.contains("monospace") => self.inline_code = true,
            _ => {}
        }
    }
}

fn font_weight_is_bold(value: &str) -> bool {
    if value == "bold" {
        return true;
    }
    match value.parse::<u16>() {
        Ok(weight) => weight >= 600,
        Err(_) => false,
    }
}

fn style_attributes(fragment: &str) -> Vec<String> {
    let lower = fragment.to_ascii_lowercase();
    let mut cursor = 0;
    let mut values = Vec::new();
    while let Some(relative_start) = lower[cursor..].find("style") {
        let start = cursor + relative_start + "style".len();
        let Some(equals_relative) = lower[start..].find('=') else {
            break;
        };
        let value_start = start + equals_relative + 1;
        let Some((value, next_cursor)) = attribute_value(fragment, value_start) else {
            break;
        };
        values.push(value);
        cursor = next_cursor;
    }
    values
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

fn parse_color(value: &str) -> Option<[u8; 4]> {
    let trimmed = value.trim();
    if let Some(color) = parse_hex_color(trimmed) {
        return Some(color);
    }
    if let Some(color) = parse_rgb_color(trimmed) {
        return Some(color);
    }
    named_color(trimmed)
}

fn parse_hex_color(value: &str) -> Option<[u8; 4]> {
    let hex = value.strip_prefix('#')?;
    match hex.len() {
        3 => Some([
            hex_pair(hex.as_bytes()[0] as char)?,
            hex_pair(hex.as_bytes()[1] as char)?,
            hex_pair(hex.as_bytes()[2] as char)?,
            255,
        ]),
        6 => Some([
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
            255,
        ]),
        _ => None,
    }
}

fn hex_pair(character: char) -> Option<u8> {
    let value = character.to_digit(16)? as u8;
    Some(value * 17)
}

fn parse_rgb_color(value: &str) -> Option<[u8; 4]> {
    let body = value
        .strip_prefix("rgb(")
        .and_then(|value| value.strip_suffix(')'))?;
    let channels = body
        .split(',')
        .map(|part| part.trim().parse::<u8>().ok())
        .collect::<Option<Vec<_>>>()?;
    let [red, green, blue] = channels.as_slice() else {
        return None;
    };
    Some([*red, *green, *blue, 255])
}

fn named_color(value: &str) -> Option<[u8; 4]> {
    Some(match value {
        "black" => [0, 0, 0, 255],
        "white" => [255, 255, 255, 255],
        "red" => [255, 0, 0, 255],
        "green" => [0, 128, 0, 255],
        "blue" => [0, 0, 255, 255],
        "gray" | "grey" => [128, 128, 128, 255],
        _ => return None,
    })
}

#[path = "html_style_tags.rs"]
mod tags;
#[cfg(test)]
#[path = "html_style_tests.rs"]
mod tests;
