# Supplied PPTX first-frame measurements

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
