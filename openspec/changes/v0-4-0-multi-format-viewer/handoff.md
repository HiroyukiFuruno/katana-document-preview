# v0.4.1 Multi-format Viewer Handoff

## Ownership

- KDV owns format detection, private engine adapters, bounded preflight,
  isolated processes, neutral artifacts, diagnostics, viewer state, and commands.
- KUC owns only generic page, grid, slide, viewport, and control contracts.
- KDV owns the document surface and uses KUC internally without exposing KUC types.
- KatanA owns file/URL intake and native host command handling, depends directly
  on KDV only for document viewing, and does not depend on KUC.
- KRR remains unchanged. Existing public APIs may be used only indirectly for
  diagram or math content embedded in an approved document artifact.

## Verified Feasibility

- PDF: `hayro` 0.7.1, 85/100, all current hard gates passed.
- DOCX: `office2pdf` 0.6.5 -> canonical PDF -> `hayro`, 85/100, conditional on
  KDV bounded OOXML preflight and isolated process enforcement.
- XLSX: `IronCalc` 0.8.3 interactive grid, 90/100. Chart and pivot capability
  remains typed unsupported.
- PPTX: `office2pdf` static slide with typed chart fallback, 85/100.
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
- `rtk ./scripts/openspec validate v0-4-0-multi-format-viewer --strict --no-interactive`
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

1. Implement and score the approved KUC/KDV paths. A proposed profile is not a
   release pass until its representative corpus reaches 80 and every hard gate
   passes.
2. Pass strict 100% coverage, cross-platform distribution, security,
   package, and release gates.
3. Publish KDV 0.4.1 with the `egui` document-surface feature, integrate the registry
   version into KatanA, then complete native acceptance and the KatanA release.
