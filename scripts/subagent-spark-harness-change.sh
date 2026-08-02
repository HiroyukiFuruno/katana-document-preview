#!/usr/bin/env bash

check_started_changes_have_handoff() {
  local file
  local handoff_file
  local pattern

  for file in "${CHANGE_FILES[@]}"; do
    [[ "$(basename "$file")" == "tasks.md" ]] || continue
    tasks_file_has_started_work "$file" || continue

    handoff_file="$(dirname "$file")/handoff.md"
    if [[ ! -f "$handoff_file" ]]; then
      fail_fast "$file" \
        "開始済みactive changeには同階層のhandoff.mdを必須化してください。"
    fi

    pattern="$(evidence_line_pattern "$handoff_file")"
    if [[ -z "$(match_evidence_lines "$pattern" "$handoff_file")" ]] &&
      ! file_has_delegation_exception "$handoff_file"; then
      fail_fast "$handoff_file" \
        "開始済みactive changeのhandoff.mdにはsubagent / Spark証跡または許可済みdelegation-exceptionを必須化してください。"
    fi
  done
}
