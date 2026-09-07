#!/usr/bin/env bash

run_sort() {
  if command -v rtk >/dev/null 2>&1; then
    rtk sort
    return
  fi

  sort
}

list_change_files() {
  if command -v rtk >/dev/null 2>&1; then
    rtk rg --files openspec/changes \
      -g 'openspec/changes/**/tasks.md' \
      -g 'openspec/changes/**/handoff.md' \
      -g '!openspec/changes/archive/**' \
      | run_sort
    return
  fi

  if command -v rg >/dev/null 2>&1; then
    rg --files openspec/changes \
      -g 'openspec/changes/**/tasks.md' \
      -g 'openspec/changes/**/handoff.md' \
      -g '!openspec/changes/archive/**' \
      | run_sort
    return
  fi

  find openspec/changes \
    -path 'openspec/changes/archive' -prune -o \
    \( -name tasks.md -o -name handoff.md \) -print \
    | run_sort
}

match_evidence_lines() {
  local pattern="$1"
  local file="$2"

  if command -v rtk >/dev/null 2>&1; then
    rtk rg -n --no-filename "${pattern}" "$file" || true
    return
  fi

  if command -v rg >/dev/null 2>&1; then
    rg -n --no-filename "${pattern}" "$file" || true
    return
  fi

  grep -En "${pattern}" "$file" || true
}

fail_fast() {
  printf 'FAIL: %s\n' "$1" >&2
  printf '  %s\n' "$2" >&2
  exit 1
}

extract_backtick_field_values() {
  local field="$1"
  local payload="$2"
  local rest="$payload"
  local pattern

  pattern="${field}:[[:space:]]*\`([^\`]*)\`"
  while [[ "$rest" =~ $pattern ]]; do
    printf '%s\n' "${BASH_REMATCH[1]}"
    rest="${rest#*"${BASH_REMATCH[0]}"}"
  done
}

payload_has_exact_backtick_field_value() {
  local field="$1" expected="$2" payload="$3" value

  while IFS= read -r value; do
    if [[ "$value" == "$expected" ]]; then
      return 0
    fi
  done < <(extract_backtick_field_values "$field" "$payload")

  return 1
}

payload_has_only_exact_backtick_field_value() {
  local field="$1" expected="$2" payload="$3" value found=0

  while IFS= read -r value; do
    found=1
    [[ "$value" == "$expected" ]] || return 1
  done < <(extract_backtick_field_values "$field" "$payload")

  [[ "$found" -eq 1 ]]
}

validate_file_references() {
  local workspace_dir="$1"
  local context="$2"
  local payload="$3"
  local file_ref
  local resolved_path

  while IFS= read -r file_ref; do
    case "$file_ref" in
      "" | "." | "./" | ".." | "../" | *"://"*)
        fail_fast "$context" "subagent / Spark証跡のfile参照が不正です: ${file_ref}"
        ;;
    esac

    if [[ "$file_ref" = /* ]]; then
      resolved_path="$file_ref"
    else
      resolved_path="${workspace_dir}/${file_ref}"
    fi

    if [[ ! -e "$resolved_path" ]]; then
      fail_fast "$context" "subagent / Spark証跡のfile参照が存在しません: ${file_ref}"
    fi
  done < <(extract_backtick_field_values "file" "$payload")
}

should_check_external_policy() {
  local mode="$1"
  local global_agents_file="$2"
  local task_delegation_skill_file="$3"

  case "$mode" in
    1)
      return 0
      ;;
    0)
      return 1
      ;;
    auto)
      [[ -f "$global_agents_file" && -f "$task_delegation_skill_file" ]]
      return
      ;;
    *)
      fail_fast "KDV_SUBAGENT_HARNESS_CHECK_EXTERNAL" \
        "allowed values are auto, 0, or 1."
      ;;
  esac
}

check_required_terms() {
  local file="$1"
  shift
  local source
  local term

  source="$(<"$file")"
  for term in "$@"; do
    if [[ "$source" != *"$term"* ]]; then
      fail_fast "${file}" "required subagent / Spark policy term is missing: ${term}"
    fi
  done
}

check_reasoning_medium_policy() {
  local file="$1"

  if ! grep -Eq 'reasoning[^`[:cntrl:]]*`medium`' "$file"; then
    fail_fast "$file" \
      'required subagent / Spark policy term is missing: reasoning `medium`'
  fi
}

check_main_orchestrator_policy() {
  local file="$1"

  if ! grep -Eq 'main[[:space:]]+(agent|Codex)[^[:cntrl:]]*設計[^[:cntrl:]]*統合' "$file"; then
    fail_fast "$file" \
      "required subagent / Spark policy term is missing: main agent/Codex must own design and integration."
  fi
}

check_external_delegation_policy() {
  local file="$1"

  check_required_terms "$file" \
    "$MODEL_TOKEN" \
    "分離できる実装・調査" \
    "親モデル継承" \
    "省略しない" \
    "単純な一手作業" \
    "直列のクリティカルパス" \
    "書き込み範囲を明示できない"

  if ! grep -Eq '(原則[[:space:]]+subagent|既定で許可)' "$file"; then
    fail_fast "$file" \
      "required subagent / Spark policy term is missing: proactive subagent delegation."
  fi

  if ! grep -Eq 'ユーザー[^[:cntrl:]]*(禁止|使わない)' "$file"; then
    fail_fast "$file" \
      "required subagent / Spark policy term is missing: user prohibition exception."
  fi

  if ! grep -Eq '(subagent[^[:cntrl:]]*分離できる実装・調査|分離できる実装・調査[^[:cntrl:]]*subagent)' "$file"; then
    fail_fast "$file" \
      "required subagent / Spark policy term is missing: delegable work ownership."
  fi

  check_reasoning_medium_policy "$file"
  check_main_orchestrator_policy "$file"
}

normalize_policy_text() {
  local source="$1"

  printf '%s' "$source" | tr '[:upper:]' '[:lower:]' | tr -d '[:punct:]'
}

check_forbidden_terms_absent() {
  local file="$1"
  shift
  local source
  local normalized_source
  local term
  local normalized_term

  source="$(<"$file")"
  normalized_source="$(normalize_policy_text "$source")"
  for term in "$@"; do
    normalized_term="$(normalize_policy_text "$term")"
    if [[ "$normalized_source" == *"$normalized_term"* ]]; then
      fail_fast "${file}" "forbidden optional subagent policy term is present: ${term}"
    fi
  done
}

recipe_contains_command() {
  local file="$1"
  local recipe="$2"
  local command="$3"

  awk -v recipe="${recipe}:" -v command="$command" '
    $0 == recipe { in_recipe = 1; next }
    in_recipe && $0 ~ /^[^[:space:]#]/ { exit 1 }
    in_recipe {
      line = $0
      sub(/^[[:space:]]+/, "", line)
      if (line ~ /^#/) next
      sub(/^[@-]+[[:space:]]*/, "", line)
      if (line == command) { found = 1; exit 0 }
    }
    END { if (!found) exit 1 }
  ' "$file"
}
