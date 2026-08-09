# Self-Review: Framework-Neutral Multi-Format Session

## Scope

- KDV `v0.5.0` provides one framework-neutral `DocumentSession` for PDF, DOCX, XLSX, and PPTX.
- KDV uses KUC internally for generic layout, geometry, hit-test, and interaction behavior without exposing KUC types in its public API.
- KDV uses KRR only through the existing rendering boundary and does not introduce Chromium, WebView, PDFium, or a KDV/KUC mixed crate.
- KatanA integration and public release verification are intentionally outside this local KDV review and remain open tasks.

## Boundary Review

- `DocumentSession::open`, `apply`, `frame`, `info`, and consuming `close` form the unified lifecycle.
- `DocumentSession::apply` returns KDV-owned `DocumentSessionEvent`; viewer and grid events are preserved instead of discarded.
- Unsupported command and format combinations return typed `UnsupportedCommand` errors before mutation.
- PDF, DOCX, and PPTX support page or slide navigation, zoom, fit, and resize; grid, copy, and external-open commands are rejected until the capability exists.
- XLSX supports sheet navigation, grid interaction, resize, and copy; zoom, fit, and external-open commands are rejected until the capability exists.
- `DocumentSessionInfo` exposes document identity, revision, MIME type, format, capabilities, and diagnostics without leaking KUC or vendor types.
- KDV owns document semantics and neutral frames. Platform I/O and UI-framework projection remain host responsibilities.

## Verification

- OpenSpec strict validation: PASS.
- Release contract and approved multi-format scorecard: PASS.
- Public dependency and document-surface boundary guards: PASS.
- Rust formatting, strict clippy, AST lint, workspace tests, and headless Storybook checks: PASS.
- Unified session contract tests cover all four formats, supported events, typed unsupported commands, unchanged state on rejection, metadata, and close semantics: PASS.
- Strict coverage: Functions `3015/3015` and Lines `24679/24679`, with zero uncovered functions and zero uncovered lines.
- Package verification and crates.io publish dry-run: PASS.

## Findings

- The pre-correction session silently ignored paged grid commands, accepted XLSX zoom and fit without applying them, and discarded KUC interaction events. The approved command contract removes all three silent behaviors.
- The subagent evidence harness previously interpreted generic `command:` evidence as a subagent invocation. It now validates command-only evidence only when an actual `multi_agent_v1.spawn_agent` call is present, with regression coverage.
- No local blocker remains for KDV `v0.5.0` publication.
- The release is not complete until the pull request passes macOS, Linux, and Windows CI and both GitHub Release and crates.io publication are verified.

## Conclusion

PASS for the local KDV `v0.5.0` release gate. Three-OS CI and public artifact verification remain mandatory before KatanA consumes the registry release.
