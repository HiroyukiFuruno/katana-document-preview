## ADDED Requirements

### Requirement: multi-format viewer は実装前にengineと品質profileを確定しなければならない

システムは、PDF / DOCX / XLSX / PPTXのproduction実装を開始する前に、formatごとの目標品質、既存engine、依存方式、license、配布方式、性能、安全性、未対応機能を比較し、採用案についてユーザーの明示承認を得なければならない（MUST）。

#### Scenario: engine候補を比較する

- **WHEN** KDV `v0.4.0` のmulti-format viewerを計画する
- **THEN** KDVはPDF / DOCX / XLSX / PPTXごとにrepresentative corpusとtrusted referenceを固定する
- **THEN** KDVは表示差分、first frame、navigation latency、peak memory、artifact cache sizeを測定する
- **THEN** KDVはdirect / transitive license、cross-platform配布、security update経路を記録する
- **THEN** KDVはunsupported featureをtyped capabilityまたはdiagnosticsとして記録する
- **THEN** KDVはPDF / DOCXの`static-page`、XLSXの`interactive-grid`、PPTXの`static-slide`を別の表示契約として評価する

#### Scenario: engine選定が未承認である

- **WHEN** formatのengine、quality profile、dependency modelが未承認である
- **THEN** KDVはそのformatのproduction dependencyを追加しない
- **THEN** KDVはそのformatの `ViewerSource` variantを正規経路へ追加しない
- **THEN** KDVは独自parser、独自layout engine、silent fallbackを暫定実装しない

#### Scenario: format別profileを変更する

- **WHEN** static layout hard gateを満たさないformatに別の表示契約を提案する
- **THEN** KDVは既存score thresholdまたはhard gateを緩和しない
- **THEN** KDVは表示対象、unsupported機能、corpus、hard gate、scoreを別profileとして定義する
- **THEN** KDVは変更後のprofileについてユーザーの明示承認を得る

#### Scenario: 禁止されたengine方式を評価する

- **WHEN** Chromium、WebView、PDFium、独自PDF parser、独自OOXML parser、独自Office layout engineが候補に含まれる
- **THEN** KDVはその候補を採用しない
- **THEN** KDVは却下理由をfeasibility evidenceへ記録する

### Requirement: document viewerの責務をKDVへ固定しなければならない

システムは、PDF / DOCX / XLSX / PPTXのformat routing、engine session、worker、materialization、neutral artifact、viewer state、diagnostics、viewer commandを統合したdocument sessionをKDVの責務として実装しなければならない（MUST）。

#### Scenario: PDFまたはOffice文書を開く

- **WHEN** ホストが選定済みengineに対応するPDF / DOCX / XLSX / PPTX sourceを渡す
- **THEN** KDVはsource identity、revision、MIME、format、capability、diagnosticsを保持する
- **THEN** KDVはformat routing、engine session、worker lifecycle、render scale、XLSX materializationを所有する
- **THEN** KDVはengine固有型をneutral page / document / sheet / slide artifactへ変換する
- **THEN** KDVはnavigation、zoom、fit、copy、openのうち対応済みcommandを公開する
- **THEN** KUCはneutral artifactとgeneric controlsだけを表示する
- **THEN** KatanAはsource intake、host command処理、中立frameのapplication backend投影だけを担当する

#### Scenario: 統一document sessionを操作する

- **WHEN** ホストがKDV document source、初期viewport、必要なplatform worker resourceを渡す
- **THEN** KDVはformat固有sessionを公開せず、統一されたopen / apply / frame / close契約を提供する
- **THEN** pointer座標、navigation、zoom、fit、resize、scrollはKDV所有commandとして受け取る
- **THEN** KDVはpointer座標をKUC hit-testへ委譲し、KUC interaction結果をKDV所有eventへ変換する
- **THEN** ホストはPDF / Office engine、KUC grid、cell materializationを直接操作しない

#### Scenario: formatが対応するcommandを適用する

- **WHEN** ホストが現在のformatとcapabilityで対応済みのnavigation、zoom、fit、resize、grid interaction、copy、open commandを適用する
- **THEN** KDVはstateまたはKUC interactionを一度だけ更新する
- **THEN** KDVは結果をKDV所有のdocument session eventとして返し、破棄しない
- **THEN** KatanAはcopyまたはopen eventに必要なplatform I/Oだけを実行する

#### Scenario: formatが対応しないcommandを適用する

