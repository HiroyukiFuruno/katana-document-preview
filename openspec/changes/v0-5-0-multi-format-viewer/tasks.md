# Tasks: katana-document-viewer v0.5.0 backend-neutral multi-format viewer

## Priority and boundary invariant

- 正しい設計と実装境界の確定・維持を、検証、ゴール達成、リリース速度より優先する。
- KUCは汎用UI Frameworkとしてlayout、geometry、hit-test、interaction stateを所有する。
- KDV/KLEはKUCに依存するdomain libraryであり、application UI backendへ依存しない。
- KDVはKUC型をpublic APIへ露出せず、KDV所有のframe / command / eventを公開する。
- KatanAはKDVだけを直接利用し、現行egui input / paintを中立契約へ投影する。KatanAでdocument geometryやhit-testを再実装しない。
- この不変条件がOpenSpec、public API、依存graph、自動guardで一致するまでpublish / releaseへ進まない。

## 0. Boundary design gate

- [x] 0.1 KUC / KDV / KLE / KatanAの所有権をdesign、spec、project正本で一致させる。証跡: file: `design.md` / file: `specs/multi-format-viewer/spec.md` / file: `openspec/project.md`
- [x] 0.2 KDV public APIにKUC型またはapplication backend型が露出しないことを検証する。証跡: test: `dependency_tests` / command: `rtk just release-check`
- [x] 0.3 KDV/KatanAにKUCのlayout、geometry、hit-test、interaction stateを再実装させない設計とsource guardを定義する。KatanA実装への適用確認は7.4で行う。証跡: file: `design.md` / file: `scripts/document-surface-boundary-check.sh` / KatanA file: `scripts/release/check-multi-format-document-contract.py`
- [x] 0.4 KatanAのdocument viewerがKDV以外のKUC / KRR / document engineへ直接依存しない設計とdependency guardを定義する。KatanA実装への適用確認は7.4で行う。証跡: file: `handoff.md` / KatanA file: `scripts/release/check-multi-format-document-contract.py`
- [x] 0.5 上記4項目の設計レビュー、自動guard定義、ユーザー承認をrelease task開始条件として満たす。証跡: user approval `2026-08-09` / command: `rtk just release-check`
- [x] 0.6 統一sessionのtyped command/event/info/close是正案についてユーザーの明示承認を得る。証跡: user approval `2026-08-09`

## Definition of Ready (DoR)

- [x] KDV browser session adapterは公開済み `v0.3.5` で完了している
- [x] KRRの正本仕様はPDF / Word / Excel / PPTX viewer renderingをKDVの責務として固定している
- [x] KUC `v0.3.0` のgeneric 2D grid-line visibilityを公開crate/tagから確認している。証跡: GitHub Release `v0.3.0` / crates.io `katana-ui-core 0.3.0` / tag commit `1256fdd08ecc01bcc09066180e1a05d0503ba382`
- [x] PDF export paginationを独立したKDV `v0.6.0` targetへ繰り延べている
- [x] formatごとのengine、quality profile、dependency modelを比較し、ユーザーの明示承認を得ている。証跡: file: `openspec/changes/v0-5-0-multi-format-viewer/evidence/benchmark-summary.json`

## Definition of Done (DoD)

- [x] PDF / DOCX / XLSX / PPTXを承認済みquality profileで表示できる
- [x] 独自parser、独自layout engine、Chromium、WebView、PDFiumを利用していない
- [x] KRRにdocument viewer APIを追加せず、KDVがformat adapterとviewer semanticsを所有している
- [x] macro / script非実行、external resource blocking、resource limits、crash isolationを検証している。証跡: test: `multi_format_office_preflight_contract` / file: `openspec/changes/v0-5-0-multi-format-viewer/evidence/office2pdf.json`
- [x] unsupported feature、非対応command、failureがtyped capability / diagnostics / errorとして追跡可能である。証跡: test: `multi_format_document_session_contract` / error: `DocumentSessionError::UnsupportedCommand`
- [x] strict coverage 100% / uncovered 0を閾値緩和・除外追加なしで満たしている
- [x] macOS / Linux / Windowsのrelease artifactとdependency supply chainを検証している。証跡: test: GitHub Actions PR #31 macOS / Ubuntu / Windows jobs / URL: `https://github.com/HiroyukiFuruno/katana-document-viewer/pull/31/checks`
- [x] `rtk ./scripts/openspec validate v0-5-0-multi-format-viewer --strict --no-interactive` が通る
- [x] `rtk just check` とKDV `v0.4.0` release gateが通る。証跡: test: `just-check` / test: `v0.4.0-release-check`
- [x] KDVがegui/eframeへ依存せず、中立frame契約を公開した`v0.5.0`のstrict gateと公開確認が完了している。証跡: command: `gh release view v0.5.0` / command: `cargo info katana-document-viewer@0.5.0 --registry crates-io` / URL: `https://github.com/HiroyukiFuruno/katana-document-viewer/releases/tag/v0.5.0` / URL: `https://crates.io/crates/katana-document-viewer/0.5.0` / URL: `https://github.com/HiroyukiFuruno/katana-document-viewer/pull/35`

