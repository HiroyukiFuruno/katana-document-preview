# Dependency maintenance

Checked on 2026-08-29 before the KDV patch release. The KRR update is tracked
by <https://github.com/HiroyukiFuruno/katana-document-viewer/issues/44>.

Rechecked on 2026-08-30 with `rtk proxy cargo outdated --workspace`.
It found no KDV-owned compatible direct dependency update. Its only reports
were `embedded-io` (a `postcard` compatibility feature), wasm-only
`getrandom`, and removed target-specific transitive packages; no lockfile
change is appropriate for those rows.

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
- 2026-09-02に`katana-ui-core` 0.3.3のcrates.io公開を確認し、公開KDV runtime
  dependencyをregistry-onlyの`0.3.3`へ更新した。lockfileにはregistry sourceと
  checksumを記録し、KDV公開crateにpath/git overrideを残さない。
- 同じ解決で互換な推移依存を`libredox` 0.1.23、`rust_decimal` 1.43.0、
  `smallvec` 1.16.0へ更新した。`cargo outdated --workspace`で確認できる
  KDV所有の他の互換direct updateはない。
- `katana-ui-core-storybook`はcrates.ioに公開されておらず、KUC 0.3.3の公開
  core APIにもKDV Storybookが必要とするpresentation/canvas/host APIはない。
  そのためGit tag `v0.3.0`は開発専用Storybook dependencyにだけ残し、公開KDV
  crate、package検査、registry consumerには含めない。これはKDV-local gate、
  Draft PR準備、CIを止める理由にしない。
- KUC 0.3.3にはセル四辺ごとのstyle/color border型・描画APIもない。Issue #48の
  KDV metadata保持とfidelity計測は完了済みで、thin projectionはその公開KUC APIを
  前提とする別の未完了項目として`tasks.md`に維持する。
