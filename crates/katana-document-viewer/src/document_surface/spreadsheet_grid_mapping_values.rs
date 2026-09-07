use crate::{SpreadsheetDataBarArtifact, SpreadsheetIconArtifact, SpreadsheetRatingArtifact};
use katana_ui_core::molecule::{GridDataBar, GridIcon, GridRating};

pub(super) fn data_bar(value: SpreadsheetDataBarArtifact) -> GridDataBar {
    GridDataBar {
        positive_color: value.positive_color,
        negative_color: value.negative_color,
        fill_ratio_basis_points: ratio_basis_points(value.value),
        axis_ratio_basis_points: ratio_basis_points(value.axis_position),
        gradient: value.gradient,
        show_value: value.show_value,
    }
}

pub(super) fn icon(value: SpreadsheetIconArtifact) -> GridIcon {
    GridIcon {
        name: value.name,
        color: value.color,
        show_value: value.show_value,
    }
}

pub(super) fn rating(value: SpreadsheetRatingArtifact) -> GridRating {
    GridRating {
        icon_name: value.icon_name,
        count: value.count,
        maximum: value.maximum,
        color: value.color,
        show_value: value.show_value,
    }
}

pub(super) fn ratio_basis_points(value: f64) -> u16 {
    if !value.is_finite() {
        return 0;
    }
    (value.clamp(0.0, 1.0) * 10_000.0).round() as u16
}

#[cfg(test)]
mod tests {
    use super::ratio_basis_points;

    #[test]
    fn ratio_basis_points_clamps_non_finite_and_out_of_range_values() {
        assert_eq!(0, ratio_basis_points(f64::NAN));
        assert_eq!(0, ratio_basis_points(-0.5));
        assert_eq!(10_000, ratio_basis_points(1.5));
    }
}
