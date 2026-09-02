use super::borders;
use ironcalc::base::types::{Border, BorderItem, BorderStyle, Color, Theme};

#[test]
fn border_artifact_preserves_each_supported_side_without_ui_projection() {
    let border = Border {
        left: Some(BorderItem {
            style: BorderStyle::Thin,
            color: Color::Rgb("#B7C4CE".to_owned()),
        }),
        bottom: Some(BorderItem {
            style: BorderStyle::Double,
            color: Color::Rgb("#183B66".to_owned()),
        }),
        ..Border::default()
    };
    let artifact = borders(&border, &Theme::default());
    assert_eq!(
        Some(("thin", Some("#B7C4CE"))),
        artifact
            .left
            .as_ref()
            .map(|side| (side.style.as_str(), side.color.as_deref()))
    );
    assert_eq!(
        Some(("double", Some("#183B66"))),
        artifact
            .bottom
            .as_ref()
            .map(|side| (side.style.as_str(), side.color.as_deref()))
    );
    assert!(artifact.right.is_none());
    assert!(artifact.top.is_none());
}