- **WHEN** paged documentへgrid commandを渡す、またはXLSXへzoom、fit、open targetを渡す
- **THEN** KDVはformatとcommand kindを含むtyped unsupported command errorを返す
- **THEN** KDVはstateを変更せず、silent successまたは別経路へのfallbackを行わない

#### Scenario: 統一document sessionのmetadataとlifecycleを扱う

- **WHEN** ホストがopen済みsessionのmetadataを取得またはsessionを終了する
- **THEN** KDVはidentity、revision、MIME、format、capabilities、diagnosticsをKDV所有info DTOで返す
- **THEN** KDVは明示的なconsuming close契約を提供する
- **THEN** KatanAはformat engineまたはworker lifecycleを直接管理しない

#### Scenario: application UI backendとの境界を検証する

- **WHEN** KDVのmanifest、source、public APIを検査する
- **THEN** KDVはKUC 0.3.0をrequired registry dependencyとして内部利用する
- **THEN** KDVはegui、eframe、または別application UI backendへ依存しない
- **THEN** KDVはKUC型をpublic field、return type、re-export、type aliasとして公開しない
- **THEN** KDVはKDV所有のpage/grid frame、command、eventだけを公開する
- **THEN** 現行KatanAだけがegui inputとpaint処理をapplication backend boundaryで変換する
- **THEN** KDVとKatanAはKUCのlayout、geometry、hit-test、interaction stateを再実装しない

#### Scenario: KRRとの責務境界を検証する

- **WHEN** PDF / DOCX / XLSX / PPTX viewerの依存関係とsource usageを検査する
- **THEN** KRRにdocument parser、PDF page renderer、Office layout engine、viewer state、viewer commandを追加していない
- **THEN** KDVはKRR内部実装やCLIを直接呼び出していない
- **THEN** 文書内diagramまたはmathを解決する場合だけ既存KRR public APIを間接利用する

#### Scenario: XLSXに2次元仮想グリッドが必要である

- **WHEN** 承認済みXLSX profileがrow / column両方向のvirtualizationを必要とする
- **THEN** KDVはformat semanticsを含まないgeneric grid contractをKUCの別changeへhandoffする
- **THEN** KUCの公開releaseが完了するまでKDVはprivate代替gridを追加しない

### Requirement: PDF viewerを安全な静的閲覧として提供しなければならない

システムは、承認済みPDF engineをKDV adapter経由で利用し、静的ページ閲覧を提供しなければならない（MUST）。

#### Scenario: PDFをpreviewする

- **WHEN** ホストが有効なPDF sourceを渡す
- **THEN** KDVはpage count、page geometry、rotation、cropを保持する
- **THEN** KDVは前後移動、page index jump、zoom、fitを提供する
- **THEN** linkまたはtext selectionをengineが提供しない場合はcapabilityで無効を示す
- **THEN** unsupported featureを別rendererへsilent fallbackしない

#### Scenario: PDF pageをcanonical artifactとして表示する

- **WHEN** PDFまたは承認済みDOCX変換がcanonical PDF artifactを生成する
- **THEN** KDVは承認済みpure Rust PDF engineをprivate worker adapter経由で利用する
- **THEN** KDVはengine固有型をKUCまたはKatanAへ露出しない
- **THEN** KDVは同じpage artifactとviewer commandをPDF / DOCXで共有する

#### Scenario: 保護または破損PDFを開く

- **WHEN** PDFがpassword protected、corrupt、resource limit超過のいずれかである
- **THEN** KDVは原因、format、operation、source identityを含むtyped diagnosticsを返す
- **THEN** KDV workerまたはhost processを停止させない

### Requirement: Office viewerを承認済みprofileで提供しなければならない

システムは、DOCX / XLSX / PPTXを承認済みのformat別profileとして提供しなければならない（MUST）。

#### Scenario: Office文書を表示する

- **WHEN** ホストがDOCX / XLSX / PPTX sourceを渡す
- **THEN** DOCXはdocument/page、XLSXはsheet、PPTXはslide単位のnavigationを提供する
- **THEN** KDVは実際に選択したprofileとunsupported featureをcapabilityへ記録する
- **THEN** interactive-gridをExcel互換page layoutと表示しない
- **THEN** static-pageまたはstatic-slideをMicrosoft Officeとのpixel identityと表示しない

#### Scenario: DOCXをstatic pageとして表示する

