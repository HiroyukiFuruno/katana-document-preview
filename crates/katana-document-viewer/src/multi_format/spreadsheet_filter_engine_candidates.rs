use super::{
    MAX_FILTER_VALUES, SpreadsheetCellArtifact, SpreadsheetEngineError, SpreadsheetEngineSession,
    filter_rows, filter_sheet,
};
use std::collections::BTreeSet;

pub(in crate::multi_format) fn candidates(
    engine: &SpreadsheetEngineSession,
    sheet_index: usize,
    column: usize,
    limit: usize,
) -> Result<(Vec<String>, bool), SpreadsheetEngineError> {
    let sheet = filter_sheet(engine, sheet_index, column)?;
    if limit == 0 || limit > MAX_FILTER_VALUES {
        return Err(SpreadsheetEngineError::FilterValueLimit {
            actual: limit,
            limit: MAX_FILTER_VALUES,
        });
    }
    let mut values = BTreeSet::new();
    let mut truncated = false;
    let mut collect_candidates =
        |_: std::ops::Range<usize>, cells: Vec<SpreadsheetCellArtifact>| {
            collect_candidate_values(&mut values, &mut truncated, limit, cells)
        };
    engine.visit_filter_grid(
        sheet_index,
        &[column],
        filter_rows(sheet),
        &mut collect_candidates,
    )?;
    Ok((values.into_iter().collect(), truncated))
}

fn collect_candidate_values(
    values: &mut BTreeSet<String>,
    truncated: &mut bool,
    limit: usize,
    cells: Vec<SpreadsheetCellArtifact>,
) -> Result<(), SpreadsheetEngineError> {
    for cell in cells {
        if values.len() == limit && !values.contains(&cell.display_text) {
            *truncated = true;
            continue;
        }
        values.insert(cell.display_text);
    }
    Ok(())
}
