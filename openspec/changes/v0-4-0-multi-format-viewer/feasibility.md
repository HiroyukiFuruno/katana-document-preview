# Feasibility: KDV v0.4.0 PDF / Office viewer

## Status

2026-07-30時点で候補比較とmacOS arm64の実測を完了し、ユーザーから次の形式別
profileについて明示承認を得た。production実装はこの選定だけを対象に開始できるが、
実装後の再計測で80点以上かつ全hard gate passになるまでrelease-approvedへ変更しない。

- PDF: pure Rust `hayro` 0.7.1を推奨する。初期評価85/100、hard gate pass。
- DOCX: pure Rust `office2pdf` 0.6.5 -> canonical PDF -> `hayro`を推奨する。
  85/100。KDVのbounded preflightとisolated workerを実装した場合だけhard gate pass。
- XLSX: `office2pdf`は65/100でquality hard gate fail。`IronCalc` 0.8.0は
  formula/style modelとして有望だがnative viewerを提供せず、chart未対応のため、
  interactive-grid profileとKUC 2D gridを承認・検証するまで採用しない。
- PPTX: `office2pdf`は80/100だがchartの向きとlayoutを誤るためstatic-layout hard
  gate fail。SlideGlance 0.1.3も67/100でquality/distribution hard gate fail。
- Office readable: `office_oxide` 0.1.8を不採用とする。54/100、layout hard gate fail。
- Office layout-faithful: ONLYOFFICE Document Builder 9.4.0はLibreOfficeより小さく、
  禁止binary dependencyは検出しなかったが、無償出力のwatermark、商用license、
  XLSX chart/PPTX text fidelityにより64/100、license/quality hard gate fail。
- Office layout-faithful: LibreOffice 26.2.5.2 -> canonical PDF -> `hayro`は
  要求品質を描画できるが、初期評価69/100に加えて、変換中にPDFiumとWebKitを
  実際にloadするため禁止dependency hard gate fail。不採用とする。

合格基準は80/100かつ全hard gate passで固定し、閾値を緩和しない。PDFとDOCXには
採用可能な経路がある。XLSXとPPTXには現行profileの全hard gateを満たす候補がない。

## Fixed Constraints

- KDVがPDF / DOCX / XLSX / PPTXのviewer semanticsとadapterを所有する。
- KRRはdocument parsing、PDF page rendering、Office layout、viewer state、
  viewer commandを所有しない。
- KUCはneutral modelとcontrolsを表示し、document formatをparseしない。
- KatanAはthin hostであり、document formatをparseまたはrenderしない。
- Chromium、WebView、PDFium、独自PDF parser、独自OOXML parser、
  独自Office layout engineを使用しない。
- coverage thresholdとquality gateを緩和しない。

## Scoring Contract

| Dimension | Weight | Pass evidence |
| --- | ---: | --- |
| Visual fidelity | 30 | fixed corpus、trusted reference、page count、raster diff |
| Format coverage | 20 | required structure、navigation unit、typed unsupported capability |
| Security / isolation | 20 | active content非実行、外部取得遮断、resource limit、crash isolation |
| Performance | 10 | cold first frame、navigation、peak RSS、lazy cache budget |
| Distribution | 10 | macOS / Linux / Windows artifact、hash pin、update path |
| License | 10 | direct / transitive licenseとredistribution obligation |

Hard gateはlayout profile適合、active content非実行、外部取得遮断、resource limit、
crash isolation、3 OS配布、license確認、strict 100% coverageである。総合点が80点以上
でもhard gateが一つでも未達ならreleaseしない。

## Scorecard

