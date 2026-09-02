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
- [x] 4.4 Make close/drop cleanup idempotent and prove process, workspace, frame, and cache live counts return to baseline across PDF, XLSX, and PPTX ten-cycle sessions. delegation-exception: `直列のクリティカルパス`
- [x] 4.5 Measure the supplied PPTX corpus, remove the dominant avoidable stage, and record the before/after first-frame result. delegation-exception: `直列のクリティカルパス`

## 5. Verification, dependency maintenance, and release

- [x] 5.1 Run focused ZIP, spreadsheet filter, performance, and lifecycle regression suites. delegation-exception: `直列のクリティカルパス`
- [ ] 5.2 Update compatible direct/transitive dependencies and lockfile, including public KRR `0.4.19` with caret-compatible semver (not an exact pin), without lowering lint, coverage, security, score, or acceptance gates. delegation-exception: `直列のクリティカルパス` / Issue: <https://github.com/HiroyukiFuruno/katana-document-viewer/issues/44> / 証跡: `evidence/dependency-maintenance.md`.
  - [x] KRR and every non-KUC dependency are updated or rechecked against the registry; the lockfile updates compatible `libredox`、`rust_decimal`、`smallvec` and needs no further compatible update.
  - [x] Public KDV runtime uses registry-only `katana-ui-core` 0.3.3 with no path/git override.
  - [ ] Replace the remaining development-only KUC Storybook Git/tag dependency after the required Storybook API is published to the registry. KUC `v0.3.3` Git Storybook is not adopted because its new `eframe`/`egui` graph fails the neutral Storybook boundary gate. KUC owner issue: <https://github.com/HiroyukiFuruno/katana-ui-core/issues/35>. KatanA resolves KUC `0.3.3` while the KDV Storybook still resolves `0.3.0`, so the KRR `0.4.19` diagram-reference crop remains at `88/95` until that neutral public boundary is available.
- [/] FB-2026-08-29-001: 全体のKUC blockerでも、KDV内で独立して進められる品質ゲート、Draft PR、CI準備を停止しない。後続の状況確認はこの実装指示を置換しないため、報告前に未完了項目を実行可能・外部入力待ち・公開待ちへ分け、実行可能な項目を継続する。グローバルポリシーの表記差は`reasoning`の`medium`基準を意味で検証し、回帰テストで維持する。delegation-exception: `直列のクリティカルパス` / 証跡: file: `scripts/check-subagent-spark-harness-policy-tests.sh`.
<!-- subagent-spark-harness-strict-start -->
- [ ] 5.3 Run strict OpenSpec validation, `just check`, 100% coverage, boundary, scorecard, package, publish dry-run, and three-OS CI gates. delegation-exception: `直列のクリティカルパス`
  - [x] strict OpenSpec validation、`just release-check`（`just check`、100% line/function coverage、boundary、scorecard、package、publish dry-run を含む）を 2026-08-31 に通過。証跡: `just release-check`。delegation-exception: `直列のクリティカルパス`
  - [ ] three-OS CI は、同一差分を commit/push した Draft PR 上で実行する公開 gate。
- [ ] 5.4 Publish the KDV patch release and verify the GitHub Release and crates.io artifact. delegation-exception: `直列のクリティカルパス`
- [ ] 5.5 Update KatanA to the exact published registry version with no path/git override and rerun the supplied-file and ten-cycle packaged acceptance suites. delegation-exception: `直列のクリティカルパス`

## 6. v0.5.6 issue coverage added after owner-layer audit

- [x] 6.1 Issue #46: add the exact KatanA data-descriptor DOCX SHA-256 fixture, verify all 20 local headers defer CRC/sizes, and prove the isolated worker generates a frame without lowering archive limits. delegation-exception: `直列のクリティカルパス` / Issue: <https://github.com/HiroyukiFuruno/katana-document-viewer/issues/46>.
  証跡: `evidence/issue-coverage.md`.
- [x] 6.2 Issue #47: cover typed string, numeric, blank, multiple-value, and clear AutoFilter behavior from a real XLSX-derived fixture while preserving grid frame visibility. delegation-exception: `直列のクリティカルパス` / Issue: <https://github.com/HiroyukiFuruno/katana-document-viewer/issues/47>.
  証跡: `evidence/issue-coverage.md`.
