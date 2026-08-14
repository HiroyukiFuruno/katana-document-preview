# v0.5.0 Backend-neutral Multi-format Viewer Handoff

## Ownership

- KDV owns format detection, private engine adapters, bounded preflight,
  isolated processes, neutral artifacts, diagnostics, viewer state, and commands.
- KUC owns only generic page, grid, slide, viewport, and control contracts.
- KDV owns the backend-neutral document frame and uses KUC internally without
  exposing KUC types or depending on egui/eframe.
- KatanA owns file/URL intake, native host command handling, and the current
  application-backend projection. It depends directly on KDV only for document
  viewing and does not depend on KUC in this release.
- KRR remains unchanged. Existing public APIs may be used only indirectly for
  diagram or math content embedded in an approved document artifact.

## Verified Feasibility

- PDF: `hayro` 0.7.1, 85/100, all current hard gates passed.
- DOCX: official `office2pdf` 0.6.7 (including PR #745)
  -> canonical PDF -> `hayro`, 85/100, conditional on
  KDV bounded OOXML preflight and isolated process enforcement.
- XLSX: `IronCalc` 0.8.3 interactive grid, 90/100. Chart and pivot capability
  remains typed unsupported.
- PPTX: official `office2pdf` 0.6.7 static slide with typed chart fallback, 85/100.
- LibreOffice, ONLYOFFICE, docMentis, Aspose, Pandoc + Typst, `rwml`,
  SlideGlance, BetterOffice, and `office_oxide` failed one or more fixed hard
  gates and are not production fallbacks.

## Verified Gates

- `rtk just multi-format-scorecard-script-test` passes.
- `rtk just multi-format-scorecard-check` passes.
- `rtk just check` passes, including clippy, AST lint, KUC boundary,
  1,588 workspace tests, Storybook tests, scorecard, and handoff harness.
- `rtk just coverage` passes with `--fail-under-lines 100` and
  `--fail-uncovered-lines 0`; 1,587 tests passed and 1 was ignored.
- `rtk ./scripts/openspec validate v0-5-0-multi-format-viewer --strict --no-interactive`
  passes.
- Release mode correctly fails with
  the first selected candidate that has not yet passed every hard gate.
- PDF / DOCX `static-page`, XLSX `interactive-grid`, and PPTX
  `static-slide-with-typed-chart-fallback` were explicitly approved on
  2026-07-30.
- Score weights remain 30 / 20 / 20 / 10 / 10 / 10, minimum score remains 80,
  every hard gate remains mandatory, and threshold relaxation remains disabled.

## Delegation

- Feasibility integration and harness repair remain in the main task. delegation-exception: `ユーザーがsubagent利用を禁止` / file: `.codex/workflows/subagent-spark-policy.md` / file: `Justfile` / file: `scripts/subagent-spark-harness-change.sh` / file: `scripts/subagent-spark-harness-evidence.sh` / file: `scripts/subagent-spark-harness-diff.sh` / file: `scripts/check-subagent-spark-harness-change-tests.sh` / file: `scripts/check-subagent-spark-harness-coverage-tests.sh` / file: `scripts/check-subagent-spark-harness-diff-tests.sh`

## Remaining Preconditions

1. Implement the typed command/event/info/close correction approved on
   2026-08-09 and reject the previous silent-command behavior in contract tests.
2. Implement and score the approved KUC/KDV paths. A proposed profile is not a
   release pass until its representative corpus reaches 80 and every hard gate
   passes.
3. Pass strict 100% coverage, cross-platform distribution, security,
   package, and release gates.
4. Publish KDV 0.5.0 without an application UI backend dependency, integrate
   the registry version into KatanA, then complete native acceptance and the
   KatanA release.
