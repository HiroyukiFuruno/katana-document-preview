# Tasks: katana-document-viewer v0.6.0 PDF export pagination

## Definition of Ready (DoR)

- [ ] `v0.1.x` のrender/export foundationがarchive済みである
- [ ] `v0.2.0` のneutral viewer contractとKUC viewer modeが利用可能である
- [ ] KatanA側が渡すpagination profile JSONの必須fieldが合意済みである
- [ ] KUC font metric profileをPDF paginationへ使える境界が確認できている

## Definition of Done (DoD)

- [ ] `rtk ./scripts/openspec validate v0-6-0-pdf-export-pagination --strict --no-interactive` が通る
- [ ] `rtk just check` が通る
- [ ] 固有JSON fixture、正規化済みprofile snapshot、`PaginationPlan` snapshot、PDF page count検査が通る
- [ ] profile JSON欠落、不正version、未知field、必須field欠落の失敗テストが通る
- [ ] viewer previewとPDF exportが同じ `PaginationPlan` を使うことを自動テストで確認している

---

## 1. pagination profile JSON契約を定義する

- [ ] 1.1 `KdvPdfPaginationProfile` とschema versionを定義する
- [ ] 1.2 page preset、orientation、margins、font metric profile、scale、heading break、keep-together、forced break markerを必須契約にする
- [ ] 1.3 固有fixture JSONを追加する
- [ ] 1.4 JSON正規化snapshotを追加する
- [ ] 1.5 profile欠落、不正version、未知field、必須field欠落をFail Fastにする

---

## 2. PaginationPlanを生成する

- [ ] 2.1 `BuildGraph` と `KdvPdfPaginationProfile` から `PaginationPlan` を生成する
- [ ] 2.2 `PaginationPlan` にpage index、block id、source range、break reasonを含める
- [ ] 2.3 Markdown本文を再parseして改ページ対象を推測していないことをテストする
- [ ] 2.4 h1 / h2 heading break ruleをprofile JSONから適用する
- [ ] 2.5 code block、diagram、table、heading-with-nextのkeep-togetherをprofile JSONから適用する
- [ ] 2.6 forced break markerをprofile JSONから適用する

---

## 3. KUC pagination preview modeを実装する

- [ ] 3.1 `PaginationPlan` をviewer inputへ追加する
- [ ] 3.2 page boundary、break reason、block source rangeをpreviewで確認できるようにする
- [ ] 3.3 preview操作は保存ダイアログやファイル保存を直接実行せず、host commandを返す
- [ ] 3.4 previewとPDF exportが同じ `PaginationPlan` を参照することをテストする

---

## 4. PDF exportへ接続する

- [ ] 4.1 PDF export requestにpagination profile JSONを渡す入口を追加する
- [ ] 4.2 `PaginationPlan` に従ってPDF pageを生成する
- [ ] 4.3 PDF page countと `PaginationPlan` のpage countを検査する
- [ ] 4.4 heading、code block、diagram、tableの代表fixtureで分断回避を検査する
- [ ] 4.5 `ExportQualityGate` のPDF signature、page tree、page image、link annotation検査を維持する

---

## 5. Final Verification

- [ ] 5.1 `rtk just check` を実行する
- [ ] 5.2 `rtk ./scripts/openspec validate v0-6-0-pdf-export-pagination --strict --no-interactive` を実行する
- [ ] 5.3 生成したPDF artifactと `*.pagination-plan.json` の絶対パスを提示し、必要な場合はユーザー目視確認を依頼する
