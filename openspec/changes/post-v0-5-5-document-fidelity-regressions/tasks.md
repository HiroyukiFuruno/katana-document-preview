## 1. Reproduction and ownership evidence

delegation-exception: `直列のクリティカルパス` / file:
`openspec/changes/post-v0-5-5-document-fidelity-regressions/handoff.md`

- [x] 1.1 Add a real data-descriptor DOCX/XLSX/PPTX fixture and record the exact local-header/central-directory behavior. delegation-exception: `直列のクリティカルパス`
- [x] 1.2 Run the supplied XLSX/PPTX corpus through KDV and record typed failures and stage-level first-frame timings. delegation-exception: `直列のクリティカルパス`
- [x] 1.3 Record child-process, cache-entry, artifact-byte, and process-count baselines for ten mixed-document open/frame/close cycles. delegation-exception: `直列のクリティカルパス`

## 2. Office ZIP compatibility

- [x] 2.1 Replace string-matched deferred-length handling with a typed local-header scan outcome. delegation-exception: `直列のクリティカルパス`
- [x] 2.2 Accept valid data-descriptor/ZIP64 packages only after central-directory, CRC, duplicate, path, relationship, and resource-limit validation. delegation-exception: `直列のクリティカルパス`
- [x] 2.3 Add regression tests proving valid variants pass and corrupt or unsafe variants remain rejected before process spawn. delegation-exception: `直列のクリティカルパス`

## 3. Interactive spreadsheet filtering

- [x] 3.1 Add neutral AutoFilter range, column, criterion, candidate, visibility, and diagnostic artifact types. delegation-exception: `直列のクリティカルパス`
- [x] 3.2 Parse bounded worksheet AutoFilter metadata for model and streaming spreadsheet backends. delegation-exception: `直列のクリティカルパス`
- [x] 3.3 Add isolated-process requests/responses for candidate extraction and filter evaluation without materializing the full sheet into host IPC. delegation-exception: `直列のクリティカルパス`
- [x] 3.4 Add KDV document commands/events and rebuild grid row visibility while preserving source row indices, selection, scroll, frozen panes, and authored hidden rows. delegation-exception: `直列のクリティカルパス`
- [x] 3.5 Add unit, protocol, session, and rendered-frame tests for apply, clear, unsupported criteria, and large-sheet limits. delegation-exception: `直列のクリティカルパス`

## 4. Performance and lifecycle

- [x] 4.1 Add `DEBUG=true`-only stage tracing for preflight, spawn, convert, decode, frame, cache, close, and drop. delegation-exception: `直列のクリティカルパス`
- [x] 4.2 Key immutable Office conversion artifacts by content, format, and process settings and prevent navigation, resize, or repeat-frame reconversion. delegation-exception: `直列のクリティカルパス`
- [x] 4.3 Bound page/grid/artifact caches by entries and bytes with deterministic eviction tests. delegation-exception: `直列のクリティカルパス`
- [x] 4.4 Make close/drop cleanup idempotent and prove process, workspace, frame, and cache live counts return to baseline. delegation-exception: `直列のクリティカルパス`
- [x] 4.5 Measure the supplied PPTX corpus, remove the dominant avoidable stage, and record the before/after first-frame result. delegation-exception: `直列のクリティカルパス`

## 5. Verification, dependency maintenance, and release

- [x] 5.1 Run focused ZIP, spreadsheet filter, performance, and lifecycle regression suites. delegation-exception: `直列のクリティカルパス`
- [ ] 5.2 Update compatible direct/transitive dependencies and lockfile, including exact public KRR `=0.4.19`, without lowering lint, coverage, security, score, or acceptance gates. delegation-exception: `直列のクリティカルパス` / Issue: <https://github.com/HiroyukiFuruno/katana-document-viewer/issues/44> / 証跡: `evidence/dependency-maintenance.md`.
<!-- subagent-spark-harness-strict-start -->
- [ ] 5.3 Run strict OpenSpec validation, `just check`, 100% coverage, boundary, scorecard, package, publish dry-run, and three-OS CI gates. delegation-exception: `直列のクリティカルパス`
- [ ] 5.4 Publish the KDV patch release and verify the GitHub Release and crates.io artifact. delegation-exception: `直列のクリティカルパス`
- [ ] 5.5 Update KatanA to the exact published registry version with no path/git override and rerun the supplied-file and ten-cycle packaged acceptance suites. delegation-exception: `直列のクリティカルパス`
