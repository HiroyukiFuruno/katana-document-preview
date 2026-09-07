#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo_bin="${CARGO:-cargo}"
embedded_kuc_root="$repo_root/crates/katana-ui-core"
metadata_file=""

cleanup_metadata_file() {
  if [[ -n "$metadata_file" && -f "$metadata_file" ]]; then
    rm -f "$metadata_file"
  fi
}
trap cleanup_metadata_file EXIT

cd "$repo_root"

if [[ -n "${KUC_ROOT:-}" ]]; then
  echo "document-surface-boundary-check: KUC_ROOT overrides are forbidden; use the registry artifact" >&2
  exit 1
fi

packages=(
  katana-document-viewer
  kdv-storybook
)

vendor_pattern='(^|[[:space:]├└─│])((eframe|egui|gpui|floem|floem_reactive|floem_renderer|winit|vello|katana-ui-core-(egui|gpui|floem)|katana-document-preview-egui))[[:space:]]'
source_pattern='eframe|egui|gpui|floem|winit|vello|katana-ui-core-(egui|gpui|floem)|katana-document-preview-egui'
storybook_sidebar="$repo_root/tools/kdv-storybook/src/sidebar.rs"
storybook_sidebar_sources=(
  "$repo_root/tools/kdv-storybook/src/sidebar.rs"
  "$repo_root/tools/kdv-storybook/src/sidebar_settings.rs"
)
storybook_source="$repo_root/tools/kdv-storybook/src"
storybook_forbidden_tree_pattern='CollapsingHeader|Button::selectable|TreeView::new|TreeNode::new|KucTreeCanvasRenderer|KucTextCanvasRenderer|KucControlCanvasRenderer|KucTreeViewCanvasRenderer|mod kuc_|egui|gpui|floem|winit|vello'
storybook_forbidden_toggle_pattern='self\.dark[[:space:]]*=[[:space:]]*!self\.dark|[*]value[[:space:]]*=[[:space:]]*![*]value|interaction\.[[:alnum:]_]+[[:space:]]*=[[:space:]]*!interaction\.'
storybook_forbidden_settings_hit_pattern='sidebar_hit_walk|SETTINGS_ROW_HEIGHT|hit_settings_node|from_state_id|settings-field:'
storybook_forbidden_settings_state_pattern='mode_from_label|apply_interaction_field|ViewerMode|ViewerInteractionConfig|hover_highlight_enabled|selection_enabled|image_controls_enabled|diagram_controls_enabled'
storybook_forbidden_direct_host_action_pattern='props\(\)\.common\.host_actions|UiHostActionSpec'
storybook_forbidden_media_hit_pattern='mouse_media_layout|kdv-diagram-frame|kdv-code-frame|BUTTON_SLOT_WIDTH|DEFAULT_LINE_HEIGHT|style_classes|viewer[.](image|diagram|code)[.]'
storybook_forbidden_kuc_media_boundary_pattern='UiMediaControlTarget|UiMediaControlAction|media_control_action'
storybook_forbidden_state_action_pattern='TASK_STATE_PREFIX|kdv-task-state:|task_state_id|starts_with[(]TASK_STATE_PREFIX[)]'
storybook_forbidden_mouse_host_geometry_pattern='link_action_matches_document_line|LinkAction|matches_document_line|hovered_link_uses_pointer|hovered_media_control_uses_pointer|ACCORDION_HEADER_HEIGHT|node_cursor|ViewerTarget|props[(][)][.]state_id'
storybook_forbidden_task_host_geometry_pattern='CHECKBOX_HIT_WIDTH|UiHostActionPlan|UiTaskMarker|full_target|row_rect|fn hit[(]'
storybook_forbidden_task_style_collection_pattern='kdv-task-checkbox|style_classes'
storybook_forbidden_window_presentation_pattern='(^|[^[:alnum:]_])present_frame[[:space:]]*[(]|fn should_present_physical_frame_directly|scale_factor[[:space:]]*[(][[:space:]]*[)][[:space:]]*>[[:space:]]*1[.]0'
kdv_kuc_forbidden_manual_interactive_pattern='cursor[(]UiCursor::Pointer[)]'
kuc_forbidden_viewer_semantic_pattern='ViewerMediaControl|UiMediaControl|media_control_action|viewer[.](image|diagram|code)|ui[.]diagram|diagram[.]zoom|kdv-|MediaControl'
forbidden_duplicate_viewer_media_prefix_pattern='viewer[.](image|diagram|code)[.]'
forbidden_adapter_media_control_contract_pattern='"(fit|open|copy|reveal-in-os|zoom-in|zoom-out|fullscreen|copy-source|pan-up|pan-left|reset-view|pan-right|trackpad-help|pan-down|copy-code)"'
kuc_storybook_forbidden_kdv_overlay_pattern='kdv-diagram-frame|kdv-diagram-toolbar|kdv-diagram-top-controls|DIAGRAM_FRAME_CLASS|DIAGRAM_CONTROLS_CLASS|DIAGRAM_TOP_CONTROLS_CLASS'
kuc_storybook_forbidden_viewer_media_action_pattern='viewer[.](image|diagram|code)[.]|ui[.]diagram|diagram[.]zoom|surface[.]overlay[.](zoom|pan|copy-source|fullscreen|full|copy|in)'
kuc_storybook_forbidden_document_rule_pattern='kdv-document-rule|DOCUMENT_RULE_CLASS'
kuc_storybook_forbidden_document_media_pattern='kdv-document-media|DOCUMENT_MEDIA_CLASS|DOCUMENT_MEDIA_VERTICAL_MARGIN'
kuc_storybook_forbidden_code_frame_pattern='kdv-document-code|kdv-code-frame|kdv-code-controls|CODE_FRAME_CLASS|CODE_CONTROLS_CLASS|CODE_CONTROL_MARGIN|CODE_CONTROLS_WIDTH'
kuc_storybook_forbidden_alert_pattern='kdv-alert-'
kuc_storybook_forbidden_list_pattern='kdv-list'
kuc_storybook_forbidden_quote_heading_pattern='kdv-document-quote|kdv-document-heading|QUOTE_DEPTH_CLASS_PREFIX'
neutral_source_roots=(
  "$repo_root/crates/katana-document-viewer/src"
  "$repo_root/tools/kdv-storybook/src"
)
neutral_package_roots=(
  "$repo_root/crates/katana-document-viewer"
  "$repo_root/tools/kdv-storybook"
)
neutral_extra_roots=()
document_viewer_manifest="$repo_root/crates/katana-document-viewer/Cargo.toml"
document_surface_source="$repo_root/crates/katana-document-viewer/src/document_surface"
kuc_workspace_manifest="$repo_root/Cargo.toml"