- [x] 6.3 Issue #49: normalize DEBUG-only first-frame trace stages for archive intake, worker spawn, runtime init, package parse, transfer, conversion, parse/layout, frame publication, and raster across static Office and XLSX workers without changing limits or cache lifecycle. delegation-exception: `直列のクリティカルパス` / Issue: <https://github.com/HiroyukiFuruno/katana-document-viewer/issues/49> / 証跡: file: `evidence/issue-coverage.md`.
  - [x] Windows AppContainer経路でも `office.worker_spawn` をDEBUG-onlyで計測し、profiling stage verifierにplatform別のsource契約を追加する。delegation-exception: `直列のクリティカルパス` / 証跡: `scripts/feasibility/verify-office-profiling-stages.py`.
- [x] 6.4 Issue #49: run the supplied PPTX corpus through repeated cold first-frame measurement and record p50/p95, RSS delta, dominant stage, unchanged-source reuse, and ten-cycle cleanup evidence. delegation-exception: `直列のクリティカルパス` / Issue: <https://github.com/HiroyukiFuruno/katana-document-viewer/issues/49> / 証跡: file: `evidence/pptx-performance.md`.
  - [x] Add a cold-process PPTX measurement harness that requires stage traces and records per-run/p50/p95/RSS evidence without substituting a generated fixture. delegation-exception: `直列のクリティカルパス`
  - [x] Execute the harness against the three supplied PPTX files after their source paths or attachments are available. delegation-exception: `直列のクリティカルパス`
- [x] 6.5 Issue #48: LibreOffice 26.8.0.3 / 72 dpi source renderer、fixture hash、viewport、DOCX/XLSX baseline/candidate scoreを固定し、KDVのXLSX border metadataをworkerから公開frameまで保持する。KatanA固有補正は行わない。delegation-exception: `直列のクリティカルパス` / Issue: <https://github.com/HiroyukiFuruno/katana-document-viewer/issues/48> / 証跡: file: `evidence/fidelity-reference.json`, `evidence/fidelity-baseline.json`, `evidence/issue-coverage.md`.
- [ ] 6.8 Issue #48: 公開KUCがセル四辺ごとのstyle/color border型と描画を提供した後、KDVの保持済みborder metadataをthin projectionへ接続し、`border_visual_missing_count`を0へ改善して同一fidelity harnessを再実行する。KDV内のmetadata保持・score取得は完了済み。delegation-exception: `直列のクリティカルパス` / Issue: <https://github.com/HiroyukiFuruno/katana-document-viewer/issues/48> / KUC registry prerequisite: published per-side custom cell border API.
- [x] 6.6 Enforce the KRR 0.4.19 / KDV V8 singleton locally with locked manifest/lockfile checks, `cargo tree -d`, inverse-tree ownership, and a public consumer link test. delegation-exception: `直列のクリティカルパス` / Issue: <https://github.com/HiroyukiFuruno/katana-document-viewer/issues/44> / 証跡: file: `evidence/dependency-maintenance.md`.
- [ ] 6.7 After publishing, build and link a fresh crates.io-only consumer of the exact KDV version; it must resolve KDV and V8 from registry without path/git overrides or duplicate V8. delegation-exception: `直列のクリティカルパス` / Issue: <https://github.com/HiroyukiFuruno/katana-document-viewer/issues/44> / 証跡: file: `evidence/issue-coverage.md`.
  - [x] Consumer verifierはroot以外のpath/git sourceをすべて拒否し、公開後build前にregistryのexact KDV/KRR/V8 packageを必須化する。delegation-exception: `直列のクリティカルパス` / 証跡: `scripts/release/verify-registry-consumer-link.py --self-test`.
- [x] FB-2026-08-29-002: KatanAの7.7 KiB XLSX cold/warm差（初回open 3,820 ms / frame 3,857 ms、warm open 14–16 ms / frame 42–45 ms、steady RSS +912 KiB）を#49のstage profilingへ反映し、既存v0.5.6 gateを下げない。delegation-exception: `直列のクリティカルパス` / Issue: <https://github.com/HiroyukiFuruno/katana-document-viewer/issues/49#issuecomment-5461739569> / 証跡: file: `evidence/pptx-performance.md`.
