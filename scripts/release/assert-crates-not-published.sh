#!/usr/bin/env bash
set -euo pipefail

version="$(bash "$(dirname "$0")/verify-version.sh" "${1:-}" | awk -F= '$1 == "version_bare" { print $2 }')"
for crate in katana-document-viewer; do
  if cargo info "${crate}@${version}" --registry crates-io >/dev/null 2>&1; then
    echo "${crate} ${version} is already published on crates.io." >&2
    exit 1
  fi
done

echo "KDV ${version} is unpublished"
