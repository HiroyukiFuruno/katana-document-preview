# Resource lifecycle evidence

- `DocumentSession::close(&mut self)` is idempotent; close and Drop release the
  runtime, spreadsheet worker, temporary workspace, PDF page cache, spreadsheet
  cell cache, and retained PDF artifact lease.
- `DocumentResourceSnapshot` exposes live sessions, spreadsheet workers,
  workspaces, retained artifact bytes, PDF cache entries/bytes, and spreadsheet
  cell cache entries/bytes for regression diagnosis.
- PDF cache remains entry/byte bounded. Spreadsheet visible-cell cache is capped
  at 8,192 entries and 4 MiB with deterministic LRU eviction tests.
- `rtk cargo test -p katana-document-viewer --test
  multi_format_resource_lifecycle_contract --locked -- --test-threads=1`:
  ten alternating PDF/XLSX open/frame/close cycles returned every counter to the
  baseline and passed.
- Stage logs are emitted only when `DEBUG=true` and cover preflight, workspace,
  worker conversion, output/decode, render/cache, close, and Drop.
