# v0.5.6 owner-layer issue coverage

## #46: exact DOCX data descriptor

- KDV representative DOCX has the same SHA-256 as KatanA's source fixture:
  `ce8ec76f77935e63824331f783bbb73f086333c6b331dda5a6d01c1e5f69d0dc`.
- `scripts/feasibility/verify-data-descriptor-docx.py --write` produces the
  exact 40,336-byte fixture SHA-256
  `a1b7e22021218d314bc2d90c526d6d682981828b67cef6e61d8cb2a71ef5742a`.
- The verifier proves all 20 entries set bit 3 and have local CRC/size
  `0/0/0`; `word/document.xml` central-directory sizes are `1383/4907`.
- Focused preflight and isolated-worker frame regressions passed on 2026-08-29.

## #47: XLSX AutoFilter typed API

- Existing public `SpreadsheetFilterCommand`, `SpreadsheetFilterEvent`, and
  `SpreadsheetFrameMetadata` remain the KDV/KatanA boundary.
- The XLSX-derived fixture regression now covers string candidates, numeric
  `98`, blank values, multiple selected values, and Clear while keeping source
  row indices and visible-row state.

## #49: PPTX stage tracing

- `DEBUG=true` emits static Office `office.archive_intake`,
  `office.package_parse`, `office.transfer_to_worker`, `office.worker_spawn`,
  `office.runtime_init`, `office.conversion`, `office.parse_layout`,
  `office.transfer_from_worker`, `office.frame_publication`, and
  `office.raster` stages. XLSX additionally exposes the corresponding
  `spreadsheet.worker_spawn`, `spreadsheet.runtime_init`,
  `spreadsheet.package_parse`, and `spreadsheet.frame_publication` stages.
- `DEBUG=true` runs of the exact data-descriptor DOCX worker-frame regression
  and the unified DOCX/PPTX session regression observed the static-worker
  stages through `office.raster` and `office.frame_publication`. The unified
  XLSX session regression observed all four `spreadsheet.*` stages. The
  supplied PPTX corpus p50/p95 and RSS evidence remains an explicit pending
  task.
- The ten-cycle resource regression now opens, frames, closes, and drops the
  representative PPTX alongside PDF and XLSX, returning process, workspace,
  frame, and cache counts to the baseline after every cycle.
- KatanA's current #49 evidence reports a 7.7 KiB XLSX cold/warm differential:
  first open 3,820 ms / first frame 3,857 ms; warm opens 14–16 ms / frames
  42–45 ms; ten cycles released resources and steady RSS was +912 KiB. The new
  stage contract isolates the suspected spawn/runtime-init/package-parse and
  frame-publication contributions without treating this single observation as
  the required supplied-corpus p50/p95 result.

## #48: DOCX/XLSX objective fidelity

- The pinned external comparison oracle is LibreOffice 26.8.0.3
  (`bce0998afefdbc355585ca324285661a2170ba77`) with `pdftoppm` 26.05.0 at
  72 dpi. It is comparison-only: KDV does not add LibreOffice to its runtime
  dependency graph.
- `evidence/fidelity-reference.json` pins the exact DOCX/XLSX fixture SHA-256,
  source viewport, renderer command/version, and regression tolerances.
  `evidence/fidelity-baseline.json` is a real verified run, not a
  self-comparison: DOCX has two source/candidate pages with mean normalized
  MAE `0.0287398` and RMSE `0.11015745`; the record also retains each source
  PDF/page artifact hash, bbox, page, and native-size delta.
- XLSX has two worksheets with zero KDV data-metadata gaps for text, font,
  fill, border, merged cells, and row/column/worksheet geometry. KDV now
  preserves IronCalc border metadata through worker artifact and public frame.
  The record separately reports the honest remaining visual gaps:
  24 custom-border cells and two source PDF pages cannot be represented by the
  continuous KUC 0.3.3 grid surface. Those are bounded regression tolerances,
  not a claim of visual equivalence; `tasks.md` 6.8 will connect the preserved
  metadata when a published KUC version exposes custom cell borders.

## V8 consumer integrity

- The local singleton verifier passed for `v8 152.2.0` with KDV and KRR 0.4.19
  in the inverse graph and a public KDV API link test.
- The temporary registry consumer template has no path/git dependency. Its
  actual fresh-resolve build runs only after the exact KDV version is visible on
  crates.io.
