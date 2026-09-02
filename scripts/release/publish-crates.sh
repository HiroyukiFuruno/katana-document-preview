#!/usr/bin/env bash
set -euo pipefail

version="$(bash "$(dirname "$0")/verify-version.sh" "${1:-}" | awk -F= '$1 == "version_bare" { print $2 }')"
publish_attempts="${PUBLISH_ATTEMPTS:-3}"
publish_retry_delay_seconds="${PUBLISH_RETRY_DELAY_SECONDS:-10}"

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

publish_with_retry() {
  local attempt
  local delay
  for attempt in $(seq 1 "${publish_attempts}"); do
    if is_published katana-document-viewer; then
      return
    fi
    if cargo publish -p katana-document-viewer --locked --token "${CARGO_REGISTRY_TOKEN}"; then
      return
    fi
    if [[ "${attempt}" == "${publish_attempts}" ]]; then
      echo "KDV ${version} publish failed after ${publish_attempts} attempts." >&2
      exit 1
    fi
    delay=$((publish_retry_delay_seconds * attempt))
    echo "KDV ${version} publish failed; retrying in ${delay}s (${attempt}/${publish_attempts})." >&2
    sleep "${delay}"
  done
}

if is_published katana-document-viewer; then
  echo "KDV ${version} is already published; skipping publication."
else
  require_clean_worktree
  require_token

  if ! is_published katana-document-viewer; then
    publish_with_retry
    wait_until_published katana-document-viewer
  fi
fi

python3 "$(dirname "$0")/verify-registry-consumer-link.py" "${version}"
