use super::SpreadsheetActiveFilters;
use crate::multi_format::{SpreadsheetFilterCriterion, SpreadsheetSheetArtifact};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct SpreadsheetPersistedFilterEngine;

impl SpreadsheetPersistedFilterEngine {
    pub(super) fn persisted_filters(
        sheets: &[SpreadsheetSheetArtifact],
    ) -> SpreadsheetActiveFilters {
        sheets
            .iter()
            .map(|sheet| {
                let Some(filter) = &sheet.auto_filter else {
                    return BTreeMap::new();
                };
                filter
                    .columns
                    .iter()
                    .filter_map(|column| {
                        let values = persisted_values(&column.criteria)?;
                        Some((column.column, values))
                    })
                    .collect()
            })
            .collect()
    }
}

fn persisted_values(criteria: &[SpreadsheetFilterCriterion]) -> Option<BTreeSet<String>> {
    let mut values = BTreeSet::new();
    for criterion in criteria {
        match criterion {
            SpreadsheetFilterCriterion::Values(selected) => values.extend(selected.iter().cloned()),
            SpreadsheetFilterCriterion::Blank => {
                values.insert(String::new());
            }
            SpreadsheetFilterCriterion::NonBlank | SpreadsheetFilterCriterion::Unsupported(_) => {
                return None;
            }
        }
    }
    (!values.is_empty()).then_some(values)
}
