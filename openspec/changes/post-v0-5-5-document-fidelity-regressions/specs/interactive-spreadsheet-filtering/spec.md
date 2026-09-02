## ADDED Requirements

### Requirement: XLSX AutoFilterを型付きmetadataとして保持しなければならない
KDVはXLSX worksheetのAutoFilter範囲、対象列、対応criteria、未対応criteria診断をneutral spreadsheet artifactとして保持しなければならない（MUST）。

#### Scenario: AutoFilterを持つworksheetを開く
- **WHEN** hostがAutoFilter定義を持つXLSXを開く
- **THEN** KDVはsheet metadataにheader範囲と列ごとのfilter状態を返す
- **THEN** KDVはKUCまたはKatanA固有型を公開artifactへ含めない

#### Scenario: 未対応criteriaを含む
- **WHEN** worksheetがKDV未対応のcustom/dynamic criteriaを含む
- **THEN** KDVはそのcriteriaを黙って適用せずtyped diagnosticを返す

### Requirement: Filter候補と適用をtyped command/eventで提供しなければならない
KDVは列の候補値取得、選択値適用、filter解除をtyped commandとして受け、適用結果をtyped eventと表示行状態で返さなければならない（MUST）。

#### Scenario: 候補値を選択して絞り込む
- **WHEN** hostがfilter列の候補値集合を指定する
- **THEN** KDV workerは元row indexを維持して一致行だけをvisibleにする
- **THEN** KDVは可視行数と適用列をevent/frame metadataへ返す

#### Scenario: Filterを解除する
- **WHEN** hostがsheetのfilter解除commandを送る
- **THEN** KDVは元からhiddenのrowを除きfilter起因の非表示を解除する

#### Scenario: 文字列・数値・空白・複数値を適用する
- **WHEN** hostが実XLSX fixtureの文字列、数値、空白、同一列の複数値をtyped commandで指定する
- **THEN** KDVは元row indexを保ったvisible-row stateとgrid frame metadataを返し、Clear後はfilter起因の非表示を残さない

### Requirement: Filterは大規模sheetでもworker境界とresource limitを維持しなければならない
KDVはfilter評価を隔離spreadsheet worker内で行い、全cellをhostへ転送せず既存resource limitを維持しなければならない（MUST）。

#### Scenario: 大規模sheetを絞り込む
- **WHEN** filter範囲が通常のmaterialize上限を超える
- **THEN** KDVはbounded worker処理でvisibilityを返し、host IPCへ全rowの全cellを送らない
