## ADDED Requirements

### Requirement: Document sessionの保持資源に上限を設けなければならない
KDVはrendered page、grid cell、decoded artifactのcacheを個数とbytesの明示上限内に保たなければならない（MUST）。

#### Scenario: 多数のslideを連続表示する
- **WHEN** hostがcache上限を超えるslide/pageを順に表示する
- **THEN** KDVは再利用順に古いentryを解放し、保持個数とbytesが上限を超えない

### Requirement: CloseとDropでworkerとcacheを解放しなければならない
KDVは明示closeまたはsession Drop後にworker process、temporary workspace、frame、texture source、artifact cacheを解放しなければならない（MUST）。

#### Scenario: Sessionを明示closeする
- **WHEN** hostがOffice document sessionをcloseする
- **THEN** workerはbounded時間内に終了し、workspaceとcacheのlive countはbaselineへ戻る

#### Scenario: CloseせずsessionをDropする
- **WHEN** hostが未close sessionをDropする
- **THEN** KDVは同じcleanupを実行し、次のdocument openを妨げない

### Requirement: 混在文書の反復切替で資源が増加し続けてはならない
KDVはHTML以外のPDF/DOCX/XLSX/PPTXを十回open/frame/closeしてもworker数、cache entry、owned bytesが単調増加してはならない（MUST NOT）。

#### Scenario: OfficeとPDFを十回切り替える
- **WHEN** acceptance harnessがPDF/DOCX/XLSX/PPTXを十回open/frame/closeする
- **THEN** 最終live resource countは開始baselineへ戻り、RSS deltaは設定budget内に収まる