resolve_cargo_package_root() {
  local package="$1"
  local source_prefix="$2"
  local version="$3"
  if [[ -z "$metadata_file" ]]; then
    metadata_file="$(mktemp)"
    "$cargo_bin" metadata --locked --format-version 1 --all-features > "$metadata_file"
  fi
python3 - "$package" "$source_prefix" "$version" "$metadata_file" <<'PY'
import json
import pathlib
import sys

package_name = sys.argv[1]
source_prefix = sys.argv[2]
version = sys.argv[3]
metadata_path = sys.argv[4]
with open(metadata_path, encoding="utf-8") as metadata_file:
    metadata = json.load(metadata_file)
for package in metadata["packages"]:
    source = package.get("source") or ""
    if (
        package["name"] == package_name
        and package["version"] == version
        and source.startswith(source_prefix)
    ):
        print(pathlib.Path(package["manifest_path"]).parent.as_posix())
        sys.exit(0)
sys.exit(1)
PY
}

resolve_cargo_package_tree() {
  local package="$1"
  local source_prefix="$2"
  local version="$3"
  if [[ -z "$metadata_file" ]]; then
    metadata_file="$(mktemp)"
    "$cargo_bin" metadata --locked --format-version 1 --all-features > "$metadata_file"
  fi
python3 - "$package" "$source_prefix" "$version" "$metadata_file" <<'PY'
import json
import sys

package_name = sys.argv[1]
source_prefix = sys.argv[2]
version = sys.argv[3]
metadata_path = sys.argv[4]
with open(metadata_path, encoding="utf-8") as metadata_file:
    metadata = json.load(metadata_file)

packages = {package["id"]: package for package in metadata["packages"]}
roots = [
    package["id"]
    for package in metadata["packages"]
    if package["name"] == package_name
    and package["version"] == version
    and (package.get("source") or "").startswith(source_prefix)
]
if len(roots) != 1:
    sys.exit(1)

nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
pending = roots
visited = set()
while pending:
    package_id = pending.pop()
    if package_id in visited:
        continue
    visited.add(package_id)
    package = packages[package_id]
    print(f'{package["name"]} v{package["version"]}')
    pending.extend(dependency["pkg"] for dependency in nodes[package_id]["deps"])
PY
}

