## Purpose

KDV v0.1.0 のUI非依存render/export foundation、KMM/KRR境界、HTML/PDF/PNG/JPG export品質契約を定義する。

## Requirements

### Requirement: KDVはapplication UI backend非依存の文書成果物契約を提供しなければならない

システムは、汎用UI FrameworkであるKUCを内部利用しつつ、egui / eframeなどのapplication UI backendに依存しない `katana-document-viewer` crateで、文書入力、文書snapshot、成果物（artifact）、診断を表すKDV所有型を提供しなければならない（MUST）。

#### Scenario: KMM documentをsnapshot化する

- **WHEN** ホストがKMM公開データ型（public DTO）をKDVへ渡す
- **THEN** KDVは `DocumentSource` と `DocumentSnapshot` として保持する
- **THEN** KMM内部parser型やrenderer内部型をKDV public APIへ露出しない
- **THEN** source revisionとdocument idを持ち、後続の評価と書き出し（export）で同じsnapshotを参照できる

#### Scenario: 成果物manifestを返す

- **WHEN** KDVがpreview / HTML / PDF / PNG / JPEG のいずれかを生成する
- **THEN** KDVは `ArtifactManifest` を返す
- **THEN** manifestはartifact id、format、source revision、diagnostics、生成backendを含む
- **THEN** manifestはsnapshot testで比較できる安定した構造を持つ

### Requirement: KDV forge はapplication UI backendに依存してはならない

システムは、`katana-document-viewer::forge` をUIなしでcompileできるようにしなければならない（MUST）。

#### Scenario: no-UI dependency guardを実行する

- **WHEN** `cargo tree -p katana-document-viewer` を実行する
- **THEN** `katana-ui-core` はKDVの内部UI Framework依存として含まれてよい
- **THEN** `egui`、`eframe`、`winit`、`vello` は含まれない
- **THEN** KUC型はKDV public APIへ露出しない
- **THEN** KRRへは公開契約またはadapter境界だけで接続する

#### Scenario: forge pipelineを構築する

- **WHEN** ホストが `BuildRequest` を渡す
- **THEN** KDVは `BuildGraph` を作成する
- **THEN** `BuildGraph` は書き出し（export）と描画評価で共有される
- **THEN** KDVはMarkdown本文を独自に再parseして別の正本を作らない

### Requirement: Markdown描画評価を自動テストで検証できなければならない

システムは、KUCの実画面に依存せず、Markdown描画評価を中間成果物で検証できなければならない（MUST）。

#### Scenario: CommonMark / GFM / KatanA互換fixtureを評価する

- **WHEN** CommonMark、GFM、KatanA互換の検証用入力（fixture）をKDVへ渡す
- **THEN** KDVは同じ `DocumentSnapshot` から評価用artifactを生成する
- **THEN** fixtureごとに期待するartifact manifestとdiagnosticsを比較できる
- **THEN** spec Scenario名または対応表から、どの要求を検証しているか追跡できる

#### Scenario: Markdown標準の全記法をKMM入力として評価する

- **WHEN** KMMがCommonMarkのブロック要素とインライン要素を含む公開データ型（public DTO）を渡す
- **THEN** KDVはKMM DTOを正本として `BuildGraph` を作る
- **THEN** KDVはMarkdown本文を独自に再parseしない
- **THEN** Markdown標準の必須記法は、raw sourceとdiagnosticsだけで完了扱いにしてはならない
- **THEN** KDVはCommonMark / GFMのfixture matrixに、KMM DTO化済み、未実装、外部backend待ちのどれに該当するかを記録する

#### Scenario: KMM coverage gapを補完parseしない

- **WHEN** KMM v0がCommonMark / GFMの一部記法を専用DTOへ構造化していない
- **THEN** KDVはその記法を独自parserで補完しない
- **THEN** KDVはcoverage gapを未完了の検証項目として記録する
- **THEN** 後続のKMM改善でDTOが追加された場合、同じfixture idで期待manifestを合格条件へ更新できる

