use super::{SpreadsheetBorderSideArtifact, SpreadsheetCellBorderArtifact};
use ironcalc::base::types::{Border, BorderItem, Color, Theme};

pub(super) fn borders(border: &Border, theme: &Theme) -> SpreadsheetCellBorderArtifact {
    SpreadsheetCellBorderArtifact {
        left: border_side(border.left.as_ref(), theme),
        right: border_side(border.right.as_ref(), theme),
        top: border_side(border.top.as_ref(), theme),
        bottom: border_side(border.bottom.as_ref(), theme),
    }
}

fn border_side(side: Option<&BorderItem>, theme: &Theme) -> Option<SpreadsheetBorderSideArtifact> {
    side.map(|side| SpreadsheetBorderSideArtifact {
        style: side.style.to_string(),
        color: color(&side.color, theme),
    })
}

pub(super) fn color(color: &Color, theme: &Theme) -> Option<String> {
    let value = color.to_rgb(theme);
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
#[path = "spreadsheet_engine_cell_border_tests.rs"]
mod tests;
