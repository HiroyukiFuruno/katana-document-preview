#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly HARNESS_SCRIPT="${SCRIPT_DIR}/check-subagent-spark-harness.sh"
readonly AGENT_ID="019e7c36-0a98-79b2-8b2e-bebe22223877"

fail_test() {
  printf 'FAIL: %s\n' "$1" >&2
  exit 1
}

write_workspace() {
  local workspace="$1"

  mkdir -p \
    "${workspace}/.codex/workflows" \
    "${workspace}/.github/workflows" \
    "${workspace}/openspec/changes/subagent-policy"
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
      - name: Mandatory harness
        run: |
          just check-subagent-harness
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

  printf -- '<!-- subagent-spark-harness-strict-start -->\n- [/] subagent証跡を残す。証跡: agent: `%s` / model: `gpt-5.3-codex-spark` / reasoning: `medium` / file: `dummy` / command: `multi_agent_v1.spawn_agent` / verify: `rtk just check-subagent-harness` / close: `multi_agent_v1.close_agent`\n' \
    "$AGENT_ID" >"${workspace}/openspec/changes/subagent-policy/tasks.md"
  printf -- '- [/] handoff証跡を残す。証跡: agent: `%s` / model: `gpt-5.3-codex-spark` / reasoning: `medium` / file: `dummy` / command: `multi_agent_v1.spawn_agent` / verify: `rtk just check-subagent-harness` / close: `multi_agent_v1.close_agent`\n' "$AGENT_ID" >"${workspace}/openspec/changes/subagent-policy/handoff.md"
}

run_harness() {
  local workspace="$1"
  KDV_SUBAGENT_HARNESS_WORKSPACE_DIR="$workspace" \
    KDV_SUBAGENT_HARNESS_CHECK_EXTERNAL=0 \
    bash "$HARNESS_SCRIPT" >"${workspace}/stdout" 2>"${workspace}/stderr"
}

run_harness_with_external_policy() {
  local workspace="$1"
  HOME="${workspace}/home" \
    KDV_SUBAGENT_HARNESS_WORKSPACE_DIR="$workspace" \
    KDV_SUBAGENT_HARNESS_CHECK_EXTERNAL=1 \
    bash "$HARNESS_SCRIPT" >"${workspace}/stdout" 2>"${workspace}/stderr"
}

run_harness_with_default_external_policy() {
  local workspace="$1"
  HOME="${workspace}/home" \
    KDV_SUBAGENT_HARNESS_WORKSPACE_DIR="$workspace" \
    bash "$HARNESS_SCRIPT" >"${workspace}/stdout" 2>"${workspace}/stderr"
}

