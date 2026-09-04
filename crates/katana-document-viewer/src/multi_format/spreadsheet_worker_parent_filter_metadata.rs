use crate::multi_format::{
    SpreadsheetAutoFilterArtifact, SpreadsheetFilterColumnArtifact, SpreadsheetFilterCommand,
    SpreadsheetFilterCriterion,
};

pub(super) fn update_filter_criteria(
    filter: &mut SpreadsheetAutoFilterArtifact,
    command: &SpreadsheetFilterCommand,
) {
    match command {
        SpreadsheetFilterCommand::ApplyValues { column, values, .. } => {
            filter_column(filter, *column).criteria =
                vec![SpreadsheetFilterCriterion::Values(values.clone())];
        }
        SpreadsheetFilterCommand::Clear {
            column: Some(column),
            ..
        } => clear_filter_column(filter, *column),
        SpreadsheetFilterCommand::Clear { column: None, .. } => clear_all_filter_columns(filter),
        SpreadsheetFilterCommand::Candidates { .. } => {}
    }
}

fn clear_filter_column(filter: &mut SpreadsheetAutoFilterArtifact, column: usize) {
    if let Some(filter_column) = filter
        .columns
        .iter_mut()
        .find(|candidate| candidate.column == column)
    {
        filter_column.criteria.clear();
    }
}

fn clear_all_filter_columns(filter: &mut SpreadsheetAutoFilterArtifact) {
    for filter_column in &mut filter.columns {
        filter_column.criteria.clear();
    }
}

fn filter_column(
    filter: &mut SpreadsheetAutoFilterArtifact,
    column: usize,
) -> &mut SpreadsheetFilterColumnArtifact {
    if let Some(index) = filter
        .columns
        .iter()
        .position(|candidate| candidate.column == column)
    {
        return &mut filter.columns[index];
    }
    let index = filter
        .columns
        .iter()
        .position(|candidate| candidate.column > column)
        .unwrap_or(filter.columns.len());
    filter.columns.insert(
        index,
        SpreadsheetFilterColumnArtifact {
            column,
            criteria: Vec::new(),
            candidates: Vec::new(),
        },
    );
    &mut filter.columns[index]
}

#[cfg(test)]
mod tests {
    use super::update_filter_criteria;
    use crate::multi_format::{
        SpreadsheetAutoFilterArtifact, SpreadsheetCoordinate, SpreadsheetFilterColumnArtifact,
        SpreadsheetFilterCommand, SpreadsheetFilterCriterion, SpreadsheetFilterRange,
    };

    #[test]
    fn candidates_do_not_change_the_selected_filter_criteria() {
        let mut filter = SpreadsheetAutoFilterArtifact {
            range: SpreadsheetFilterRange {
                start: SpreadsheetCoordinate::new(0, 0),
                end: SpreadsheetCoordinate::new(1, 1),
            },
            columns: vec![SpreadsheetFilterColumnArtifact {
                column: 0,
                criteria: vec![SpreadsheetFilterCriterion::Values(vec!["North".to_owned()])],
                candidates: Vec::new(),
            }],
            filtered_out_rows: vec![1],
            diagnostics: Vec::new(),
        };
        let expected = filter.clone();

        update_filter_criteria(
            &mut filter,
            &SpreadsheetFilterCommand::Candidates {
                sheet_index: 0,
                column: 0,
                limit: 16,
            },
        );

        assert_eq!(expected, filter);
    }
}
