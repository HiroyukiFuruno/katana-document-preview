## Context

`v0.1.x` でPDF/PNG/JPGのnative surface exportは成立しているが、現状のpage planは固定の内部ルールに近い。`v0.6.0` では、KatanAが期待する用紙、余白、見出し前改ページ、keep-together、強制改ページ記法を明示した固有JSONを入力にし、viewer確認とPDF出力が同じ `PaginationPlan` を使うようにする。

画面上では、PDF保存前にページごとの区切り、分断回避、強制改ページの結果を確認するpreview modeが必要になる。保存ダイアログや実ファイル保存はKDVではなくホストが行う。

## Goals / Non-Goals

**Goals:**

- `KdvPdfPaginationProfile` JSON schemaを定義する。
- `BuildGraph` とpagination profile JSONから `PaginationPlan` を作る。
- KDVがKUCを内部利用して `PaginationPlan` を確認できるbackend-neutral preview modeを提供する。
- PDF artifactのpage countと `PaginationPlan` を一致させる。
- profile JSON欠落、未知version、不正fieldをFail Fastにする。

**Non-Goals:**

- KDVが保存ダイアログやファイル保存の副作用を持たない。
- Markdown本文を再parseして改ページ対象を推測しない。
- byte完全一致のPDF snapshotを合格条件にしない。
- Office/PDF外部viewerの汎用表示は `v0.4.0` で扱う。

## Decisions

### 固有JSONを改ページ契約の正本にする

`KdvPdfPaginationProfile` は少なくとも次の情報を持つ。

```json
{
  "schema_version": "kdv.pdf-pagination-profile.v1",
  "page": {
    "preset": "a4",
    "orientation": "portrait"
  },
  "margins_mm": {
    "top": 18,
    "right": 16,
    "bottom": 18,
    "left": 16
  },
  "render": {
    "font_metric_profile": "kuc-default-v1",
    "scale": "1.0"
  },
  "rules": {
    "heading_breaks": [
      { "level": 1, "before": "always" },
      { "level": 2, "before": "auto" }
    ],
    "keep_together": ["heading_with_next", "code_block", "diagram", "table"],
    "forced_break_markers": ["thematic_break"]
  }
}
```

KDVはこのJSONから正規化済みprofileを作り、unknown field、unknown version、必要field欠落をエラーにする。暗黙fallbackでページ設定を補完しない。

### PaginationPlanをviewerとPDFで共有する

`PaginationPlan` はpage size、margin、page index、block id、source range、break reasonを持つ。KDVのKUC-backed preview modeとPDF exportは同じ `PaginationPlan` を参照する。KUC型はKDV public APIへ露出せず、KatanAはKDV所有の中立frameだけを現在のapplication backendへ投影する。

### 既存surface page planを置き換え対象として扱う

既存の `export_surface::page_plan` は内部実装の出発点として使えるが、`v0.6.0` ではprofile JSONを受け取らない固定ルールのまま完了にしない。

## Risks / Trade-offs

- profile JSONの粒度が粗いと、KatanA側の期待とPDF artifactがずれる。
- font metric profileがKUC実描画とずれると、previewとPDFの改ページが一致しない。
- previewを見た目だけで確認すると不一致を見逃すため、`PaginationPlan` JSONとPDF page countを自動検査する。
