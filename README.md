<p align="center">
  <img src="assets/kdv-icon.png" width="128" alt="kdv icon">
</p>

<h1 align="center">kdv</h1>

<p align="center">
  Vendor-neutral Markdown viewer and export library for
  <a href="https://github.com/HiroyukiFuruno/KatanA">KatanA</a>.
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  <a href="https://github.com/HiroyukiFuruno/katana-document-viewer/actions/workflows/test-and-build.yml"><img src="https://github.com/HiroyukiFuruno/katana-document-viewer/actions/workflows/test-and-build.yml/badge.svg" alt="CI"></a>
  <img src="https://img.shields.io/badge/status-scaffolding-orange" alt="Status: scaffolding">
</p>

---

## Design

The published crate owns the complete document-viewer boundary:

```
KatanA -> katana-document-viewer -> katana-ui-core / katana-render-runtime
```

KatanA enables KDV's optional `egui` host feature and uses only KDV document
surface, input, state, capability, and diagnostic APIs. KDV uses KUC internally;
KUC types are not part of KDV's public API, and KatanA does not depend on KUC.
KDV does not own editor-viewer synchronization control; KatanA commands viewer
or editor.

HTML/PDF/PNG/JPG export belongs to KDV so viewer display and export share the
same render pipeline. Diagram and math rendering are delegated through KRR
(katana-render-runtime). Unsupported diagram or Markdown semantics stay in KDV
as diagnostics and raw source until KMM or KRR exposes the needed public
contract.

`v0.1.0` starts with the UI-independent artifact/forge/export foundation. It
depends on KMM for Markdown structure and KRR for direct render runtime
boundaries. KDV does not fill KMM parser gaps by reparsing Markdown; unsupported
or not-yet-structured Markdown semantics are carried as diagnostics and raw
source until KMM/KRR provide the needed public contract.

## Status

Scaffolding. The crates.io package is `katana-document-viewer`.

## License

MIT