write_external_policy() {
  local workspace="$1"
  local task_delegation_line="$2"

  mkdir -p \
    "${workspace}/home/.codex/skills/task-delegation-triage"

  cat >"${workspace}/home/.codex/AGENTS.md" <<'AGENTS'
分離できる実装・調査がある場合、原則 subagent へ移譲する。
subagent は `gpt-5.3-codex-spark` を使う。
subagent 起動時はモデルと reasoning を必ず明示し、reasoning は `medium` を基準に作業内容に応じて調整する。
親モデル継承を理由に省略しない。
例外は、ユーザーが明示的に禁止した場合、単純な一手作業、直列のクリティカルパス、書き込み範囲を明示できない場合に限る。
main Codex は設計、計画、レビュー、統合判断、ユーザー対話を担当する。
subagent には分離できる実装・調査を渡す。
完了済み subagent は速やかに閉じる。
AGENTS

  cat >"${workspace}/home/.codex/skills/task-delegation-triage/SKILL.md" <<SKILL
許可済みの作業で、分離できる実装・調査がある場合は、原則 subagent へ移譲する。
subagent は原則 \`gpt-5.3-codex-spark\` / reasoning \`medium\` を明示する。親モデル継承を理由に省略しない。
例外は、単純な一手作業、直列のクリティカルパス、書き込み範囲を明示できない作業、ユーザーがsubagent利用を禁止した場合に限る。
main agent は設計、計画、レビュー、統合判断、ユーザー対話を担当する。
${task_delegation_line}
SKILL
}

delete_policy_line() {
  local workspace="$1"
  local term="$2"

  sed -i.bak "/${term}/d" \
    "${workspace}/.codex/workflows/subagent-spark-policy.md"
}

expect_policy_failure() {
  local label="$1"
  local term="$2"
  local expected="$3"
  local workspace
  workspace="$(mktemp -d)"
  trap 'rm -rf "$workspace"' RETURN

  write_workspace "$workspace"
  delete_policy_line "$workspace" "$term"

  if run_harness "$workspace"; then
    fail_test "${label} should fail"
  fi

  if ! grep -Fq -- "$expected" "${workspace}/stderr"; then
    fail_test "${label} should explain ${expected}"
  fi
}

expect_pass() {
  local workspace
  workspace="$(mktemp -d)"
  trap 'rm -rf "$workspace"' RETURN

  write_workspace "$workspace"
  run_harness "$workspace" || fail_test "valid policy should pass"
}

expect_missing_policy_test_command_fails() {
  local workspace
  workspace="$(mktemp -d)"
  trap 'rm -rf "$workspace"' RETURN

  write_workspace "$workspace"
  sed -i.bak '/check-subagent-spark-harness-policy-tests.sh/d' \
    "${workspace}/Justfile"

  if run_harness "$workspace"; then
    fail_test "missing policy test command should fail"
  fi

  if ! grep -Fq -- "check-subagent-spark-harness-policy-tests.sh" \
    "${workspace}/stderr"; then
    fail_test "missing policy test command should explain recipe gap"
  fi
}

expect_legacy_external_reasoning_phrase_passes() {
  local workspace
  workspace="$(mktemp -d)"
  trap 'rm -rf "$workspace"' RETURN

  write_workspace "$workspace"
  write_external_policy "$workspace" \
    "subagent には分離できる実装・調査を渡す。"
  sed -i.bak \
    's/reasoning は `medium` を基準に作業内容に応じて調整する/reasoning `medium` を明示する/' \
    "${workspace}/home/.codex/AGENTS.md"

  run_harness_with_external_policy "$workspace" || \
    fail_test "legacy external reasoning phrase should pass"
}

expect_missing_external_reasoning_policy_fails() {
  local workspace
  workspace="$(mktemp -d)"
  trap 'rm -rf "$workspace"' RETURN

  write_workspace "$workspace"
  write_external_policy "$workspace" \
    "subagent には分離できる実装・調査を渡す。"
  sed -i.bak '/reasoning は `medium` を基準/d' \
    "${workspace}/home/.codex/AGENTS.md"

  if run_harness_with_external_policy "$workspace"; then
    fail_test "missing external reasoning policy should fail"
  fi

  if ! grep -Fq -- 'reasoning `medium`' "${workspace}/stderr"; then
    fail_test "missing external reasoning policy should explain reasoning baseline"
  fi
}

expect_legacy_external_main_role_phrase_passes() {
  local workspace
  workspace="$(mktemp -d)"
  trap 'rm -rf "$workspace"' RETURN

  write_workspace "$workspace"
  write_external_policy "$workspace" \
    "subagent には分離できる実装・調査を渡す。"
  sed -i.bak 's/main Codex/main agent/' \
    "${workspace}/home/.codex/AGENTS.md"

  run_harness_with_external_policy "$workspace" || \
    fail_test "legacy external main role phrase should pass"
}

expect_missing_external_main_role_fails() {
  local workspace
  workspace="$(mktemp -d)"
  trap 'rm -rf "$workspace"' RETURN

  write_workspace "$workspace"
  write_external_policy "$workspace" \
    "subagent には分離できる実装・調査を渡す。"
  sed -i.bak '/main Codex は設計/d' \
    "${workspace}/home/.codex/AGENTS.md"

  if run_harness_with_external_policy "$workspace"; then
    fail_test "missing external main role should fail"
  fi

  if ! grep -Fq -- 'main agent/Codex must own design and integration.' \
    "${workspace}/stderr"; then
    fail_test "missing external main role should explain orchestrator ownership"
  fi
}

expect_missing_external_cleanup_policy_fails() {
  local workspace
  workspace="$(mktemp -d)"
  trap 'rm -rf "$workspace"' RETURN

  write_workspace "$workspace"
  write_external_policy "$workspace" \
    "subagent には分離できる実装・調査を渡す。"
  sed -i.bak '/完了済み subagent/d' \
    "${workspace}/home/.codex/AGENTS.md"

  if run_harness_with_external_policy "$workspace"; then
    fail_test "missing external cleanup policy should fail"
  fi

  if ! grep -Fq -- '完了済み subagent' "${workspace}/stderr"; then
    fail_test "missing external cleanup policy should explain cleanup requirement"
  fi
}

expect_default_external_policy_is_opt_in() {
  local workspace
  workspace="$(mktemp -d)"
  trap 'rm -rf "$workspace"' RETURN

  write_workspace "$workspace"
  write_external_policy "$workspace" \
    "環境やAGENTSでSpark指定がある場合だけ、subagentは指定modelを使う。"

  run_harness_with_default_external_policy "$workspace" || \
    fail_test "default external policy check should be opt-in"
}

expect_conditional_external_skill_fails() {
  local workspace
  workspace="$(mktemp -d)"
  trap 'rm -rf "$workspace"' RETURN

  write_workspace "$workspace"
  write_external_policy "$workspace" \
    "環境やAGENTSでSpark指定がある場合だけ、subagentは指定modelを使う。"

  if run_harness_with_external_policy "$workspace"; then
    fail_test "conditional external skill should fail"
  fi

  if ! grep -Fq -- "環境やAGENTSでSpark指定がある場合" \
    "${workspace}/stderr"; then
    fail_test "conditional external skill should explain forbidden conditional term"
  fi
}

expect_pass
expect_policy_failure "missing exception rule" "例外は" "例外は"
expect_policy_failure "missing file reference mandate" "file:" "required subagent / Spark policy term is missing: file:"
expect_policy_failure "missing main role" "main agent は設計" "main agent は設計"
expect_policy_failure "missing subagent role" "subagent には" "subagent には分離できる実装・調査"
expect_policy_failure "missing cleanup rule" "完了済み subagent" "完了済み subagent"
expect_missing_policy_test_command_fails
expect_legacy_external_reasoning_phrase_passes
expect_missing_external_reasoning_policy_fails
expect_legacy_external_main_role_phrase_passes
expect_missing_external_main_role_fails
expect_missing_external_cleanup_policy_fails
expect_default_external_policy_is_opt_in
expect_conditional_external_skill_fails

echo "check-subagent-spark-harness-policy-tests: ok"