| Candidate | Visual | Coverage | Security | Performance | Distribution | License | Total | Decision |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `hayro` PDF | 25 | 14 | 18 | 8 | 10 | 10 | **85** | 推奨、hard gate pass |
| `office2pdf` DOCX -> PDF -> `hayro` | 26 | 18 | 15 | 6 | 10 | 10 | **85** | 推奨、KDV security gate実装を条件にpass |
| `office2pdf` XLSX -> PDF -> `hayro` | 12 | 12 | 15 | 6 | 10 | 10 | **65** | 不採用、quality hard gate fail |
| `office2pdf` PPTX -> PDF -> `hayro` | 22 | 17 | 15 | 6 | 10 | 10 | **80** | 不採用、static-layout hard gate fail |
| SlideGlance PPTX -> SVG | 17 | 12 | 16 | 10 | 2 | 10 | **67** | 不採用、quality/distribution hard gate fail |
| `office_oxide` readable | 7 | 10 | 12 | 5 | 10 | 10 | **54** | 不採用、layout hard gate fail |
| ONLYOFFICE Document Builder -> PDF -> `hayro` | 20 | 15 | 14 | 6 | 9 | 0 | **64** | 不採用、license/quality hard gate fail |
| LibreOffice -> PDF -> `hayro` | 26 | 18 | 10 | 5 | 3 | 7 | **69** | 不採用、禁止dependency/security/distribution未達 |

## Corpus and Method

`assets/fixtures/multi-format/`に決定的生成可能なcorpusを固定した。
`fixture-manifest.json`は各fixtureのSHA-256、OOXML entry数、展開後byte数を保持する。

- representative PDF: KDV既存13 page PDF。font、image、複数pageを含む。
- representative DOCX: heading、Japanese text、list、table、image、page break。
- representative XLSX: 3 sheets、style、merged cell、formula、chart、Japanese text。
- representative PPTX: 2 slides、absolute geometry、Japanese text、image、chart。
- stress: 100,000 cell XLSX、34,389,487 byteへ展開するDOCX。
- security: external image relationship、macro marker。

PDFとLibreOffice生成PDFは同一page geometryでHayroとPopplerをrasterizeし、
normalized MAE / RMSE / changed-channel ratioを比較した。Officeの意味的抽出は
`office_oxide`のIR、HTML、plain textを固定fixtureで確認した。

## PDF Result

`hayro` 0.7.1は13 page corpusを全page描画し、Popplerとの差分は次のとおりだった。

- mean normalized MAE: `0.003308`
- mean normalized RMSE: `0.033349`
- mean changed-channel ratio: `0.034348`
- first page: `33.699 ms`
- all 13 pages: `360.214 ms`
- generated PNG cache: `2,174,378 bytes`
- peak RSS: `138,526,720 bytes`
- Cargo closure: 74 packages、missing license 0、native `links` 0

Hayro出力はtransparentなので、white backgroundはrenderer設定で与えず、KDV側で
RGBAをalpha compositeする。rendererへwhite backgroundを直接指定するとfixture内の
image colorが失われたため、この挙動を回帰契約にする。

既知のunsupportedはencrypted/password PDF、blending/isolation、knockout group、
color-key maskなどである。silent fallbackせずtyped capability/diagnosticsへ公開する。

## Pure Rust Office Result

### `office2pdf` 0.6.5

`office2pdf`はDOCX / XLSX / PPTXを独自IRへ読み、Typst 0.14でPDFを生成する
Apache-2.0のpure Rust libraryである。KDVはengine内部のOOXML/parser/layout型を
公開せず、private adapterからcanonical PDFをHayroへ渡せる。Chromium、WebView、
PDFium、LibreOffice、external serviceを必要としない。

公式releaseはmacOS arm64/x86_64、Linux arm64/x86_64 glibc、
Linux x86_64 musl、Windows x86_64を提供し、圧縮assetは26.3--29.6 MBである。
macOS arm64 asset SHA-256
`1ca2acd416166e8075bc56acc1799b774bf7189e18f5de5d764ddd0b27dd2625`
を検証した。binaryは約66.6 MBで、system `libiconv`、CoreFoundation、
libSystem以外のdynamic dependencyを持たない。ただしupstream binaryはad-hoc署名で
notarizeされていないため配布せず、productionではKatanA buildに含めたKDV workerを
通常のrelease signing対象にする。

