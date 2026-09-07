## ADDED Requirements

### Requirement: DOCX/XLSX fidelityのsource rendererを固定しなければならない
KDVはDOCX/XLSXの客観fidelity scoreを採用する前に、source renderer名、version、reference viewport、fixture SHA-256、生成コマンドを固定しなければならない（MUST）。

#### Scenario: reference artifactを更新する
- **WHEN** source rendererまたはfixtureを更新する
- **THEN** harnessはrenderer/version、viewport、fixture hash、reference artifact hashを記録し、自己比較をreferenceとして採用しない

### Requirement: DOCX/XLSXのelementとgeometryを数値化しなければならない
KDVはtext、font、border、fill、merged cell、row/column geometry、pagination/worksheet構造について、source rendererとの差分とmissing element数を数値化しなければならない（MUST）。

#### Scenario: representative Office fixtureを比較する
- **WHEN** 同一fixtureをsource rendererとKDVで描画する
- **THEN** DOCX/XLSX別にbaselineとcandidate score、element missing count、bbox/track/page/worksheet deltaを保存し、記録済みtoleranceを超える回帰をfailする

### Requirement: Owner boundaryを保持しなければならない
KDVは比較結果からOffice conversion、artifact、font、layout、grid mappingのowner-layerを改善してよいが、KatanA固有styleで差分を補正してはならない（MUST NOT）。

#### Scenario: KatanAがfixtureを表示する
- **WHEN** KatanAが公開KDVのframeをthin projectionする
- **THEN** fidelity評価と改善はKDV/office2pdf owner layerに残り、KatanAはrenderer固有の座標・style補正を持たない

### Requirement: KDVはXLSX border metadataを公開frameまで保持しなければならない
KDVはXLSX source artifactに存在するleft/right/top/bottom borderのstyleとcolorを、worker artifactから公開grid frameまで保持しなければならない（MUST）。KUCが個別borderを描画できない間も、KDVはmetadataを捨てたりKatanA固有補正で視覚差を隠したりしてはならない（MUST NOT）。

#### Scenario: XLSXにcell borderがある
- **WHEN** source XLSXのセルにborder styleまたはcolorがある
- **THEN** KDV candidate captureは同一cellのborder metadataを公開し、harnessはmetadata欠落とvisual projection未対応を別の数値として保存する
