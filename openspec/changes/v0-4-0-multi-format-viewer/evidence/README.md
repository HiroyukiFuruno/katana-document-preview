# Multi-format feasibility evidence

このdirectoryはKDV `v0.4.0` engine選定の再現可能な証跡であり、
production runtime assetではない。

## Index

- `benchmark-summary.json`: scoring contract、candidate score、timing、memory、
  security、distributionの集約値
- `libreoffice-loaded-images.json`: Office変換中のprocess image検査結果
- `onlyoffice-document-builder.json`: 軽量Office engineのasset、dependency、性能、
  security、license、visual評価
- `office2pdf.json`: pure Rust Office -> Typst PDF engineのformat別品質、asset、
  supply chain、性能、8 GB expansion failureとsecurity要件
- `additional-native-candidates.json`: `rwml`、SlideGlance、IronCalc、
  BetterOffice、Pandoc + Typst、docMentis WASMの追加比較
- `fixture-manifest.json`: fixture SHA-256、OOXML entry数、展開後byte数
- `cargo-supply-chain.json`: Hayro / office_oxide dependency closureとlicense
- `selected-runtime-supply-chain.json`: production採用したoffice2pdf / IronCalcの
  dependency closure、license欠落、native link、build script一覧
- `linux-sandbox-supply-chain.json`: Linux Office workerのLandlock / seccomp二重遮断、
  direct crate pin、license、実動作検証条件
- `metrics/pdf-hayro.json`: KDV既存13 page PDFのHayro測定
- `metrics/*-hayro-poppler-diff.json`: 同一geometryでのraster差分
- `metrics/office-oxide-*.json`: pure Rust readable profileのIR/HTML測定
- `metrics/libreoffice-*-hayro.json`: Office -> PDF後のHayro測定
- `screenshots/`: representative page/sheet/slide画像と独立reference
  `onlyoffice-*-evaluation.png`は無償版watermarkと表示差分を含む不採用証跡。
  `office2pdf-*`、`rwml-*`、`pandoc-typst-*`はpure Rust/lightweight候補の
  actual outputであり、trusted referenceではない。

## Reproduction

1. `scripts/feasibility/generate-multi-format-corpus.py`でfixtureを再生成する。
2. `tools/multi-format-feasibility/`をrelease buildする。
3. PDF/Office出力を同一page geometryでrenderする。
4. `scripts/feasibility/compare-raster-corpus.py`でreferenceとの差分を測定する。
5. `scripts/feasibility/audit-cargo-metadata.py`でdependency closureを再生成する。
6. `just multi-format-scorecard-check`で固定100点rubric、score内訳、
   hard gate、採否、4 formatの提案profileを検証する。

`just multi-format-scorecard-script-test`は閾値変更、score合計不一致、
hard gate未達候補の推奨、未承認releaseを拒否するself-testである。
`release-contract-check`は同じvalidatorを`--require-approved`で実行し、
4 formatすべてについて明示承認、80点以上、全hard gate pass、release-approved
decisionが揃うまで失敗する。KDVのpreflight、isolated worker、KUC adapterの
契約テスト完了後に4 profileは明示承認済みであり、release gateは
`--require-approved`でも成功しなければならない。

macOS arm64以外のLibreOffice runtime、sandbox、hashは未検証であり、
artifactの存在確認だけを完了扱いにしてはならない。

`office2pdf`はmacOS arm64 upstream binaryのhashとruntimeを確認したが、
upstream binaryはad-hoc署名かつnotarizeされていない。productionではbinaryを
取得実行せず、KDV workerをKatanA release buildの一部として署名する。
