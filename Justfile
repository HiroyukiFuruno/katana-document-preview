set shell := ["bash", "-uc"]

REPO_ROOT := justfile_directory()
RTK := env_var_or_default("RTK", `command -v rtk 2> /dev/null || true`)
RTK_CMD := if RTK == "" { "" } else { RTK + " " }
JOBS := env_var_or_default("JOBS", "2")
CARGO := env_var_or_default("CARGO", "cargo")
VERSION := env_var_or_default("VERSION", `awk -F '"' '/^version = / { print $2; exit }' Cargo.toml`)
VERSION_BARE := replace(VERSION, "v", "")
TAG := "v" + VERSION_BARE
COVERAGE_MIN_LINES := "100"
COVERAGE_MAX_UNCOVERED_LINES := "0"
COVERAGE_TARGET_PACKAGES := "-p katana-document-viewer"
COVERAGE_IGNORE_FILENAME_REGEX := "(^|/)(tests?|examples)(/|$)|(^|/)[^/]*(test|tests)[^/]*\\.rs$|/crates/kdv-linter/"
RELEASE_REPO := env_var_or_default("RELEASE_REPO", "HiroyukiFuruno/katana-document-viewer")
KAL_VERSION := env_var_or_default("KAL_VERSION", "0.5.1")
KAL_ROOT := env_var_or_default("KAL_ROOT", REPO_ROOT + "/target/kal")
KAL := env_var_or_default("KAL", KAL_ROOT + "/bin/kal")
STORYBOOK_TARGET_DIR := env_var_or_default("STORYBOOK_TARGET_DIR", REPO_ROOT + "/target")
STORYBOOK_FRAMES := env_var_or_default("STORYBOOK_FRAMES", "0")
KDV_FMT_PACKAGES := "--package kdv-linter --package katana-document-viewer --package kdv-storybook"
# Release gates consume the exact crates.io artifact; legacy local KUC recipes remain opt-in.
KUC_ROOT := env_var_or_default("KUC_ROOT", replace(REPO_ROOT, "/katana-document-viewer", "/katana-ui-core"))

export RUSTFLAGS := env_var_or_default("RUSTFLAGS", "-D warnings")

default: help

# Show available tasks
help:
    @just --list --unsorted

fmt:
    {{CARGO}} fmt {{KDV_FMT_PACKAGES}}

fmt-check:
    {{CARGO}} fmt {{KDV_FMT_PACKAGES}} -- --check

lint:
    {{CARGO}} clippy -j {{JOBS}} --workspace --all-targets --all-features --locked -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::todo -D clippy::unimplemented -D clippy::dbg_macro -D clippy::panic -D clippy::wildcard_imports

ast-lint:
    {{CARGO}} test -p kdv-linter --test ast_linter --locked ast_linter_workspace_rules -- --exact

kdv-lint: ast-lint

test:
    {{CARGO}} test --workspace --all-targets --all-features --locked --exclude kdv-storybook
    {{CARGO}} test -p kdv-storybook --locked -- --test-threads=1 --skip katana_intro_text_keeps_readable_frame_band_heights --skip katana_language_link_underline_reaches_frame_pixels --skip direct_html_margin_left_fixture_reaches_frame_pixels --skip storybook_score_visual_ --skip storybook_sample_ --skip storybook_preview_crop_score_ --skip storybook_score_export_surface_excludes_overlay_controls --skip mouse_click_uses_external_scroll_for_scroll_independent_scene
    {{CARGO}} test -p kdv-storybook --locked mouse_click_uses_external_scroll_for_scroll_independent_scene -- --test-threads=1

# Backward-compatible test entrypoint
unit-test: test

# cargo llvm-cov は専用targetを使うため、cache復元後に同じ場所のV8 link artifactを再生成する。
coverage-v8-refresh:
    {{CARGO}} clean -p v8 --target-dir target/llvm-cov-target

# Run coverage as a required full-check gate
coverage: coverage-v8-refresh
    DEBUG=true {{CARGO}} llvm-cov {{COVERAGE_TARGET_PACKAGES}} --all-targets --all-features --locked --ignore-filename-regex '{{COVERAGE_IGNORE_FILENAME_REGEX}}' --summary-only --show-missing-lines --fail-under-functions 100 --fail-under-lines {{COVERAGE_MIN_LINES}} --fail-uncovered-functions 0 --fail-uncovered-lines {{COVERAGE_MAX_UNCOVERED_LINES}}

# Show missing coverage lines without relaxing the coverage gate
coverage-missing: coverage-v8-refresh
    DEBUG=true {{CARGO}} llvm-cov {{COVERAGE_TARGET_PACKAGES}} --all-targets --all-features --locked --ignore-filename-regex '{{COVERAGE_IGNORE_FILENAME_REGEX}}' --show-missing-lines --fail-under-functions 100 --fail-under-lines {{COVERAGE_MIN_LINES}} --fail-uncovered-functions 0 --fail-uncovered-lines {{COVERAGE_MAX_UNCOVERED_LINES}}

# Run the local quality gate
check: fmt-check lint ast-lint storybook-entrypoint-check document-surface-boundary-check test release-target-script-test multi-format-scorecard-script-test multi-format-scorecard-check data-descriptor-fixture-check office-profiling-stage-check office-performance-harness-check office-fidelity-harness-check v8-runtime-check check-subagent-harness
    @echo "checks passed"