---

## 1. Feasibility gate

- [x] 1.1 PDF / DOCX / XLSX / PPTXのrepresentative corpusとtrusted referenceを固定する
- [x] 1.2 PDFは `hayro`、XLSXは `calamine`、DOCXは `docx-rs` 相当を含むpure Rust候補を比較する
- [x] 1.3 Office static layoutはsandboxed LibreOffice変換を含む既存engine候補を比較する
- [x] 1.4 Chromium / WebView / PDFium / hand-written parser・layoutを候補から除外し、理由を記録する
- [x] 1.5 reference image差分、first frame、navigation latency、peak memory、cache sizeを測定する
- [x] 1.6 direct / transitive license、cross-platform配布、security update経路を監査する
- [x] 1.7 macro非実行、external resource blocking、archive expansion / time / memory limit、cleanupを検証する。証跡: file: `openspec/changes/v0-5-0-multi-format-viewer/evidence/office2pdf.json` / file: `openspec/changes/v0-5-0-multi-format-viewer/evidence/onlyoffice-document-builder.json` / file: `openspec/changes/v0-5-0-multi-format-viewer/evidence/libreoffice-loaded-images.json`
- [x] 1.8 PDF / DOCX採用候補とXLSX / PPTXのformat別profile選択肢をscore・hard gate付きで提示する。証跡: file: `openspec/changes/v0-5-0-multi-format-viewer/feasibility.md` / file: `openspec/changes/v0-5-0-multi-format-viewer/evidence/benchmark-summary.json`
- [x] 1.9 fixed score、hard gate、採否、4 format selectionをlocal/release gateで機械検証する。証跡: `scripts/feasibility/verify-multi-format-scorecard.py` / `rtk just multi-format-scorecard-script-test` / `rtk just multi-format-scorecard-check`
- [x] 1.10 ユーザーの明示承認後にだけproduction dependencyとformat実装へ進む。証跡: file: `openspec/changes/v0-5-0-multi-format-viewer/evidence/benchmark-summary.json`

---

## 2. Neutral viewer contract

