# 提供 PPTX の first-frame / lifecycle 証跡

## v0.5.6 実コーパス cold-process 測定（2026-08-31）

3 件の提供 PPTX を、Cargo のビルドを測定区間の外へ出したうえで、fixture
ごとに 5 回ずつ別プロセスで実行した。各サンプルは同じ KDV セッションで
初回 frame、resize 後 frame、繰り返し frame を生成する。元の Office package
は再変換されず、stage trace の `office.conversion` は必ず 1 回であることを
harness が検証する。測定前に expected SHA-256 を照合するため、同名の生成
fixture への差し替えも失敗する。

| Fixture（SHA-256） | Source | cold first-frame p50 / p95 | matched no-op 基準の peak RSS delta p50 / p95 | 支配 stage（p50） |
| --- | ---: | ---: | ---: | --- |
| [`librechat_entra_oidc_vs_saml`](supplied-pptx-first-frame/librechat_entra_oidc_vs_saml.json) (`0d034a…8d9cdb`) | 5.6 MiB | 1,255 / 4,471 ms | +244.7 / +246.7 MiB | `office.conversion` 1,187 ms（内 `parse_layout` 1,109 ms） |
| [`【チャット型AIエージェント】フェーズ1 プロジェクト&PoC提案_r3_20260616`](supplied-pptx-first-frame/chat-ai-agent-phase1-r3_20260616.json) (`ebe463…4e37f4`) | 17.8 MiB | 3,848 / 6,977 ms | +680.3 / +693.7 MiB | `office.conversion` 3,753 ms（内 `parse_layout` 3,685 ms） |
| [`libre-chat_vs_loom`](supplied-pptx-first-frame/libre-chat_vs_loom.json) (`34f462…b3af57`) | 39.0 MiB | 2,054 / 5,263 ms | +773.3 / +775.5 MiB | `office.conversion` 1,830 ms（内 `parse_layout` 1,698 ms） |

RSS delta は、同一 integration-test binary の no-op ignored test を同数の別
プロセスで走らせた peak RSS の p50 を、各 fixture 実行の peak RSS から引いた
値である。OS 全体の RSS や worker process の合算値ではないため、KDV の
first-frame 経路に帰属する比較指標としてのみ扱う。完全な fixture hash、5 個の
raw sample、baseline、p50/p95 は各 JSON に保持する。

`office.conversion` は worker 全体を囲む span であり、`office.parse_layout` は
その内側の span である。両者を加算しない。archive/package intake、worker spawn、
runtime init、frame publication は p50 で支配的ではなく、3 frame 合計の raster は
100 ms、95 ms、248 ms だった。現時点で安全上限や隔離を下げずに除去できる
KDV 固有の支配 stage は確認できず、Office 変換（主に parse/layout）が残る主要因で
ある。

## 未変更 source の再利用と 10-cycle cleanup

`supplied_pptx_reuses_its_source_and_cleans_up_after_ten_cycles` は 3 fixture を
それぞれ 10 回、open → first frame → resize → repeat frame → close した。各 close
直後に `DocumentResourceSnapshot`（session、worker、workspace、artifact、page/grid
cache の live count/bytes）が開始時 baseline と一致することを確認する。実行結果は
`1 passed`、77.15 秒である。

同じ corpus を用いる cold-process harness でも、全 15 サンプルで
`office.conversion` は 1 回、`office.frame_publication` と `office.raster` は各 3 回
だった。したがって、初回・resize 後・繰り返し frame の間に未変更 source の
再変換は発生していない。

## 2026-08-28 の事前計測

Measured through the ignored supplied-corpus KDV document-session contract with
`DEBUG=true` on 2026-08-28.

| Fixture | Source bytes | KDV session open | First PDF frame | Dominant KDV stage |
| --- | ---: | ---: | ---: | --- |
| `librechat_entra_oidc_vs_saml.pptx` | 5.6 MiB | 6,210 ms | 37 ms | office2pdf engine 3,068 ms; remaining worker lifecycle about 3.1 s |
| `【チャット型AIエージェント】フェーズ1 プロジェクト&PoC提案_r3_20260616.pptx` | 18 MiB | 9,872 ms | 42 ms | office2pdf engine 6,163 ms; remaining worker lifecycle about 3.7 s |
| `libre-chat_vs_loom.pptx` | 39 MiB | 5,727 ms | 132 ms | office2pdf engine 1,898 ms; remaining worker lifecycle about 3.8 s |

