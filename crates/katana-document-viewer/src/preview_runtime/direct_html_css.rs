use crate::preview_runtime::direct_html_css_attrs::DirectHtmlCssAttrs;

#[derive(Debug, Clone)]
pub(crate) struct DirectHtmlCss {
    rules: Vec<CssRule>,
}

impl DirectHtmlCss {
    pub(crate) fn parse(css: &str) -> Self {
        Self {
            rules: parse_rules(css),
        }
    }

    pub(crate) fn apply_to_line(&self, line: &str) -> String {
        let mut output = String::with_capacity(line.len());
        let mut cursor = 0;
        while let Some(relative_start) = line[cursor..].find('<') {
            let tag_start = cursor + relative_start;
            output.push_str(&line[cursor..tag_start]);
            let fragment = &line[tag_start..];
            let Some(relative_end) = DirectHtmlCssAttrs::html_tag_end(fragment) else {
                output.push_str(fragment);
                return output;
            };
            let tag_end = tag_start + relative_end;
            let tag = &line[tag_start..=tag_end];
            output.push_str(&self.apply_to_tag(tag));
            cursor = tag_end + 1;
        }
        output.push_str(&line[cursor..]);
        output
    }

    fn apply_to_tag(&self, tag: &str) -> String {
        let Some(tag_name) = DirectHtmlCssAttrs::tag_name(tag) else {
            return tag.to_string();
        };
        let mut matching_rules = self
            .rules
            .iter()
            .enumerate()
            .filter(|(_, rule)| rule.matches(tag, &tag_name))
            .collect::<Vec<_>>();
        matching_rules.sort_by_key(|(source_order, rule)| (rule.specificity(), *source_order));
        let declarations = matching_rules
            .into_iter()
            .flat_map(|(_, rule)| rule.declarations.iter().cloned())
            .collect::<Vec<_>>();
        if declarations.is_empty() {
            return tag.to_string();
        }
        merge_style_attribute(tag, &declarations)
    }
}

#[derive(Debug, Clone)]
struct CssRule {
    selector: String,
    declarations: Vec<String>,
}

impl CssRule {
    fn matches(&self, tag: &str, tag_name: &str) -> bool {
        if self.selector.eq_ignore_ascii_case("body") {
            return true;
        }
        if let Some(class) = self.selector.strip_prefix('.') {
            return DirectHtmlCssAttrs::class_list(tag)
                .iter()
                .any(|candidate| candidate == class);
        }
        if let Some(id) = self.selector.strip_prefix('#') {
            return DirectHtmlCssAttrs::attribute_value(tag, "id").is_some_and(|value| value == id);
        }
        self.selector.eq_ignore_ascii_case(tag_name)
    }

    fn specificity(&self) -> u8 {
        if self.selector.eq_ignore_ascii_case("body") {
            0
        } else if self.selector.starts_with('#') {
            100
        } else if self.selector.starts_with('.') {
            10
        } else {
            1
        }
    }
}

fn parse_rules(css: &str) -> Vec<CssRule> {
    let mut rules = Vec::new();
    for chunk in css.split('}') {
        let Some((selector, body)) = chunk.split_once('{') else {
            continue;
        };
        let selector = selector.trim();
        if selector.is_empty() || selector.contains(',') || selector.contains(' ') {
            continue;
        }
        let declarations = parse_declarations(body);
        if !declarations.is_empty() {
            rules.push(CssRule {
                selector: selector.to_string(),
                declarations,
            });
        }
    }
    rules
}

fn parse_declarations(body: &str) -> Vec<String> {
    body.split(';')
        .filter_map(|declaration| {
            let (name, value) = declaration.split_once(':')?;
            let name = name.trim();
            let value = value.trim();
            if supported_property(name) && !value.is_empty() {
                Some(format!("{name}: {value}"))
            } else {
                None
            }
        })
        .collect()
}

fn supported_property(name: &str) -> bool {
    matches!(
        name.trim().to_ascii_lowercase().as_str(),
        "color"
            | "font-weight"
            | "font-style"
            | "font-family"
            | "text-align"
            | "text-decoration"
            | "background"
            | "background-color"
    )
}

fn merge_style_attribute(line: &str, declarations: &[String]) -> String {
    let Some(tag_end) = DirectHtmlCssAttrs::html_tag_end(line) else {
        return line.to_string();
    };
    let tag = &line[..=tag_end];
    let inherited = declarations.join("; ");
    if let Some((range, existing)) = DirectHtmlCssAttrs::style_attribute_range(tag) {
        return replace_style_attribute(line, range, merged_style_value(&inherited, &existing));
    }
    insert_style_attribute(line, tag, tag_end, inherited)
}

fn merged_style_value(inherited: &str, existing: &str) -> String {
    if existing.trim().is_empty() {
        return inherited.to_string();
    }
    format!("{inherited}; {existing}")
}

fn replace_style_attribute(
    line: &str,
    range: std::ops::Range<usize>,
    merged_style: String,
) -> String {
    let merged_style = escape_style_attribute_value(&merged_style);
    format!(
        "{}style=\"{}\"{}",
        &line[..range.start],
        merged_style,
        &line[range.end..]
    )
}

fn insert_style_attribute(line: &str, tag: &str, tag_end: usize, inherited: String) -> String {
    let insert_at = style_insert_position(tag, tag_end);
    let inherited = escape_style_attribute_value(&inherited);
    format!(
        "{} style=\"{}\"{}",
        &line[..insert_at],
        inherited,
        &line[insert_at..]
    )
}

fn escape_style_attribute_value(value: &str) -> String {
    value.replace('"', "&quot;")
}

fn style_insert_position(tag: &str, tag_end: usize) -> usize {
    if tag.ends_with("/>") {
        return tag_end.saturating_sub(1);
    }
    tag_end
}

#[cfg(test)]
#[path = "direct_html_css_tests.rs"]
mod tests;