check_kuc_core_semantic_boundary() {
  local label="$1"
  local root="$2"
  local semantic_paths=("$root/src")

  if [[ -d "$root/tests" ]]; then
    semantic_paths+=("$root/tests")
  fi

  if grep -R -n -E "$kuc_forbidden_viewer_semantic_pattern" "${semantic_paths[@]}"; then
    echo "document-surface-boundary-check: viewer media semantics leaked into $label" >&2
    exit 1
  fi
}

check_embedded_kuc_source_boundary() {
  local label="$1"
  local root="$2"
  local source_paths=("$root/Cargo.toml" "$root/src")

  if grep -R -n -E "$source_pattern" "${source_paths[@]}"; then
    echo "document-surface-boundary-check: vendor runtime source reference leaked into $label" >&2
    exit 1
  fi

  check_kuc_core_semantic_boundary "$label" "$root"
}

check_file_tree_facade_contract() {
  local label="$1"
  local file="$2"
  local file_dir
  local module_files=()

  if [[ ! -f "$file" ]]; then
    return
  fi

  file_dir="$(dirname "$file")"
  module_files+=("$file")
  if [[ -f "$file_dir/file_tree_model.rs" ]]; then
    module_files+=("$file_dir/file_tree_model.rs")
  fi
  if [[ -f "$file_dir/file_tree_types.rs" ]]; then
    module_files+=("$file_dir/file_tree_types.rs")
  fi

  if ! grep -q 'TreeView::new' "${module_files[@]}"; then
    echo "document-surface-boundary-check: FileTree in $label must build KUC TreeView" >&2
    exit 1
  fi

  if ! grep -q 'TreeViewHitTestInput' "${module_files[@]}"; then
    echo "document-surface-boundary-check: FileTree in $label must delegate hit-test to KUC TreeView" >&2
    exit 1
  fi

  if ! grep -q 'TreeViewAction::SelectNode' "${module_files[@]}"; then
    echo "document-surface-boundary-check: FileTree in $label must map KUC TreeView selection action" >&2
    exit 1
  fi

  if ! grep -q 'TreeViewAction::ToggleNode' "${module_files[@]}"; then
    echo "document-surface-boundary-check: FileTree in $label must map KUC TreeView directory toggle action" >&2
    exit 1
  fi

  if grep -n -E 'UiNodeKind::(Text|Row|Column)|Text::new|Row::new|Column::new' "${module_files[@]}"; then
    echo "document-surface-boundary-check: FileTree in $label must stay a TreeView facade, not build a separate row UI" >&2
    exit 1
  fi
}

