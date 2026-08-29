# Dependency maintenance

Checked on 2026-08-29 before the KDV patch release. The KRR update is tracked
by <https://github.com/HiroyukiFuruno/katana-document-viewer/issues/44>.

- `katana-render-runtime` was updated from 0.4.16 through 0.4.19 to the
  caret-compatible registry requirement `0.4.19` (not an exact pin); the
  lockfile contains the crates.io source and checksum.
- `epaint_default_fonts` was updated from 0.35.0 to 0.36.1. All 424 focused
  export-surface tests passed after the font dependency change.
- `office2pdf-katana` remains the latest published exact version, `=0.6.10`.
- The remaining direct dependency requirements are current compatible releases.
- The shared direct `v8` requirement was updated from 150.0.0 to 152.2.0 to
  match public KRR 0.4.19. The resolved graph MUST contain one V8 version so
  KDV does not link two runtimes or violate the shared runtime ABI boundary.
- The `embedded-io` 0.4.0 report is a transitive compatibility feature of
  `postcard` alongside 0.6.1, not a direct stale KDV dependency. The wasm-only
  `getrandom` report comes through `office2pdf-katana`/`umya-spreadsheet`.
