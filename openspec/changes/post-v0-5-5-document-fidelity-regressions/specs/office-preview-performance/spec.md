## ADDED Requirements

### Requirement: Office初回表示のstage時間を診断可能にしなければならない
KDVは`DEBUG=true`の場合だけpreflight、worker spawn、convert、artifact decode、first frameの経過時間とsource identityを出力しなければならない（MUST）。

#### Scenario: DEBUGを有効にしてPPTXを開く
- **WHEN** `DEBUG=true`でPPTX sessionを開きfirst frameを取得する
- **THEN** 各stageの経過時間が同一session/sourceへ関連付けて出力される

#### Scenario: 通常のrelease実行を行う
- **WHEN** `DEBUG`が未設定またはfalseである
- **THEN** stage traceは出力されず、表示結果と制御フローは変わらない

### Requirement: 不変sourceを不要に再変換してはならない
KDVは同一content、format、worker設定のpaged Office sourceについて、session内のnavigation、resize、再frame取得でoffice2pdf変換を再実行してはならない（MUST NOT）。

#### Scenario: PPTXのslideを切り替えて戻る
- **WHEN** hostが同一sessionで複数slideを表示して既表示slideへ戻る
- **THEN** KDVは既存artifact/frame cacheを再利用し、convert stageを再実行しない
