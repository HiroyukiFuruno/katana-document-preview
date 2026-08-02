# KDV multi-format feasibility tool

This isolated, non-publishable workspace measures candidate engines without
changing KDV production dependencies.

```text
cargo run --release -- office <input.docx|xlsx|pptx> <output-directory>
cargo run --release -- pdf <input.pdf> <output-directory>
```

Generated artifacts and metrics are evidence only. They are not part of the KDV
runtime or acceptance path.