#### Scenario: 数式とGitHub alertを評価対象に含める

- **WHEN** KMM documentにinline math、fenced math、`$$` 内側の半角スペースを許容するKatanA互換math、GitHub alertが含まれる
- **THEN** KDVはGitHub alertをKMM DTOの評価対象として扱う
- **THEN** KDVはmathを評価対象として扱い、raw sourceとdiagnosticsだけで完了扱いにしてはならない
- **THEN** KDVはmathとalertの評価結果をartifact manifestとdiagnosticsへ保持する
- **THEN** HTML/PDF/PNG/JPG書き出し（export）はその評価結果を再利用する

#### Scenario: KatanA独自解釈を評価対象に含める

- **WHEN** KMM documentに中央寄せHTML、badge row、legacy note、description list、`[-]` / `[/]` task marker、Draw.io直接code block、`.drawio` / `.xml` 参照、ZenUML、日本語、HTML entityが含まれる
- **THEN** KDVはKatanA互換fixtureとして評価する
- **THEN** KDVはKMM DTOとKRR結果からartifact manifestを生成する
- **THEN** ZenUMLはMermaid fence内の `zenuml` contentとして扱い、KDVはKMMにZenUML専用enumを要求しない
- **THEN** KDVはKatanA独自解釈をexport形式ごとに別解釈しない

#### Scenario: 外部描画失敗時にsourceを失わない

- **WHEN** Mermaid、Draw.io、ZenUML、PlantUML、mathの外部描画またはKRR境界のSVG生成が失敗する
- **THEN** KDVは元sourceをdiagnosticsとartifact manifestに保持する
- **THEN** KDVは独自rendererへfallbackしない
- **THEN** 後続viewerはraw表示に必要な情報をartifactから取り出せる

### Requirement: export は同じ中間成果物から生成されなければならない

システムは、HTML/PDF/PNG/JPG書き出し（export）を、viewer表示と同じ中間成果物から生成できる契約にしなければならない（MUST）。

#### Scenario: export requestを処理する

- **WHEN** ホストが complete theme object を含む `ExportRequest` と `ExportFormat` を渡す
- **THEN** KDVは `BuildGraph` から `ExportOutput` を生成する
- **THEN** formatごとの `ArtifactManifest` を返す
- **THEN** export artifact のbytesは0 byteであってはならない
- **THEN** export処理はKMM DTOを独自に再parseしない
- **THEN** HTML exportはcomplete theme objectからCSS変数を生成する

#### Scenario: HTML/PDF/PNG/JPGの互換性を維持する

- **WHEN** 同じ `BuildGraph` からHTML/PDF/PNG/JPGを生成する
- **THEN** KDVはHTMLを先に検査基準として固定し、HTMLが未合格の状態でPDF/PNG/JPGを完了扱いしてはならない
- **THEN** HTMLの合格判定は自動テストだけで確定せず、利用者が実artifactを確認して明示OKを出すまで合格扱いしてはならない
- **THEN** Markdown本文、数式、図形、table、code block、GitHub alert、KatanA独自解釈は形式間で同じ意味と見た目を保持する
- **THEN** PDF/PNG/JPGはHTMLで成立した評価済みsemanticsをsurfaceへ投影し、形式ごとにMarkdown ASTを再解釈してはならない
- **THEN** HTMLだけはaccordionの開閉操作を保持できる
- **THEN** PDF/PNG/JPGはHTMLの見え方を正とし、accordion開閉操作とリンククリック可否を除くすべての見た目をHTMLと同等にする
- **THEN** PDF/PNG/JPGではaccordion本文を開いた状態の静的内容として描画する
- **THEN** PDFではリンク注釈を保持し、PDF viewerから外部URLまたは文書内対象へ遷移できる
- **THEN** PNG/JPGではリンク注釈を保持しなくてよいが、リンク文字列の色、下線、本文位置はHTML/PDFと互換でなければならない

