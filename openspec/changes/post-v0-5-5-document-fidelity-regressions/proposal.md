## Why

KatanA v0.22.41でOffice文書を実利用した結果、XLSXのAutoFilter欠落、DOCX/XLSX/PPTXのZIP互換エラー、PPTX初回表示の長時間化、文書切替後の資源滞留が確認された。これらはKDVが所有する文書モデル・worker・session境界で直し、KatanA固有の解析や回避処理を増やさずに再発防止する必要がある。

## What Changes

- XLSX AutoFilterの範囲・列・選択候補・適用条件を型付きartifactとして公開し、filter command/eventと表示行状態をKDV sessionへ追加する。
- データ記述子を持つ正当なOOXML ZIPを受理しつつ、重複entry、破損payload、resource limitの拒否を維持する。
- KatanAで再現した全20 entryのDOCX data-descriptor fixtureをKDVへ固定し、preflightと実workerのfirst frameまで回帰化する。
- PPTX/XLSXのcold/warm差をspawn、runtime init、package parse、conversion、frame publicationまでstage別に計測し、同一sourceの不要な再変換を防止する。
- page/grid/artifact cacheとworker lifecycleを上限付きにし、close/drop後にprocess・frame・cacheを解放する。
- KDVとKRRのV8を単一のregistry版へ解決し、local graph、公開API consumer link、公開後のfresh registry consumerで二重linkを拒否する。
- DOCX/XLSXの客観fidelityは、合意済みsource rendererのversion、viewport、fixture hash、element/geometry scoreを固定してから比較する。
- 実Office corpusと生成fixtureを用いた回帰・性能・資源解放gateを追加する。

## Capabilities

### New Capabilities

- `interactive-spreadsheet-filtering`: XLSX AutoFilter metadata、typed command/event、候補値、表示行状態の契約。
- `office-package-compatibility`: 正当なOOXML ZIP variantsを受理し、安全性違反はtyped errorで拒否する契約。
- `office-preview-performance`: Office workerのstage計測、同一source再利用、初回表示budgetの契約。
- `document-session-resource-lifecycle`: session cache上限とclose/drop時のworker・frame・artifact解放契約。
- `office-preview-fidelity`: DOCX/XLSXのsource-renderer referenceと客観element/geometry比較の契約。
- `runtime-dependency-integrity`: KDV/KRRのV8単一化とregistry consumer link検証の契約。

### Modified Capabilities

- なし。

## Impact

- KDVのspreadsheet artifact、worker protocol、document session、neutral grid frame、Office preflight、paged conversion cache、release consumer検証を変更する。
- 公開APIには追加の型・command・event・metadataが加わるが、既存variantと既存hostの挙動は維持する。
- KatanAは公開されたKDV版へ更新し、egui表示と入力だけを担当する。
