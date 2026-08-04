## Context

KDV `v0.3.x` はin-process Rust/V8 HTML browser session adapterとして完了し、
`v0.3.5` まで公開された。以前 `v0.5.0` へ繰り延べたmulti-format viewerを
`v0.4.0` へ戻し、PDF / DOCX / XLSX / PPTXを次のdocument viewer対象にする。

KRRの正本 `renderer-runtime-interface` はCSV / PDF / Word / Excel / PPTX viewer
renderingをKDVへ移譲している。したがって、KDV側の古い「KRR候補」という分類は
撤回し、format adapterとviewer semanticsをKDVへ固定する。

## Goals / Non-Goals

**Goals:**

- PDF / DOCX / XLSX / PPTXを閲覧可能にする既存engineを評価する。
- custom parser / layout engineを作らず、adapterとneutral viewer contractに限定する。
- formatごとの品質profileとunsupported範囲を明示する。
- KDV `ViewerSource`、viewer state、diagnostics、host command、KUC bridgeを追加する。

**Non-Goals:**

- KRRへPDF / Office viewer APIを追加しない。
- KatanAまたはKUCへformat parserを追加しない。
- Microsoft Officeとpixel-identicalなlayout engineを独自実装しない。
- PDF編集、annotation追加、署名、form入力、OCRを追加しない。
- Office編集、macro実行、外部link自動取得、再計算を追加しない。
- PPTX animation、transition、embedded media playbackを追加しない。
- CSV / SVG / WebP / AVIFとPDF export paginationを本releaseへ混ぜない。

## Decisions

### D1. Document viewer ownershipはKDVへ固定する

責務は次のとおり固定する。

- KDV: format detection、engine adapter、neutral artifact、viewer state、diagnostics、
  navigation、zoom、fit、copy、open command。
- KUC: neutral page/sheet/slide model、viewport、generic controls。
- KDV: KUCを内部利用するdocument surface、host入力変換、page/grid表示。
- KatanA: file/URL intake、KDV commandのhost処理、KDV document surfaceの表示。
  KUCへ直接依存せず、KUC型を保持しない。
- KRR: 変更なし。文書内diagram/mathを既存KRR APIで解決する場合だけ間接利用する。

### D2. Engine selectionを実装前gateにする

`feasibility.md` のcandidate matrixを使い、各formatで次を測定する。

- representative corpusのreference image差分
- unsupported featureとdiagnostics
- cold start / first frame / navigation latency
- peak memoryとcache size
- macOS / Linux / Windowsの配布方式
- direct / transitive license
- untrusted documentに対するsandboxとresource limit

比較結果とquality profileをユーザーが承認するまでproduction dependencyと
`ViewerSource` variantを追加しない。

評価スコアはvisual fidelity 30、format coverage 20、security/isolation 20、
performance 10、distribution 10、license 10の100点とする。80点以上かつ全hard
gate passをrelease条件とし、閾値緩和を禁止する。不合格候補をproduction dependency
へ追加せず、新規候補も同じrubricで評価する。

### D3. format別quality profileを混同しない

- `static-page`: PDF / DOCXをpage artifactとして表示する。reference image差分、
  page geometry、header / footer、table、imageをhard gateで検証する。
- `interactive-grid`: XLSXをcell modelと2次元virtualized gridで表示する。
  formula、style、merge、row / column geometry、conditional formattingを検証し、
  chart / pivot tableは対応しない限りtyped capabilityで無効化する。
- `static-slide`: PPTXをslide artifactとして表示する。text、image、shape、table、
  chartの意味と配置をhard gateで検証する。

`interactive-grid` をExcel互換page layoutと呼ばず、`static-page` / `static-slide` も
Microsoft Officeとのpixel identityを保証しない。formatごとに選択したprofileと
未対応機能をcapabilityとdiagnosticsで公開する。profile変更は評価閾値の緩和ではなく、
別の表示契約としてcorpus、hard gate、scoreを再定義し、ユーザーの明示承認を必要とする。

### D4. Neutral contracts

- `ViewerSource::Pdf`: bytes/path、identity、revision、MIME。
- `ViewerSource::Office`: format、bytes/path、identity、revision、MIME。
- `PdfDocumentArtifact`: page count、page geometry、rendered page、link/text capability。
- `OfficeDocumentArtifact`: profile、page/sheet/slide metadata、rendered/static artifact、
  semantic model、diagnostics。
- `ViewerCommand`: previous/next、index jump、zoom、fit、copy、open。

engine固有型、KUC型、KatanA型をKDV public APIへ露出しない。
既存の`katana-document-viewer` crateのoptional `egui` featureがKUCを内部利用し、
KDV所有の`DocumentSurfaceFrame` / `DocumentSurfaceCommand` / `DocumentSurfaceHost`を
公開する。KatanAはこのKDV APIだけを利用し、KUC dependency、`UiNode`、`GridAction`、
page/grid painterを持たない。KDVとKUCを混成した別crateを作らない。

### D4.1 v0.4.1 release correction

`v0.4.0`ではcore crate公開後に別presentation crateのuploadが403となり、さらに
KatanAがKDVを飛び越えてKUCへ直接依存する誤った境界が判明した。`v0.4.1`は別crateを
削除し、KDV所有のdocument surfaceへ置き換える。release対象は既存KDV crateだけとし、
dependency方向を`KatanA -> KDV -> KUC/KRR`へ固定する。

### D4.2 v0.4.2 Windows AppContainer correction