- [x] 2.1 承認済みformatだけに `ViewerSource::Pdf` と `ViewerSource::Office` を追加する
- [x] 2.2 source identity、revision、MIME、format、capability、typed diagnosticsを定義する
- [x] 2.3 PDF page、Office document / sheet / slideのneutral artifactを定義する
- [x] 2.4 navigation、index jump、zoom、fit、copy、openのneutral commandと結果eventを定義する。証跡: type: `DocumentSessionCommand` / type: `DocumentSessionEvent`
- [x] 2.5 engine固有型、KUC型、KatanA型をKDV public APIへ露出しない
- [x] 2.6 unsupported format / feature / commandをsilent successまたはfallbackせずtyped diagnostics / errorへ落とす。証跡: test: `multi_format_document_session_contract`
- [x] 2.7 統一sessionがsource metadataを返す`info`と明示的な`close`を提供する。証跡: API: `DocumentSession::info` / API: `DocumentSession::close`

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
- [x] 4.7 Office engineを別processへ隔離し、network deny、dedicated temp、timeout、memory limit、kill、cleanup、crash isolationを実装する。delegation-exception: `ユーザーがsubagent利用を禁止` / file: `openspec/changes/v0-5-0-multi-format-viewer/evidence/linux-sandbox-supply-chain.json` / test: `multi_format::office_worker_constraints::network_seccomp::tests`
- [x] 4.8 XLSX `interactive-grid`が承認された場合だけIronCalc model adapterを統合する
- [x] 4.9 PPTX chart fallback profileが承認された場合だけ `office2pdf` static slide adapterを統合する
- [x] 4.10 representative corpusのreference diffと性能budgetをformat別契約テストへ固定する
- [x] 4.11 高圧縮DOCXの約8 GB memory回帰をpreflightまたはprocess limitで拒否する契約テストを追加する
- [x] 4.12 Windows AppContainer profile folder内のdocument専用workspaceへworkerをstageし、workspace / input / staged workerへ明示ACLを付与した実Windows起動を検証する。delegation-exception: `ユーザーがsubagent利用を禁止` / 証跡: CI run `30868560652` / Windows job `91865557624` / test: `Run Windows AppContainer worker acceptance` success / URL: `https://github.com/HiroyukiFuruno/katana-document-viewer/actions/runs/30868560652/job/91865557624`
- [x] 4.13 Windows AppContainer起動へ親processの完全なUnicode環境blockをcase-insensitive順で渡し、`TEMP` / `TMP`だけをdocument workspaceへ置換する。delegation-exception: `ユーザーがsubagent利用を禁止` / 証跡: failure: CI run `30865519677` / `CreateProcessW 0x800700CB` / file: `windows_worker_profile.rs` / test: `just check`, `just coverage`, `v0.4.2-release-check`, `cargo xwin check`
- [x] 4.14 Windows verbatim pathの `?` をURL queryとして切断せず、direct imageの拡張子判定とfile URI生成を正規化する。delegation-exception: `ユーザーがsubagent利用を禁止` / 証跡: failure: CI run `30869935308` / Windows job `91869713856` / test: `windows_verbatim_image_source_keeps_extension_and_valid_file_uri`, `windows_verbatim_document_path_still_plans_direct_image_asset`, `loader_materializes_visible_direct_image_asset`
- [x] 4.15 同一process内の並列document openで永続Windows AppContainer profileを重複作成せず共有し、失敗時は再試行可能なまま各document workspaceを分離する。delegation-exception: `ユーザーがsubagent利用を禁止` / 証跡: failure: CI run `30874173658` / Windows job `91882129198` / test: `multi_format_office_worker_contract`
- [x] 4.16 KDV linterのsource scopeとtest path判定をpath separator非依存にし、Windowsでruleが偽陰性にならない契約テストを追加する。delegation-exception: `ユーザーがsubagent利用を禁止` / 証跡: failure: CI run `30876360020` / Windows job `91888519478` / test: `path_matching_is_separator_independent`, `kdv-linter --lib`
- [x] 4.17 KDV linterのworkspace length baselineをOS非依存な正規化相対パスで照合し、Windowsの混在separatorでも既存baselineだけを許可する。delegation-exception: `ユーザーがsubagent利用を禁止` / 証跡: failure: CI run `30878695303` / Windows job `91895238013` / test: `contains_windows_mixed_separator_file_length_debt`, `contains_windows_mixed_separator_function_length_debt`, `rejects_windows_workspace_prefix_collision`
- [x] 4.18 Storybookのdirect image source契約をWindows verbatim pathの文字列表現へ依存させず、正規化document IDが同じ実ファイルへ解決でき、`//?/`を外部へ漏らさないことを検証する。delegation-exception: `ユーザーがsubagent利用を禁止` / 証跡: failure: CI run `30881147000` / Windows job `91902623148` / test: `direct_image_fixture_source_uses_absolute_file_uri`, `relative_direct_image_fixture_source_uses_absolute_file_uri`, `windows_extended_drive_path_becomes_a_regular_document_id`
- [/] 4.19 Office workerへsource commit、SHA-256、OFL licenseを固定したCarlito / Noto Sans JPをstageし、既存office2pdf font search APIだけで3 OSの日本語glyphとtext box配置を再現可能にする。OOXML書換え・独自parser・独自layout補正は禁止する。delegation-exception: `ユーザーがsubagent利用を禁止` / 証跡: test: `pptx_isolated_worker_preserves_slide_profile_and_fallback_diagnostics` success with upstream fix commit / KatanA CI run `31354395814` のLinux配置崩れ / Windows日本語欠け字 / KDV CI run `31369299467`: Windows Calibri環境は約22px行間、CalibriなしのLinux / macOSは約11px行間 / root: `office2pdf 0.6.5`の`powerpoint_line_box_em`が`font_paths`を参照しない / 残件: 3 OS再検証
- [x] 4.20 `office2pdf`のPowerPoint paragraph line metricsがrenderingと同じfont fallback chainを使う上流修正を採用する。delegation-exception: `ユーザーがsubagent利用を禁止` / 証跡: test: `multi_format_office_worker_contract` `9/9` / upstream issue `#705` / PR `#745` / merge commit `c528eef467aaf9ca4873acf5c8bedb07b7ae5596` / maintenance commit `a80e63a3b4ab111df54aa809a525a76b7a25533c` / crates.io `office2pdf-katana 0.6.6` / GitHub Release `office2pdf-katana-v0.6.6` / registry checksum `bcb4241bb2edfa2e1a52f49ee4804c0b6fb6ff30b18124daece20166a5c98fe8` / KDV release contract success。上流`v0.6.5`へ`#745`と必要なfont resolution helperだけをbackportしたApache-2.0 maintenance crateをexact registry dependencyとして使用する。KDV内のOOXML書換え・独自font substitution parser・独自layout補正は禁止し、公式互換版公開後は後続KDV patchで公式crateへ戻す。
- [/] 4.21 公式`office2pdf 0.6.7`がPR `#745` merge commitを含むことをtag sourceで検証し、KDV `v0.5.3`でexact registry dependencyへ戻す。delegation-exception: `直列のクリティカルパス` / 証跡: test: `multi_format_office_worker_contract` 9/9 success / command: `rtk just coverage` success, functions `3021/3021`, lines `24728/24728`, uncovered `0` / upstream issue `#959` closed / GitHub Release `v0.6.7` tag commit `8f34766a1d1567b9d81d606e45ea690987a7c6ed` / compare `c528eef...v0.6.7` status `ahead`, behind `0` / crates.io `office2pdf 0.6.7` unyanked / checksum `0cd39889efe9f4bc36ea89ffc30ad5ecf7e1cd3a33b5af76604d54ca26f764c3` / 残件: 3 OS・公開確認
- [x] 4.22 外部OOXML relationshipを型別に扱い、標準hyperlinkだけはisolated workerへ原文のまま渡して表示を継続する。remote image、template、data connectionは従来どおりworker起動前に拒否し、workerのnetwork denyを維持する。OOXML書換え、独自parser、独自layout補正は禁止する。delegation-exception: `直列のクリティカルパス` / 証跡: `multi_format_office_preflight_contract` `10/10` / `multi_format_office_worker_contract` `10/10` / `rtk just coverage` success, functions `3024/3024`, lines `24747/24747`, uncovered `0`

