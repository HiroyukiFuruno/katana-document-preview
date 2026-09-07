#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/subagent-spark-harness-lib.sh"
source "${SCRIPT_DIR}/subagent-spark-harness-contracts.sh"
source "${SCRIPT_DIR}/subagent-spark-harness-verify.sh"
source "${SCRIPT_DIR}/subagent-spark-harness-ci.sh"
source "${SCRIPT_DIR}/subagent-spark-harness-evidence.sh"
source "${SCRIPT_DIR}/subagent-spark-harness-terms.sh"
source "${SCRIPT_DIR}/subagent-spark-harness-change.sh"
source "${SCRIPT_DIR}/subagent-spark-harness-diff.sh"

readonly DEFAULT_WORKSPACE_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
readonly WORKSPACE_DIR="$(cd "${KDV_SUBAGENT_HARNESS_WORKSPACE_DIR:-${DEFAULT_WORKSPACE_DIR}}" && pwd)"
readonly JUSTFILE="${WORKSPACE_DIR}/Justfile"
readonly REPO_POLICY_FILE="${WORKSPACE_DIR}/.codex/workflows/subagent-spark-policy.md"
readonly CI_WORKFLOW_FILE="${WORKSPACE_DIR}/.github/workflows/test-and-build.yml"
readonly MODEL_TOKEN="gpt-5.3-codex-spark"
readonly REASONING_TOKEN='reasoning `medium`'
readonly REASONING_FIELD_VALUE="medium"
readonly SPAWN_COMMAND_TOKEN="multi_agent_v1.spawn_agent"
readonly CLOSE_COMMAND_TOKEN="multi_agent_v1.close_agent"
readonly DELEGATION_KEYWORD_PATTERN='([Ss]ubagent|[Ss]park|サブエージェント|移譲|委譲|[Ww]orker|[Ee]xplorer|delegate[[:space:]_-]*=[[:space:]_-]*subagent)'
readonly STRICT_TASK_MARKER='<!-- subagent-spark-harness-strict-start -->'
readonly DELEGATION_EXCEPTION_PATTERN='delegation-exception:[[:space:]]*`?(単純な一手作業|直列のクリティカルパス|書き込み範囲を明示できない|ユーザーがsubagent利用を禁止)`?'
readonly CHANGES_ROOT="${WORKSPACE_DIR}/openspec/changes"
readonly CHECK_EXTERNAL_POLICY="${KDV_SUBAGENT_HARNESS_CHECK_EXTERNAL:-0}"
readonly GLOBAL_AGENTS_FILE="${HOME}/.codex/AGENTS.md"
readonly TASK_DELEGATION_SKILL_FILE="${HOME}/.codex/skills/task-delegation-triage/SKILL.md"
readonly OPTIONAL_DELEGATION_TERMS=(
  "subagentは任意"
  "subagent は任意"
  "subagent任意"
  "subagentを使う場合"
  "subagent を使う場合"
  "subagentを使うなら"
  "subagent を使うなら"
  "サブエージェントは任意"
  "サブエージェント は任意"
  "サブエージェント任意"
  "サブエージェントを使う場合"
  "サブエージェント を使う場合"
  "サブエージェントを使うなら"
  "サブエージェント を使うなら"
  "サブエージェントが必要な場合"
  "サブエージェント必要な場合"
  "subagentは必要なら"
  "subagent は必要なら"
  "subagentを必要なら"
  "subagent を必要なら"
  "subagentが必要なら"
  "subagent が必要なら"
  "サブエージェントは必要なら"
  "サブエージェント は必要なら"
  "サブエージェントを必要なら"
  "サブエージェント を必要なら"
  "サブエージェントが必要なら"
  "サブエージェント が必要なら"
  "任意運用"
  "任意で使う"
  "必要ならsubagent"
  "必要なら subagent"
  "必要ならサブエージェント"
  "必要なら サブエージェント"
  "必要に応じてsubagent"
  "必要に応じて subagent"
  "必要に応じてサブエージェント"
  "必要に応じて サブエージェント"
  "subagentは必要に応じて"
  "subagent は必要に応じて"
  "サブエージェントは必要に応じて"
  "サブエージェント は必要に応じて"
  "subagentを必要に応じて"
  "subagent を必要に応じて"
  "サブエージェントを必要に応じて"
  "サブエージェント を必要に応じて"
  "optional subagent"
  "if using subagent"
  "when using subagent"
  "if needed, use subagent"
  "when subagent is needed"
  "use subagent as needed"
  "subagent as needed"
)
readonly CONDITIONAL_SPARK_POLICY_TERMS=(
  "環境やAGENTSでSpark指定がある場合"
  "指定がある環境では"
  "ユーザーや環境で指定された routing"
  "未指定なら理由付きで既定"
)

CHANGE_FILES=()

collect_harness_targets() {
  local relative_file
  local target_file

  CHANGE_FILES=()

  while IFS= read -r relative_file; do
    if [[ "$relative_file" = /* ]]; then
      target_file="$relative_file"
    else
      target_file="${WORKSPACE_DIR}/${relative_file}"
    fi

    if [[ ! -f "$target_file" ]]; then
      continue
    fi

    CHANGE_FILES+=("$target_file")
  done < <(
    cd "${WORKSPACE_DIR}" &&
      list_change_files || true
  )

  if [[ ${#CHANGE_FILES[@]} -eq 0 ]]; then
    fail_fast "${CHANGES_ROOT}" "openspec/changes 配下に tasks.md / handoff.md が見つかりません"
  fi
}

for required_file in "$JUSTFILE" "$REPO_POLICY_FILE" "$CI_WORKFLOW_FILE"; do
  if [[ ! -f "$required_file" ]]; then
    fail_fast "missing file" "${required_file}"
  fi
done

collect_harness_targets

check_started_changes_have_handoff
check_policy_contracts
check_justfile_contract
check_ci_contract

for required_file in "${CHANGE_FILES[@]}"; do
  check_forbidden_terms_absent \
    "$required_file" \
    "${OPTIONAL_DELEGATION_TERMS[@]}" \
    "${CONDITIONAL_SPARK_POLICY_TERMS[@]}"
  check_forbidden_regex_terms_absent "$required_file"
  check_file "$required_file"
done

check_changed_harness_files_have_evidence

echo "check-subagent-spark-harness: ok"
