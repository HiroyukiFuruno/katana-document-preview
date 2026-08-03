<!-- KDV v0.5.0 deferred change -->

## ADDED Requirements

### Requirement: PDF pagination は固有JSON profileを必須入力にしなければならない

システムは、PDF改ページ制御を暗黙デフォルトだけで決めず、version付き `KdvPdfPaginationProfile` JSONを必須入力として受け取らなければならない（MUST）。

#### Scenario: pagination profile JSONを読み込む

- **WHEN** ホストが `KdvPdfPaginationProfile` JSONをPDF export requestへ渡す
- **THEN** KDVは `schema_version`、page設定、margin設定、font metric profile、heading break rule、keep-together rule、forced break markerを検証する
- **THEN** KDVは検証済みprofileから `PaginationPlan` を生成する
- **THEN** JSONの正規化結果をsnapshot testで比較できる

#### Scenario: pagination profile JSONが欠落または不正である

- **WHEN** PDF pagination exportにprofile JSONが渡されない、または未知の `schema_version` が渡される
- **THEN** KDVはPDF exportを開始しない
- **THEN** KDVは暗黙fallbackで用紙、余白、改ページルールを補完しない
- **THEN** diagnosticsに不正なfield名と理由を残す

### Requirement: PDF export はページ境界制御と事前viewer確認を提供しなければならない

システムは、Markdown を PDF export する際、`KdvPdfPaginationProfile` と `BuildGraph` から生成した `PaginationPlan` に従い、見出し・コードブロック・ダイアグラム・テーブルがページ途中で分断されないようにする改ページルールを実装し、export 前にページ分割viewer確認を提供しなければならない（MUST）。

#### Scenario: 改ページルールを適用する

- **WHEN** Markdown を PDF として export する
- **THEN** h1 / h2 見出し前の改ページ有無はpagination profile JSONに従う
- **THEN** コードブロック・ダイアグラム・テーブルのkeep-together挙動はpagination profile JSONに従う
- **THEN** 強制改ページ記法はpagination profile JSONで指定されたmarkerだけが尊重される
- **THEN** KDVはMarkdown本文を再parseして改ページルールを推測しない

#### Scenario: export 前にページ分割をviewerで確認する

- **WHEN** ユーザーが PDF export を実行する
- **THEN** ホストは保存ダイアログを開く前にページ分割viewerを表示できる
- **THEN** viewerと実際のPDF出力が同じ `PaginationPlan` を使う
- **THEN** KDVは保存ダイアログやファイル書き込みの副作用を直接実行しない

#### Scenario: PaginationPlanを検査する

- **WHEN** E2E fixtureからPDF pagination previewとPDF artifactを生成する
- **THEN** KDVは `*.pagination-plan.json` を出力してpage count、block order、page break reasonを比較できる
- **THEN** PDF artifactのpage countは `PaginationPlan` と一致する
- **THEN** heading、code block、diagram、tableの代表fixtureはページ途中で不自然に分断されない
