# Self-Review: Windows AppContainer and Source Paths

## No Issues

- The worker remains inside the AppContainer with no unsandboxed fallback.
- The executable, input, and per-document workspace ACL scope is unchanged.
- The parent environment is preserved as before, while `TEMP` and `TMP` are replaced with the per-document workspace.
- The Unicode environment block is sorted case-insensitively before `CreateProcessW`.
- Windows verbatim drive and UNC paths are normalized before URL query or fragment parsing.
- Direct images from canonical Windows paths retain their extension, file URI, and lazy asset request.
- No public API or KUC boundary changed.
- `just check`, strict coverage, `v0.4.2-release-check`, OpenSpec validation, and Windows cross-compilation pass.
- Strict coverage remains Functions `3013/3013` and Lines `24653/24653`, with zero uncovered functions or lines.

## Findings

- CI run `30865519677` reproduced `CreateProcessW` error `0x800700CB` because the previous explicit environment block was partial and unsorted.
- GitHub Actions run `30868560652`, Windows job `91865557624`, passed the real AppContainer worker acceptance on Windows Server 2025.
- CI run `30869935308`, Windows job `91869713856`, exposed a direct-image regression after the AppContainer acceptance passed. The `?` in Rust's canonical `\\?\D:\...` path was incorrectly treated as a URL query delimiter, so the image extension and asset request were lost.
- CI run `30874173658`, Windows job `91882129198`, confirmed the direct-image fix and all 1674 unit tests, then exposed concurrent Office opens calling `CreateAppContainerProfile` for the same persistent profile at the same time. The profile is now initialized once per process, shared by DOCX/PPTX/XLSX workers, and failed initialization remains retryable; document workspaces remain unique and independently removed.
- Windows ACL failures now identify the exact file or directory resource in the typed worker error instead of returning only the underlying `GetNamedSecurityInfoW` message.
- CI run `30876360020`, Windows job `91888519478`, passed AppContainer acceptance and the Office/XLSX worker contracts, then exposed eight KDV linter tests whose source scopes compared `/` against native Windows paths. All linter path matching now normalizes both path and expected fragment before matching, with a cross-platform separator contract test.
- `windows_verbatim_image_source_keeps_extension_and_valid_file_uri`, `windows_verbatim_unc_image_source_keeps_extension_and_valid_file_uri`, `windows_verbatim_document_path_still_plans_direct_image_asset`, and `loader_materializes_visible_direct_image_asset` pass after normalizing path syntax before URL syntax.

## Conclusion

PASS for local, cross-compiled, and previously exercised real Windows AppContainer runtime gates. The final three-OS matrix on the corrected source-path commit remains the release gate.
