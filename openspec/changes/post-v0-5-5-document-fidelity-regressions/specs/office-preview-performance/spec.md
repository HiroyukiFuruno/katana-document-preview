## ADDED Requirements

### Requirement: Office初回表示のstage時間を診断可能にしなければならない
KDVは`DEBUG=true`の場合だけarchive intake、package parse、worker spawn、runtime init、worker transfer、conversion、engine parse/layout、artifact decode、frame publication、raster first frameの経過時間とsource identityを出力しなければならない（MUST）。XLSXは同じ責務を`spreadsheet.*` stageで出力しなければならない（MUST）。

#### Scenario: DEBUGを有効にしてPPTXを開く
- **WHEN** `DEBUG=true`でPPTX sessionを開きfirst frameを取得する
- **THEN** `office.archive_intake`、`office.package_parse`、`office.transfer_to_worker`、`office.worker_spawn`、`office.runtime_init`、`office.conversion`、`office.parse_layout`、`office.transfer_from_worker`、`office.frame_publication`、`office.raster`が同一session/sourceへ関連付けて出力される

#### Scenario: DEBUGを有効にしてXLSXを開く
- **WHEN** `DEBUG=true`でXLSX sessionを開きfirst frameを取得する
- **THEN** `spreadsheet.worker_spawn`、`spreadsheet.runtime_init`、`spreadsheet.package_parse`、`spreadsheet.frame_publication`が出力され、cold/warm差をstageごとに比較できる

#### Scenario: 通常のrelease実行を行う
- **WHEN** `DEBUG`が未設定またはfalseである
- **THEN** stage traceは出力されず、表示結果と制御フローは変わらない

### Requirement: 不変sourceを不要に再変換してはならない
KDVは同一content、format、worker設定のpaged Office sourceについて、session内のnavigation、resize、再frame取得でoffice2pdf変換を再実行してはならない（MUST NOT）。

#### Scenario: PPTXのslideを切り替えて戻る
- **WHEN** hostが同一sessionで複数slideを表示して既表示slideへ戻る
- **THEN** KDVは既存artifact/frame cacheを再利用し、convert stageを再実行しない

### Requirement: 実PPTX corpusのfirst-frame分布とRSSを記録しなければならない
KDVは各supplied PPTXについてcold first-frameのp50/p95、stage別時間、RSS deltaを記録し、10-cycle close後にartifact/cache/workerがbaselineへ戻ることを証明しなければならない（MUST）。

#### Scenario: supplied PPTXを反復測定する
- **WHEN** acceptance harnessが同一fixtureをcold open/frame/closeする
- **THEN** p50/p95、RSS delta、支配stageが記録され、unchanged sourceの再frame/resizeでconversionが再実行されない