| Format | Cold conversion | Warm conversion | Peak RSS | Output |
| --- | ---: | ---: | ---: | --- |
| DOCX | 5.40 s | 1.61 s | 306,085,888 B | 2 pages |
| XLSX | 5.37 s | 未計測 | 275,791,872 B | 3 pages |
| PPTX | 5.34 s | 未計測 | 274,137,088 B | 2 slides |
| 100,000 cell XLSX (`--streaming`) | 4.69 s | n/a | 1,636,073,472 B | 18 pages |

DOCXはlandscape geometry、2 page break、header/footer、table、image、日本語を保持し、
trusted reference page 1との差分はMAE `0.039577`、RMSE `0.156813`、
changed-channel ratio `0.105716`だった。Hayro first pageは11.240 ms、2 pagesは
16.298 msである。Word pixel identityではないが、選定したstatic-page profileを
満たすため推奨する。

XLSXはcached valueのないformula resultを空欄にし、conditional formattingを欠落し、
chartをfallback boxへ置換し、`Target`列を別pageへ分割した。PPTXは基本geometry、
image、日本語、2 slidesを保持したが、chartを縦棒から横棒へ変え、category/seriesの
向きとlayoutを誤った。PPTX raster差分は平均MAE `0.037598`、RMSE `0.139357`、
changed-channel ratio `0.085872`である。総合80点でもstatic-layout hard gateを
満たさないため、閾値を緩和して採用しない。

security fixtureではexternal relationshipへHTTP requestを行わず、macro markerも
実行しなかった。一方、engine自身にZIP expansion、time、memory limitはない。
72,795 byteのcompressed DOCXをpreflightなしで変換すると15.79 s、
peak RSS 8,034,746,368 bytesまで増えた。従ってKDVはengine call前にOOXML entry数、
展開byte、active content、external relationshipを上限付きで検査し、変換を別process
で実行してtimeout、memory limit、kill、cleanupを保証しなければならない。

### Additional native candidates

- `rwml` 0.1.1はDOCXを0.09 s、peak RSS 25,133,056 bytesで2 page PDFへ変換した。
  body、table、image、日本語、page breakは保持したが、image geometryがreferenceより
  大きく、2 page目のheader/footerをdiagnosticなしで欠落した。公開から約1か月であり、
  高品質な`office2pdf` DOCX経路を置き換えない。
- SlideGlance 0.1.3はpure Rust PPTX -> SVG/PNG libraryで、library pathはparse
  12 ms、peak RSS約7.3 MBだった。chart title/axis label欠落、text clipping、
  crates.io未公開により67/100で不採用。
- `IronCalc` 0.8.0は代表XLSXを10 ms未満、100,000 cellsを0.06 s、
  peak RSS 69,533,696 bytesで読み、empty cached resultのformulaを正しく再計算した。
  ただしchart/pivot tableは未対応で、Rust crateはnative rendererを提供しない。
  official rendererはbrowser canvasを必要とする。KUC generic 2D gridを新規実装する
  interactive-grid profileをユーザーが承認した場合だけ別OpenSpecで再評価する。
- BetterOffice PPTX 0.0.4はbounded parserとshaped-text display listを持つが、
  table/chartをplaceholderにし、raster/KUC backendを提供しない。欠けたrendererを
  KDV/KUCで作る経路になるため不採用。
- Pandoc 3.10.1 + Typst 0.15.1は0.61 sでDOCXを変換し外部取得もしなかったが、
  page break、header/footer、landscape geometryを失い1 pageへcollapseした。
- docMentis udoc WASM 0.7.11は18.6 MBで複数formatをCPU renderできたが、
  free licenseは競合viewer利用を禁止し、server permitとtelemetryを必須にする。
  XLSX formula/chart欠落もあるためlicense/offline/privacy/quality hard gate fail。

### Library screening

