#!/usr/bin/env bash

check_policy_contracts() {
  check_required_terms "$REPO_POLICY_FILE" \
    "許可済みの作業で、分離できる実装・調査" \
    "原則 subagent" \
    "作業済みactive tasks.md" \
    "$MODEL_TOKEN" \
    "$REASONING_TOKEN" \
    "model:" \
    "reasoning:" \
    "file:" \
    "$SPAWN_COMMAND_TOKEN" \
    "close:" \
    "$CLOSE_COMMAND_TOKEN" \
    "検証コマンドは \`verify:\`" \
    "移譲 / 委譲 / worker / explorer" \
    "<!-- subagent-spark-harness-strict-start -->" \
    "delegation-exception:" \
    "親モデル継承" \
    "省略しない" \
    "例外は" \
    "単純な一手作業" \
    "直列のクリティカルパス" \
    "書き込み範囲を明示できない" \
    "ユーザーがsubagent利用を禁止" \
    "main agent は設計" \
    "subagent には分離できる実装・調査" \
    "完了済み subagent" \
    "速やかに閉じ"
  check_forbidden_terms_absent \
    "$REPO_POLICY_FILE" \
    "${OPTIONAL_DELEGATION_TERMS[@]}" \
    "${CONDITIONAL_SPARK_POLICY_TERMS[@]}"
  check_forbidden_regex_terms_absent "$REPO_POLICY_FILE"

  if ! should_check_external_policy \
    "$CHECK_EXTERNAL_POLICY" "$GLOBAL_AGENTS_FILE" "$TASK_DELEGATION_SKILL_FILE"; then
    return
  fi

  for required_file in "$GLOBAL_AGENTS_FILE" "$TASK_DELEGATION_SKILL_FILE"; do
    if [[ ! -f "$required_file" ]]; then
      fail_fast "missing external policy file" "${required_file}"
    fi
  done

  check_external_delegation_policy "$GLOBAL_AGENTS_FILE"
  check_required_terms "$GLOBAL_AGENTS_FILE" "完了済み subagent"
  check_forbidden_terms_absent \
    "$GLOBAL_AGENTS_FILE" \
    "${OPTIONAL_DELEGATION_TERMS[@]}" \
    "${CONDITIONAL_SPARK_POLICY_TERMS[@]}"
  check_forbidden_regex_terms_absent "$GLOBAL_AGENTS_FILE"

  check_external_delegation_policy "$TASK_DELEGATION_SKILL_FILE"
  check_forbidden_terms_absent "$TASK_DELEGATION_SKILL_FILE" \
    "${OPTIONAL_DELEGATION_TERMS[@]}" \
    "${CONDITIONAL_SPARK_POLICY_TERMS[@]}"
  check_forbidden_regex_terms_absent "$TASK_DELEGATION_SKILL_FILE"
}

check_justfile_contract() {
  if ! grep -Eq '^check:.*(^|[[:space:]])check-subagent-harness([[:space:]]|$)' "$JUSTFILE"; then
    fail_fast "$JUSTFILE" "just check must include check-subagent-harness."
  fi

  if ! recipe_contains_command "$JUSTFILE" \
    "check-subagent-harness" \
    "bash scripts/check-subagent-spark-harness.sh"; then
    fail_fast "$JUSTFILE" "check-subagent-harness must run scripts/check-subagent-spark-harness.sh."
  fi

  if ! recipe_contains_command "$JUSTFILE" \
    "check-subagent-harness" \
    "bash scripts/check-subagent-spark-harness-tests.sh"; then
    fail_fast "$JUSTFILE" "check-subagent-harness must run scripts/check-subagent-spark-harness-tests.sh."
  fi

  if ! recipe_contains_command "$JUSTFILE" \
    "check-subagent-harness" \
    "bash scripts/check-subagent-spark-harness-edge-tests.sh"; then
    fail_fast "$JUSTFILE" "check-subagent-harness must run scripts/check-subagent-spark-harness-edge-tests.sh."
  fi

  if ! recipe_contains_command "$JUSTFILE" \
    "check-subagent-harness" \
    "bash scripts/check-subagent-spark-harness-policy-tests.sh"; then
    fail_fast "$JUSTFILE" "check-subagent-harness must run scripts/check-subagent-spark-harness-policy-tests.sh."
  fi

  if ! recipe_contains_command "$JUSTFILE" \
    "check-subagent-harness" \
    "bash scripts/check-subagent-spark-harness-change-tests.sh"; then
    fail_fast "$JUSTFILE" "check-subagent-harness must run scripts/check-subagent-spark-harness-change-tests.sh."
  fi

  if ! recipe_contains_command "$JUSTFILE" \
    "check-subagent-harness" \
    "bash scripts/check-subagent-spark-harness-verify-tests.sh"; then
    fail_fast "$JUSTFILE" "check-subagent-harness must run scripts/check-subagent-spark-harness-verify-tests.sh."
  fi

  if ! recipe_contains_command "$JUSTFILE" \
    "check-subagent-harness" \
    "bash scripts/check-subagent-spark-harness-coverage-tests.sh"; then
    fail_fast "$JUSTFILE" "check-subagent-harness must run scripts/check-subagent-spark-harness-coverage-tests.sh."
  fi

  if ! recipe_contains_command "$JUSTFILE" \
    "check-subagent-harness" \
    "bash scripts/check-subagent-spark-harness-ci-tests.sh"; then
    fail_fast "$JUSTFILE" "check-subagent-harness must run scripts/check-subagent-spark-harness-ci-tests.sh."
  fi

  if ! recipe_contains_command "$JUSTFILE" \
    "check-subagent-harness" \
    "bash scripts/check-subagent-spark-harness-diff-tests.sh"; then
    fail_fast "$JUSTFILE" "check-subagent-harness must run scripts/check-subagent-spark-harness-diff-tests.sh."
  fi
}

check_ci_contract() {
  if ! ci_runs_command "$CI_WORKFLOW_FILE" "just check-subagent-harness"; then
    fail_fast "$CI_WORKFLOW_FILE" "CI must run just check-subagent-harness."
  fi
}
