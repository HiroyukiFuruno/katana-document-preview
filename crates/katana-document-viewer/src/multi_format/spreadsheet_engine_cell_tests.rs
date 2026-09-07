use super::{
    Alignment, CfDataBar, CfIcon, CfRating, HorizontalAlignment, SpreadsheetCellMaterializer,
    SpreadsheetHorizontalAlignment, SpreadsheetVerticalAlignment, Style, Theme, VerticalAlignment,
};
use ironcalc::base::cf_types::{ExtendedStyle, Icon};
use ironcalc::base::types::Color;

fn data_bar() -> CfDataBar {
    CfDataBar {
        positive_color: Color::Rgb("#00ff00".to_owned()),
        negative_color: Color::Rgb("#ff0000".to_owned()),
        is_gradient: true,
        value: 0.75,
        axis_position: 0.25,
        show_value: false,
    }
}

#[test]
fn empty_conditional_formatting_is_not_applied() {
    let base = Style::default();
    let artifact = SpreadsheetCellMaterializer::conditional_formatting(
        &base,
        ExtendedStyle {
            style: base.clone(),
            icon: None,
            data_bar: None,
            rating: None,
        },
        &Theme::default(),
    );
    assert!(!artifact.applied);
    assert!(artifact.data_bar.is_none());
    assert!(artifact.icon.is_none());
    assert!(artifact.rating.is_none());
}

#[test]
fn populated_conditional_formatting_keeps_all_artifacts() {
    let base = Style::default();
    let artifact = SpreadsheetCellMaterializer::conditional_formatting(
        &base,
        ExtendedStyle {
            style: base.clone(),
            data_bar: Some(data_bar()),
            icon: Some(CfIcon {
                icon: Icon::Star,
                color: Color::Rgb("#0000ff".to_owned()),
                show_value: true,
            }),
            rating: Some(CfRating {
                icon: Icon::Heart,
                count: 3,
                max: 5,
                color: Color::Rgb("#ff00ff".to_owned()),
                show_value: false,
            }),
        },
        &Theme::default(),
    );
    assert!(artifact.applied);
    assert!(artifact.data_bar.is_some());
    assert!(artifact.icon.is_some());
    assert!(artifact.rating.is_some());
}

#[test]
fn conditional_formatting_components_keep_engine_values() {
    let theme = Theme::default();
    let data_bar = SpreadsheetCellMaterializer::data_bar(data_bar(), &theme);
    assert_eq!(0.75, data_bar.value);
    assert!(data_bar.gradient);
    let icon = SpreadsheetCellMaterializer::icon(
        CfIcon {
            icon: Icon::Star,
            color: Color::Rgb("#0000ff".to_owned()),
            show_value: true,
        },
        &theme,
    );
    assert_eq!("Star", icon.name);
    let rating = SpreadsheetCellMaterializer::rating(
        CfRating {
            icon: Icon::Heart,
            count: 3,
            max: 5,
            color: Color::Rgb("#ff00ff".to_owned()),
            show_value: false,
        },
        &theme,
    );
    assert_eq!((3, 5), (rating.count, rating.maximum));
}

fn assert_horizontal(cases: &[(HorizontalAlignment, SpreadsheetHorizontalAlignment)]) {
    for (input, expected) in cases {
        let alignment = Alignment {
            horizontal: input.clone(),
            ..Alignment::default()
        };
        assert_eq!(
            expected,
            &SpreadsheetCellMaterializer::horizontal_alignment(&alignment)
        );
    }
}

#[test]
fn common_horizontal_alignments_map_to_neutral_types() {
    assert_horizontal(&[
        (
            HorizontalAlignment::General,
            SpreadsheetHorizontalAlignment::General,
        ),
        (
            HorizontalAlignment::Left,
            SpreadsheetHorizontalAlignment::Left,
        ),
        (
            HorizontalAlignment::Center,
            SpreadsheetHorizontalAlignment::Center,
        ),
        (
            HorizontalAlignment::CenterContinuous,
            SpreadsheetHorizontalAlignment::CenterContinuous,
        ),
    ]);
}

#[test]
fn extended_horizontal_alignments_map_to_neutral_types() {
    assert_horizontal(&[
        (
            HorizontalAlignment::Right,
            SpreadsheetHorizontalAlignment::Right,
        ),
        (
            HorizontalAlignment::Fill,
            SpreadsheetHorizontalAlignment::Fill,
        ),
        (
            HorizontalAlignment::Justify,
            SpreadsheetHorizontalAlignment::Justify,
        ),
        (
            HorizontalAlignment::Distributed,
            SpreadsheetHorizontalAlignment::Distributed,
        ),
    ]);
}

fn assert_vertical(cases: &[(VerticalAlignment, SpreadsheetVerticalAlignment)]) {
    for (input, expected) in cases {
        let alignment = Alignment {
            vertical: input.clone(),
            ..Alignment::default()
        };
        assert_eq!(
            expected,
            &SpreadsheetCellMaterializer::vertical_alignment(&alignment)
        );
    }
}

#[test]
fn every_vertical_alignment_maps_to_neutral_types() {
    assert_vertical(&[
        (
            VerticalAlignment::Bottom,
            SpreadsheetVerticalAlignment::Bottom,
        ),
        (
            VerticalAlignment::Center,
            SpreadsheetVerticalAlignment::Center,
        ),
        (VerticalAlignment::Top, SpreadsheetVerticalAlignment::Top),
        (
            VerticalAlignment::Justify,
            SpreadsheetVerticalAlignment::Justify,
        ),
        (
            VerticalAlignment::Distributed,
            SpreadsheetVerticalAlignment::Distributed,
        ),
    ]);
}
