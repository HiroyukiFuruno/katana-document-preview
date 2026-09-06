# Dependency maintenance

## 2026-09-06 KUC v0.3.7 public adoption

- KUC `v0.3.7` is publicly available as tag
  `df8df4251584ab59aa33d778ea8863172514e402`, GitHub Release, and crates.io
  package. `cargo info katana-ui-core@0.3.7` downloaded the registry artifact
  and reported its `raster-host` feature.
- KDV now uses exact registry `katana-ui-core = "=0.3.7"` in the workspace
  and document-surface manifests. `cargo update -p katana-ui-core --precise
  0.3.7` recorded the crates.io source and checksum
  `9e8dfdba6ce3480c373dea39db73edd53215e651f6fa4637be4d455530fb7b3e`;
  no path or git override was introduced.
- The following final-source contract checks passed: release contract,
  document-surface boundary, release DoD self-test, V8 runtime singleton
  (`152.2.0`), dependency tests (4), Storybook score-gate contract tests
  (24), format, AST lint, and strict workspace clippy.
- This is dependency-maintenance evidence only. The KatanA-generated
  candidate reference, final strict release gate, three-OS PR checks, KDV
  publication, fresh public KDV consumer, and KatanA packaged acceptance
  remain distinct required gates.

## 2026-09-05 KUC v0.3.6 public adoption and dependency refresh

- KUC `v0.3.6` has a remote tag, a published GitHub Release, and a crates.io
  artifact. A fresh isolated consumer pinned to `katana-ui-core = "=0.3.6"`
  built both normally and with `--locked`; its lockfile resolves the registry
  source with the published checksum and contains no path/git override.
- KDV now pins `katana-ui-core = "=0.3.6"` in both the workspace and document
  surface manifests. The release-contract and document-surface boundary
  verifiers require this exact registry artifact and reject the previous
  `0.3.5` pin, Git/path overrides, and a second Storybook alias.
- `just update` refreshed compatible non-KUC lockfile entries: `cc` `1.4.5`,
  `find-msvc-tools` `0.1.12`, `js-sys`/`wasm-bindgen` `0.3.105`/`0.2.128`,
  `syn` `3.0.5`, `tinyvec` `1.13.2`, and `zstd-sys`
  `2.1.0+zstd.1.5.7`.
- The registry-resolved KUC Storybook smoke passed `143/30/54/121` tests.
  The locked graph resolves one `v8 152.2.0` shared by KDV and public KRR
  `0.4.19`; `cargo tree -d`, inverse-tree inspection, and the V8 singleton
  linker verifier were run without a duplicate V8 root.
- The KUC `0.3.5` fidelity capture remains historical evidence of the public
  per-side border API. KDV `0.5.6` must refresh the independent KatanA
  reference only after its own published registry artifact is adopted; this
  is not substituted by the local KDV check.

This evidence does not assert that the Draft PR review, three-OS CI, GitHub
Release, crates.io KDV artifact, fresh public KDV consumer, or KatanA
acceptance is complete. Those remain explicitly open in `tasks.md` (5.3,
5.4, 5.5, and 6.7).

## 2026-09-04 historical pre-v0.3.6 local preflight

The KUC publication boundary and the KDV local release preflight are complete.
This evidence does not assert that the Draft PR review, three-OS CI, GitHub
Release, crates.io artifact, fresh public consumer, or KatanA acceptance is
complete. Those remain explicitly open in `tasks.md` (5.3, 5.4, 5.5, and 6.7).

## 2026-09-04 v0.5.6 release-check監査

- 実行日時: `2026-09-04T23:19:24+0900`。実行元は`release/v0.5.6`の候補作業ツリー
  （基底commit `404b6e0aec0428cc6ded69b187d7368a3adb4108`へ、coverage回帰を追加した
  未commit差分）である。commit/push後のPR最終HEADでは、KatanA候補reference更新を含めて
  同じgateを再実行する。
