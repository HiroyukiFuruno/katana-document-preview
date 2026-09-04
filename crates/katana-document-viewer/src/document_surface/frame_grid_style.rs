#[path = "frame_grid_borders.rs"]
mod borders;

pub use borders::{DocumentGridBorderSide, DocumentGridCellBorders};

use katana_ui_core::render_model::{
    UiGridCellAppearance, UiGridDataBar, UiGridHorizontalAlignment, UiGridIcon, UiGridRating,
    UiGridVerticalAlignment,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DocumentGridCellAppearance {
    pub font_family: String,
    pub font_size_px: u16,
    pub text_color: Option<String>,
    pub fill_color: Option<String>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub horizontal_alignment: DocumentGridHorizontalAlignment,
    pub vertical_alignment: DocumentGridVerticalAlignment,
    pub wrap_text: bool,
    pub data_bar: Option<DocumentGridDataBar>,
    pub icon: Option<DocumentGridIcon>,
    pub rating: Option<DocumentGridRating>,
    pub borders: DocumentGridCellBorders,
}

impl From<&UiGridCellAppearance> for DocumentGridCellAppearance {
    fn from(value: &UiGridCellAppearance) -> Self {
        Self {
            font_family: value.font_family.clone(),
            font_size_px: value.font_size_px,
            text_color: value.text_color.clone(),
            fill_color: value.fill_color.clone(),
            bold: value.bold,
            italic: value.italic,
            underline: value.underline,
            strike: value.strike,
            horizontal_alignment: DocumentGridHorizontalAlignment::from(value.horizontal_alignment),
            vertical_alignment: DocumentGridVerticalAlignment::from(value.vertical_alignment),
            wrap_text: value.wrap_text,
            data_bar: value.data_bar.as_ref().map(DocumentGridDataBar::from),
            icon: value.icon.as_ref().map(DocumentGridIcon::from),
            rating: value.rating.as_ref().map(DocumentGridRating::from),
            borders: DocumentGridCellBorders::from(&value.borders),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DocumentGridHorizontalAlignment {
    #[default]
    General,
    Left,
    Center,
    Right,
    Fill,
    Justify,
    Distributed,
}

impl From<UiGridHorizontalAlignment> for DocumentGridHorizontalAlignment {
    fn from(value: UiGridHorizontalAlignment) -> Self {
        match value {
            UiGridHorizontalAlignment::General => Self::General,
            UiGridHorizontalAlignment::Left => Self::Left,
            UiGridHorizontalAlignment::Center => Self::Center,
            UiGridHorizontalAlignment::Right => Self::Right,
            UiGridHorizontalAlignment::Fill => Self::Fill,
            UiGridHorizontalAlignment::Justify => Self::Justify,
            UiGridHorizontalAlignment::Distributed => Self::Distributed,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DocumentGridVerticalAlignment {
    #[default]
    Bottom,
    Center,
    Top,
    Justify,
    Distributed,
}

impl From<UiGridVerticalAlignment> for DocumentGridVerticalAlignment {
    fn from(value: UiGridVerticalAlignment) -> Self {
        match value {
            UiGridVerticalAlignment::Bottom => Self::Bottom,
            UiGridVerticalAlignment::Center => Self::Center,
            UiGridVerticalAlignment::Top => Self::Top,
            UiGridVerticalAlignment::Justify => Self::Justify,
            UiGridVerticalAlignment::Distributed => Self::Distributed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentGridDataBar {
    pub positive_color: Option<String>,
    pub negative_color: Option<String>,
    pub fill_ratio_basis_points: u16,
    pub axis_ratio_basis_points: u16,
    pub gradient: bool,
    pub show_value: bool,
}

impl From<&UiGridDataBar> for DocumentGridDataBar {
    fn from(value: &UiGridDataBar) -> Self {
        Self {
            positive_color: value.positive_color.clone(),
            negative_color: value.negative_color.clone(),
            fill_ratio_basis_points: value.fill_ratio_basis_points,
            axis_ratio_basis_points: value.axis_ratio_basis_points,
            gradient: value.gradient,
            show_value: value.show_value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentGridIcon {
    pub name: String,
    pub color: Option<String>,
    pub show_value: bool,
}

impl From<&UiGridIcon> for DocumentGridIcon {
    fn from(value: &UiGridIcon) -> Self {
        Self {
            name: value.name.clone(),
            color: value.color.clone(),
            show_value: value.show_value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentGridRating {
    pub icon_name: String,
    pub count: u32,
    pub maximum: u32,
    pub color: Option<String>,
    pub show_value: bool,
}

impl From<&UiGridRating> for DocumentGridRating {
    fn from(value: &UiGridRating) -> Self {
        Self {
            icon_name: value.icon_name.clone(),
            count: value.count,
            maximum: value.maximum,
            color: value.color.clone(),
            show_value: value.show_value,
        }
    }
}