check_file_tree_facade_contract_for_root() {
  local label="$1"
  local root="$2"
  local candidates=()

  if [[ ! -d "$root/src" ]]; then
    return
  fi

  if ! grep -R -q 'pub struct FileTree' "$root/src" 2>/dev/null; then
    echo "document-surface-boundary-check: FileTree type is missing in $label" >&2
    exit 1
  fi

  while IFS= read -r file; do
    candidates+=("$file")
  done < <(grep -R -l -E '^impl[[:space:]]+FileTree[[:space:]]*\{' "$root/src" 2>/dev/null || true)

  if [[ ${#candidates[@]} -eq 0 ]]; then
    echo "document-surface-boundary-check: FileTree implementation is missing in $label" >&2
    exit 1
  fi

  for file in "${candidates[@]}"; do
    check_file_tree_facade_contract "$label" "$file"
  done
}

for package in "${packages[@]}"; do
  tree="$("$cargo_bin" tree -p "$package" --locked)"
  if printf '%s\n' "$tree" | grep -E "$vendor_pattern"; then
    echo "document-surface-boundary-check: vendor runtime dependency leaked into $package" >&2
    exit 1
  fi
done

if ! grep -q '^katana-ui-core = "=0[.]3[.]7"$' "$document_viewer_manifest"; then
  echo "document-surface-boundary-check: KDV document surface must use exact registry KUC 0.3.7" >&2
  exit 1
fi

if grep -n -E '(^|[[:space:]])(eframe|egui)[[:space:]]*=' "$document_viewer_manifest"; then
  echo "document-surface-boundary-check: KDV must remain independent from egui and eframe" >&2
  exit 1
fi

if [[ -e "$repo_root/crates/katana-document-viewer-kuc" ]] || grep -q '"crates/katana-document-viewer-kuc"' "$kuc_workspace_manifest"; then
  echo "document-surface-boundary-check: the cross-layer katana-document-viewer-kuc crate is forbidden" >&2
  exit 1
fi

if ! grep -Eq '^katana-ui-core = \{ version = "=0[.]3[.]7",.*raster-host' "$kuc_workspace_manifest" || \
   grep -Eq '^katana-ui-core-storybook[[:space:]]*=' "$kuc_workspace_manifest"; then
  echo "document-surface-boundary-check: KDV workspace must use one exact registry KUC v0.3.7 raster-host dependency" >&2
  exit 1
fi

if grep -n -E 'katana-ui-core[^\n]*(git|path)[[:space:]]*=' "$kuc_workspace_manifest" "$document_viewer_manifest"; then
  echo "document-surface-boundary-check: KDV must not override registry KUC with git/path" >&2
  exit 1
fi

if ! grep -R -q 'GenericGrid' "$document_surface_source" || ! grep -R -q 'ImageSurface' "$document_surface_source"; then
  echo "document-surface-boundary-check: KDV document surface must reuse KUC grid and image surface" >&2
  exit 1
fi

if grep -R -n -E 'use[[:space:]]+(hayro|ironcalc|office2pdf|quick_xml|xmltree|zip)' "$document_surface_source"; then
  echo "document-surface-boundary-check: format engines must remain outside the KDV presentation surface" >&2
  exit 1
fi

if grep -R -n -E 'struct[[:space:]]+Grid(Cell|Layout|Viewport|Selection|HitTest)|fn[[:space:]]+(hit_test|plan_grid_layout|navigate_selection)' "$document_surface_source"; then
  echo "document-surface-boundary-check: KDV document surface must not duplicate KUC grid geometry or interaction" >&2
  exit 1
fi

kuc_core_source_root="$(resolve_cargo_package_root katana-ui-core registry+ 0.3.7)"

if [[ ! -f "$kuc_core_source_root/Cargo.toml" ]]; then
  echo "document-surface-boundary-check: registry katana-ui-core v0.3.7 source is unavailable" >&2
  exit 1
fi

kuc_manifest="$kuc_core_source_root/Cargo.toml"
if ! grep -q '^raster-host[[:space:]]*=' "$kuc_manifest" || \
   ! grep -q '^pub mod raster_host;' "$kuc_core_source_root/src/lib.rs"; then
  echo "document-surface-boundary-check: KUC v0.3.7 must expose the public raster-host API" >&2
  exit 1
fi

kuc_tree="$(resolve_cargo_package_tree katana-ui-core registry+ 0.3.7)"
# KUC の任意 backend は未有効化でもソースに存在するため、解決済み依存グラフで判定する。
if printf '%s\n' "$kuc_tree" | grep -E "$vendor_pattern"; then
  echo "document-surface-boundary-check: vendor runtime dependency leaked into katana-ui-core" >&2
  exit 1
fi

check_kuc_core_semantic_boundary "katana-ui-core" "$kuc_core_source_root"
check_file_tree_facade_contract_for_root "katana-ui-core" "$kuc_core_source_root"

if [[ -f "$embedded_kuc_root/Cargo.toml" ]]; then
  check_embedded_kuc_source_boundary "embedded KDV crates/katana-ui-core" "$embedded_kuc_root"
  if grep -R -q 'pub struct FileTree' "$embedded_kuc_root/src" 2>/dev/null; then
    check_file_tree_facade_contract_for_root "embedded KDV crates/katana-ui-core" "$embedded_kuc_root"
  fi
fi

for package_root in "${neutral_package_roots[@]}"; do
  for target in build.rs examples benches; do
    path="$package_root/$target"
    if [[ -e "$path" ]]; then
      neutral_extra_roots+=("$path")
    fi
  done
done

matches="$(
  find "${neutral_source_roots[@]}" "${neutral_extra_roots[@]}" \
    -type f \
    \( -name '*.rs' -o -name 'Cargo.toml' \) \
    ! -name '*tests.rs' \
    ! -path '*/tests/*' \
    -print0 \
    | xargs -0 grep -n -E "$source_pattern" || true
)"

if [[ -n "$matches" ]]; then
  printf '%s\n' "$matches"
  echo "document-surface-boundary-check: vendor runtime source reference leaked into KDV/KUC neutral implementation" >&2
  exit 1
fi

if ! grep -q 'FileTree::render' "$storybook_sidebar"; then
  echo "document-surface-boundary-check: Storybook sidebar must render file selection through KUC FileTree" >&2
  exit 1
fi

if ! grep -R -q 'SettingsList::new' "${storybook_sidebar_sources[@]}"; then
  echo "document-surface-boundary-check: Storybook sidebar must render settings through KUC SettingsList" >&2
  exit 1
fi

if grep -R -n -E "$storybook_forbidden_tree_pattern" "$storybook_source"; then
  echo "document-surface-boundary-check: vendor or independent tree UI leaked into vendor-free Storybook" >&2
  exit 1
fi

if grep -R -n -E "$storybook_forbidden_toggle_pattern" "$storybook_source"; then
  echo "document-surface-boundary-check: Storybook settings toggles must go through KUC SettingsListAction" >&2
  exit 1
fi

if grep -R -n -E "$storybook_forbidden_settings_hit_pattern" "$storybook_source"; then
  echo "document-surface-boundary-check: Storybook settings hit-test must go through KUC SettingsList::hit_test" >&2
  exit 1
fi

if [[ -e "$storybook_source/settings_action_value.rs" ]]; then
  echo "document-surface-boundary-check: Storybook settings value conversion must not live in a detached helper that bypasses KUC SettingsList actions" >&2
  exit 1
fi

if grep -R -n -E "$storybook_forbidden_direct_host_action_pattern" "$storybook_source"; then
  echo "document-surface-boundary-check: Storybook host actions must go through KUC UiHostActionPlan" >&2
  exit 1
fi

window_presentation_matches="$(
  find "$storybook_source" \
    -type f \
    -name '*.rs' \
    ! -name '*_tests.rs' \
    ! -path '*/tests/*' \
    -print0 \
    | xargs -0 grep -n -E "$storybook_forbidden_window_presentation_pattern" || true
)"

if [[ -n "$window_presentation_matches" ]]; then
  printf '%s\n' "$window_presentation_matches"
  echo "document-surface-boundary-check: Storybook window presentation must go through KUC present_frame_for_window" >&2
  exit 1
fi

if grep -R -n -E "$storybook_forbidden_media_hit_pattern" "$storybook_source"/mouse_media*.rs; then
  echo "document-surface-boundary-check: Storybook media hit-test must go through KUC Storybook renderer hit rects" >&2
  exit 1
fi

if grep -R -n -E "$storybook_forbidden_kuc_media_boundary_pattern" "$storybook_source"; then
  echo "document-surface-boundary-check: Storybook must not depend on KUC viewer media control semantics" >&2
  exit 1
fi

if grep -R -n -E "$storybook_forbidden_state_action_pattern" \
  "$storybook_source"/window_command.rs \
  "$storybook_source"/mouse_task.rs; then
  echo "document-surface-boundary-check: Storybook task actions must go through KUC TaskControlAction" >&2
  exit 1
fi

if grep -n -E "$storybook_forbidden_mouse_host_geometry_pattern" \
  "$storybook_source"/mouse.rs \
  "$storybook_source"/mouse_cursor.rs \
  "$storybook_source"/window_accordion.rs; then
  echo "document-surface-boundary-check: Storybook link/accordion hover and click must use KUC host action hit rects" >&2
  exit 1
fi

if grep -n -E "$storybook_forbidden_task_host_geometry_pattern" \
  "$storybook_source"/mouse_task.rs; then
  echo "document-surface-boundary-check: Storybook task hit-test must use KUC host action hit rects and KDV task state" >&2
  exit 1
fi

if grep -n -E "$storybook_forbidden_task_style_collection_pattern" \
  "$storybook_source"/preview_interaction_command_support.rs; then
  echo "document-surface-boundary-check: Storybook task action collection must use KUC host action plans instead of style classes" >&2
  exit 1
fi

if [[ -d "$storybook_source/kuc_bridge" ]]; then
  echo "document-surface-boundary-check: KDV Storybook must not own kuc_bridge; use KUC document_viewer host contract" >&2
  exit 1
fi

core_adapter_reference_matches="$(
  grep -R -n -E 'katana_document_viewer_kuc|katana-document-viewer-kuc' \
    "$repo_root/crates/katana-document-viewer/Cargo.toml" \
    "$repo_root/crates/katana-document-viewer/src" \
    || true
)"