- 実行コマンド: `rtk proxy just VERSION=v0.5.6 release-check`。
- 結果: exit `0`。release contract、`just check`、boundary、scorecard、
  data-descriptor DOCX worker、Office profiling、V8 singleton、subagent harnessを通過した。
- strict coverage: functions `3531/3531`、lines `29061/29061`（各`100%`）、
  uncovered functions/lines `0`。
- 隔離 package buildは`868 files`を検証し、`cargo publish --dry-run`は成功した。
  `assert-crates-not-published.sh v0.5.6`は未公開を確認した。
- このローカルpreflightは公開・下流採用の証明ではない。three-OS CI、Draft PRの
  fresh-head review、GitHub Release、crates.io artifact、公開registryのみを使うconsumer、
  KatanAのpackaged acceptanceはそれぞれ別の必須gateとして残る。

## 2026-09-05 v0.5.6 release-check再実行

- 実行日時: `2026-09-05T11:50:37+0900`。実行元は`release/v0.5.6`の候補作業ツリーで、
  Retina図表スクロール境界の回帰を追加した未commit差分である。
- 実行コマンド: `rtk proxy just VERSION=v0.5.6 release-check`。
- 結果: exit `0`。release contract、`just check`、boundary、scorecard、
  data-descriptor DOCX worker、Office profiling、V8 singleton、subagent harnessを通過した。
- strict coverage: functions `3532/3532`、lines `29064/29064`（各`100%`）、
  uncovered functions/lines `0`。
- 隔離 package buildは`868 files`、`14.9MiB`（圧縮後`7.3MiB`）を検証し、
  `cargo publish --dry-run`は成功した。`assert-crates-not-published.sh v0.5.6`は未公開を確認した。
- 容量不足で中断した先行runは、cleanかつ`origin/master`統合済みだった役割済みKUC
  worktreeと再生成可能なKDV build outputを安全に整理してから再実行した環境要因であり、
  source差分・閾値・検証範囲は変更していない。このローカルpreflightは公開・下流採用の
  証明ではない。three-OS CI、Draft PRのfresh-head review、GitHub Release、crates.io artifact、
  公開registryのみを使うconsumer、KatanAのpackaged acceptanceはそれぞれ別の必須gateとして残る。

### Historical failed attempt

The earlier 2026-09-04 attempt at `404b6e0` failed strict coverage at
functions `99.97%` and lines `99.90%`. It is superseded by the successful
local preflight above; the added regressions keep the required thresholds at
100% without lowering any gate.

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

## 2026-09-06 KatanA candidate-registry reference refresh

- KDV `0.5.6` was packaged from the clean release candidate and served as the
  only KDV package from a disposable loopback sparse registry. The package
  SHA-256 was `3196397aec631fc31524d70647c7fdc12479be16698f955a09deb48bdca07020`.
- A disposable KatanA clone resolved KDV `0.5.6` from that registry, KRR
  `0.4.19` and KUC `0.3.7` from crates.io, and exactly one V8 `152.2.0`; no
  KDV path, git, or patch override was used.
- Its real screenshot runner generated `sample-export.png` (1280x19067,
  SHA-256 `88a3342dadc95e46fa1db5e5c942017670e399d8199eca63b57ff73622675a1c`),
  `sample-full.png` (2560x4800,
  `3ec7ab6e530ec68acce6655c881af858f6bd3a36a5437be52b2a0e5c970849de`), and
  `sample-diagrams-full.png` (2560x4800,
  `11cced597dacbe2e641fd850b49d6744721ec2e3dfd3e41fed175994d63f4321`).
- The candidate's `sample_diagrams.md` frame did not draw the diagram bodies.
  Its dark crop was normalised with the established physical crop
  `2374x4450+64+202`, Box filtering, and 1280x2400 output only to diagnose
  the result; the existing tracked references were restored immediately. The
  objective KDV Storybook comparison was `3/95`, so neither the reference nor
  the threshold was changed. The focused score, a corrected final release
  gate, public registry consumer, and KatanA packaged acceptance remain
  required.

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
