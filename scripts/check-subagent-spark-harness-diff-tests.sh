#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly HARNESS_SCRIPT="${SCRIPT_DIR}/check-subagent-spark-harness.sh"
readonly AGENT_ID="019e7c36-0a98-79b2-8b2e-bebe22223877"

fail_test() {
  printf 'FAIL: %s\n' "$1" >&2
  exit 1
}

payload_with_file() {
  local file_ref="$1"
  local verify_command="${2:-rtk just check-subagent-harness}"

  printf 'agent: `%s` / model: `gpt-5.3-codex-spark` / reasoning: `medium` / file: `%s` / command: `multi_agent_v1.spawn_agent` / verify: `%s` / close: `multi_agent_v1.close_agent`' \
    "$AGENT_ID" "$file_ref" "$verify_command"
}

write_workspace() {
  local workspace="$1"
  local payload="$2"

  mkdir -p \
    "${workspace}/.codex/workflows" \
    "${workspace}/.github/workflows" \
    "${workspace}/openspec/changes/subagent-diff" \
    "${workspace}/scripts"
  : >"${workspace}/dummy"
  : >"${workspace}/scripts/subagent-spark-harness-verify.sh"
  : >"${workspace}/README.md"

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
    '<!-- subagent-spark-harness-strict-start -->' "$payload" \
    >"${workspace}/openspec/changes/subagent-diff/tasks.md"
  printf -- '- [/] handoff証跡を残す。証跡: agent: `%s` / model: `gpt-5.3-codex-spark` / reasoning: `medium` / file: `dummy` / command: `multi_agent_v1.spawn_agent` / verify: `rtk just check-subagent-harness` / close: `multi_agent_v1.close_agent`\n' "$AGENT_ID" >"${workspace}/openspec/changes/subagent-diff/handoff.md"
}

git_in_fixture_without_hook_environment() {
  local workspace="$1"
  shift

  env \
    -u GIT_ALTERNATE_OBJECT_DIRECTORIES \
    -u GIT_COMMON_DIR \
    -u GIT_DIR \
    -u GIT_INDEX_FILE \
    -u GIT_OBJECT_DIRECTORY \
    -u GIT_WORK_TREE \
    git -C "$workspace" "$@"
}

commit_workspace() {
  local workspace="$1"

  git_in_fixture_without_hook_environment "$workspace" init -q
  git_in_fixture_without_hook_environment "$workspace" add .
  git_in_fixture_without_hook_environment "$workspace" \
    -c user.name="KDV Harness" \
    -c user.email="kdv-harness@example.invalid" \
    commit --no-gpg-sign -q -m "initial"
}

run_harness() {
  local workspace="$1"
  KDV_SUBAGENT_HARNESS_WORKSPACE_DIR="$workspace" \
    KDV_SUBAGENT_HARNESS_CHECK_EXTERNAL=0 \
    bash "$HARNESS_SCRIPT" >"${workspace}/stdout" 2>"${workspace}/stderr"
}

expect_failure() {
  local label="$1"
  local payload="$2"
  local mutate_command="$3"
  local expected="$4"
  local workspace
  workspace="$(mktemp -d)"
  trap "rm -rf '$workspace'" RETURN

  write_workspace "$workspace" "$payload"
  commit_workspace "$workspace"
  bash -c "$mutate_command" bash "$workspace"

  if run_harness "$workspace"; then
    fail_test "${label} should fail"
  fi

  if ! grep -Fq -- "$expected" "${workspace}/stderr"; then
    fail_test "${label} should explain ${expected}"
  fi
}

expect_pass() {
  local label="$1"
  local payload="$2"
  local mutate_command="$3"
  local workspace
  workspace="$(mktemp -d)"
  trap "rm -rf '$workspace'" RETURN

  write_workspace "$workspace" "$payload"
  commit_workspace "$workspace"
  bash -c "$mutate_command" bash "$workspace"
  run_harness "$workspace" || fail_test "${label} should pass"
}

expect_exception_pass() {
  local workspace
  workspace="$(mktemp -d)"
  trap "rm -rf '$workspace'" RETURN

  write_workspace "$workspace" "$(payload_with_file 'dummy')"
  commit_workspace "$workspace"
  printf '%s\n%s\n' \
    '<!-- subagent-spark-harness-strict-start -->' \
    '- [x] main agentで直列作業を行う。delegation-exception: `ユーザーがsubagent利用を禁止` / file: `scripts/subagent-spark-harness-verify.sh`' \
    >"${workspace}/openspec/changes/subagent-diff/tasks.md"
  printf '# changed\n' >>"${workspace}/scripts/subagent-spark-harness-verify.sh"
  run_harness "$workspace" || fail_test "changed harness file with allowed exception should pass"
}

expect_pass \
  "changed harness file with evidence" \
  "$(payload_with_file 'scripts/subagent-spark-harness-verify.sh')" \
  'printf "# changed\n" >>"$1/scripts/subagent-spark-harness-verify.sh"'
expect_exception_pass
expect_pass \
  "changed non-harness file without evidence" \
  "$(payload_with_file 'dummy')" \
  'printf "changed\n" >>"$1/README.md"'
expect_failure \
  "changed harness file without file evidence" \
  "$(payload_with_file 'dummy')" \
  'printf "# changed\n" >>"$1/scripts/subagent-spark-harness-verify.sh"' \
  "変更されたsubagent / Sparkハーネス関連ファイル"
expect_failure \
  "changed nested harness file without file evidence" \
  "$(payload_with_file 'dummy')" \
  'mkdir -p "$1/scripts/nested"; printf "# changed\n" >"$1/scripts/nested/subagent-spark-harness-extra.sh"' \
  "変更されたsubagent / Sparkハーネス関連ファイル"
expect_failure \
  "changed deeply nested harness file without file evidence" \
  "$(payload_with_file 'dummy')" \
  'mkdir -p "$1/scripts/vendor/internal"; printf "# changed\n" >"$1/scripts/vendor/internal/subagent-spark-harness-extra.sh"' \
  "変更されたsubagent / Sparkハーネス関連ファイル"
expect_pass \
  "changed nested harness file with evidence" \
  "$(payload_with_file 'scripts/nested/subagent-spark-harness-extra.sh')" \
  'mkdir -p "$1/scripts/nested"; printf "# changed\n" >"$1/scripts/nested/subagent-spark-harness-extra.sh"'
expect_pass \
  "changed deeply nested harness file with evidence" \
  "$(payload_with_file 'scripts/vendor/internal/subagent-spark-harness-extra.sh')" \
  'mkdir -p "$1/scripts/vendor/internal"; printf "# changed\n" >"$1/scripts/vendor/internal/subagent-spark-harness-extra.sh"'
expect_failure \
  "invalid dot file reference" \
  "$(payload_with_file '.')" \
  ':' \
  "file参照が不正"
expect_failure \
  "invalid url file reference" \
  "$(payload_with_file 'https://example.invalid/file')" \
  ':' \
  "file参照が不正"
expect_failure \
  "empty rtk verify command" \
  "$(payload_with_file 'dummy' 'rtk ')" \
  ':' \
  "verifyは rtk で始まる再現可能"

echo "check-subagent-spark-harness-diff-tests: ok"