---

## 5. KUC bridge and conditional handoff

- [x] 5.1 KUCの既存page viewport、virtualized list、image surface、slide controls、generic 2D gridを再利用する
- [x] 5.2 KDV document surfaceはKUCを内部利用し、format semanticsまたはKUC型をKatanAへ露出しない
- [x] 5.3 XLSX profileが2次元virtualized gridを必要とする場合だけKUCの別OpenSpecを作成する
- [x] 5.4 KUC `v0.3.0` 公開release完了後にregistry dependencyとして取り込む
- [x] 5.5 private table/grid代替実装をKDVへ追加していないことを機械検証する。証跡: test: `document-surface-boundary-check` / file: `scripts/document-surface-boundary-check.sh`
- [x] 5.6 PPTX chart fallbackはKDV typed diagnosticとして保持し、KUCにOffice semanticsを追加しない
- [x] 5.7 KDV coreからegui/eframe dependency、feature、host widgetを除去し、KUCを内部利用するbackend-neutral document frameを公開する。証跡: file: `crates/katana-document-viewer/Cargo.toml` / file: `document_surface/frame.rs` / test: `dependency_tests`, `document_surface_tests` / command: `rtk just release-check`
- [x] 5.8 KUC core crates.io `0.3.0`をrequired dependency、開発用Storybook一式を同一 `v0.3.0` tagとし、sibling path dependencyを禁止する。証跡: file: `crates/katana-document-viewer/Cargo.toml` / file: `Cargo.toml` / test: `document-surface-boundary-check` / command: `rtk just release-check`
- [x] 5.9 XLSX sheetのgrid-line visibilityをKUC typed render propsへ欠落なく渡す
- [x] 5.10 pointer inputをKDV commandとして受け、KUC hit-test / interactionへ委譲した結果を破棄せずKDV所有eventへ変換する。KUC `GridEvent`をKDV public APIへ露出しない。delegation-exception: `ユーザーがsubagent利用を禁止`
- [x] 5.11 KDV中立frameがKUCの計算したpage pixels、grid geometry/style、selection、scroll stateをKUC型なしで保持し、geometryを再計算しない。delegation-exception: `ユーザーがsubagent利用を禁止` / 証跡: test: `neutral_grid_frame_keeps_backend_independent_geometry`, `rendered_page_becomes_a_document_surface_without_reinterpreting_pixels` / command: `rtk just release-check`

