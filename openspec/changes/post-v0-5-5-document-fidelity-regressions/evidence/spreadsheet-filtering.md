# Spreadsheet filtering evidence

- Candidate extraction, selected-value evaluation, and clear are typed worker
  requests/responses. Worker responses contain candidates or original row
  indices, not full-sheet cell payloads.
- Evaluation chunks ranges larger than `max_materialized_cells`; a regression
  test uses a two-cell materialization limit across a four-row AutoFilter range.
- KDV exposes `SpreadsheetFilterCommand`, `SpreadsheetFilterEvent`, and
  `SpreadsheetFrameMetadata`. KUC receives only generic hidden row tracks.
- Filter-hidden rows are combined with authored hidden rows. Rebuilding the grid
  keeps frozen panes, scroll position, and selection, moving a newly hidden
  selection to the nearest visible original row.
- `rtk cargo test -p katana-document-viewer --all-targets --locked filter_ --
  --test-threads=1`: 9 passed.
- `rtk just ast-lint`: passed after responsibility splits.
