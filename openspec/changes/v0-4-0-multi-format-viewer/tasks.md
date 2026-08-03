# Tasks: katana-document-viewer v0.4.0 multi-format viewer

## Definition of Ready (DoR)

- [x] KDV browser session adapterは公開済み `v0.3.5` で完了している
- [x] KRRの正本仕様はPDF / Word / Excel / PPTX viewer renderingをKDVの責務として固定している
- [x] KUC `v0.3.0` のgeneric 2D grid-line visibilityを公開crate/tagから確認している。証跡: GitHub Release `v0.3.0` / crates.io `katana-ui-core 0.3.0` / tag commit `1256fdd08ecc01bcc09066180e1a05d0503ba382`
- [x] PDF export paginationを独立したKDV `v0.5.0` targetへ繰り延べている
- [x] formatごとのengine、quality profile、dependency modelを比較し、ユーザーの明示承認を得ている。証跡: file: `openspec/changes/v0-4-0-multi-format-viewer/evidence/benchmark-summary.json`

## Definition of Done (DoD)

- [x] PDF / DOCX / XLSX / PPTXを承認済みquality profileで表示できる
- [x] 独自parser、独自layout engine、Chromium、WebView、PDFiumを利用していない
- [x] KRRにdocument viewer APIを追加せず、KDVがformat adapterとviewer semanticsを所有している
- [x] macro / script非実行、external resource blocking、resource limits、crash isolationを検証している。証跡: test: `multi_format_office_preflight_contract` / file: `openspec/changes/v0-4-0-multi-format-viewer/evidence/office2pdf.json`
- [x] unsupported featureとfailureがtyped capability / diagnosticsとして追跡可能である
- [x] strict coverage 100% / uncovered 0を閾値緩和・除外追加なしで満たしている
- [x] macOS / Linux / Windowsのrelease artifactとdependency supply chainを検証している。証跡: test: GitHub Actions PR #31 macOS / Ubuntu / Windows jobs / URL: `https://github.com/HiroyukiFuruno/katana-document-viewer/pull/31/checks`
- [x] `rtk ./scripts/openspec validate v0-4-0-multi-format-viewer --strict --no-interactive` が通る
- [x] `rtk just check` とKDV `v0.4.0` release gateが通る。証跡: test: `just-check` / test: `v0.4.0-release-check`

---

## 1. Feasibility gate

- [x] 1.1 PDF / DOCX / XLSX / PPTXのrepresentative corpusとtrusted referenceを固定する
- [x] 1.2 PDFは `hayro`、XLSXは `calamine`、DOCXは `docx-rs` 相当を含むpure Rust候補を比較する
- [x] 1.3 Office static layoutはsandboxed LibreOffice変換を含む既存engine候補を比較する
- [x] 1.4 Chromium / WebView / PDFium / hand-written parser・layoutを候補から除外し、理由を記録する
- [x] 1.5 reference image差分、first frame、navigation latency、peak memory、cache sizeを測定する
- [x] 1.6 direct / transitive license、cross-platform配布、security update経路を監査する
- [x] 1.7 macro非実行、external resource blocking、archive expansion / time / memory limit、cleanupを検証する。証跡: file: `openspec/changes/v0-4-0-multi-format-viewer/evidence/office2pdf.json` / file: `openspec/changes/v0-4-0-multi-format-viewer/evidence/onlyoffice-document-builder.json` / file: `openspec/changes/v0-4-0-multi-format-viewer/evidence/libreoffice-loaded-images.json`
- [x] 1.8 PDF / DOCX採用候補とXLSX / PPTXのformat別profile選択肢をscore・hard gate付きで提示する。証跡: file: `openspec/changes/v0-4-0-multi-format-viewer/feasibility.md` / file: `openspec/changes/v0-4-0-multi-format-viewer/evidence/benchmark-summary.json`
- [x] 1.9 fixed score、hard gate、採否、4 format selectionをlocal/release gateで機械検証する。証跡: `scripts/feasibility/verify-multi-format-scorecard.py` / `rtk just multi-format-scorecard-script-test` / `rtk just multi-format-scorecard-check`
- [x] 1.10 ユーザーの明示承認後にだけproduction dependencyとformat実装へ進む。証跡: file: `openspec/changes/v0-4-0-multi-format-viewer/evidence/benchmark-summary.json`

