#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly HARNESS_SCRIPT="${SCRIPT_DIR}/check-subagent-spark-harness.sh"
readonly AGENT_ID="019e7c36-0a98-79b2-8b2e-bebe22223877"

fail_test() {
  printf 'FAIL: %s\n' "$1" >&2
  exit 1
}

valid_payload() {
  printf 'agent: `%s` / model: `gpt-5.3-codex-spark` / reasoning: `medium` / file: `dummy` / command: `multi_agent_v1.spawn_agent` / verify: `rtk just check-subagent-harness` / close: `multi_agent_v1.close_agent`' "$AGENT_ID"
}

write_workspace() {
  local workspace="$1"
  local handoff_body="$2"

  mkdir -p \
    "${workspace}/.codex/workflows" \
    "${workspace}/.github/workflows" \
    "${workspace}/openspec/changes/subagent-change"
  : >"${workspace}/dummy"
  cat >"${workspace}/Justfile" <<'JUSTFILE'
check: check-subagent-harness

check-subagent-harness:
    bash scripts/check-subagent-spark-harness.sh
    bash scripts/check-subagent-spark-harness-tests.sh
    bash scripts/check-subagent-spark-harness-edge-tests.sh
    bash scripts/check-subagent-spark-harness-policy-tests.sh
    bash scripts/check-subagent-spark-harness-change-tests.sh
    bash scripts/check-subagent-spark-harness-verify-tests.sh
    bash scripts/check-subagent-spark-harness-coverage-tests.sh
    bash scripts/check-subagent-spark-harness-ci-tests.sh
    bash scripts/check-subagent-spark-harness-diff-tests.sh
JUSTFILE
  cat >"${workspace}/.github/workflows/test-and-build.yml" <<'WORKFLOW'
name: test
jobs:
  test:
    steps:
      - name: Run subagent/Spark harness check
        run: just check-subagent-harness
WORKFLOW
  cat >"${workspace}/.codex/workflows/subagent-spark-policy.md" <<'POLICY'
- 許可済みの作業で、分離できる実装・調査は原則 subagent へ移譲する。
- 作業済みactive tasks.mdには少なくとも1件のsubagent / Spark証跡を残す。
- subagent は `gpt-5.3-codex-spark` / reasoning `medium` を明示する。
- OpenSpec証跡には `file:` と command: `multi_agent_v1.spawn_agent` と close: `multi_agent_v1.close_agent` を残す。
- OpenSpec証跡の検証コマンドは `verify:` に分ける。
- 移譲 / 委譲 / worker / explorer も subagent証跡の対象にする。
- 作業済みactive `tasks.md` は `<!-- subagent-spark-harness-strict-start -->` 以降の作業行に `証跡:` または `delegation-exception:` を必須化する。
- 親モデル継承や `fork_context` 制約を理由に、model / reasoning の明示を省略しない。
- 例外は、単純な一手作業、直列のクリティカルパス、書き込み範囲を明示できない作業、ユーザーがsubagent利用を禁止した場合に限る。
- `delegation-exception:` は許可済みの例外理由だけを受け付ける。
- main agent は設計、計画、レビュー、統合判断、ユーザー対話を担当する。
- subagent には分離できる実装・調査を渡し、同じファイルや同じ責務を重ねない。
- 完了済み subagent は速やかに閉じ、新しい並列作業の枠を塞がない。
- OpenSpec証跡には `model:` と `reasoning:` を残す。
POLICY
  printf '%s\n- [/] subagent証跡を残す。証跡: %s\n' \
    '<!-- subagent-spark-harness-strict-start -->' "$(valid_payload)" \
    >"${workspace}/openspec/changes/subagent-change/tasks.md"
  printf '%s\n' "$handoff_body" \
    >"${workspace}/openspec/changes/subagent-change/handoff.md"
}

run_harness() {
  local workspace="$1"
  KDV_SUBAGENT_HARNESS_WORKSPACE_DIR="$workspace" \
    KDV_SUBAGENT_HARNESS_CHECK_EXTERNAL=0 \
    bash "$HARNESS_SCRIPT" >"${workspace}/stdout" 2>"${workspace}/stderr"
}

expect_failure() {
  local label="$1"
  local handoff_body="$2"
  local expected="$3"
  local workspace
  workspace="$(mktemp -d)"
  trap "rm -rf '$workspace'" RETURN

  write_workspace "$workspace" "$handoff_body"
  if run_harness "$workspace"; then fail_test "${label} should fail"; fi
  if ! grep -Fq -- "$expected" "${workspace}/stderr"; then
    fail_test "${label} should explain ${expected}"
  fi
}

expect_pass() {
  local workspace
  workspace="$(mktemp -d)"
  trap "rm -rf '$workspace'" RETURN

  write_workspace "$workspace" "- [/] handoff証跡を残す。証跡: $(valid_payload)"
  run_harness "$workspace" || fail_test "valid handoff evidence should pass"
}

expect_handoff_exception_pass() {
  local workspace
  workspace="$(mktemp -d)"
  trap "rm -rf '$workspace'" RETURN

  write_workspace "$workspace" \
    '- delegation-exception: `ユーザーがsubagent利用を禁止`'
  run_harness "$workspace" || fail_test "allowed handoff delegation exception should pass"
}

expect_pass
expect_handoff_exception_pass
expect_failure "empty handoff evidence" "" "handoff.mdにはsubagent / Spark証跡または"
expect_failure "plain handoff without evidence" "- handoffだけを書く。" "handoff.mdにはsubagent / Spark証跡または"

echo "check-subagent-spark-harness-change-tests: ok"