`v0.4.1`のWindows経路は、KatanAのrelease directoryにあるworker fileへACLを付与して
AppContainerから直接起動していた。file ACLだけでは親directoryのtraverse権限を保証
できず、KatanA Windows release buildで`CreateProcessW`がworker開始前に失敗した。
`v0.4.2`はAppContainer profile folder内に各document専用workspaceを作成し、workerを固定名で
stageする。workspace、既存input、staged workerへ明示ACLを付与してから、そのstaged path
だけを起動する。KatanAのinstall directory、ユーザーprofile、Temp配下へACLを付けず、
unsandboxed fallbackも追加しない。network deny、memory / time limit、job close時kill、
dedicated workspace cleanupは既存契約を維持する。

### D5. Feasibility評価後の候補状態

- PDFはpure Rust `hayro` 0.7.1を推奨する。85/100、hard gate pass。
- DOCXはpure Rust `office2pdf` 0.6.5 -> canonical PDF -> `hayro`を推奨する。
  85/100でstatic-page hard gateを満たす。ただしKDVのbounded OOXML preflightと
  isolated workerを必須条件とする。
- XLSXの`office2pdf` static-pageは65/100で、formula結果、conditional formatting、
  chart、column paginationのhard gateを満たさないため不採用とする。`IronCalc`
  0.8.0はformula、style、merge、row / column geometry、conditional formattingの
  modelを提供するがRust native rendererを持たない。`interactive-grid` profileを
  明示承認する場合に限り、KUC generic 2D gridと組み合わせる候補とする。
- PPTXの`office2pdf` static-slideは80/100だが、chartの方向と意味が変わるため
  static layout hard gateを満たさない。chartをtyped unsupported diagnosticとする
  profile変更を明示承認しない限り採用しない。
- ONLYOFFICE Document Builder 9.4.0は軽量かつ禁止binary dependencyを検出しないが、
  commercial licenseなしではwatermark/API制限があり、代表XLSX/PPTXにも表示差分が
  あるため不採用とする。
- LibreOffice 26.2.5.2 -> canonical PDF -> Hayroはlayout品質を満たすが、
  Office変換中にLibreOffice同梱PDFiumとmacOS WebKitを実際にloadするため不採用とする。
- `docMentis`、Aspose、Pandoc + Typst、`rwml`、SlideGlance、BetterOfficeは、
  license、distribution、native dependency、layout fidelity、renderer欠落の
  いずれかのhard gateを満たさないため不採用とする。

従って、PDF / DOCXには採用候補があり、XLSX `interactive-grid`とPPTX typed chart
fallbackを含むformat別profileは2026-07-30にユーザーが明示承認した。production
dependencyとformat実装はこの選定に限定する。評価中に未隔離ONLYOFFICE /
LibreOfficeがexternal relationshipを取得し、`office2pdf`の小さな高圧縮DOCXが
約8 GBまでmemoryを消費したため、未隔離fallbackを設けない。

## Responsibility Matrix

| Format | Target | Engine adapter owner | Viewer / command owner | KUC model | KRR |
| --- | --- | --- | --- | --- | --- |
| PDF | `static-page` | KDV | KDV | page viewport / page list | no change |
| DOCX | `static-page` | KDV | KDV | page viewport / page list | no change |
| XLSX | conditional `interactive-grid` | KDV | KDV | conditional generic 2D grid | no change |
| PPTX | conditional `static-slide` | KDV | KDV | slide viewport / controls | no change |

KUC v0.3.0のscroll area、split pane、virtualized list、image surface、slide control、
generic 2D gridを再利用する。KDV側にprivate grid、cell geometry、hit-test、selection
engineを作らず、KUC `GenericGrid` が返す可視座標だけをIronCalc workerへ要求する。
公開KDV document surfaceはKUC coreをcrates.io `0.3.0`から取得する。非公開Storybook supportと
その型を共有する開発用KUC coreだけは同一 `v0.3.0` tagから取得し、どちらの経路にも
sibling path dependencyを使用しない。

## Security

- macroとembedded scriptを実行しない。
- external link、remote image、template、data connectionを自動取得しない。
- ZIP/XML展開量、page/sheet/slide/cell数、処理時間、メモリに上限を設ける。
- Office engineへ渡す前にKDVがOOXML central directoryとrelationshipをbounded
  preflightし、active content、external relationship、圧縮率、展開量、entry数を検査する。
- password protected、corrupt、unsupported featureをtyped diagnosticsにする。
- Office変換はpure Rust engineでも別worker processに隔離し、sandbox、dedicated
  temporary directory、network deny、timeout、memory limit、kill、cleanup、
  crash isolationを必須にする。8 GB memory回帰fixtureをpreflightまたはworker limitで
  engine実行前後に確実に拒否する。
- unsupported時に別rendererへsilent fallbackしない。

## Release Order

1. conditional KUC grid contract（必要な場合のみ）
2. KDV `v0.4.0` multi-format viewer
3. KatanA adjacent releaseでpublished KDVをintake
4. KDV `v0.5.0` PDF export pagination

## Risks / Trade-offs

- pure Rust profileは配布しやすいが、formatによってOffice layout fidelityが異なる。
- external Office engineはlayout fidelityを上げられるが、非Rust依存、配布サイズ、
  cold start、license、security updateの負担が増える。
- `office2pdf`はpure RustでもTypst初期化と変換時memoryが大きく、in-process利用できない。
- XLSX `interactive-grid`は計算結果とcell semanticsを優先する代わりに印刷page layout、
  chart、pivot tableを保証しない。
- PPTX chart fallbackを許可するとslide navigationは提供できるが、chartを含むslideの
  static fidelity hard gateは別profileとして再定義が必要になる。
- engineを抽象化しすぎるとunsupported差分が隠れるため、capabilityを明示する。
- PDF / Officeを一度に実装するとscopeが広いため、共通contract後はformat単位で
  quality gateを通す。
