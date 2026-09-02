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
  local evidence="$2"

  mkdir -p \
    "${workspace}/.codex/workflows" \
    "${workspace}/.github/workflows" \
    "${workspace}/openspec/changes/subagent-coverage"
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

  printf '%s\n%s\n' '<!-- subagent-spark-harness-strict-start -->' "$evidence" \
    >"${workspace}/openspec/changes/subagent-coverage/tasks.md"
  printf -- '- [/] handoff証跡を残す。証跡: %s\n' "$(valid_payload)" >"${workspace}/openspec/changes/subagent-coverage/handoff.md"
}

run_harness() {
  local workspace="$1"
  KDV_SUBAGENT_HARNESS_WORKSPACE_DIR="$workspace" \
    KDV_SUBAGENT_HARNESS_CHECK_EXTERNAL=0 \
    bash "$HARNESS_SCRIPT" >"${workspace}/stdout" 2>"${workspace}/stderr"
}

expect_failure() {
  local label="$1"
  local evidence="$2"
  local expected="$3"
  local workspace
  workspace="$(mktemp -d)"
  trap "rm -rf '$workspace'" RETURN

  write_workspace "$workspace" "$evidence"

  if run_harness "$workspace"; then
    fail_test "${label} should fail"
  fi

  if ! grep -Fq -- "$expected" "${workspace}/stderr"; then
    fail_test "${label} should explain ${expected}"
  fi
}

expect_pass_without_keyword() {
  local workspace
  workspace="$(mktemp -d)"
  trap "rm -rf '$workspace'" RETURN

  write_workspace "$workspace" \
    "- [/] 監査証跡を残す。証跡: $(valid_payload)"
  run_harness "$workspace" || fail_test "valid agent evidence should pass"
}

expect_agent_evidence_without_keyword_is_checked() {
  expect_failure \
    "agent evidence without subagent keyword" \
    "- [/] 監査証跡を残す。証跡: agent: \`${AGENT_ID}\` / model: \`gpt-5.3-codex-spark\` / file: \`dummy\` / command: \`multi_agent_v1.spawn_agent\` / verify: \`rtk just check-subagent-harness\` / close: \`multi_agent_v1.close_agent\`" \
    "reasoningは medium"
}

expect_active_tasks_without_evidence_fails() {
  expect_failure \
    "active tasks without subagent evidence" \
    "- [x] 実装を進める。" \
    "subagent / Spark証跡または許可済みdelegation-exception"
}

expect_active_tasks_with_allowed_exception_passes() {
  local workspace
  workspace="$(mktemp -d)"
  trap "rm -rf '$workspace'" RETURN

  write_workspace "$workspace" \
    '- [x] main agentで直列作業を行う。delegation-exception: `ユーザーがsubagent利用を禁止`'
  run_harness "$workspace" || fail_test "allowed active-task delegation exception should pass"
}

expect_strict_line_without_evidence_fails() {
  expect_failure \
    "strict completed task without evidence" \
    "- [/] 監査証跡を残す。証跡: $(valid_payload)
- [x] 追加実装を進める。" \
    "strict marker以降の作業行"
}
expect_strict_line_with_allowed_exception_passes() {
  local workspace
  workspace="$(mktemp -d)"
  trap "rm -rf '$workspace'" RETURN

  write_workspace "$workspace" \
    "- [/] 監査証跡を残す。証跡: $(valid_payload)
- [x] 直列作業を行う。delegation-exception: \`直列のクリティカルパス\` / 証跡: file: \`dummy\`"
  run_harness "$workspace" || fail_test "allowed delegation exception should pass"
}
expect_optional_delegation_regex_terms_fail() {
  local variant
  for variant in \
    "必要ならば subagent を使う" \
    "subagent は if required then 使う" \
    "必要な場合にはサブエージェントを使う"; do
    expect_failure \
      "optional delegation regex ${variant}" \
      "- [/] ${variant}。証跡: $(valid_payload)" \
      "forbidden optional subagent policy term"
  done
}
expect_started_change_without_handoff_fails() {
  local workspace
  workspace="$(mktemp -d)"
  trap "rm -rf '$workspace'" RETURN
  write_workspace "$workspace" "- [/] 監査証跡を残す。証跡: $(valid_payload)"
  mv "${workspace}/openspec/changes/subagent-coverage/handoff.md" \
    "${workspace}/openspec/changes/subagent-coverage/handoff.md.missing"
  if run_harness "$workspace"; then fail_test "started change without handoff should fail"; fi
  if ! grep -Fq -- "handoff.md" "${workspace}/stderr"; then
    fail_test "started change without handoff should explain handoff.md"
  fi
}
expect_non_task_handoff_command_only_evidence_is_checked() {
  local workspace
  workspace="$(mktemp -d)"
  trap "rm -rf '$workspace'" RETURN

  write_workspace "$workspace" \
    "- [/] 監査証跡を残す。証跡: $(valid_payload)"
  printf '%s\n' "- [/] subagent証跡記録。証跡: command: \`multi_agent_v1.spawn_agent\`" \
    >"${workspace}/openspec/changes/subagent-coverage/handoff.md"

  if run_harness "$workspace"; then
    fail_test "non-task handoff command-only evidence should fail"
  fi
  if ! grep -Fq -- "実行agent id" "${workspace}/stderr"; then
    fail_test "non-task handoff command-only evidence should explain agent id"
  fi
}

expect_generic_command_evidence_is_not_subagent_evidence() {
  local workspace
  workspace="$(mktemp -d)"
  trap "rm -rf '$workspace'" RETURN

  write_workspace "$workspace" \
    '- [/] main agentで検証する。delegation-exception: `ユーザーがsubagent利用を禁止`'
  printf '%s\n' '- 通常の検証結果。delegation-exception: `ユーザーがsubagent利用を禁止` / 証跡: command: `rtk just release-check`' \
    >"${workspace}/openspec/changes/subagent-coverage/handoff.md"

  if ! run_harness "$workspace"; then
    cat "${workspace}/stderr" >&2
    fail_test "generic command evidence should not be parsed as subagent evidence"
  fi
}

expect_unquoted_extra_command_field_fails() {
  expect_failure \
    "unquoted extra command field" \
    "- [/] subagent証跡に未引用commandを混ぜる。証跡: agent: \`${AGENT_ID}\` / model: \`gpt-5.3-codex-spark\` / reasoning: \`medium\` / file: \`dummy\` / command: rtk just check / command: \`multi_agent_v1.spawn_agent\` / verify: \`rtk just check-subagent-harness\` / close: \`multi_agent_v1.close_agent\`" \
    "command: は backtick付き"
}

expect_pass_without_keyword
expect_agent_evidence_without_keyword_is_checked
expect_active_tasks_without_evidence_fails
expect_active_tasks_with_allowed_exception_passes
expect_strict_line_without_evidence_fails
expect_strict_line_with_allowed_exception_passes
expect_optional_delegation_regex_terms_fail
expect_started_change_without_handoff_fails
expect_non_task_handoff_command_only_evidence_is_checked
expect_generic_command_evidence_is_not_subagent_evidence
expect_unquoted_extra_command_field_fails

echo "check-subagent-spark-harness-coverage-tests: ok"