- `calamine` 0.36.1はcell/value/formula/image metadataを読むspreadsheet readerであり、
  Excelのpage layout engineではない。
- `docx-rs` 0.4.21はDOCX object modelを読み書きできるが、Word pagination rendererを
  提供しない。
- `pptx`系crateはpackage/object accessを主目的とし、PowerPoint slide rendererを
  提供しない。
- `office_oxide` 0.1.8はDOCX/XLSX/PPTXを統一IR/HTMLへ変換できるため、
  readable profileの代表として実測した。

### Measured result

- DOCX: parse `3.097 ms`、HTML 777 bytes。page breakを含んでも1 sectionで、
  Word paginationとlayoutを保持しない。
- XLSX: parse `0.625 ms`、HTML 1,017 bytes。style、cached formula result、
  chart、Japanese Notes rowが必要品質で保持されない。
- PPTX: parse `0.713 ms`、HTML 279 bytes。absolute position、style、image data、
  chartを保持しない。
- 100,000 cell XLSX: peak RSS `310,525,952 bytes`、IR JSON `202,045,818 bytes`。
- expanded DOCX: peak RSS `306,774,016 bytes`、HTML約33.6 MB。

小規模文書のparseは速いが、HTMLは意味的断片であり、DOCX page、XLSX layout、
PPTX slide fidelityを満たさない。また評価版には展開量、cell数、memoryの強制上限が
ない。従ってpure Rust readable profileをKatanAのOffice viewerとして採用しない。

## Lightweight Office Engine Result

### ONLYOFFICE Document Builder 9.4.0

公式release artifactはmacOS arm64 40,000,600 bytes、展開後190,896 KiBであり、
LibreOfficeの297,407,265 bytes / 819,188 KiBより小さい。GitHub release APIは
macOS arm64/x86_64、Linux arm64/x86_64、Windows x64/x86のasset sizeとSHA-256を
提供している。macOS binariesはAscensio System SIAのDeveloper IDで署名済みだった。

native C++ binaryとJavaScriptCoreを使用する。全executable/frameworkのMach-O
dependencyとbinary nameを検査し、Chromium、WebView/WebKit、PDFium、CEFは
検出しなかった。`doctrenderer.framework`はmacOS JavaScriptCoreへlinkするため、
pure Rustではないが、browser/WebView dependencyではない。

| Format | Cold conversion | Peak RSS | Output |
| --- | ---: | ---: | --- |
| DOCX | 3.95 s | 296,779,776 B | 2 page PDF |
| XLSX | 0.99 s | 226,000,896 B | 1 page PDF |
| PPTX | 1.21 s | 278,003,712 B | 2 slide PDF |
| 100,000 cell XLSX | 28.83 s | 518,078,464 B | 210 page PDF |

DOCXはpage break/table/imageを保持し、PPTXのabsolute geometry/image/chartも概ね
保持した。一方、代表XLSXのchart seriesはformula resultを正しく表示せず、
代表PPTXのchart titleを欠落し、右側textをwrapせずclipした。full licenseなしでは
`Api`/`Asc` initializationが失敗するため、recalculationによる改善を検証できない。

無償版は全生成文書へ`Unregistered Version` watermarkを入れ、
`license is invalid`を報告する。公式documentationはwatermark除去とfull featureに
commercial licenseが必要としている。KDVはMITであり、AGPL packageまたはcommercial
SDKの再配布・統合条件を法務確認せず採用できない。

security fixtureでは未隔離childがexternal relationshipをHTTP GETした。
macOS `deny network*` sandbox下では取得が拒否され、PDF生成は継続した。これは
LibreOfficeより隔離しやすいが、macro/script preflight、archive/memory/time limit、
strict write sandbox、3 OS runtimeは未検証である。

軽量性は確認できたが、現時点ではlicense/quality hard gateを満たさない。商用license
または制限なし30日licenseをユーザーが明示的に用意する場合だけ、watermarkなしの
同一corpusを再評価できる。

