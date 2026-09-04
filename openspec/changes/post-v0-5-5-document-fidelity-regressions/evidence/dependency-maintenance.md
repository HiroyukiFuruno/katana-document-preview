# Dependency maintenance

## 2026-09-04 completion evidence before the release gates

The KUC publication boundary is complete, but this evidence does not assert
that KDV `just check`, coverage, PR, release, or KatanA acceptance is complete.
Those remain explicitly open in `tasks.md` (5.3, 5.4, 5.5, and 6.7).

- KUC `0.3.5` is public at its tag, GitHub Release, and crates.io artifact.
  KDV resolves the exact registry package and has no path/git override.
- `just update` was executed after KUC publication. `toml` `1.1.5` was
  adopted. `tinyvec` remains `1.12.0` because `1.13.0` does not resolve its
  `vec` macro on the current toolchain. `generic-array` remains `0.14.7`
  because `crypto-common` `0.1.7` requires `generic-array =0.14.7`, making the
  newer compatible-looking candidate non-resolvable in this graph.
- The registry Storybook smoke completed successfully with `142/27/54/116`
  tests. This is the KDV registry-resolved consumer smoke, not a sibling KUC
  checkout test.
- The fidelity record verifies `custom border=true` and
  `border_visual_missing_count=0` using the same harness and the KUC `0.3.5`
  per-side border projection.
- The V8 singleton and consumer link checks passed: the locked graph resolves
  one V8 runtime, and the consumer link test resolves the registry packages
  without duplicate V8 or a path/git override.

Reproducible evidence commands:

```text
rtk proxy just update
rtk proxy just storybook-kuc-smoke
rtk proxy python3 scripts/release/verify-v8-runtime-singleton.py
rtk proxy cargo test -p katana-document-viewer --test v8_runtime_link_contract --locked
rtk proxy env PATH=/Applications/LibreOffice.app/Contents/MacOS:$PATH python3 scripts/feasibility/measure-office-fidelity.py --verify-record openspec/changes/post-v0-5-5-document-fidelity-regressions/evidence/fidelity-baseline.json
```

These results complete the dependency-maintenance and border-projection
subitems only. The release gate, public KDV artifact, and downstream KatanA
acceptance still require their own later evidence.

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
- **Historical (superseded on 2026-09-04):** 2026-09-02に
  `katana-ui-core` 0.3.3のcrates.io公開を確認し、公開KDV runtime dependencyを
  registry-onlyの`0.3.3`へ更新した。この中間状態はその後のKUC 0.3.5公開で置換され、
  現行のKDV公開候補には`0.3.5`以外のKUC version、path、git overrideを残さない。
- 同じ解決で互換な推移依存を`libredox` 0.1.23、`rust_decimal` 1.43.0、
  `smallvec` 1.16.0へ更新した。`cargo outdated --workspace`で確認できる
  KDV所有の他の互換direct updateはない。
- **Historical (superseded on 2026-09-04):** `katana-ui-core-storybook`はKUC
  v0.3.3で`publish = false`であり、KUC 0.3.3公開core APIにもKDV Storybookが必要とする
  presentation/canvas/host APIはなかった。そのためv0.3.0のGit Storybook packageを
  開発専用に残していたが、KUC 0.3.5のpublic `raster-host` APIに置換済みである。
  現行KDV候補のmanifest/lockfile/consumerには`katana-ui-core-storybook`、Git tag、
  またはKUCのpath overrideを残さない。
- **Historical (superseded on 2026-09-04):** 2026-09-02のKatanA candidate
  `sample_diagrams.md` cropは、KUC 0.3.3 runtimeとGit Storybook/core 0.3.0の混在で
  `88/95`だった。これはKUC 0.3.5 public boundary前の診断であり、現行のKDV v0.5.6
  release evidenceではない。KDV v0.5.6のregistry採用後にKatanAから独立生成する
  reference artifactとacceptanceを、公開後DoDとして別途実施する。
- **Historical (superseded on 2026-09-04):** KUC 0.3.3にはセル四辺ごとの
  style/color border型・描画APIがなかった。KUC 0.3.5の公開APIでthin projectionと
  fidelity計測が完了し、現行recordは`border_visual_missing_count=0`である。
