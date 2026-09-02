## Context

KDV 0.5.5はXLSXを別processのIronCalc/streaming engineとKUC GenericGridで表示し、DOCX/PPTXを別processのoffice2pdfでpage/slideへ変換する。XLSX AutoFilter、local-header data descriptor、session cache、worker cleanupのowner-layer実装は導入済みだが、KatanAで再現した全20 entryのDOCX fixture、V8の公開consumer link、DOCX/XLSXのsource-renderer fidelity比較をrelease DoDとして固定する必要がある。

## Goals / Non-Goals

**Goals:**

- KDV所有の型だけでXLSX filter metadata、criteria、command/event、表示行を表現する。
- 正当なOOXML ZIP variantを受理し、安全上の検査は維持する。
- workerのstage時間と再利用を計測可能にし、同一sourceの重複変換をなくす。
- cache上限とsession close/drop後の資源解放を決定論的に検証する。

**Non-Goals:**

- Excelの編集、数式再計算、任意式filter、sort、pivot tableは実装しない。
- OOXMLを書き換えない。Office layoutをKDVで再実装しない。
- KUCへOffice固有型またはfilter semanticsを追加しない。
- KatanAへZIP解析、Office変換、filter評価を移さない。

## Decisions

1. AutoFilterは`SpreadsheetSheetArtifact`へ範囲と列定義を追加し、criteriaはKDVの公開commandとしてsessionへ渡す。候補値抽出と行評価は既存の隔離spreadsheet workerで実行する。全行をhostへmaterializeする案は、大規模sheetでIPC量とメモリ上限を破るため採用しない。
2. filter適用結果は元row indexの可視性として保持し、KDVがgrid用row trackを再構築する。KUC GenericGridは汎用hidden trackだけを受け取り、Office semanticsを知らない。
3. AutoFilter XMLはOOXML worksheetの`autoFilter`、`filterColumn`、`filters/filter`だけをbounded streaming parseする。未対応criteriaは診断として保持し、誤った絞り込みを行わない。
4. ZIP local-header走査はdata descriptorを「中央directory検証へ委譲可能な状態」として型で判定する。error文字列一致だけに依存する案を廃止し、実descriptor fixtureで中央directory・CRC・entry安全検査が維持されることを固定する。
5. PPTXはsource identityと変換設定からcache keyを作り、同一sessionの不変入力では変換artifactを再利用する。global無制限cacheは採用せず、session所有の個数・bytes上限付きLRUとする。
6. worker spawn、runtime init、package parse、preflight、convert、decode、frame publication、rasterを`DEBUG=true`時だけstage traceへ出す。XLSXには同じ責務の`spreadsheet.*` stageを出し、release版の通常出力と処理分岐は変えない。
7. `close`と`Drop`の両方がworker shutdown、temporary workspace、frame/cacheを解放する。二重closeは成功する冪等操作とする。
8. KDV direct `v8 =152.2.0`とKRR 0.4.19のV8を単一registry packageへ解決する。local `cargo tree -d`と公開KDV APIを参照するconsumer linkに加え、publish後はpath/gitなしのtemporary registry consumerをfresh resolveして同じ条件を再確認する。
9. DOCX/XLSX fidelityのsource rendererはLibreOffice 26.8.0.3、72 dpi、`representative.docx`/`representative.xlsx`のSHA-256、DOCX 842x596/XLSX 596x842 viewportに固定する。KDV runtimeへLibreOffice等を追加するのではなく、比較harnessのsourceとしてのみ扱う。XLSX border metadataはworker artifactから公開frameまでKDVが保持し、KUCのcustom border描画が公開された時だけthin projectionを接続する。

## Risks / Trade-offs

- [Risk] AutoFilterのOOXML表現は広く、初回実装で全criteriaを扱えない → 対応subsetを型で明示し、未対応条件は適用せずdiagnosticへ出す。
- [Risk] 行visibilityの更新でselection/scroll位置が無効になる → 可視な最寄りrowへ正規化し、eventでhostへ通知する。
- [Risk] cacheにより古いartifactを返す → content hash、format、worker設定をkeyに含め、closeで破棄する。
- [Risk] data descriptor受理がarchive検査を弱める → 中央directory、CRC、重複名、展開量、relationship検査は必須のままにする。
- [Risk] KDV/KRRが異なるV8を解決するとconsumer binaryが二重静的runtimeをlinkする → version、lockfile、duplicate tree、local link、registry linkを別々に必須化する。
- [Risk] source renderer未合意のままfidelity scoreを採用すると比較対象が恣意的になる → LibreOffice 26.8.0.3 / 72 dpi / fixture hash / viewport / artifact hashをfidelity recordへ固定し、変更時は再計測を必須にする。
- [Risk] KUC 0.3.3はcellごとのcustom border描画を持たず、KDVだけで視覚差分を隠すとowner boundaryを越える → KDVはmetadataを保持してscoreへ露出し、visual gapはKUC公開版のthin projection待ちとして数値化する。

## Migration Plan

1. 追加型とworker protocolを後方互換なvariant/fieldとして導入する。
2. KDVの生成fixture・実Office corpus・full gateを通し、patch versionを公開する。
3. KatanAをexact registry versionへ更新し、KatanA側はframe metadataとcommand/eventをeguiへ投影する。
4. 問題時はKatanA dependencyを直前の公開KDVへ戻せる。文書ファイルの移行はない。

## Open Questions

- Issue #48のKUC custom cell border描画をどの公開版で受けるか。KDV側のsource artifact、border metadata、fidelity scoreは固定済みであり、KUC registry versionだけがvisual projectionの前提である。