### Additional screening

- Aspose Words/Cells/Slides for C++は3 OS対応の独立layout engineだが、2026-07-30の
  NuGet packageは合計348,002,043 bytesでLibreOffice DMGより大きい。3製品の
  proprietary licenseが必要で、無償評価はwatermark/文書量制限を持つため不採用。
- ONLYOFFICE Docs/Desktop Editorsはbrowser editorまたはfull desktop suiteであり、
  Chromium/WebView非依存と軽量配布の要件を満たさない。
- macOS Quick Look、Windows Preview Handler/Office COM、Linux desktop thumbnailは
  OS固有で品質契約が異なり、headless 3 OS CIを構成できない。
- Microsoft/Google/Adobe remote conversionはlocal/offlineと文書非送信の要件に反する。
- Pandoc等のsemantic converterはXLSXとOffice pagination/slide layoutを統一して
  提供しない。Pandoc + Typstは実測でもDOCXのpage break、header/footer、
  landscape geometryを失った。

## Layout-faithful Office Result

### Rejected candidate

LibreOffice 26.2.5.2をheadlessでDOCX/XLSX/PPTXからcanonical PDFへ変換し、
PDF pageをHayroでlazy rasterizeする。KDVはLibreOffice固有型を公開せず、
Office source、capability、diagnostics、page/sheet/slide identityを所有する。

ただしmacOS arm64で100,000 cell XLSXを変換中のprocess imageを`vmmap`で検査すると、
LibreOffice同梱`libpdfiumlo.dylib`とmacOS WebKit/WebCore/WebKitLegacyが実際に
loadされていた。変換対象はOffice文書でありPDF importではない。この経路は
PDFium/WebView系非依存のhard gateを満たさないため、production候補から除外する。

### Fidelity and performance

| Format | LibreOffice cold conversion | Peak RSS | Hayro first page | Hayro all pages | Poppler mean RMSE |
| --- | ---: | ---: | ---: | ---: | ---: |
| DOCX | 5.97 s | 336,740,352 B | 12.228 ms | 18.664 ms | 0.053511 |
| XLSX | 4.00 s | 343,523,328 B | 6.353 ms | 10.674 ms | 0.033183 |
| PPTX | 4.09 s | 377,470,976 B | 9.149 ms | 15.851 ms | 0.062502 |

同一profileのDOCX再変換は2.06 sだった。100,000 cell XLSXは5.61 s、
peak RSS 431,046,656 bytes、218 pagesとなった。218 pagesを全描画すると3.65 s、
cache約71.9 MBになるため、productionではfirst page優先のlazy renderとbounded
page cacheが必須である。

生成結果はDOCXのpage break/table/image、XLSXのstyle/formula/chart、
PPTXのabsolute layout/image/Japanese/chartを保持した。PPTXはmacOS Quick Look
referenceとも構造一致を確認した。DOCX/XLSXのQuick Lookはthumbnail semanticsにより
page/tableを省略またはcropするためtrusted full-layout oracleには使用しない。

### Security result

- 未隔離LibreOfficeはexternal image relationshipに対してHTTP `OPTIONS` 1回、
  `GET` 2回を実行した。未隔離実行は不採用である。
- macOS sandboxで外部networkをdenyし、localhostとUNIX socketだけを許可すると
  representative DOCX変換は成功した。
- networkを全面denyするとLibreOfficeはexit 0でもPDFを生成しなかった。
  したがってexit codeだけで成功判定せず、expected output、MIME、page countを検証する。
- dedicated directory以外のwriteをdenyするstrict file sandboxは現時点で変換に失敗した。
  production採用にはOS別sandbox、OOXML preflight、timeout/kill、memory limit、
  dedicated temp、cleanup、child crash isolationの実装と契約テストが必要である。

macro-markerは実行可能macroではなくactive-content preflight用markerである。
productionでは変換前にOOXML relationship/content typeを上限付きで検査し、
macro/script、external relationship、template、data connectionを拒否する。