#### Scenario: export debug出力を利用者指定ディレクトリに保存する

- **WHEN** 利用側が入力Markdownと出力先ディレクトリを指定する
- **THEN** KDVは出力先ディレクトリ配下にHTML/PDF/PNG/JPGファイルを生成する
- **THEN** 生成後に各ファイルの容量が0 byteではないことを検証する
- **THEN** `*.manifest.toml` のような形式別sidecarファイルを成果物フォルダへ残さない

#### Scenario: 4形式のexport成果物をscore gateで検証する

- **WHEN** E2E fixtureからHTML/PDF/PNG/JPGの4形式を書き出す
- **THEN** KDVは `ExportQualityGate` で4形式の実ファイルを評価する
- **THEN** HTMLはKDV root、stylesheet、inline Markdown、link、GFM alert、task state、KRR runtime、raw Markdown漏れの検査を通過する
- **THEN** PDFはPDF signature、page tree、page image、link annotationの検査を通過する
- **THEN** PNG/JPGはsignature、decode、文書として十分な寸法、非blank領域の検査を通過する
- **THEN** 0 byte、壊れたsignature、decode不能、raw Markdown漏れ、PDF link annotation欠落、blank画像のいずれかをfatal failureとして扱う
- **THEN** scoreが満点ではない場合、またはfatal failureが1件でも残る場合、4形式exportを合格扱いしてはならない

#### Scenario: native surface exportで評価済みblockを視覚表現として保持する

- **GIVEN** KMM評価済みMarkdownにKRRが返したSVG図形とコードブロックが含まれる
- **WHEN** KDVがPDF/PNG/JPGを生成する
- **THEN** KDVはSVG sourceやSVG内CSSを本文テキストとして出力しない
- **THEN** 図形はRust側surface上でrasterizeされた図形blockとして描画される
- **THEN** コードブロックはテーマ由来の背景と枠を持つblockとして描画される

#### Scenario: 形式別にdiagnosticsを保持する

- **WHEN** HTML/PDF/PNG/JPGのいずれかで外部描画やasset解決に失敗する
- **THEN** KDVは `ForgeDiagnostics` と `ArtifactDiagnostics` に失敗理由を保持する
- **THEN** HTMLでは詳細情報を属性または補助metadataとして保持できる
- **THEN** PDF/PNG/JPGでは代表メッセージとraw sourceを失わない

### Requirement: KRR direct backend境界を固定しなければならない

システムは、KDV v0.1.0でKRRを利用するが、KRR側のpublic API縮小やCLI delegate化を行ってはならない（MUST NOT）。

#### Scenario: KRR direct renderingの対象を限定する

- **WHEN** KDVがKRRへ直接 `RenderInput` を渡す
- **THEN** KDVはKRR public APIで直接表現できるMermaid / Draw.io / PlantUML / MathJaxを渡す
- **THEN** KDVは上位から受け取った完全テーマ（theme）をKRR `RenderThemeSnapshot` に変換し、`RenderContext.theme` として渡す
- **THEN** KRRの暗色表示判定は `RenderThemeSnapshot.mode` を使い、KDV側で別の暗黙fallbackを持たない
- **THEN** その他未対応kindはKRRに偽装せず、KDV diagnosticsとして返す
- **AND** 数式（math）はKRR境界を使い、HTML/PDF/PNG/JPGで同じSVGを出力・描画する
- **AND** KRR adapterは、受け取ったTeX文字列をSVG化するだけでMarkdown AST解析を行わない
- **THEN** ZenUMLはKRR Mermaid runtimeの互換入力として扱える場合だけMermaid経路へ渡す

#### Scenario: KRR結果をKDV artifactへ変換する

- **WHEN** KRRがrender outputを返す
- **THEN** KDVはその結果をKDV `Artifact` へ変換する
- **THEN** backend固有型をKDV public APIへ漏らさない
- **THEN** KRR既存利用者の挙動をこのchangeで変更しない