# Run release-line mapping tests without contacting external services.
release-target-script-test:
    python3 scripts/release/verify-openspec-release-target.py --self-test

# Verify the fixed multi-format score rubric and recorded candidate evidence.
multi-format-scorecard-script-test:
    python3 scripts/feasibility/verify-multi-format-scorecard.py --self-test

multi-format-scorecard-check:
    python3 scripts/feasibility/verify-multi-format-scorecard.py

data-descriptor-fixture-check:
    python3 scripts/feasibility/verify-data-descriptor-docx.py

office-profiling-stage-check:
    python3 scripts/feasibility/verify-office-profiling-stages.py --self-test
    python3 scripts/feasibility/verify-office-profiling-stages.py

office-performance-harness-check:
    python3 scripts/feasibility/measure-office-first-frame.py --self-test

office-fidelity-harness-check:
    python3 scripts/feasibility/measure-office-fidelity.py --self-test
    python3 scripts/feasibility/measure-office-fidelity.py --verify-record openspec/changes/post-v0-5-5-document-fidelity-regressions/evidence/fidelity-baseline.json

v8-runtime-check:
    python3 scripts/release/verify-v8-runtime-singleton.py --self-test
    python3 scripts/release/verify-v8-runtime-singleton.py

# Verify subagent / Spark delegation evidence is explicitly recorded
check-subagent-harness:
    bash scripts/check-subagent-spark-harness-tests.sh
    bash scripts/check-subagent-spark-harness-edge-tests.sh
    bash scripts/check-subagent-spark-harness-policy-tests.sh
    bash scripts/check-subagent-spark-harness-change-tests.sh
    bash scripts/check-subagent-spark-harness-verify-tests.sh
    bash scripts/check-subagent-spark-harness-coverage-tests.sh
    bash scripts/check-subagent-spark-harness-ci-tests.sh
    bash scripts/check-subagent-spark-harness-diff-tests.sh
    bash scripts/check-subagent-spark-harness.sh

# Ensure KDV/KUC preview entrypoints stay vendor-runtime independent
document-surface-boundary-check:
    CARGO="{{CARGO}}" {{RTK_CMD}}bash scripts/document-surface-boundary-check.sh
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked no_reintroduced_manual_storybook_action_contracts -- --test-threads=1

# Open the vendor-free KUC preview Storybook window
storybook:
    {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --interactive --frames {{STORYBOOK_FRAMES}}

# Smoke-test the Storybook renderer without opening an interactive window
storybook-window-smoke:
    {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --smoke --frames 2

# Smoke-test selectable Storybook text reaches the OS clipboard.
storybook-clipboard-smoke:
    old_clip=$(bash scripts/clipboard-read.sh 2>/dev/null || true); restore_clipboard() { printf "%s" "$old_clip" | bash scripts/clipboard-write.sh; }; trap restore_clipboard EXIT; {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --clipboard-smoke; payload=$(bash scripts/clipboard-read.sh); case "$payload" in *"KDV settings"*"Hover highlight"*"katana/sample.md"*"KatanA Rendering"*"Regression"*"Test"*) printf "clipboard-smoke: ok bytes=%s\n" "${#payload}" ;; *) printf "clipboard-smoke: missing expected text\n" >&2; exit 1 ;; esac

# Smoke-test selectable Storybook text reaches the OS clipboard through the keyboard copy path.
storybook-clipboard-keyboard-smoke:
    old_clip=$(bash scripts/clipboard-read.sh 2>/dev/null || true); restore_clipboard() { printf "%s" "$old_clip" | bash scripts/clipboard-write.sh; }; trap restore_clipboard EXIT; {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --clipboard-keyboard-smoke; payload=$(bash scripts/clipboard-read.sh); case "$payload" in *"KDV settings"*"Hover highlight"*"katana/sample.md"*"KatanA Rendering"*"Regression"*"Test"*) printf "clipboard-keyboard-smoke: ok bytes=%s\n" "${#payload}" ;; *) printf "clipboard-keyboard-smoke: missing expected text\n" >&2; exit 1 ;; esac

storybook-clipboard-drag-smoke:
    old_clip=$(bash scripts/clipboard-read.sh 2>/dev/null || true); restore_clipboard() { printf "%s" "$old_clip" | bash scripts/clipboard-write.sh; }; trap restore_clipboard EXIT; {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --clipboard-drag-smoke; payload=$(bash scripts/clipboard-read.sh); case "$payload" in *"KDV settings"*"Hover highlight"*"katana/sample.md"*"KatanA Rendering"*"Regression"*"Test"*) printf "clipboard-drag-smoke: ok bytes=%s\n" "${#payload}" ;; *) printf "clipboard-drag-smoke: missing expected text\n" >&2; exit 1 ;; esac

