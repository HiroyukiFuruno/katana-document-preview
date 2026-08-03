#!/usr/bin/env bash
set -euo pipefail

version="$(bash "$(dirname "$0")/verify-version.sh" "${1:-}" | awk -F= '$1 == "version_bare" { print $2 }')"

require_clean_worktree() {
  if git diff --quiet && git diff --cached --quiet; then
    return
  fi
  echo "working tree must be clean before publishing." >&2
  exit 1
}

require_token() {
  if [[ -n "${CARGO_REGISTRY_TOKEN:-}" ]]; then
    return
  fi
  echo "CARGO_REGISTRY_TOKEN is required." >&2
  exit 1
}

is_published() {
  cargo info "$1@${version}" --registry crates-io >/dev/null 2>&1
}

wait_until_published() {
  local crate="$1"
  for _ in {1..30}; do
    if is_published "$crate"; then
      return
    fi
    sleep 10
  done
  echo "${crate} ${version} did not become visible on crates.io in time." >&2
  exit 1
}

if is_published katana-document-viewer; then
  echo "KDV ${version} is already published; skipping."
  exit 0
fi

require_clean_worktree
require_token

if ! is_published katana-document-viewer; then
  cargo publish -p katana-document-viewer --locked --token "${CARGO_REGISTRY_TOKEN}"
  wait_until_published katana-document-viewer
fi
