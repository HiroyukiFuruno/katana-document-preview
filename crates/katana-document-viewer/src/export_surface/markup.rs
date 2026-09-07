use super::SurfaceBadge;
use crate::export_surface_text::SurfaceTextParser;
use katana_markdown_model::{KmmNode, ListItemNode};

#[path = "export_surface_markup_alert.rs"]
mod export_surface_markup_alert;
#[path = "export_surface_markup_html.rs"]
mod export_surface_markup_html;
#[path = "export_surface_markup_html_style.rs"]
mod export_surface_markup_html_style;

pub(super) use self::export_surface_markup_alert::{
    alert_body_lines, alert_color, alert_label_text,
};
#[cfg(test)]
pub(super) use self::export_surface_markup_alert::{alert_icon_name, alert_title};
pub(super) use self::export_surface_markup_html::SurfaceHtmlMarkup;

pub(super) struct SurfaceDetailsParts<'a> {
    pub(super) summary: &'a str,
    pub(super) body: &'a str,
}

impl<'a> SurfaceDetailsParts<'a> {
    pub(super) fn parse(fragment: &'a str) -> Option<Self> {
        let trimmed = fragment.trim();
        if !trimmed.starts_with("<details") {
            return None;
        }
        let summary_start = trimmed.find("<summary>")? + "<summary>".len();
        let summary_end = trimmed.find("</summary>")?;
        let body_start = summary_end + "</summary>".len();
        let body_end = trimmed.rfind("</details>")?;
        let body = Self::strip_div(&trimmed[body_start..body_end]);
        Some(Self {
            summary: &trimmed[summary_start..summary_end],
            body,
        })
    }

    fn strip_div(value: &'a str) -> &'a str {
        let trimmed = value.trim();
        let lower = trimmed.to_ascii_lowercase();
        if !lower.starts_with("<div") {
            return trimmed;
        }
        let Some(open_end) = trimmed.find('>') else {
            return trimmed;
        };
        let body = &trimmed[open_end + 1..];
        let body = body.trim_end();
        if body.to_ascii_lowercase().ends_with("</div>") {
            return &body[..body.len() - "</div>".len()];
        }
        body
    }
}

pub(super) fn list_marker_text(item: &ListItemNode, ordered: bool) -> String {
    if let Some(marker) = &item.task_marker {
        return format!("{} ", task_marker_text(marker));
    }
    if ordered {
        let number = item
            .ordered_number
            .or_else(|| ordered_number_from_marker(&item.marker))
            .unwrap_or(1);
        return format!("{number}. ");
    }
    "• ".to_string()
}

fn ordered_number_from_marker(marker: &str) -> Option<usize> {
    marker
        .trim_end_matches('.')
        .trim_end_matches(')')
        .parse::<usize>()
        .ok()
}

fn task_marker_text(marker: &str) -> &'static str {
    match marker {
        "[x]" => "☑",
        "[ ]" => "☐",
        "[-]" => "⊟",
        "[/]" => "◩",
        _ => "☐",
    }
}

pub(super) fn legacy_note_quote(raw: &str) -> Option<(String, String)> {
    let mut lines = raw
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix('>'));
    let title = lines
        .next()?
        .trim()
        .strip_prefix("**")?
        .strip_suffix("**")?
        .trim()
        .to_string();
    if !is_legacy_note_title(&title) {
        return None;
    }
    let body = lines
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(SurfaceTextParser::inline_markdown_text)
        .collect::<Vec<_>>()
        .join(" ");
    (!body.is_empty()).then_some((title, body))
}

pub(super) fn legacy_note_children(children: &[KmmNode]) -> Option<(String, String)> {
    let (first, rest) = children.split_first()?;
    let title = SurfaceTextParser::inline_text(first).trim().to_string();
    if !is_legacy_note_title(&title) {
        return None;
    }
    let body = rest
        .iter()
        .map(SurfaceTextParser::inline_text)
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    (!body.is_empty()).then_some((title, body))
}

fn is_legacy_note_title(title: &str) -> bool {
    matches!(title, "Note" | "Tip" | "Important" | "Warning" | "Caution")
}

#[cfg(test)]
#[path = "markup_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "markup_details_tests.rs"]
mod details_tests;
