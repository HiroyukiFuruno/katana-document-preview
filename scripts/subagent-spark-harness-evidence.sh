#!/usr/bin/env bash

evidence_line_pattern() {
  local file="$1"
  local evidence_token

  evidence_token="(${DELEGATION_KEYWORD_PATTERN}|agent:[[:space:]]*|model:[[:space:]]*.*gpt-5[.]3-codex-spark)"

  case "$(basename "$file")" in
    tasks.md)
      printf '^[[:space:]]*-[[:space:]]*\\[[[:space:]xX/-]\\][[:space:]].*%s' \
        "$evidence_token"
      ;;
    *)
      printf '^[[:space:]]*-[[:space:]].*%s' "$evidence_token"
      ;;
  esac
}

check_evidence_payload() {
  local context="$1"
  local payload="$2"
  local file_ref
  local has_file_reference=0

  validate_backtick_fields \
    "$context" "$payload" agent model reasoning file command verify close
  validate_agent_ids "$context" "$payload"

  if ! payload_has_only_exact_backtick_field_value \
    "model" "$MODEL_TOKEN" "$payload"; then
    fail_fast "$context" \
      "subagent / Spark証跡のmodelは gpt-5.3-codex-spark だけにしてください。"
  fi

  if ! payload_has_only_exact_backtick_field_value \
    "reasoning" "$REASONING_FIELD_VALUE" "$payload"; then
    fail_fast "$context" \
      "subagent / Spark証跡のreasoningは medium だけにしてください。"
  fi

  while IFS= read -r file_ref; do
    has_file_reference=1
    break
  done < <(extract_backtick_field_values "file" "$payload")
  if [[ "$has_file_reference" -eq 0 ]]; then
    fail_fast "$context" \
      "subagent / Spark証跡のfile: フィールドは1件以上必須です。"
  fi

  if ! payload_has_only_exact_backtick_field_value \
    "command" "$SPAWN_COMMAND_TOKEN" "$payload"; then
    fail_fast "$context" \
      "subagent / Spark証跡のcommandは \`${SPAWN_COMMAND_TOKEN}\` だけにしてください。検証コマンドは verify: に分けてください。"
  fi

  validate_verify_commands "$context" "$payload"
  validate_file_references "$WORKSPACE_DIR" "$context" "$payload"

  if ! payload_has_only_exact_backtick_field_value \
    "close" "$CLOSE_COMMAND_TOKEN" "$payload"; then
    fail_fast "$context" \
      "subagent / Spark証跡には close: \`${CLOSE_COMMAND_TOKEN}\` を必須化してください。"
  fi
}

tasks_file_has_started_work() {
  local file="$1"

  grep -Eq '^[[:space:]]*-[[:space:]]*\[[xX/-]\]' "$file"
}

task_line_has_started_work() {
  local line="$1"

  [[ "$line" =~ ^[[:space:]]*-[[:space:]]*\[[xX/-]\][[:space:]] ]]
}

check_delegation_exception() {
  local context="$1"
  local line="$2"

  if [[ "$line" =~ $DELEGATION_EXCEPTION_PATTERN ]]; then
    return
  fi

  fail_fast "$context" \
    "delegation-exception は 単純な一手作業 / 直列のクリティカルパス / 書き込み範囲を明示できない / ユーザーがsubagent利用を禁止 のいずれかを明示してください。"
}

file_has_delegation_exception() {
  local file="$1"
  local line
  local line_no=0

  while IFS= read -r line; do
    line_no=$((line_no + 1))
    [[ "$line" == *"delegation-exception:"* ]] || continue
    check_delegation_exception "${file} (line ${line_no})" "$line"
    return 0
  done <"$file"

  return 1
}

check_strict_task_line() {
  local file="$1"
  local line_no="$2"
  local line="$3"
  local payload

  if [[ "$line" == *"delegation-exception:"* ]]; then
    check_delegation_exception "${file} (line ${line_no})" "$line"
    return
  fi

  if [[ "$line" == *"証跡:"* ]]; then
    payload="${line#*証跡:}"
    check_evidence_payload "${file} (line ${line_no})" "$payload"
    return
  fi

  fail_fast "${file} (line ${line_no})" \
    "strict marker以降の作業行には subagent / Spark の 証跡: または delegation-exception: を必須化してください。"
}

check_strict_task_section() {
  local file="$1"
  local line
  local line_no=0
  local marker_seen=0
  local started=0

  while IFS= read -r line; do
    line_no=$((line_no + 1))
    if [[ "$line" == *"$STRICT_TASK_MARKER"* ]]; then
      marker_seen=1
      continue
    fi
    if task_line_has_started_work "$line"; then
      started=1
    fi
    if [[ "$marker_seen" -eq 1 ]] && task_line_has_started_work "$line"; then
      check_strict_task_line "$file" "$line_no" "$line"
    fi
  done <"$file"

  if [[ "$started" -eq 1 ]] && [[ "$marker_seen" -eq 0 ]]; then
    fail_fast "$file" \
      "作業済みactive tasks.mdには ${STRICT_TASK_MARKER} を置き、以降の作業行をstrict検査してください。"
  fi
}

check_file() {
  local file="$1"
  local task
  local line_no
  local evidence
  local payload
  local pattern
  local matched=0
  local line

  pattern="$(evidence_line_pattern "$file")"

  while IFS= read -r task; do
    matched=1
    line_no="${task%%:*}"
    evidence="${task#*:}"
    evidence="${evidence# }"

    if [[ "$evidence" == *"delegation-exception:"* ]]; then
      check_delegation_exception "${file} (line ${line_no})" "$evidence"
      continue
    fi

    if [[ "$evidence" != *"証跡:"* ]]; then
      fail_fast "${file} (line ${line_no})" \
        "subagent / Spark項目に証跡がありません。 [ ] / [/] / [x] 行には 証跡: を必須化してください。"
    fi

    payload="${evidence#*証跡:}"
    check_evidence_payload "${file} (line ${line_no})" "$payload"
  done < <(match_evidence_lines "$pattern" "$file")

  if [[ "$(basename "$file")" == "tasks.md" ]] &&
    [[ "$matched" -eq 0 ]] &&
    tasks_file_has_started_work "$file" &&
    ! file_has_delegation_exception "$file"; then
    fail_fast "$file" \
      "作業済みactive tasks.mdにはsubagent / Spark証跡または許可済みdelegation-exceptionを残してください。"
  fi

  if [[ "$(basename "$file")" == "tasks.md" ]]; then
    check_strict_task_section "$file"
  fi

  line_no=0
  while IFS= read -r line; do
    line_no=$((line_no + 1))
    if [[ "$line" != *"証跡:"* ]] || [[ "$line" != *"${SPAWN_COMMAND_TOKEN}"* ]]; then
      continue
    fi
    if [[ "$line" =~ $pattern ]]; then
      continue
    fi
    check_evidence_payload "${file} (line ${line_no})" "${line#*証跡:}"
  done <"$file"
}
