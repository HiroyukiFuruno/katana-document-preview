#!/usr/bin/env bash

is_harness_owned_path() {
  local path="$1"
  local name="${path##*/}"

  case "$path" in
    Justfile | \
      .github/workflows/test-and-build.yml | \
      .codex/workflows/subagent-spark-policy.md | \
      scripts/check-subagent-spark-harness.sh)
      return 0
      ;;
  esac

  case "$name" in
    check-subagent-spark-harness*.sh | subagent-spark-harness*.sh)
      [[ "$path" == scripts/* ]] && return 0
      ;;
  esac

  return 1
}

git_in_workspace_without_hook_environment() {
  env \
    -u GIT_ALTERNATE_OBJECT_DIRECTORIES \
    -u GIT_COMMON_DIR \
    -u GIT_DIR \
    -u GIT_INDEX_FILE \
    -u GIT_OBJECT_DIRECTORY \
    -u GIT_WORK_TREE \
    git -C "$WORKSPACE_DIR" "$@"
}

git_worktree_available() {
  git_in_workspace_without_hook_environment \
    rev-parse --is-inside-work-tree >/dev/null 2>&1
}

status_path_from_line() {
  local line="$1"
  local path="${line:3}"

  if [[ "$path" == *" -> "* ]]; then
    path="${path##* -> }"
  fi

  printf '%s\n' "$path"
}

changed_status_is_deleted() {
  local status="$1"

  [[ "$status" != "??" && "$status" == *D* ]]
}

collect_changed_harness_files() {
  local line
  local path
  local status

  if ! git_worktree_available; then
    return
  fi

  while IFS= read -r line; do
    [[ -n "$line" ]] || continue

    status="${line:0:2}"
    path="$(status_path_from_line "$line")"
    is_harness_owned_path "$path" || continue
    changed_status_is_deleted "$status" && continue

    printf '%s\n' "$path"
  done < <(
    git_in_workspace_without_hook_environment \
      status --porcelain=v1 --untracked-files=all
  )
}

normalize_evidence_file_ref() {
  local file_ref="$1"

  if [[ "$file_ref" = "$WORKSPACE_DIR"/* ]]; then
    printf '%s\n' "${file_ref#"$WORKSPACE_DIR/"}"
    return
  fi

  printf '%s\n' "${file_ref#./}"
}

evidence_references_file() {
  local expected="$1"
  local evidence_file
  local line
  local payload
  local file_ref

  for evidence_file in "${CHANGE_FILES[@]}"; do
    while IFS= read -r line; do
      if [[ "$line" == *"証跡:"* ]]; then
        payload="${line#*証跡:}"
      elif [[ "$line" == *"delegation-exception:"* ]]; then
        payload="$line"
      else
        continue
      fi

      while IFS= read -r file_ref; do
        if [[ "$(normalize_evidence_file_ref "$file_ref")" == "$expected" ]]; then
          return 0
        fi
      done < <(extract_backtick_field_values "file" "$payload")
    done <"$evidence_file"
  done

  return 1
}

check_changed_harness_files_have_evidence() {
  local changed_file

  while IFS= read -r changed_file; do
    [[ -n "$changed_file" ]] || continue

    if ! evidence_references_file "$changed_file"; then
      fail_fast "$changed_file" \
        "変更されたsubagent / Sparkハーネス関連ファイルはOpenSpec証跡の file: で参照してください。"
    fi
  done < <(collect_changed_harness_files)
}
