## Why

KDV `v0.3.x` のbrowser session adapterが公開済み `v0.3.5` で完了したため、
次のminor releaseでは、以前から計画されていたPDFとMicrosoft Office文書のviewerを
優先する。

KRRの正本仕様はCSV / PDF / Word / Excel / PPTX viewer renderingをKDVへ移譲済みで
あり、KRRへdocument parser、Office layout engine、PDF viewer APIを追加してはならない。
KDVは成熟した既存engine/libraryをadapter経由で利用し、PDF parser、Office parser、
layout engineを独自実装しない。

このchangeはKDV `v0.4.0` をPDF / DOCX / XLSX / PPTX viewerへ繰り上げる。
PDF export paginationは独立したKDV `v0.5.0` changeへ繰り延べる。

## What Changes

### Feasibility decision gate

- PDF / DOCX / XLSX / PPTXごとに、目標品質、既存engine、license、配布方式、
  初回表示時間、メモリ、unsupported機能を比較する。
- Chromium、WebView、PDFium、独自parser、独自Office layout engineを候補から除外する。
- Rust-firstを維持し、PDF / DOCXの`static-page`、XLSXの`interactive-grid`、
  PPTXの`static-slide`を別の表示契約として評価する。
- engineを採用する前にfixture corpusとreference imageを使った機械比較を行い、
  結果をユーザーへ提示して承認を得る。

### PDF viewer

- KDV adapterが選定済みPDF engineを呼び、page artifactとdiagnosticsを作る。
- KDVはsource identity、page navigation、zoom、fit、selection/copy/open commandを持つ。
- KUCは解釈済みpage artifactとcontrolsを表示する。

### Office viewer（DOCX / XLSX / PPTX）

- DOCXはcanonical PDFを介した`static-page`、XLSXはcell modelを使う
  `interactive-grid`、PPTXは`static-slide`としてformat別に評価する。
- DOCXは文書ページ、XLSXはsheet、PPTXはslide単位のnavigationを提供する。
- macro実行、外部link自動取得、Office再計算、編集、PPTX animation再生は行わない。
- unsupported chart / pivot / active contentはsilent fallbackせずtyped diagnosticsにする。
- 2D virtualized gridが必要な場合だけKUCの別changeへhandoffする。

### Deferred

- CSV / SVG / WebP / AVIF viewerは本releaseへ混ぜず、後続changeへ残す。
- PDF export paginationはKDV `v0.5.0` へ繰り延べる。

## Capabilities

### New Capabilities

- `multi-format-viewer-boundary`: PDF / Office viewerのengineと品質境界を確定する
- `pdf-viewer`: PDFページ表示・ナビゲーション
- `office-viewer`: DOCX / XLSX / PPTXの内容表示

## Impact

- `crates/katana-document-viewer/` — format adapter、neutral artifact、viewer state、diagnostics
- KUC bridge — page/sheet/slide viewportとgeneric controls
- `katana-ui-core` — 公開済みv0.2.0契約のgeneric 2D gridと既存surface/controlを利用する
- `katana-document-viewer-kuc` — KDV neutral artifactをKUC表示modelへ変換する公開adapter crateを追加する
- KatanA — published KDVを利用するthin host integration
- `katana-render-runtime` — 変更なし