---

## 2. Neutral viewer contract

- [x] 2.1 承認済みformatだけに `ViewerSource::Pdf` と `ViewerSource::Office` を追加する
- [x] 2.2 source identity、revision、MIME、format、capability、typed diagnosticsを定義する
- [x] 2.3 PDF page、Office document / sheet / slideのneutral artifactを定義する
- [x] 2.4 navigation、index jump、zoom、fit、copy、openのneutral commandを定義する
- [x] 2.5 engine固有型、KUC型、KatanA型をKDV public APIへ露出しない
- [x] 2.6 unsupported format / featureをsilent fallbackせずtyped diagnosticsへ落とす

---

## 3. PDF viewer

- [x] 3.1 承認済みPDF engineをprivate adapterで統合する
- [x] 3.2 page count、geometry、rotation、crop、rendered pageをneutral artifactへ変換する
- [x] 3.3 前後移動、page index jump、zoom、fitを提供する
- [x] 3.4 link / text selectionはengine capabilityに従って有効化し、未対応を明示する
- [x] 3.5 password protected、corrupt、resource limit超過をtyped diagnosticsで区別する
- [x] 3.6 representative corpusのreference diffと性能budgetを契約テストへ固定する

---

## 4. Office viewer

- [x] 4.1 DOCX / XLSX / PPTXを承認済みformat別profileで統合する
- [x] 4.2 DOCX document/page、XLSX sheet、PPTX slideのneutral artifactを定義する
- [x] 4.3 profile名、対応機能、unsupported featureをcapabilityへ記録する
- [x] 4.4 DOCXは `office2pdf` -> canonical PDF -> `hayro`をKDV private adapterとして統合する
- [x] 4.5 macro / scriptを実行せず、external resourceを自動取得しない
- [x] 4.6 bounded OOXML preflightでentry数、圧縮率、展開量、active content、external relationshipを検査する
- [x] 4.7 Office engineを別processへ隔離し、network deny、dedicated temp、timeout、memory limit、kill、cleanup、crash isolationを実装する。delegation-exception: `ユーザーがsubagent利用を禁止` / file: `openspec/changes/v0-4-0-multi-format-viewer/evidence/linux-sandbox-supply-chain.json` / test: `multi_format::office_worker_constraints::network_seccomp::tests`
- [x] 4.8 XLSX `interactive-grid`が承認された場合だけIronCalc model adapterを統合する
- [x] 4.9 PPTX chart fallback profileが承認された場合だけ `office2pdf` static slide adapterを統合する
- [x] 4.10 representative corpusのreference diffと性能budgetをformat別契約テストへ固定する
- [x] 4.11 高圧縮DOCXの約8 GB memory回帰をpreflightまたはprocess limitで拒否する契約テストを追加する
- [ ] 4.12 Windows AppContainer workerをdocument専用workspaceへstageし、workspace / input / staged workerへ明示ACLを付与した実Windows起動を検証する。delegation-exception: `ユーザーがsubagent利用を禁止` / 証跡: PR作成後のWindows CI実行URLと実AppContainer契約テスト結果を追記する

---

## 5. KUC bridge and conditional handoff

- [x] 5.1 KUCの既存page viewport、virtualized list、image surface、slide controls、generic 2D gridを再利用する
- [x] 5.2 KDV document surfaceはKUCを内部利用し、format semanticsまたはKUC型をKatanAへ露出しない
- [x] 5.3 XLSX profileが2次元virtualized gridを必要とする場合だけKUCの別OpenSpecを作成する
- [x] 5.4 KUC `v0.3.0` 公開release完了後にregistry dependencyとして取り込む
- [x] 5.5 private table/grid代替実装をKDVへ追加していないことを機械検証する。証跡: test: `document-surface-boundary-check` / file: `scripts/document-surface-boundary-check.sh`
- [x] 5.6 PPTX chart fallbackはKDV typed diagnosticとして保持し、KUCにOffice semanticsを追加しない
- [x] 5.7 KDV coreの`egui` optional featureがKUCを内部利用するdocument surfaceを所有し、KatanAがKUCへ直接依存またはformat別presentation変換を持たないようにする
- [x] 5.8 公開featureはKUC core crates.io `0.3.0`、開発用Storybook一式は同一 `v0.3.0` tagを使用し、sibling path dependencyを禁止する。証跡: file: `crates/katana-document-viewer/Cargo.toml` / file: `Cargo.toml` / test: `document-surface-boundary-check`
- [x] 5.9 XLSX sheetのgrid-line visibilityをKUC typed render propsへ欠落なく渡す