### Distribution and license

- macOS arm64 DMG: 297,407,265 bytes、SHA-256
  `c99fb4fe574437fc4cb820a4ca15271bca325920861f7139858b36d7f9df78ad`を実測確認。
- 展開後macOS application: 819,188 KiB。
- macOS x86_64 DMG、Linux x86_64 DEB archive、Windows x86_64 MSIの公式artifact
  存在は確認したが、hash、signature、実行、sandboxは未検証である。
- LibreOffice本体はMPL 2.0を中心とするが、同梱third-party noticeにはLGPL/GPLを含む
  多数のlicenseがある。再配布方式、notice、source offer等はrelease前の法務監査対象。
- full bundleのKDV同梱またはon-demand取得は、重量と禁止dependencyの双方により
  採用しない。

## Rejected Candidates

- PDFium / `pdfium-render`: 外部C++ PDF engineとnative binary supply chainが必要。
- Chromium / browser / WebView Office preview: local Rust-first viewer、
  offline local-file contract、明示された禁止事項に反する。
- hand-written OOXML/PDF parserまたはlayout: Microsoft Office/PDF behaviorの
  再実装となり、ownershipとmaintenance budgetを超える。
- pure Rust Office semantic HTML: content extractionには使えるが、
  KatanAで要求するOffice layout表示を満たさない。

## Decision Required

production実装へ進める状態は次のとおりである。

1. PDFはpure Rust `hayro` 0.7.1をKDV private adapterとして承認可能である。
2. DOCXはpure Rust `office2pdf` 0.6.5 -> PDF -> Hayroを、KDVのbounded preflightと
   isolated worker実装を条件に承認可能である。
3. XLSXは現行static layout profileに合格候補がない。IronCalc + KUC generic
   2D gridを`interactive-grid` profileとして新設する案だけがRust-firstで実現可能性を
   持つが、chart/pivot tableをtyped unsupportedとする設計承認が必要である。
4. PPTXは`office2pdf`が80点に達したがchart semantics/layoutを誤るためhard gate
   failである。chartをtyped unsupported/fallbackとするstatic-slide profile変更を
   承認しない限り採用できない。
5. ONLYOFFICE Document Builderは軽量だが、commercial licenseなしではwatermarkと
   API制限があり、現状のvisual fidelityも未合格である。
6. LibreOfficeはforbidden-dependency/security/distribution hard gate failである。

PDF/DOCXだけを先行releaseすることも、XLSX/PPTXのprofileを変更することも現行DoDを
変更するため自動では行わない。production実装へ進むには、PDF/DOCXの採用に加えて、
XLSX `interactive-grid`とPPTX typed chart fallbackは2026-07-30に明示承認された。
次のgateはKUC/KDV実装後のformat別再計測、80点以上、全hard gate passである。

## Evidence

- `evidence/benchmark-summary.json`: score、timing、memory、distribution、security結果
- `evidence/libreoffice-loaded-images.json`: Office変換中にloadされた禁止dependency
- `evidence/onlyoffice-document-builder.json`: 軽量engineのasset、性能、license、
  dependency、security、visual評価
- `evidence/fixture-manifest.json`: corpus SHA-256と展開量
- `evidence/cargo-supply-chain.json`: Cargo dependency/license closure
- `evidence/metrics/`: engine metricsとPoppler raster diff
- `evidence/screenshots/`: PDF/DOCX/XLSX/PPTXの固定render証跡
- `assets/fixtures/multi-format/README.md`: corpus生成方法と期待構造
- `tools/multi-format-feasibility/`: production dependencyから隔離した評価runner

## Release Order

1. 全formatで承認済みengineが揃ってからKDV private adapterへ実装する。
2. KDV `v0.4.0`を80/100以上かつ全hard gate passでreleaseする。
3. KatanAの隣接patchで公開済みKDVを統合する。
4. KDV `v0.5.0` PDF export paginationを実装する。