if [[ -n "$core_adapter_reference_matches" ]]; then
  printf '%s\n' "$core_adapter_reference_matches"
  echo "document-surface-boundary-check: KDV must not reference the forbidden cross-layer crate" >&2
  exit 1
fi

duplicate_viewer_media_prefix_matches="$(
  find \
    "$repo_root/tools/kdv-storybook/src" \
    -type f \
    -name '*.rs' \
    ! -name '*tests.rs' \
    ! -name 'tests.rs' \
    -print0 \
    | xargs -0 grep -n -E "$forbidden_duplicate_viewer_media_prefix_pattern" || true
)"

if [[ -n "$duplicate_viewer_media_prefix_matches" ]]; then
  printf '%s\n' "$duplicate_viewer_media_prefix_matches"
  echo "document-surface-boundary-check: viewer media action prefix must be owned by KDV core ViewerMediaControlAction" >&2
  exit 1
fi

if ! grep -R -q 'SettingsListAction::UpdateField' "$storybook_source"; then
  echo "document-surface-boundary-check: Storybook settings actions must use KUC SettingsListAction::UpdateField" >&2
  exit 1
fi

"$cargo_bin" test -p kdv-linter --locked storybook_contract_flags_manual_hit_test_and_action_synthesis -- --test-threads=1
"$cargo_bin" test -p kdv-linter --locked storybook_contract_ignores_window_presentation_terms_in_tests -- --test-threads=1

echo "document-surface-boundary-check: ok"
