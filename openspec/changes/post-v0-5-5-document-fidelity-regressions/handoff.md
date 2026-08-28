# v0.5.6 Document Fidelity Regression Handoff

## Ownership

- KDV owns Office package preflight, isolated conversion and spreadsheet processes,
  neutral document artifacts, spreadsheet filter evaluation, bounded caches, and
  document-session resource cleanup.
- KUC owns only generic grid rendering and hidden-track behavior. It must not gain
  XLSX, AutoFilter, Office process, or KatanA host semantics.
- KatanA owns file-tree selection, menus, and native UI projection of KDV's typed
  commands, events, diagnostics, and frame metadata.
- KRR owns HTML layout and script/style evaluation. KDV may consume only its
  published registry release and does not duplicate HTML layout behavior.

## Verified Work

- Real non-seekable DOCX/XLSX/PPTX fixtures exercise data-descriptor local
  headers and central-directory metadata.
- Typed ZIP local-header validation accepts valid descriptor and small ZIP64
  packages while retaining CRC, duplicate-name, path, relationship, and resource
  checks.
- XLSX AutoFilter range, columns, supported value/blank criteria, and unsupported
  criteria diagnostics are parsed into neutral sheet artifacts for both model
  and streaming backends.
- Focused Office preflight and spreadsheet suites pass. The initial full
  `just check` reached the handoff policy gate and identified this missing file;
  it must be rerun after the remaining implementation.

## Execution

- The integrated validation and release path remains in the main task.
  delegation-exception: `直列のクリティカルパス` / file:
  `openspec/changes/post-v0-5-5-document-fidelity-regressions/tasks.md`

## Release Work

1. Candidate extraction, filter evaluation, typed commands/events, and grid
   visibility preservation are implemented and covered by contract tests.
2. DEBUG-only stage tracing, bounded caches, idempotent cleanup, and ten-cycle
   resource evidence are implemented.
3. The supplied XLSX/PPTX corpus is measured; the spreadsheet startup path uses
   the lightweight process while PPTX retains the isolated conversion boundary.
4. Publish KDV only after all release gates pass, then integrate its exact
   registry version into KatanA and rerun packaged acceptance.