The first frame raster is not the bottleneck. Conversion and isolated-process
startup dominate KDV time. The 39 MiB end-to-end test spent about 13 additional
seconds before KDV session-open tracing began, so host file ingestion must be
timed separately in KatanA. Reducing the macOS memory-monitor poll rate from 10
ms to 100 ms made monitor shutdown bounded (54 ms in the measured 18 MiB run)
and removed the prior long post-test tail; memory-limit enforcement remains
covered.

A dependency-pruned conversion-only worker prototype reduced the 5.6 MiB case
by only about 350 ms while adding another roughly 90 MiB debug binary, so it was
not retained. A warmed release worker opened that fixture in 2,353 ms and
rendered its first page in 36 ms; 1,178 ms was office2pdf conversion and the
remaining launch/isolation cost was about 1.1 s. The dominant remaining delay is
therefore office2pdf document conversion plus the mandatory isolated process,
not KDV page rasterization or repeated conversion. Navigation, resize, and
repeat frames reuse a content/format/worker-settings conversion key and the
retained bounded PDF artifact.

## v0.5.6 lifecycle regression

`multi_format_resource_lifecycle_contract` now runs ten PDF, XLSX, and
representative PPTX open/frame/close cycles. It asserts the process, workspace,
frame, and cache live counts return to the captured baseline after every cycle.
The direct supplied-corpus ten-cycle contract above is the Issue #49 evidence;
this generic regression remains a complementary coverage guard.

## KatanA #49 XLSX cold/warm evidence

KatanA's current release rebuild recorded the following 7.7 KiB XLSX result in
[Issue #49](https://github.com/HiroyukiFuruno/katana-document-viewer/issues/49#issuecomment-5461739569):

- First session open: 3,820 ms; first frame: 3,857 ms.
- Subsequent session opens: 14–16 ms; subsequent frames: 42–45 ms.
- Ten cycles released all resources; steady RSS delta was +912 KiB.

This identifies cold spawn/runtime initialization and package parse as the
next profiling boundary. It is source-side evidence, not a replacement for the
KDV supplied-PPTX p50/p95/RSS DoD.

## Reproducible supplied-corpus measurement harness

`scripts/feasibility/measure-office-first-frame.py` builds the ignored
single-fixture acceptance binary once outside the measured interval, then runs
that binary in separate processes. It requires every Office trace stage,
captures peak RSS through the platform `time` utility, and writes the individual
samples plus p50/p95 summaries. It refuses to overwrite prior evidence or to
accept a non-PPTX fixture or a mismatched expected SHA-256, so a generated
representative file cannot silently replace the supplied corpus or Cargo startup
cannot inflate KDV timing.

```text
python3 scripts/feasibility/measure-office-first-frame.py \
  --fixture /absolute/path/to/supplied-corpus/target.pptx \
  --expected-sha256 <source-sha256> \
  --iterations 5 \
  --output /absolute/path/to/evidence/target-first-frame.json
```

The supplied PPTX files were subsequently located outside the worktrees and
measured on 2026-08-31. The current reports, including the matched no-op RSS
delta and raw samples, are linked in the v0.5.6 table above.

KDV validated the added stage spans with:

```text
DEBUG=true cargo test -p katana-document-viewer --test multi_format_office_worker_contract --locked exact_katana_data_descriptor_docx_worker_generates_a_frame -- --test-threads=1 --nocapture
DEBUG=true cargo test -p katana-document-viewer --test multi_format_document_session_contract --locked docx_and_pptx_use_the_unified_paged_session -- --test-threads=1 --nocapture
DEBUG=true cargo test -p katana-document-viewer --test multi_format_document_session_contract --locked xlsx_uses_the_unified_session_for_sheet_grid_and_materialization -- --test-threads=1 --nocapture
```

Those runs emitted the required static Office and XLSX stages, including
worker spawn, runtime init, package parse, conversion where applicable, and
frame publication.