---

## 6. Ownership and release gates

- [x] 6.1 source usageとdependency declarationの両方でKRR document viewer依存がないことを検査する
- [x] 6.2 Chromium / WebView / PDFiumおよび禁止engineの依存がないことを検査する
- [x] 6.3 `multi-format-viewer` release contractを実装し、current target `v0.5.x` を機械検証する。証跡: test: `release-contract-check` / file: `scripts/release/verify-release-contract.py`
- [x] 6.4 PDF export paginationだけがdeferred `v0.6.0` であることを機械検証する。証跡: file: `openspec/release-targets.json` / file: `openspec/changes/v0-6-0-pdf-export-pagination/tasks.md`
- [x] 6.5 command境界是正後にstrict coverage 100% / uncovered 0、AST lint、clippy、package verify、publish dry-runを再実行する。証跡: command: `rtk just release-check` / functions `3015/3015` / lines `24679/24679` / uncovered functions `0` / uncovered lines `0` / package success / publish dry-run success
- [x] 6.6 macOS / Linux / Windowsのartifactとruntime dependencyを検証する。証跡: test: GitHub Actions PR #31 macOS / Ubuntu / Windows jobs / URL: `https://github.com/HiroyukiFuruno/katana-document-viewer/pull/31/checks`
- [x] 6.7 Linux CIでtest moduleをproduction coverage対象から分離し、親子process profileを保持したままstrict coverage 100% / uncovered 0を再通過する。証跡: release-preflight run `30773465236`
- [x] 6.8 `v0.4.0` core crateの公開と別presentation crateの403およびcross-layer依存を検出し、別crateを削除した`v0.4.1` KDV document surfaceへrelease contractを修正する
- [x] 6.9 KDV/KUC混成crateの不存在、KUC型のpublic API非露出、`KatanA -> KDV -> KUC/KRR`の依存方向をrelease gateへ固定する。証跡: file: `scripts/document-surface-boundary-check.sh` / test: `document-surface-boundary-check`
- [x] 6.10 KDV `v0.4.1` strict gate、GitHub Release、crates.io publicationを確認する。証跡: test: GitHub Release `v0.4.1` / URL: `https://github.com/HiroyukiFuruno/katana-document-viewer/releases/tag/v0.4.1` / crate: `katana-document-viewer 0.4.1`
- [x] 6.11 Windows AppContainer回帰を修正したKDV `v0.4.2`のstrict gate、GitHub Release、crates.io publicationを確認する。delegation-exception: `ユーザーがsubagent利用を禁止` / 証跡: command: `gh run view 30885769177` / URL: `https://github.com/HiroyukiFuruno/katana-document-viewer/releases/tag/v0.4.2` / crates.io `katana-document-viewer 0.4.2` / tag commit `dff8f1e9ebb6212181c73d0aa93d11a6a38417b1`
- [x] 6.12 KDV `v0.5.0`のstrict coverage 100% / uncovered 0、3 OS CI、package、publish dry-run、GitHub Release、crates.io publicationを確認する。証跡: command: `rtk just release-check` / functions `3015/3015` / lines `24679/24679` / uncovered functions `0` / uncovered lines `0` / URL: `https://github.com/HiroyukiFuruno/katana-document-viewer/pull/35/checks` / URL: `https://github.com/HiroyukiFuruno/katana-document-viewer/releases/tag/v0.5.0` / URL: `https://crates.io/crates/katana-document-viewer/0.5.0`
- [x] 6.13 KRR `v0.4.15`を最低registry dependencyとするKDV `v0.5.1`について、strict coverage 100% / uncovered 0、3 OS CI、package、publish dry-run、GitHub Release、crates.io publicationを確認する。証跡: command: `rtk just VERSION=0.5.1 release-check` / functions `3015/3015` / lines `24679/24679` / uncovered functions `0` / uncovered lines `0` / PR `#36` / GitHub Release `v0.5.1` / crates.io `katana-document-viewer 0.5.1`
- [/] 6.14 deterministic Office font修正を含むKDV `v0.5.2`について、strict coverage 100% / uncovered 0、3 OS CI、package、publish dry-run、GitHub Release、crates.io publicationを確認する。delegation-exception: `ユーザーがsubagent利用を禁止` / 証跡: command: `rtk just VERSION=0.5.2 release-check` success / functions `3021/3021` / lines `24728/24728` / uncovered functions `0` / uncovered lines `0` / package success / publish dry-run success / `office2pdf-katana 0.6.6` registry source・checksum・supply-chain evidence確認済み / 残件: 3 OS CI、GitHub Release、crates.io publication
- [/] 6.15 KDV `v0.5.3`の公式office2pdf復帰について、strict coverage 100% / uncovered 0、3 OS CI、package、publish dry-run、GitHub Release、crates.io publicationを確認する。delegation-exception: `直列のクリティカルパス` / 証跡: command: `rtk just VERSION=0.5.3 release-check` success / functions `3021/3021`, lines `24728/24728`, uncovered `0` / package success / publish dry-run success / `office2pdf 0.6.7` registry source / checksum `0cd39889efe9f4bc36ea89ffc30ad5ecf7e1cd3a33b5af76604d54ca26f764c3` / 残件: 3 OS CI、GitHub Release、crates.io publication
- [/] 6.16 KDV `v0.5.4`のexternal hyperlink表示修正について、strict coverage 100% / uncovered 0、3 OS CI、package、publish dry-run、GitHub Release、crates.io publicationを確認する。delegation-exception: `直列のクリティカルパス` / 証跡: command: `rtk just VERSION=0.5.4 release-check` success / functions `3024/3024` / lines `24747/24747` / uncovered `0` / `multi_format_office_preflight_contract` `10/10` / `multi_format_office_worker_contract` `10/10` / package success / publish dry-run success / 残件: 3 OS CI、GitHub Release、crates.io publication

