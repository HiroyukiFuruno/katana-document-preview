# Tasks: katana-document-viewer v0.3.0 browser session adapter

## Release Tasks

- [x] v0.3.0 adapter release contract を、legacy Storybook UI acceptance と分離する。証跡: `rtk just VERSION=0.3.0 release-verify`、`rtk just VERSION=0.3.0 release-check`、workflow contract test。
  legacy artifact の freshness/source-integrity check は維持し、adapter release は
  registry lock、ownership contract、adapter integration contract、strict quality gate
  を必須条件とする。release workflow は legacy Storybook artifact を adapter
  release gate として実行しない。
- [x] KRR `0.4.0` の crates.io 公開後、temporary local patch を除外し registry lockfile を確定する。証跡: `rtk cargo generate-lockfile` により `katana-render-runtime 0.4.0` を crates.io checksum 付きで解決。
- [x] 同一 version の未実装 PDF pagination plan を後続 version へ繰り延べ、`v0.3.0` release target と衝突しないことを機械検証する。証跡: `rtk just VERSION=0.3.0 release-target-check`
- [x] Git pre-push hook が継承する Git environment を harness とその fixture temporary workspace に漏らさず、非 Git workspace を誤判定しない。証跡: `rtk just check`
- [x] KDV の release preflight、GitHub Release、crates.io `0.3.0` 公開を確認する。
- [x] KDV `0.3.1` で連続 scroll / resize command を adapter 層で順序を保って
  coalesce し、KRR の frame raster backlog により後続 pointer input が古い表示状態へ
  適用されないことを保証する。証跡: command coalescing unit tests、KatanA headless
  acceptance 60/60、同一文書/外部文書 fragment の complete origin、raw KRR frame、
  composed screenshot の一致、`rtk just coverage` 1,579 passed / 1 ignored。
- [x] 公開済み crates.io `katana-render-runtime 0.4.3` を registry checksum 付きで
  `Cargo.lock` に解決し、KDV に path/git override や Chromium/WebView 依存がないことを
 確認する。証跡: `rtk cargo update -p katana-render-runtime --precise 0.4.3`、
  `rtk cargo tree -p katana-document-viewer --locked -e normal`。
- [x] production line coverage 100% / uncovered lines 0 を、除外追加や閾値緩和なしで
  `rtk just coverage` の release gate として通す。証跡: 20,801 / 20,801 lines、
  missed lines 0、`--fail-under-lines 100 --fail-uncovered-lines 0`。
- [x] KDV `0.3.1` は取り下げ、後続の recovery patch `0.3.2` へ置き換える。
- [x] KDV `0.3.2` で browser session 起動失敗後も実行 thread を維持し、次の navigation を復旧する。証跡:
  session を再生成できることを保証する。エラーは layer / operation / document /
  cause を保持し、公開済み KRR `0.4.4` を crates.io checksum 付きで解決する。
  `rtk cargo test -p katana-document-viewer browser_session --locked` 25 passed、
  `Cargo.lock` registry source / checksum `4b06dce4...a3530`。
- [x] production line coverage 100% / uncovered lines 0 を、除外追加や閾値緩和なしで
  `rtk just coverage` の release gate として通す。証跡: 20,933 / 20,933 lines、
  missed lines 0、`--fail-under-lines 100 --fail-uncovered-lines 0`。
- [x] KDV `0.3.2` の release preflight、GitHub Release、crates.io 公開を確認する。
- [x] KDV `0.3.3` で公開済み KRR `0.4.5` を crates.io checksum 付きで解決し、
  `document` / `window` lifecycle EventTarget を adapter が変更せず中継する。証跡:
  `Cargo.lock` checksum `4f90416a7d638dc70e51c0e9ea36ce4a04d76dd828dfc6b85d075405b74a8529`、
  `browser_session_adapter_contract` 3 tests passed。
- [x] KDV `0.3.3` の full check、strict coverage 100% / uncovered 0、release preflight、
  package verify、publish dry-run を確認する。
- [x] KDV `0.3.3` の PR CI、merge、GitHub Release、crates.io 公開を確認する。
- [x] KDV `0.3.4` で固定容量 channel を producer-side semantic mailbox に置き換え、
  連続 pointer move は最新値、scroll は差分合算、resize は最新 viewport へ合流する。
  pointer down/up、keyboard、text、focus、navigation の順序と内容は保持し、worker が
  frame を生成中でも一時的な入力集中を fatal error として host へ返さない。証跡:
  semantic mailbox unit tests 8 passed、worker tests 14 passed、公開 adapter contract
  4 passed（4,097 pointer move burst 後の link navigation / frame continuation を含む）。
- [x] 実際の `slides.html` を用いて adapter start から KRR session creation、
  first frame publication までを段階計測し、支配的な回避可能遅延を除去する。
  headless contract で first-frame latency と burst input 後の継続 frame generation を
  固定待機なしに検証する。証跡: `browser_session_worker_startup.rs` の runtime-open
  計測、`browser_session_worker.rs` の first-frame publication 計測、KatanA
  v0.22.37 headless acceptance。
- [x] KDV `0.3.4` で `just update` を実行し、全依存を対応可能な最新版へ更新する。
  strict coverage 100% / uncovered 0、full check、release preflight、package verify、
  publish dry-run を閾値緩和・除外追加なしで通す。証跡:
  `rtk just VERSION=0.3.4 release-check`、21,050 / 21,050 lines、uncovered 0、
  1,586 passed / 1 ignored、package verify / publish dry-run passed。
- [x] KDV `0.3.4` の PR CI、merge、GitHub Release、crates.io 公開を確認する。
  証跡: PR #29、release workflow `30231877757`、GitHub Release `v0.3.4`、
  crates.io `katana-document-viewer 0.3.4`。
- [x] KDV `0.3.5` で browser command queue の pending と処理中 command を区別し、
  startup 完了前、処理中、worker 停止後を idle と判定しない公開 adapter 契約を追加する。
  KatanA headless harness は frame generation とこの idle 契約の両方を待ち、
  burst 後の残存 frame を次の離散入力完了と誤認しないことを固定する。証跡:
  `browser_session_command_queue_idle_tests.rs`、公開 adapter contract、PR #30。
- [x] KDV `0.3.5` で `just update` を実行し、全依存を対応可能な最新版へ更新する。
  strict coverage 100% / uncovered 0、full check、release preflight、package verify、
  publish dry-run を閾値緩和・除外追加なしで通す。証跡:
  `rtk just VERSION=0.3.5 release-check`、21,081 / 21,081 lines、uncovered 0、
  1,587 passed / 1 ignored、package verify / publish dry-run passed。
- [x] KDV `0.3.5` の PR CI、merge、GitHub Release、crates.io 公開を確認する。
  証跡: PR #30、tag / GitHub Release / crates.io `v0.3.5`。

<!-- subagent-spark-harness-strict-start -->
- [x] KDV browser-session adapter の ownership を独立 review し、KRR への raw source/input/navigation 中継以外の HTML semantics を持たないことを確認する。証跡: agent: `019f75e7-c5e2-7293-b738-cfcc0290f921` / model: `gpt-5.3-codex-spark` / reasoning: `medium` / file: `crates/katana-document-viewer/src/browser_session.rs` / file: `crates/katana-document-viewer/src/browser_session_worker.rs` / file: `crates/katana-document-viewer/tests/browser_session_adapter_contract.rs` / command: `multi_agent_v1.spawn_agent` / verify: `rtk cargo test -p katana-document-viewer --test browser_session_adapter_contract --locked -- --test-threads=1` / close: `multi_agent_v1.close_agent`