- **WHEN** 承認済みDOCX `static-page` profileで文書を開く
- **THEN** KDVはbounded preflight後にisolated workerでcanonical PDFへ変換する
- **THEN** KDVはPDF page artifactとしてpage geometry、header / footer、table、imageを保持する
- **THEN** KDVは変換engineとPDF engineのdiagnosticsをsource identity付きで保持する

#### Scenario: XLSXをinteractive gridとして表示する

- **WHEN** ユーザーがXLSX `interactive-grid` profileを明示承認している
- **THEN** KDVはformula、style、merge、row / column geometry、conditional formattingをneutral sheet artifactへ変換する
- **THEN** KUCはformat semanticsを持たないgeneric 2D virtualized gridを表示する
- **THEN** chart、pivot table、print page layoutが未対応ならtyped capabilityとdiagnosticsで無効を示す
- **THEN** KDVはinteractive gridをExcel互換page layoutと表示しない

#### Scenario: PPTX chartを含むslideを表示する

- **WHEN** ユーザーがchart unsupportedを許容するPPTX `static-slide` profileを明示承認している
- **THEN** KDVはchartを誤った意味または配置で描画しない
- **THEN** KDVは該当chartをtyped unsupported diagnostic付きの明示fallbackとして表示する
- **THEN** text、image、shape、tableとslide geometryは承認済みhard gateを満たす

#### Scenario: Office文書を異なるOSで表示する

- **WHEN** 同じDOCXまたはPPTXをmacOS / Linux / Windowsのisolated workerで変換する
- **THEN** KDVはsource commit、SHA-256、licenseを固定したmetric-compatible Latin fontと日本語fallback fontをworker workspaceへ展開する
- **THEN** KDVは既存engineのfont search APIへ固定font directoryを渡し、OSのfont在庫だけに結果を依存させない
- **THEN** 日本語glyphを欠け字にせず、font substitutionによるtext boxの行重なりを発生させない
- **THEN** KDVはOOXMLを書換えず、独自font substitution parserまたはlayout engineを追加しない

#### Scenario: active contentを含むOffice文書を開く

- **WHEN** 文書がmacro、embedded script、external link、remote image、template、data connectionを含む
- **THEN** KDVはactive contentを実行しない
- **THEN** KDVはexternal resourceを自動取得しない
- **THEN** KDVは検出したactive contentをtyped diagnosticsへ記録する

#### Scenario: Office packageのresource limitを超える

- **WHEN** ZIP/XML展開量、page / sheet / slide / cell数、処理時間、メモリのいずれかが上限を超える
- **THEN** KDVはengine起動前にbounded OOXML preflightでentry数、圧縮率、展開量、external relationship、active contentを検査する
- **THEN** KDVは処理を中止してtyped diagnosticsを返す
- **THEN** temporary artifactをcleanupする
- **THEN** Office engineはpure Rust実装でも別worker processで実行する
- **THEN** workerはnetwork deny、timeout、memory limit、killを強制し、hostからcrashを隔離する

### Requirement: release対象をPDFとOffice viewerへ限定しなければならない

システムは、KDV `v0.5.x` のrelease contractをPDF / DOCX / XLSX / PPTX viewerに限定しなければならない（MUST）。

#### Scenario: v0.5.x release targetを検証する

- **WHEN** release gateがKDV `v0.5.x` を検証する
- **THEN** current OpenSpec changeは `v0-5-0-multi-format-viewer` である
- **THEN** PDF export paginationはKDV `v0.6.0` のdeferred targetである
- **THEN** CSV / SVG / WebP / AVIFは本changeのDoDに含まれない
- **THEN** coverage thresholdまたは既存quality gateを緩和していない

#### Scenario: feasibility scorecardを検証する

- **WHEN** local checkまたはrelease gateがmulti-format scorecardを検証する
- **THEN** visual 30、coverage 20、security 20、performance 10、distribution 10、license 10の重みを固定する
- **THEN** component scoreとtotalの一致、最低80点、全hard gate必須、閾値緩和禁止を機械検証する
- **THEN** hard gate未達候補をrecommendedまたはrelease-approvedにできない
- **THEN** PDF / DOCX / XLSX / PPTXすべてのproposed profileとcandidateを要求する

#### Scenario: 未承認または未達候補でreleaseする

- **WHEN** format別profileが未承認、80点未満、hard gate未達、release未承認のいずれかである
- **THEN** release contract checkはformatと原因を示して失敗する
- **THEN** local evidence checkの成功をrelease承認の代替にしない
