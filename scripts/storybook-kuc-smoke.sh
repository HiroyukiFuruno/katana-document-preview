#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo_bin="${CARGO:-cargo}"

if [[ ! -d "$repo_root/assets/fixtures/direct" ]]; then
  echo "direct fixture root not found: $repo_root/assets/fixtures/direct" >&2
  exit 1
fi

"$cargo_bin" test -p kdv-storybook --locked document_viewer -- --test-threads=1
"$cargo_bin" test -p katana-document-viewer --locked --lib document_surface -- --test-threads=1
"$cargo_bin" test -p katana-document-viewer --locked asset_loader -- --test-threads=1
"$cargo_bin" test -p katana-document-viewer --locked direct_ -- --test-threads=1

echo "storybook-kuc-smoke: ok"
