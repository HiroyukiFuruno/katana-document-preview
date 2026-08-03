# katana-document-viewer OpenSpec

## Project

`katana-document-preview` は未リリース・未取り込みのため、計画上は `katana-document-viewer`（KDV）へ改名する。

KDVは、KMM公開データ型（public DTO）を入力にしたMarkdown viewer、hit-test、node選択、HTML/PDF/PNG/JPG書き出し（export）を担うlibraryである。crates.io package 名は `katana-document-viewer` とし、KatanA はこれを dependency として consume する。

## Design Principles

- `katana-document-viewer` packageはartifact、viewer state、document surfaceを所有する。
- optional `egui` featureだけがKUCとeguiを内部利用し、KUC型をpublic APIへ露出しない。
- KatanAはKDVだけを直接dependencyに取り、KUCへ直接依存しない。
- viewer表示とHTML/PDF/PNG/JPG書き出し（export）は同じ描画手順（render pipeline）を使う。
- Mermaid / Draw.io / PlantUML / ZenUML互換入力 / 数式（math）のSVG生成はKRR（katana-render-runtime）を正本にする。対応backendがない場合はraw sourceとdiagnosticsを保持し、HTML/PDF/PNG/JPGで同じSVG契約を使う。
- KDVはeditor-viewer同期制御を持たない。同期制御はKatanAが担い、KatanAがviewerまたはeditorへ命令する。

## Versioning

- `v0.1.x`: KDV改名、KMM model input、文書成果物（artifact）/ forge / export のneutral契約、描画評価の自動検証基盤、KRRへ委譲する窓口（facade）の確立。KUC完成を待たずに進める。
- `v0.2.x`: KUC上のMarkdown viewer実装、hit-test、目次（TOC）、hover、選択、画像・図形操作など画面操作を伴う機能。
- `v0.3.x`: PDF書き出し（export）の改ページ制御と、KUC viewer上での事前確認。
- `v0.4.x`: PDF / CSV / Office / SVG などMarkdown以外のviewer拡張。

## Consumers

- [KatanA](https://github.com/HiroyukiFuruno/KatanA) — git tag pinned（v0.26.0 で取り込み）

---

## UI core移行方針（egui → KUC）

このセクションはエコシステム全体で共通の方針。詳細は [KatanA openspec/project.md](https://github.com/HiroyukiFuruno/KatanA/blob/master/openspec/project.md) を正とする。

### 技術選定（確定）

| 層 | 採用 |
|----|------|
| UI core | **katana-ui-core (KUC)** |
| 文字描画 | KUCのfont契約に従う |
| 2D レンダリング | KUCのrendering契約に従う |
| レイアウト | KUCのlayout契約に従う |
| アーキテクチャ参考 | KUCのstyle / theme / font / state契約 |

React / TypeScript / WebView は使用しない。KDVはKUC契約を消費する。

### eguiから脱却する理由（要約）

- カラー絵文字：epaint が SBIX/CBTF 非対応 → cosmic-text で解決
- IME 不完全：egui TextEdit の composition が壊れる → cosmic-text + winit で解決
- レイアウト拡張不可：vendor パッチなしに行間・マージンを変えられない → vello Scene への直接描画で解決
- immediate mode の再描画コスト → vello の retained 描画で解決

### このrepoの責務

`katana-document-viewer`のoptional `egui` host featureがKUCを内部利用する。
feature無効時のengine/export APIはKUCとeguiへ依存しない。KatanAはKDVのdocument
surfaceを表示するだけで、KUC型、grid action、presentation変換を所有しない。

### katana-document-viewer の移行

```
KatanA -> katana-document-viewer/egui -> katana-ui-core
```

viewer はKUCのstyle / theme / font / state契約に従う。PDF / 画像 / 図表もKDVのartifact/export契約からKUC表示へ接続する。

---

## 先行着手の切り分け

KUC完成前に進める作業は、画面部品の見た目や操作に依存しないKDV内部の契約と自動検証に限定する。

- 進める: `DocumentSource`、`DocumentSnapshot`、`Artifact*`、`BuildRequest`、`ExportRequest`、`ExportOutput`、描画評価の検証用入力（fixture）、KRR委譲境界、KMM DTO coverage gapのdiagnostics、コマンド実行口（CLI）から呼べるAPI。
- 待つ: KUC部品に乗るtoolbar、scrollbar、split pane、hover overlay、TOC panel、PDF事前確認viewerなどの画面操作。

## KMM構想での扱い

KMM構想ではP3として、P0 `katana-ast-lint`、P1 `katana-markdown-model`、P2 `katana-ui-core` の境界を受けて、KMM文書モデルをKUCで表示し、同じpipelineでexportする。

- KMM文書モデルを再実装しない。CommonMark / GFMの全記法はKDV fixture matrixで棚卸しするが、KMM v0で未構造化のものはKDVが独自parseせず、raw sourceとdiagnosticsへ保持する。
- parser内部型やrenderer内部型をviewer stateへ漏らさない。
- KUC表示を前提にする。
- unresolved metadataを画面上で確認できる入口を持つ。
- HTML/PDF/PNG/JPG exportを担う。
- editor-viewer同期制御は持たない。
- 共通AST lintを品質ゲートにする。
