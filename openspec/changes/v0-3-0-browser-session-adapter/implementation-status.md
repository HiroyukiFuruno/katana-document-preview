# Implementation Status

- [x] KDV は HTML parser、CSS/JS evaluator、WebView を持たず、KRR session adapter だけを実装した。
- [x] raw HTML と完全な document URL origin を KRR `HtmlBrowserSource` のまま渡す。
- [x] input、resize、refresh、explicit navigation、browser navigation、close を session thread 経由で伝播する。
- [x] frame coalescing と navigation/error FIFO を契約テストで確認した。
- [x] browser adapter source は regions、functions、lines ともに 100% coverage。
- [x] `rtk cargo test -p katana-document-viewer --test browser_session_adapter_contract -- --test-threads=1` は 2 tests 成功。
- [x] `rtk just coverage-missing` は browser adapter source の未カバー 0 で成功。
- [x] `rtk just ast-lint` と strict clippy を通過した。
- [x] browser session adapter は KDV `v0.3.5` までの公開patchで完了した。
- [x] 次のrelease-line manifestは `v0.4.x` をmulti-format viewerに固定し、PDF
  export paginationを `v0.6.0` へ繰り延べる。
- [x] release contract は adapter-only の KDV v0.3.0 と legacy Storybook UI acceptance
  を分離し、adapter line では registry KRR lock、ownership prohibition、integration
  contract、strict quality gate を機械検証する。証跡: `rtk just VERSION=0.3.0 release-verify`。
