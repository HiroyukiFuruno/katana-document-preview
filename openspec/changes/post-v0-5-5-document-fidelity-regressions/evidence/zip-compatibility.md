# Office ZIP compatibility evidence

- Generated non-seekable DOCX/XLSX/PPTX packages set general-purpose bit 3 and store entry CRC/sizes in data descriptors rather than local headers.
- A generated ZIP64 + data-descriptor DOCX requires extraction version 4.5 and contains the ZIP64 extra field.
- KDV validates local signature, local/central name and data offset, EOCD entry count, duplicate names, central metadata, decompressed payload CRC, paths, relationships, active content, and resource limits before worker spawn.
- The former error-message substring fallback was removed; descriptor handling is determined from the typed local-header flag and validated central metadata.
- `cargo test -p katana-document-viewer office_preflight_zip_entries`: 5 passed.
- `cargo test -p katana-document-viewer office_preflight`: 23 passed after responsibility-based test consolidation.
- `cargo test -p katana-document-viewer --test multi_format_office_preflight_contract`: 13 passed, including all three Office formats and ZIP64 descriptor coverage.
