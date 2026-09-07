# Supplied XLSX startup measurements

Measured through the ignored supplied-corpus KDV document-session contract with
`DEBUG=true` on 2026-08-28.

The original spreadsheet path launched the 130-150 MiB combined Office worker.
On `shopchannel_analysis.xlsx`, the child engine needed 4 ms but the parent did
not receive `Opened` for 3,266-4,221 ms. This isolated binary/process startup as
the dominant avoidable stage.

KDV now packages and automatically selects a dedicated spreadsheet worker,
while retaining the combined worker as a compatibility fallback.

| Fixture | Source bytes | Session open | First materialization | Result |
| --- | ---: | ---: | ---: | --- |
| `shopchannel_analysis.xlsx` | 27,857 | 361 ms debug; 618 ms cold release; 14 ms warm release | 1 ms | passed, repeat frame cache hit 0 ms |
| `【681303_発注依頼書】...xlsx` | 670,354 | 444 ms | 0 ms; scrolled frame 2 ms | passed |
| `視聴購入data_セグ別201805_20260803.xlsx` | 86,321,258 | 4,480 ms | 3 ms; scrolled frame 93 ms | passed |

The release spreadsheet worker is 7.0 MiB, versus 130 MiB for the combined
release Office worker. The large workbook spends 3,396 ms inside bounded
streaming workbook analysis; that cost scales with actual workbook content.
Host-side file reads occur before KDV tracing and remain a separate KatanA
ingestion measurement.