storybook-selection-screenshot-smoke:
    {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --selection-screenshot-smoke --screenshot-output target/kdv-storybook-selection-smoke.png; test -s target/kdv-storybook-selection-smoke.png

storybook-window-selection-screenshot-smoke:
    old_clip=$(bash scripts/clipboard-read.sh 2>/dev/null || true); restore_clipboard() { printf "%s" "$old_clip" | bash scripts/clipboard-write.sh; }; trap restore_clipboard EXIT; {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --window-selection-screenshot-smoke --screenshot-output target/kdv-storybook-window-selection-smoke.png; test -s target/kdv-storybook-window-selection-smoke.png

storybook-window-hover-screenshot-smoke:
    {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --window-hover-screenshot-smoke --screenshot-output target/kdv-storybook-window-hover-smoke.png; test -s target/kdv-storybook-window-hover-smoke.png; test -s target/kdv-storybook-window-hover-smoke-hover.png

storybook-window-hover-wide-screenshot-smoke:
    {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --window-hover-screenshot-smoke --width 2048 --height 1291 --screenshot-output target/kdv-storybook-window-hover-wide-smoke.png; test -s target/kdv-storybook-window-hover-wide-smoke.png; test -s target/kdv-storybook-window-hover-wide-smoke-hover.png

storybook-window-footnote-screenshot-smoke:
    {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --window-footnote-screenshot-smoke --screenshot-output target/kdv-storybook-window-footnote-smoke.png; test -s target/kdv-storybook-window-footnote-smoke.png; test -s target/kdv-storybook-window-footnote-smoke-reference.png; test -s target/kdv-storybook-window-footnote-smoke-definition.png

storybook-window-table-screenshot-smoke:
    {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --window-table-screenshot-smoke --screenshot-output target/kdv-storybook-window-table-smoke.png; test -s target/kdv-storybook-window-table-smoke.png

storybook-window-code-copy-screenshot-smoke:
    old_clip=$(bash scripts/clipboard-read.sh 2>/dev/null || true); restore_clipboard() { printf "%s" "$old_clip" | bash scripts/clipboard-write.sh; }; trap restore_clipboard EXIT; {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --window-code-copy-screenshot-smoke --screenshot-output target/kdv-storybook-window-code-copy-smoke.png; test -s target/kdv-storybook-window-code-copy-smoke.png; test -s target/kdv-storybook-window-code-copy-smoke-hover.png; test -s target/kdv-storybook-window-code-copy-smoke-copied.png; payload=$(bash scripts/clipboard-read.sh); case "$payload" in *"fn main"*) printf "window-code-copy-screenshot-smoke: ok bytes=%s\n" "${#payload}" ;; *) printf "window-code-copy-screenshot-smoke: missing expected code payload\n" >&2; exit 1 ;; esac

storybook-selection-contract-check-core:
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_window_visible_text_runs_are_individually_selectable_and_copyable -- --test-threads=1

storybook-slideshow-screenshot-smoke:
    {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --slideshow-screenshot-smoke --screenshot-output target/kdv-storybook-slideshow-smoke.png; test -s target/kdv-storybook-slideshow-smoke.png; test -s target/kdv-storybook-slideshow-smoke-mode.png; test -s target/kdv-storybook-slideshow-smoke-next.png; test -s target/kdv-storybook-slideshow-smoke-previous.png; test -s target/kdv-storybook-slideshow-smoke-close.png

storybook-window-slideshow-screenshot-smoke:
    {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --window-slideshow-screenshot-smoke --screenshot-output target/kdv-storybook-window-slideshow-smoke.png; test -s target/kdv-storybook-window-slideshow-smoke.png; test -s target/kdv-storybook-window-slideshow-smoke-mode.png; test -s target/kdv-storybook-window-slideshow-smoke-next.png; test -s target/kdv-storybook-window-slideshow-smoke-previous.png; test -s target/kdv-storybook-window-slideshow-smoke-close.png

storybook-window-sidebar-screenshot-smoke:
    {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --window-sidebar-screenshot-smoke --screenshot-output target/kdv-storybook-window-sidebar-smoke.png; test -s target/kdv-storybook-window-sidebar-smoke.png; test -s target/kdv-storybook-window-sidebar-smoke-file-hover.png; test -s target/kdv-storybook-window-sidebar-smoke-file-click.png; test -s target/kdv-storybook-window-sidebar-smoke-settings-hover.png; test -s target/kdv-storybook-window-sidebar-smoke-settings-click.png

storybook-window-sidebar-narrow-screenshot-smoke:
    {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --window-sidebar-screenshot-smoke --width 816 --height 518 --screenshot-output target/kdv-storybook-window-sidebar-narrow-smoke.png; test -s target/kdv-storybook-window-sidebar-narrow-smoke.png; test -s target/kdv-storybook-window-sidebar-narrow-smoke-file-hover.png; test -s target/kdv-storybook-window-sidebar-narrow-smoke-file-click.png; test -s target/kdv-storybook-window-sidebar-narrow-smoke-settings-hover.png; test -s target/kdv-storybook-window-sidebar-narrow-smoke-settings-click.png

storybook-window-sidebar-large-screenshot-smoke:
    {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --window-sidebar-screenshot-smoke --width 2048 --height 1496 --screenshot-output target/kdv-storybook-window-sidebar-large-smoke.png; test -s target/kdv-storybook-window-sidebar-large-smoke.png; test -s target/kdv-storybook-window-sidebar-large-smoke-file-hover.png; test -s target/kdv-storybook-window-sidebar-large-smoke-file-click.png; test -s target/kdv-storybook-window-sidebar-large-smoke-settings-hover.png; test -s target/kdv-storybook-window-sidebar-large-smoke-settings-click.png

storybook-window-diagram-screenshot-smoke:
    {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --window-diagram-screenshot-smoke --screenshot-output target/kdv-storybook-window-diagram-smoke.png && test -s target/kdv-storybook-window-diagram-smoke.png && test -s target/kdv-storybook-window-diagram-smoke-hover-zoom-in.png && test -s target/kdv-storybook-window-diagram-smoke-zoom-in.png && test -s target/kdv-storybook-window-diagram-smoke-hover-pan-right.png && test -s target/kdv-storybook-window-diagram-smoke-pan-right.png && test -s target/kdv-storybook-window-diagram-smoke-hover-pan-down.png && test -s target/kdv-storybook-window-diagram-smoke-pan-down.png && test -s target/kdv-storybook-window-diagram-smoke-hover-reset-view.png && test -s target/kdv-storybook-window-diagram-smoke-reset-view.png && test -s target/kdv-storybook-window-diagram-smoke-hover-pan-left.png && test -s target/kdv-storybook-window-diagram-smoke-pan-left.png && test -s target/kdv-storybook-window-diagram-smoke-hover-pan-up.png && test -s target/kdv-storybook-window-diagram-smoke-pan-up.png && test -s target/kdv-storybook-window-diagram-smoke-hover-zoom-out.png && test -s target/kdv-storybook-window-diagram-smoke-zoom-out.png && test -s target/kdv-storybook-window-diagram-smoke-hover-trackpad-help.png && test -s target/kdv-storybook-window-diagram-smoke-trackpad-help.png && test -s target/kdv-storybook-window-diagram-smoke-hover-fullscreen.png && test -s target/kdv-storybook-window-diagram-smoke-fullscreen.png

storybook-window-drawio-diagram-screenshot-smoke:
    {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --window-diagram-screenshot-smoke --diagram-smoke-fixture katana/drawio/basic/03-basic-flow.drawio --screenshot-output target/kdv-storybook-window-drawio-diagram-smoke.png && test -s target/kdv-storybook-window-drawio-diagram-smoke.png && test -s target/kdv-storybook-window-drawio-diagram-smoke-hover-zoom-in.png && test -s target/kdv-storybook-window-drawio-diagram-smoke-zoom-in.png && test -s target/kdv-storybook-window-drawio-diagram-smoke-hover-pan-right.png && test -s target/kdv-storybook-window-drawio-diagram-smoke-pan-right.png && test -s target/kdv-storybook-window-drawio-diagram-smoke-hover-pan-down.png && test -s target/kdv-storybook-window-drawio-diagram-smoke-pan-down.png && test -s target/kdv-storybook-window-drawio-diagram-smoke-hover-reset-view.png && test -s target/kdv-storybook-window-drawio-diagram-smoke-reset-view.png && test -s target/kdv-storybook-window-drawio-diagram-smoke-hover-pan-left.png && test -s target/kdv-storybook-window-drawio-diagram-smoke-pan-left.png && test -s target/kdv-storybook-window-drawio-diagram-smoke-hover-pan-up.png && test -s target/kdv-storybook-window-drawio-diagram-smoke-pan-up.png && test -s target/kdv-storybook-window-drawio-diagram-smoke-hover-zoom-out.png && test -s target/kdv-storybook-window-drawio-diagram-smoke-zoom-out.png && test -s target/kdv-storybook-window-drawio-diagram-smoke-hover-trackpad-help.png && test -s target/kdv-storybook-window-drawio-diagram-smoke-trackpad-help.png && test -s target/kdv-storybook-window-drawio-diagram-smoke-hover-fullscreen.png && test -s target/kdv-storybook-window-drawio-diagram-smoke-fullscreen.png

storybook-scroll-performance-artifact:
    {{RTK_CMD}}env KDV_STORYBOOK_SCROLL_PERFORMANCE_ARTIFACT="{{REPO_ROOT}}/target/acceptance/kdv-storybook-scroll-performance.txt" KDV_STORYBOOK_LOADED_ASSET_WAIT_SECS=60 KDV_STORYBOOK_ASSET_JOB_TIMEOUT_SECS=60 {{CARGO}} test -p kdv-storybook --release --locked scroll_performance -- --ignored --test-threads=1
    test -s target/acceptance/kdv-storybook-scroll-performance.txt
    grep -q '^scenario=large_loaded_diagram_wheel_present$' target/acceptance/kdv-storybook-scroll-performance.txt
    grep -q '^full_preview_redraw_fallback_count=0$' target/acceptance/kdv-storybook-scroll-performance.txt

storybook-acceptance-artifact:
    {{RTK_CMD}}just storybook-window-sidebar-screenshot-smoke
    {{RTK_CMD}}just storybook-window-sidebar-narrow-screenshot-smoke
    {{RTK_CMD}}just storybook-window-sidebar-large-screenshot-smoke
    {{RTK_CMD}}just storybook-window-hover-screenshot-smoke
    {{RTK_CMD}}just storybook-window-hover-wide-screenshot-smoke
    {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --window-hover-screenshot-smoke --window-smoke-fixture direct/html-margin-left.html --screenshot-output target/kdv-storybook-window-html-margin-smoke.png && test -s target/kdv-storybook-window-html-margin-smoke.png && test -s target/kdv-storybook-window-html-margin-smoke-hover.png
    {{RTK_CMD}}just storybook-window-footnote-screenshot-smoke
    {{RTK_CMD}}just storybook-window-table-screenshot-smoke
    {{RTK_CMD}}just storybook-window-code-copy-screenshot-smoke
    {{RTK_CMD}}just storybook-window-selection-screenshot-smoke
    {{RTK_CMD}}just storybook-window-diagram-screenshot-smoke
    {{RTK_CMD}}just storybook-window-drawio-diagram-screenshot-smoke
    {{RTK_CMD}}just storybook-window-slideshow-screenshot-smoke
    {{RTK_CMD}}just storybook-scroll-performance-artifact
    {{RTK_CMD}}env KDV_STORYBOOK_PREVIEW_CROP_DUMP_DIR="{{REPO_ROOT}}/target/acceptance/preview-crop-reference" {{CARGO}} test -p kdv-storybook --locked storybook_score_visual_uses_katana_ -- --test-threads=1
    {{RTK_CMD}}bash scripts/release/generate-storybook-acceptance-artifact.sh

storybook-live-acceptance-artifact:
    {{RTK_CMD}}{{CARGO}} build --release --locked -p kdv-storybook
    {{RTK_CMD}}bash scripts/release/generate-storybook-live-acceptance-artifact.sh

storybook-release-acceptance-artifacts:
    {{RTK_CMD}}just storybook-acceptance-artifact
    {{RTK_CMD}}just storybook-live-acceptance-artifact

# Run vendor-free Storybook contract tests
storybook-tool-test:
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked

# Check Storybook content parity and export score coverage
storybook-content-check:
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked preview_feature_matrix -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked direct_diagram -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked loaded_asset_scene_cache_separates_dark_and_light_diagram_theme -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked preview_requirement_matrix -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked required_kuc_roles_reach_fixture_frame_pixels -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked katana_alert_scene_keeps_title_body_and_kind_contract -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked fixture_feature_matrix -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked fixture_mermaid -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked fixture_score_matrix -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked direct_ -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked export_quality -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked surface_equivalence -- --test-threads=1

storybook-content-check-core:
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked fixture_feature_matrix -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked fixture_mermaid -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked direct_ -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked export_quality -- --test-threads=1

# Check OS color emoji rendering is preserved through KDV -> KUC Storybook
storybook-emoji-check:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib emoji -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked emoji -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked emoji -- --test-threads=1

storybook-emoji-check-core:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib emoji -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked emoji -- --test-threads=1

storybook-kuc-visual-check-core:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib ui_tree_canvas -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked presentation -- --test-threads=1

# Check KUC TreeView/FileTree rendering and sidebar integration
storybook-treeview-check:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core --locked --lib tree -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib ui_tree_canvas_tests -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib ui_tree_canvas_tree -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked file_tree -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked sidebar_file_canvas_click_rebuilds_viewer_scene_for_selected_fixture -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_window_file_tree_hover_draws_kuc_row_background -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_window_routes_visible_control_matrix -- --test-threads=1

storybook-treeview-check-core:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core --locked --lib tree -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked rendered_file_tree_directory_hover_paints_kuc_row_background -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked rendered_file_tree_directory_click_collapses_visible_rows_and_pixels -- --test-threads=1

# Check KUC SettingsList action, hover, and KDV Storybook state roundtrip
storybook-settings-contract-check:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core --locked --lib settings -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core --locked --test settings_list_contract -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core --locked --test interactive_preset_contract settings_list_fields_expose_control_interactive_preset -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core --locked --test molecule_models_contract settings_list_filters_resets_and_collapses_with_typed_events -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib generic_toggle_hover_draws_kuc_interactive_preset_border -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib settings_section_header_hover_draws_kuc_interactive_preset_border -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked settings -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked sidebar_mode_canvas_click_rebuilds_scene_as_slideshow -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked sidebar_theme_canvas_click_rebuilds_scene_as_light -- --test-threads=1

storybook-settings-contract-check-core:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core --locked --lib settings -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core --locked --test settings_list_contract -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core --locked --test interactive_preset_contract settings_list_fields_expose_control_interactive_preset -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core --locked --test molecule_models_contract settings_list_filters_resets_and_collapses_with_typed_events -- --test-threads=1

# Check KUC/KDV coordinate normalization, rendered hit rects, hover, cursor, and click routing
storybook-coordinate-contract-check:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core --locked --lib mouse_point_and_surface_must_share_the_same_coordinate_space -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib ui_tree_canvas_hit -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib row_render_and_hit_collector_share_row_layout_contract -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked window_coordinates -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked window_input_uses_kuc_core_mouse_normalizer_for_canvas_space -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked hit_rect_center -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked sidebar_hit_accepts_rendered_toggle_track_bounds_at_retina_scale -- --test-threads=1

storybook-coordinate-contract-check-core:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core --locked --lib mouse_point_and_surface_must_share_the_same_coordinate_space -- --test-threads=1

# Check hover/cursor/highlight contracts through KUC interaction state and KDV Storybook window pixels
storybook-hover-contract-check:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core --locked --lib with_hovered_node_id -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib tree_canvas_draws_hover_row_background_from_tree_hovered_id -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib tree_canvas_draws_tree_view_row_hover_border_from_kuc_contract -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib generic_button_hover_draws_kuc_interactive_preset_border -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib generic_toggle_hover_draws_kuc_interactive_preset_border -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib generic_checkbox_hover_draws_kuc_interactive_preset_border -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib settings_toggle_control_hover_draws_kuc_interactive_preset_border -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib settings_section_header_hover_draws_kuc_interactive_preset_border -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib generic_text_hover_draws_kuc_hover_background_before_text -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked hover -- --test-threads=1

storybook-hover-contract-check-core:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core --locked --lib with_hovered_node_id -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib hover_surface_preserves_viewer_node_geometry_and_semantic_id -- --test-threads=1

# Check KDV-owned media controls through KUC host action hit rects
storybook-media-control-clickability-check:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib ui_tree_canvas_hit -- --test-threads=1
    {{RTK_CMD}}just storybook-media-control-clickability-check-core

storybook-media-control-clickability-check-core:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib icon_variant_button_keeps_transparent_base_on_tree_canvas -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib diagram_controls_use_katana_icon_preset_by_default -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib every_diagram_control_uses_katana_icon_asset_source -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib katana_stroke_icon_tints_white_stroke_to_requested_color -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib diagram_control_icons_can_be_overridden_from_host_config -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib document_viewer -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib diagram_controls_keep_katana_min_container_height_for_short_surface -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked media_control -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked media -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked media_control -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked kuc_default_diagram_control_icons_match_katana_asset_files -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked diagram_control_icons_render_as_katana_glyphs_not_blocky_squares -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked diagram_controls_follow_katana_top_and_grid_layout -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_window_every_diagram_control_click_dispatches_from_kuc_hit -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_window_fullscreen_diagram_overlay_controls_dispatch_from_kuc_hits -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_window_diagram_control_click_repaints_viewer_frame -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_window_diagram_controls_survive_continuous_kuc_hit_sequence_without_asset_reload -- --test-threads=1

storybook-media-control-clickability-check-full-core:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib diagram_controls_use_katana_icon_preset_by_default -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib every_diagram_control_uses_katana_icon_asset_source -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib katana_stroke_icon_tints_white_stroke_to_requested_color -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib diagram_control_icons_can_be_overridden_from_host_config -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib document_viewer -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib diagram_controls_keep_katana_min_container_height_for_short_surface -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked media_control -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked kuc_default_diagram_control_icons_match_katana_asset_files -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked diagram_control_icons_render_as_katana_glyphs_not_blocky_squares -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked diagram_controls_follow_katana_top_and_grid_layout -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_window_every_diagram_control_click_dispatches_from_kuc_hit -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_window_fullscreen_diagram_overlay_controls_dispatch_from_kuc_hits -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_window_diagram_control_click_repaints_viewer_frame -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_window_diagram_controls_survive_continuous_kuc_hit_sequence_without_asset_reload -- --test-threads=1

# Check direct image fixtures render image surfaces and KDV-owned image controls
storybook-image-control-check:
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked image -- --test-threads=1

# Check code blocks render syntax-highlighted body and KDV-owned copy control
storybook-code-block-check:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib document_code_block -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked planner_uses_export_surface_height_for_multiline_fenced_code -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked media_control_specs_create_host_action_ids -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked code -- --test-threads=1

storybook-code-block-check-core:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib document_code_block -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked planner_uses_export_surface_height_for_multiline_fenced_code -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked media_control_specs_create_host_action_ids -- --test-threads=1

# Check task checkbox visual contract, KUC hit/action rects, and external state propagation
storybook-task-checkbox-check:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib checkbox -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib tree_canvas_renders_context_menu_node_and_returns_item_hit -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked task -- --test-threads=1

storybook-task-checkbox-check-core:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib checkbox -- --test-threads=1

# Check accordion parsing, KUC typed host action, hover, click, and open/close frame changes
storybook-accordion-check:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core --locked --test host_action_plan_contract accordion_text_action_exposes_requested_open_without_consumer_inversion -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib accordion -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked accordion -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked accordion -- --test-threads=1

storybook-accordion-check-core:
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core --locked --test host_action_plan_contract accordion_text_action_exposes_requested_open_without_consumer_inversion -- --test-threads=1
    cd "{{KUC_ROOT}}" && {{RTK_CMD}}{{CARGO}} test -p katana-ui-core-storybook --locked --lib accordion -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked accordion -- --test-threads=1

# Check link and footnote commands use rendered KUC text span hits and anchor scroll.
storybook-link-footnote-check-core:
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked link -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked footnote -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_window_footnote_reference_click_jumps_to_definition -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_window_footnote_backlink_click_jumps_to_reference -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_window_sample_footnote -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_window_sample_footnote -- --test-threads=1

# Check KMM outline -> TOC model -> Storybook command path.
storybook-toc-check-core:
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked toc -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked kuc_scene_target_reaches_kdv_toc_scroll_command -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked scroll_to_heading_command_scrolls_to_target -- --test-threads=1

# Check diagram lazy load, controls, math exclusion, cache, theme, and scroll stability
storybook-diagram-load-check:
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked katana_diagram_fixture_reaches_all_viewer_diagram_kinds -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked asset_loader -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked diagram -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked asset_job -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --release --locked fixture_switch_pending_first_frame_completes_loaded_diagram_assets -- --ignored --test-threads=1

storybook-diagram-load-check-core:
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked katana_diagram_fixture_reaches_all_viewer_diagram_kinds -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked asset_loader -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --release --locked fixture_switch_pending_first_frame_completes_loaded_diagram_assets -- --ignored --test-threads=1

# Check slideshow mode, key navigation, page index, scene reuse, and Katana-specific control visibility
storybook-slideshow-check:
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked slideshow -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked slideshow -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked sidebar_state_shows_mode_and_human_slide_index -- --test-threads=1

storybook-slideshow-check-core:
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked slideshow -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked sidebar_mode_canvas_click_rebuilds_scene_as_slideshow -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked slideshow_next_button_click_advances_page_without_scene_rebuild -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked slideshow_previous_button_click_works_after_page_scroll -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked slideshow_close_button_click_works_after_page_scroll -- --test-threads=1

# Check search highlights and navigation keep KatanA-style wrap and artifact text contracts.
storybook-search-check-core:
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked search -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked search -- --test-threads=1

# Check unsupported KMM metadata remains visible instead of being dropped from the body.
storybook-unresolved-metadata-check-core:
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked unresolved_metadata -- --test-threads=1

# Check KatanA-style preview scroll, resize, bottom spacer, and diagram cache stability
storybook-scroll-resize-contract-check:
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked scroll -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked resize -- --test-threads=1

# Check viewer interaction commands and rendered interaction state
storybook-interaction-check:
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked commands -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked preview_interaction_command_ -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked mouse -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked interaction_tests -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked scroll_tests -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked sidebar_tests -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked task_state_tests -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked search -- --test-threads=1

storybook-interaction-check-core:
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked commands -- --test-threads=1

# Smoke-test KDV Preview -> KUC render model without vendor UI runtimes
storybook-kuc-smoke:
    CARGO="{{CARGO}}" {{RTK_CMD}}bash scripts/storybook-kuc-smoke.sh

# Ensure just storybook is an interactive launch command, not a smoke-test alias
storybook-entrypoint-check:
    {{RTK_CMD}}bash scripts/check-storybook-entrypoint.sh

# Check Markdown切替時のStorybook performance budget
storybook-performance-check:
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --release --locked render_engine_keeps_diagram_fixture_inside_interactive_budget -- --ignored --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked visible_asset_load_parallel_starts_visible_diagrams_together -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked asset_worker -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --release --locked performance_tests -- --ignored --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --release --locked fixture_switch_ -- --ignored --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked asset_job_streams_partial_scene_before_completion -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked resized_loaded_diagram_scene_reuses_cached_artifacts_without_pending_reload -- --test-threads=1

storybook-performance-check-core:
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --release --locked render_engine_keeps_diagram_fixture_inside_interactive_budget -- --ignored --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked visible_asset_load_parallel_starts_visible_diagrams_together -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --release --locked performance_tests -- --ignored --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --release --locked fixture_switch_ -- --ignored --test-threads=1

# Check Storybook score gates required for KDV release acceptance.
storybook-score-audit-check:
    {{RTK_CMD}}just storybook-score-check

storybook-score-check:
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked storybook_score_gate -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked katana_reference_artifacts -- --test-threads=1
    {{RTK_CMD}}just storybook-entrypoint-check
    {{RTK_CMD}}just storybook-registry-consumer-suite
    # Supersedes private-KUC lanes: storybook-content-check-core, storybook-emoji-check-core,
    # storybook-kuc-visual-check-core, storybook-treeview-check-core,
    # storybook-interaction-check-core, storybook-coordinate-contract-check-core,
    # storybook-hover-contract-check-core, storybook-clipboard-smoke,
    # storybook-clipboard-keyboard-smoke, storybook-clipboard-drag-smoke,
    # storybook-selection-contract-check-core, storybook-selection-screenshot-smoke,
    # storybook-settings-contract-check-core, storybook-window-smoke.
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked katana_intro_text_keeps_readable_frame_band_heights -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked katana_language_link_underline_reaches_frame_pixels -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked direct_html_margin_left_fixture_reaches_frame_pixels -- --test-threads=1
    {{RTK_CMD}}just storybook-window-sidebar-screenshot-smoke
    {{RTK_CMD}}just storybook-window-sidebar-narrow-screenshot-smoke
    {{RTK_CMD}}just storybook-window-sidebar-large-screenshot-smoke
    {{RTK_CMD}}just storybook-window-hover-screenshot-smoke
    {{RTK_CMD}}just storybook-window-hover-wide-screenshot-smoke
    {{RTK_CMD}}{{CARGO}} run --release --locked -p kdv-storybook -- --window-hover-screenshot-smoke --window-smoke-fixture direct/html-margin-left.html --screenshot-output target/kdv-storybook-window-html-margin-smoke.png && test -s target/kdv-storybook-window-html-margin-smoke.png && test -s target/kdv-storybook-window-html-margin-smoke-hover.png
    {{RTK_CMD}}just storybook-window-footnote-screenshot-smoke
    {{RTK_CMD}}just storybook-window-table-screenshot-smoke
    {{RTK_CMD}}just storybook-window-selection-screenshot-smoke
    # KDV's full registry consumer suite covers the former private-KUC media/code lanes:
    # storybook-media-control-clickability-check-full-core, storybook-code-block-check-core.
    {{RTK_CMD}}just storybook-window-code-copy-screenshot-smoke
    # storybook-diagram-load-check-core is covered by the registry consumer suite.
    {{RTK_CMD}}just storybook-window-diagram-screenshot-smoke
    {{RTK_CMD}}just storybook-window-drawio-diagram-screenshot-smoke
    {{RTK_CMD}}just storybook-link-footnote-check-core
    {{RTK_CMD}}just storybook-slideshow-check-core
    {{RTK_CMD}}just storybook-slideshow-screenshot-smoke
    {{RTK_CMD}}just storybook-window-slideshow-screenshot-smoke
    {{RTK_CMD}}just storybook-search-check-core
    # Registry consumer coverage includes storybook-task-checkbox-check-core,
    # storybook-accordion-check-core, storybook-toc-check-core,
    # storybook-unresolved-metadata-check-core, storybook-scroll-resize-contract-check,
    # and storybook-performance-check-core.
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_score_visual_uses_katana_preview_crop_reference -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_sample_top_local_text_metrics_match_katana_reference -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_score_visual_uses_katana_sample_diagrams_crop_reference -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_sample_diagrams_local_svg_metrics_match_katana_reference -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_preview_crop_score_uses_scaled_canvas_pixels -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_preview_crop_score_excludes_storybook_overlay_controls -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_score_visual_uses_katana_export_png_reference -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked storybook_score_gate_keeps_interactive_controls_disabled_for_reference_comparison -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_score_export_surface_excludes_overlay_controls -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked storybook_score_audit -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked fixture_score_matrix -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked surface_equivalence -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked frame::test_modules::surface_parity_tests::storybook_frame_matches_export_surface_for_katana_viewer -- --ignored --exact --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked frame::test_modules::surface_parity_tests::storybook_frame_matches_export_surface_for_katana_viewer_diagrams -- --ignored --exact --test-threads=1

storybook-score-check-core:
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked storybook_score_gate -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked katana_reference_artifacts -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked fixture_score_matrix -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked surface_equivalence -- --test-threads=1

# Check the vendor-free Storybook contract
storybook-check: storybook-entrypoint-check document-surface-boundary-check storybook-kuc-smoke storybook-registry-consumer-suite storybook-score-check

# The published KUC raster-host API is validated only through KDV's locked
# registry consumer.  Do not make release gates depend on the private KUC
# Storybook implementation crate.
storybook-registry-consumer-suite:
    {{RTK_CMD}}{{CARGO}} test -p katana-document-viewer --locked --lib -- --test-threads=1
    {{RTK_CMD}}{{CARGO}} test -p kdv-storybook --locked -- --test-threads=1

# Verify VERSION follows the published release line
release-target-check:
    bash scripts/release/verify-version.sh "{{VERSION}}"
    python3 scripts/release/verify-release-target.py --target-version "{{TAG}}" --repo "{{RELEASE_REPO}}"
    python3 scripts/release/verify-openspec-release-target.py --target-version "{{TAG}}"

# Verify package metadata and dry-run the crates.io publish target.
release-dod-check:
    python3 scripts/release/assert-viewer-recovery-dod.py --self-test
    python3 scripts/release/assert-viewer-recovery-dod.py

# Verify the active OpenSpec release contract before packaging or publishing.
release-contract-check:
    python3 scripts/release/verify-release-contract.py --self-test
    python3 scripts/release/verify-release-contract.py --target-version "{{TAG}}"
    python3 scripts/release/verify-registry-consumer-link.py --self-test
    python3 scripts/feasibility/verify-multi-format-scorecard.py --require-approved
    {{CARGO}} test -p katana-document-viewer --test browser_session_adapter_contract --locked -- --test-threads=1
    {{RTK_CMD}}bash scripts/storybook-kuc-smoke.sh

# Verify package metadata and dry-run the crates.io publish target.
release-verify: release-contract-check check coverage
    bash scripts/release/verify-version.sh "{{VERSION}}"
    {{CARGO}} package -p katana-document-viewer --locked --allow-dirty
    {{CARGO}} publish -p katana-document-viewer --dry-run --locked --allow-dirty

# Verify release branch readiness before merging
release-check: release-target-check release-verify
    bash scripts/release/assert-crates-not-published.sh "{{VERSION}}"

# Publish the verified release to crates.io.
release-publish: release-check
    bash scripts/release/publish-crates.sh "{{VERSION}}"

# Sweep old build artifacts locally
sweep:
    @{{RTK_CMD}}cargo sweep --time 7 || true

# Remove build artifacts
clean: sweep
    {{RTK_CMD}}{{CARGO}} clean

# Update dependency crates safely
update-safe:
    {{RTK_CMD}}cargo update

# Upgrade all dependency crates and refresh the lockfile
update:
    {{RTK_CMD}}cargo upgrade -i --recursive false
    {{RTK_CMD}}cargo update

# List outdated dependency crates
outdated:
    @cp Cargo.toml Cargo.toml.bak
    @sed -e '/^\[patch\.crates-io\]/,$d' Cargo.toml.bak > Cargo.toml
    @{{RTK_CMD}}cargo outdated --workspace || (mv Cargo.toml.bak Cargo.toml && exit 1)
    @mv Cargo.toml.bak Cargo.toml

[private]
ensure-kal:
    @if [[ "{{KAL}}" == */* ]]; then \
        test -x "{{KAL}}" && "{{KAL}}" version | grep -q "kal {{KAL_VERSION}}" && exit 0; \
      elif command -v "{{KAL}}" >/dev/null 2>&1 && "{{KAL}}" version | grep -q "kal {{KAL_VERSION}}"; then \
        exit 0; \
      fi; \
      {{CARGO}} install katana-ast-lint --version "{{KAL_VERSION}}" --locked --force --root "{{KAL_ROOT}}"