---

## 7. KatanA handoff

- [x] 7.1 KDV `v0.4.2` 公開後にKatanAのadjacent patch OpenSpecへregistry versionとcapabilityを引き渡す。証跡: KatanA `release/v0.22.38`
- [x] 7.2 KatanAがformat parser / rendererを持たないことをhandoff条件にする
- [x] 7.3 PDF / DOCX / XLSX / PPTXのfile/URL intake、navigation、diagnosticsのacceptance corpusを引き渡す
- [ ] 7.4 KDV `v0.5.2`公開後にKatanAへregistry dependencyとして取り込み、KatanAだけがegui backend integrationを持つことを検証する

---

## 8. Final verification

- [x] 8.1 command境界是正後に`rtk ./scripts/openspec validate v0-5-0-multi-format-viewer --strict --no-interactive`を再実行する。証跡: command: `rtk ./scripts/openspec validate v0-5-0-multi-format-viewer --strict --no-interactive`
- [x] 8.2 command境界是正後に`rtk just check`を再実行する。証跡: command: `rtk just check`
- [x] 8.3 KDV `v0.4.0` release gateを実行する。証跡: test: `v0.4.0-release-check`
- [ ] 8.4 engine選定、quality profile、unsupported機能、security evidenceをユーザーへ報告する
- [x] 8.5 KDV `v0.4.2` release gateとWindows実worker acceptanceを実行する。delegation-exception: `ユーザーがsubagent利用を禁止` / 証跡: local `just VERSION=0.4.2 release-check` success / CI run `30868560652` / Windows job `91865557624` / `Run Windows AppContainer worker acceptance` success / URL: `https://github.com/HiroyukiFuruno/katana-document-viewer/actions/runs/30868560652/job/91865557624`
- [/] 8.6 KDV `v0.5.2`のbackend-neutral release gateを実行し、公開後にKatanA `v0.22.38`の最終gateへ進む。証跡: local `rtk just VERSION=0.5.2 release-check` success / 残件: 3 OS CIと公開

<!-- subagent-spark-harness-strict-start -->
- delegation-exception: `ユーザーがsubagent利用を禁止`
