## Why

Markdown を PDF export する際、現在はページ境界の制御がなく、見出し・コードブロック・ダイアグラムがページ途中で分断される。印刷・配布用途では読みやすいページ分割が必要。

また、用紙サイズ、余白、見出し前改ページ、keep-together、強制改ページ記法を暗黙デフォルトだけで実装すると、KatanAが期待するページ分割と高確率でずれる。`v0.5.0` は固有の pagination profile JSON を入力契約にし、同じJSONからviewer確認とPDF出力を作る。

## What Changes

- version付き `KdvPdfPaginationProfile` JSON schemaを定義する。
- `KdvPdfPaginationProfile` JSONを必須入力として読み、欠落や未知versionはexport開始前にエラーにする。
- KMM由来の `BuildGraph` と pagination profile JSON から `PaginationPlan` を生成する。
- Markdown の AST を再parseせず、ページ境界での分断を避けるための改ページルールを実装する：
  - 見出し（h1 / h2）の前に改ページを挿入する（設定可能）
  - コードブロック・ダイアグラム・テーブルが分断される場合に前ページに留めるか次ページへ送る
  - 強制改ページ記法をJSONで指定する
- PDF export 前にページ分割viewerで同じ `PaginationPlan` を確認できるようにする（KDV内部のviewer/export共通pipelineを使う）
- `katana-document-viewer-kuc` にページ分割viewer modeを追加する
- export 設定（用紙サイズ・余白・改ページルール）を `KdvPdfPaginationProfile` として注入できる設計にする

## Capabilities

### New Capabilities

- `pdf-export-pagination`: ページ境界制御付きPDF exportと事前viewer確認

### Modified Capabilities

- `render-export-foundation`: PDF export requestがpagination profile JSONと `PaginationPlan` を扱えるようにする
- `markdown-viewer-kuc-integration`: KUC viewerでPDF pagination preview modeを扱えるようにする

## Impact

- DoR: `v0.1.0` のrender/export foundationと `v0.2.0` のKUC viewer integrationが完了していること
- `crates/katana-document-viewer/` — `KdvPdfPaginationProfile`、JSON schema、`PaginationPlan`、PDF export接続の追加
- `crates/katana-document-viewer-kuc/` — pagination viewer modeの追加
- KatanA — pagination profile JSONを渡し、保存前確認と実際の保存を制御する
