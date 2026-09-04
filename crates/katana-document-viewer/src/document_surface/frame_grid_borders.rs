use katana_ui_core::render_model::{UiGridBorderLineStyle, UiGridBorderSide, UiGridCellBorders};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DocumentGridCellBorders {
    pub left: Option<DocumentGridBorderSide>,
    pub right: Option<DocumentGridBorderSide>,
    pub top: Option<DocumentGridBorderSide>,
    pub bottom: Option<DocumentGridBorderSide>,
}

impl From<&crate::SpreadsheetCellBorderArtifact> for DocumentGridCellBorders {
    fn from(value: &crate::SpreadsheetCellBorderArtifact) -> Self {
        Self {
            left: value.left.as_ref().map(DocumentGridBorderSide::from),
            right: value.right.as_ref().map(DocumentGridBorderSide::from),
            top: value.top.as_ref().map(DocumentGridBorderSide::from),
            bottom: value.bottom.as_ref().map(DocumentGridBorderSide::from),
        }
    }
}

impl From<&UiGridCellBorders> for DocumentGridCellBorders {
    fn from(value: &UiGridCellBorders) -> Self {
        Self {
            left: value
                .left
                .is_visible()
                .then(|| DocumentGridBorderSide::from(&value.left)),
            right: value
                .right
                .is_visible()
                .then(|| DocumentGridBorderSide::from(&value.right)),
            top: value
                .top
                .is_visible()
                .then(|| DocumentGridBorderSide::from(&value.top)),
            bottom: value
                .bottom
                .is_visible()
                .then(|| DocumentGridBorderSide::from(&value.bottom)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentGridBorderSide {
    pub style: String,
    pub color: Option<String>,
}

impl From<&crate::SpreadsheetBorderSideArtifact> for DocumentGridBorderSide {
    fn from(value: &crate::SpreadsheetBorderSideArtifact) -> Self {
        Self {
            style: value.style.clone(),
            color: value.color.clone(),
        }
    }
}

impl From<&UiGridBorderSide> for DocumentGridBorderSide {
    fn from(value: &UiGridBorderSide) -> Self {
        Self {
            style: grid_border_style_name(value.line_style).to_owned(),
            color: value.color.clone(),
        }
    }
}

const fn grid_border_style_name(value: UiGridBorderLineStyle) -> &'static str {
    match value {
        UiGridBorderLineStyle::None => "none",
        UiGridBorderLineStyle::Hair => "hair",
        UiGridBorderLineStyle::Thin => "thin",
        UiGridBorderLineStyle::Medium => "medium",
        UiGridBorderLineStyle::Thick => "thick",
        UiGridBorderLineStyle::Double => "double",
        UiGridBorderLineStyle::Dotted => "dotted",
        UiGridBorderLineStyle::Dashed => "dashed",
        UiGridBorderLineStyle::DashDot => "dashDot",
        UiGridBorderLineStyle::DashDotDot => "dashDotDot",
        UiGridBorderLineStyle::MediumDashed => "mediumDashed",
        UiGridBorderLineStyle::MediumDashDot => "mediumDashDot",
        UiGridBorderLineStyle::MediumDashDotDot => "mediumDashDotDot",
        UiGridBorderLineStyle::SlantDashDot => "slantDashDot",
        UiGridBorderLineStyle::Solid => "solid",
    }
}

#[cfg(test)]
mod tests {
    use super::{DocumentGridBorderSide, DocumentGridCellBorders};
    use crate::{SpreadsheetBorderSideArtifact, SpreadsheetCellBorderArtifact};
    use katana_ui_core::render_model::{
        UiGridBorderLineStyle, UiGridBorderSide, UiGridCellBorders,
    };

    #[test]
    fn artifact_borders_keep_each_authored_side() {
        let left_color = color([17, 34, 51]);
        let right_color = color([68, 85, 102]);
        let bottom_color = color([119, 136, 153]);
        let borders = SpreadsheetCellBorderArtifact {
            left: Some(artifact_side("thin", Some(&left_color))),
            right: Some(artifact_side("double", Some(&right_color))),
            top: Some(artifact_side("dotted", None)),
            bottom: Some(artifact_side("solid", Some(&bottom_color))),
        };

        let projected = DocumentGridCellBorders::from(&borders);

        assert_eq!(Some(border_side("thin", Some(&left_color))), projected.left);
        assert_eq!(
            Some(border_side("double", Some(&right_color))),
            projected.right
        );
        assert_eq!(Some(border_side("dotted", None)), projected.top);
        assert_eq!(
            Some(border_side("solid", Some(&bottom_color))),
            projected.bottom
        );
    }

    #[test]
    fn kuc_borders_project_visibility_and_every_line_style() {
        let test_color = color([170, 187, 204]);
        for (line_style, expected_name) in border_style_names() {
            let projected = DocumentGridBorderSide::from(&UiGridBorderSide {
                line_style,
                color: Some(test_color.clone()),
            });
            assert_eq!(border_side(expected_name, Some(&test_color)), projected);
        }

        let visible = UiGridCellBorders {
            left: UiGridBorderSide::solid(color([17, 34, 51])),
            right: UiGridBorderSide::solid(color([68, 85, 102])),
            top: UiGridBorderSide::solid(color([119, 136, 153])),
            bottom: UiGridBorderSide::solid(test_color),
        };
        let projected = DocumentGridCellBorders::from(&visible);
        assert!(projected.left.is_some());
        assert!(projected.right.is_some());
        assert!(projected.top.is_some());
        assert!(projected.bottom.is_some());

        let hidden = DocumentGridCellBorders::from(&UiGridCellBorders::default());
        assert_eq!(None, hidden.left);
        assert_eq!(None, hidden.right);
        assert_eq!(None, hidden.top);
        assert_eq!(None, hidden.bottom);
    }

    fn artifact_side(style: &str, color: Option<&str>) -> SpreadsheetBorderSideArtifact {
        SpreadsheetBorderSideArtifact {
            style: style.to_owned(),
            color: color.map(str::to_owned),
        }
    }

    fn border_side(style: &str, color: Option<&str>) -> DocumentGridBorderSide {
        DocumentGridBorderSide {
            style: style.to_owned(),
            color: color.map(str::to_owned),
        }
    }

    fn color(bytes: [u8; 3]) -> String {
        format!("#{:02X}{:02X}{:02X}", bytes[0], bytes[1], bytes[2])
    }

    fn border_style_names() -> [(UiGridBorderLineStyle, &'static str); 15] {
        [
            (UiGridBorderLineStyle::None, "none"),
            (UiGridBorderLineStyle::Hair, "hair"),
            (UiGridBorderLineStyle::Thin, "thin"),
            (UiGridBorderLineStyle::Medium, "medium"),
            (UiGridBorderLineStyle::Thick, "thick"),
            (UiGridBorderLineStyle::Double, "double"),
            (UiGridBorderLineStyle::Dotted, "dotted"),
            (UiGridBorderLineStyle::Dashed, "dashed"),
            (UiGridBorderLineStyle::DashDot, "dashDot"),
            (UiGridBorderLineStyle::DashDotDot, "dashDotDot"),
            (UiGridBorderLineStyle::MediumDashed, "mediumDashed"),
            (UiGridBorderLineStyle::MediumDashDot, "mediumDashDot"),
            (UiGridBorderLineStyle::MediumDashDotDot, "mediumDashDotDot"),
            (UiGridBorderLineStyle::SlantDashDot, "slantDashDot"),
            (UiGridBorderLineStyle::Solid, "solid"),
        ]
    }
}