---

## 6. Ownership and release gates

- [x] 6.1 source usageとdependency declarationの両方でKRR document viewer依存がないことを検査する
- [x] 6.2 Chromium / WebView / PDFiumおよび禁止engineの依存がないことを検査する
- [x] 6.3 `multi-format-viewer` release contractを実装し、current target `v0.4.x` を機械検証する。証跡: test: `release-contract-check` / file: `scripts/release/verify-release-contract.py`
- [x] 6.4 PDF export paginationだけがdeferred `v0.5.0` であることを機械検証する。証跡: file: `openspec/release-targets.json` / file: `openspec/changes/v0-5-0-pdf-export-pagination/tasks.md`
- [x] 6.5 strict coverage 100% / uncovered 0、AST lint、clippy、package verify、publish dry-runを通す
- [x] 6.6 macOS / Linux / Windowsのartifactとruntime dependencyを検証する。証跡: test: GitHub Actions PR #31 macOS / Ubuntu / Windows jobs / URL: `https://github.com/HiroyukiFuruno/katana-document-viewer/pull/31/checks`
- [x] 6.7 Linux CIでtest moduleをproduction coverage対象から分離し、親子process profileを保持したままstrict coverage 100% / uncovered 0を再通過する。証跡: release-preflight run `30773465236`
- [x] 6.8 `v0.4.0` core crateの公開と別presentation crateの403およびcross-layer依存を検出し、別crateを削除した`v0.4.1` KDV document surfaceへrelease contractを修正する
- [x] 6.9 KDV/KUC混成crateの不存在、KUC型のpublic API非露出、`KatanA -> KDV -> KUC/KRR`の依存方向をrelease gateへ固定する。証跡: file: `scripts/document-surface-boundary-check.sh` / test: `document-surface-boundary-check`
- [x] 6.10 KDV `v0.4.1` strict gate、GitHub Release、crates.io publicationを確認する。証跡: test: GitHub Release `v0.4.1` / URL: `https://github.com/HiroyukiFuruno/katana-document-viewer/releases/tag/v0.4.1` / crate: `katana-document-viewer 0.4.1`
- [ ] 6.11 Windows AppContainer回帰を修正したKDV `v0.4.2`のstrict gate、GitHub Release、crates.io publicationを確認する

---

## 7. KatanA handoff

- [ ] 7.1 KDV `v0.4.2` 公開後にKatanAのadjacent patch OpenSpecへregistry versionとcapabilityを引き渡す
- [x] 7.2 KatanAがformat parser / rendererを持たないことをhandoff条件にする
- [x] 7.3 PDF / DOCX / XLSX / PPTXのfile/URL intake、navigation、diagnosticsのacceptance corpusを引き渡す

---

## 8. Final verification

- [x] 8.1 `rtk ./scripts/openspec validate v0-4-0-multi-format-viewer --strict --no-interactive` を実行する
- [x] 8.2 `rtk just check` を実行する
- [x] 8.3 KDV `v0.4.0` release gateを実行する。証跡: test: `v0.4.0-release-check`
- [ ] 8.4 engine選定、quality profile、unsupported機能、security evidenceをユーザーへ報告する
- [ ] 8.5 KDV `v0.4.2` release gateとWindows実worker acceptanceを実行する。delegation-exception: `ユーザーがsubagent利用を禁止` / 証跡: release-check結果、Windows CI実行URL、実AppContainer契約テスト結果を追記する

<!-- subagent-spark-harness-strict-start -->
- delegation-exception: `ユーザーがsubagent利用を禁止`
