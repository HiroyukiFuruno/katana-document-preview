#!/usr/bin/env python3
"""Fail release while required Storybook acceptance evidence is incomplete."""

from __future__ import annotations

import pathlib
import re
import sys
import hashlib
import os
import shutil
import subprocess
import struct
import tomllib
from datetime import datetime, timezone


ROOT = pathlib.Path(__file__).resolve().parents[2]
LOCAL_CARGO_CONFIG = ROOT / ".cargo/config.toml"
CHANGE = ROOT / "openspec/changes/v0-2-0-markdown-viewer-kuc-integration"
USER_FEEDBACK = CHANGE / "user-feedback-todo.md"
REMAINING_PLAN = CHANGE / "remaining-plan.md"
STORYBOOK_USER_ACCEPTANCE = CHANGE / "storybook-user-acceptance.md"
CI_WORKFLOW = ROOT / ".github/workflows/test-and-build.yml"
PREFLIGHT_WORKFLOW = ROOT / ".github/workflows/release-preflight.yml"
RELEASE_WORKFLOW = ROOT / ".github/workflows/release.yml"
HANDOFF_FEEDBACK_LEDGER_FILES = (
    ROOT / "handoff-kdv-v0.2.0-viewer-recovery-2026-06-13.md",
    ROOT / "handoff-kdv-v0.2.0-work-instruction-2026-06-13.md",
)
CANONICAL_FEEDBACK_LEDGER_RELATIVE_PATH = USER_FEEDBACK.relative_to(ROOT).as_posix()
CANONICAL_FEEDBACK_LEDGER_PATH = USER_FEEDBACK.as_posix()
STALE_ROOT_FEEDBACK_LEDGER_PATH = (ROOT / "user-feedback-todo.md").as_posix()

OPEN_CHECKLIST_RE = re.compile(r"^- \[ \] .+", re.MULTILINE)
ACCEPTANCE_STATUS_RE = re.compile(r"^\s*status\s*:\s*(?P<status>[a-zA-Z_-]+)\s*$")
CHECKLIST_ITEM_RE = re.compile(r"^- \[(?P<status>.)\]\s+(?P<label>.+?)\s*$", re.MULTILINE)
EVIDENCE_FIELD_RE = re.compile(r"^- (?P<field>confirmed_by|confirmed_at):\s*(?P<value>.+?)\s*$")
CONFIRMED_AT_RE = re.compile(
    r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:Z|[+-]\d{2}:\d{2})$"
)
CONFIRMED_BY_PLACEHOLDERS = {"", "human reviewer", "reviewer", "user", "tbd", "n/a", "none"}
REQUIRED_EVIDENCE_FIELDS = ("confirmed_by", "confirmed_at")
HUMAN_ACCEPTANCE_NOTE_RE = re.compile(r"^\s*-\s+human acceptance:\s*(?P<note>.+?)\s*$")
HUMAN_ACCEPTANCE_NOTE_REQUIRED_TOKENS = ("just storybook", "KatanA", "interactive")
HUMAN_ACCEPTANCE_NOTE_FORBIDDEN_MARKERS = (
    "pending",
    "not human acceptance",
    "代替ではない",
    "automated evidence only",
    "headless",
    "screenshot / score gate",
)
PENDING_ACCEPTANCE_REQUIRED_OPEN_FEEDBACK_IDS = ("UF-040", "UF-042")
KUC_ISSUE_7_URL = "https://github.com/HiroyukiFuruno/katana-ui-core/issues/7"
KUC_ISSUE_8_URL = "https://github.com/HiroyukiFuruno/katana-ui-core/issues/8"
KUC_CARGO_GIT_URL = "https://github.com/HiroyukiFuruno/katana-ui-core.git"
KUC_CARGO_TAG = "v0.1.4"
KUC_CARGO_VERSION = "0.1.4"
KUC_CARGO_REV = "554f13f2c219115cbd3a2c3dc3d02fd5306c4743"
KUC_CARGO_LOCK_SOURCE = (
    f"git+{KUC_CARGO_GIT_URL}?tag={KUC_CARGO_TAG}#{KUC_CARGO_REV}"
)
KUC_INTERACTION_TARGET_STALE_CARGO_TAG = "v0.1.1"
KUC_INTERACTION_TARGET_STALE_CARGO_REV = "50eef732edaf2b8b29f55e74827a4f3ac8693243"
KUC_INTERACTION_TARGET_STALE_CARGO_LOCK_SOURCE = (
    "git+https://github.com/HiroyukiFuruno/katana-ui-core.git"
    f"?tag={KUC_INTERACTION_TARGET_STALE_CARGO_TAG}"
    f"#{KUC_INTERACTION_TARGET_STALE_CARGO_REV}"
)
KUC_REGISTRY_VERSION = "0.3.7"
KUC_REGISTRY_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"
KUC_CARGO_DEPENDENCY_NAMES = ("katana-ui-core",)
KUC_CARGO_FORBIDDEN_SIBLING_MARKERS = (
    "../katana-ui-core",
    "../../../../katana-ui-core",
)
KUC_CONTEXT_MENU_ADR = (
    CHANGE / "adrs/2026-06-23-kuc-installed-boundary-and-context-menu-actions.md"
)
KUC_INTERACTION_TARGET_ADR = (
    CHANGE / "adrs/2026-06-23-kuc-interaction-target-contract.md"
)
KUC_DOCUMENT_VIEWER_HARNESS_ADR = (
    CHANGE / "adrs/2026-06-24-document-viewer-harness-ownership.md"
)
KUC_INTERACTION_TARGET_REQUIRED_TOKENS = (
    "TreeView",
    "FileTree",
    "SettingsList",
    "Toggle",
    "Button",
    "link-like span",
    "media control",
    "typed action injection",
)
KUC_INTERACTION_TARGET_COMPLETION_REQUIRED_TOKENS = (
    "KUC #8 completion evidence:",
    KUC_ISSUE_8_URL,
    "TreeView / FileTree / SettingsList / Toggle / Button / link-like span / media control",
    "host_action_hits",
    "interaction_for_hit",
    "UiTreeInteractionTarget",
    "UiTreeSurfaceHost::interaction_target_for_hits_at",
    "cargo test -p katana-ui-core-storybook --locked ui_tree_interaction_surface",
    "cargo test -p katana-ui-core-storybook --locked ui_tree_storybook_host",
    "cargo test -p katana-ui-core --locked settings",
    "cargo test -p katana-ui-core --locked file_tree",
    "cargo test -p kdv-storybook --locked no_reintroduced_manual_storybook_action_contracts",
)
KUC_INTERACTION_TARGET_KDV_REQUIRED_TOKENS = (
    "UiTreeInteractionTarget",
    "interaction_target_for_hits_at",
    "hover_node_id",
)
KUC_DOCUMENT_VIEWER_REJECTED_COMMAND = (
    "cargo test -p katana-ui-core-storybook --locked document_viewer"
)
KDV_DOCUMENT_VIEWER_REQUIRED_COMMAND = (
    "cargo test -p kdv-storybook --locked document_viewer"
)
KUC_DOCUMENT_VIEWER_OWNERSHIP_REQUIRED_TOKENS = (
    "KDV owns the KatanA document_viewer harness",
    "katana-document-viewer",
    "katana-ui-core-storybook",
    "0 passed",
    "not release proof",
    KDV_DOCUMENT_VIEWER_REQUIRED_COMMAND,
    KUC_DOCUMENT_VIEWER_REJECTED_COMMAND,
)
REQUIRED_ACCEPTANCE_CHECKS = (
    "just storybook",
    "Markdown 本文",
    "SVG / Mermaid / Draw.io / math",
    "図形 controls",
    "footnote jump / backlink",
    "slideshow mode",
    "左 sidebar FileTree / SettingsList",
    "縦 scroll",
    "KatanA viewer / export HTML / export PDF",
)
REQUIRED_MATRIX_HEADING = "## Automated Evidence Matrix"
REQUIRED_ACCEPTANCE_ARTIFACTS = (
    "just storybook-release-acceptance-artifacts",
    "just storybook-acceptance-artifact",
    "just storybook-live-acceptance-artifact",
    "target/acceptance/kdv-storybook-acceptance-contact-sheet.png",
    "target/acceptance/kdv-storybook-text-regression-crops.png",
    "target/acceptance/kdv-storybook-katana-reference-comparison.png",
    "target/acceptance/text-regression-crops/table-section.png",
    "target/acceptance/kdv-storybook-scroll-performance.txt",
    "target/kdv-storybook-window-code-copy-smoke-copied.png",
    "target/acceptance/kdv-storybook-acceptance-artifacts.sha256",
    "target/acceptance/kdv-storybook-live-interactive.png",
    "target/acceptance/kdv-storybook-live-light-toggle.png",
    "target/acceptance/kdv-storybook-live-acceptance.log",
    "target/acceptance/kdv-storybook-live-acceptance-artifacts.sha256",
)
REQUIRED_ACCEPTANCE_MATRIX_EVIDENCE_TOKENS = (
    (
        "just storybook",
        (
            "target/release/kdv-storybook --interactive --frames 0",
            "storybook-entrypoint-check",
        ),
    ),
    (
        "Markdown 本文",
        (
            "storybook-content-check-core",
            "KatanA reference artifact",
            "direct_html_margin_left_fixture_reaches_frame_pixels",
        ),
    ),
    (
        "SVG / Mermaid / Draw.io / math",
        (
            "storybook-diagram-load-check-core",
            "KDV SVG metrics",
            "storybook-scroll-resize-contract-check",
        ),
    ),
    (
        "図形 controls",
        (
            "storybook-media-control-clickability-check-full-core",
            "diagram_controls_use_katana_icon_preset_by_default",
            "fullscreen",
        ),
    ),
    (
        "footnote jump / backlink",
        (
            "storybook-link-footnote-check-core",
            "storybook-code-block-check-core",
            "kdv-storybook-window-code-copy-smoke-copied.png",
        ),
    ),
    (
        "slideshow mode",
        (
            "storybook-slideshow-check-core",
            "storybook-window-slideshow-screenshot-smoke",
        ),
    ),
    (
        "左 sidebar FileTree / SettingsList",
        (
            "storybook-treeview-check-core",
            "storybook-settings-contract-check-core",
            "storybook-window-sidebar-screenshot-smoke",
        ),
    ),
    (
        "縦 scroll",
        (
            "storybook-scroll-resize-contract-check",
            "storybook-performance-check-core",
            "full_preview_redraw_fallback_count=0",
        ),
    ),
    (
        "KatanA viewer / export HTML / export PDF",
        (
            "storybook-score-check",
            "export HTML",
            "export PDF",
            "score",
        ),
    ),
)
REQUIRED_ACCEPTANCE_ARTIFACT_PATHS = (
    ROOT / "target/acceptance/kdv-storybook-acceptance-contact-sheet.png",
    ROOT / "target/acceptance/kdv-storybook-text-regression-crops.png",
    ROOT / "target/acceptance/kdv-storybook-katana-reference-comparison.png",
    ROOT / "target/acceptance/text-regression-crops/title-body.png",
    ROOT / "target/acceptance/text-regression-crops/language-link.png",
    ROOT / "target/acceptance/text-regression-crops/html-margin-center.png",
    ROOT / "target/acceptance/text-regression-crops/direct-html-margin-left.png",
    ROOT / "target/acceptance/text-regression-crops/hover-highlight.png",
    ROOT / "target/acceptance/text-regression-crops/wide-title-link-html.png",
    ROOT / "target/acceptance/text-regression-crops/diagram-control-icons.png",
    ROOT / "target/acceptance/text-regression-crops/table-section.png",
    ROOT / "target/acceptance/kdv-storybook-scroll-performance.txt",
    ROOT / "target/acceptance/text-regression-crops/reference-comparison/sample-top-reference.png",
    ROOT / "target/acceptance/text-regression-crops/reference-comparison/sample-top-candidate.png",
    ROOT / "target/acceptance/text-regression-crops/reference-comparison/sample-top-diff.png",
    ROOT
    / "target/acceptance/text-regression-crops/reference-comparison/sample-diagrams-reference.png",
    ROOT
    / "target/acceptance/text-regression-crops/reference-comparison/sample-diagrams-candidate.png",
    ROOT
    / "target/acceptance/text-regression-crops/reference-comparison/sample-diagrams-diff.png",
    ROOT / "target/acceptance/preview-crop-reference/katana_sample_md-preview-crop_reference.ppm",
    ROOT / "target/acceptance/preview-crop-reference/katana_sample_md-preview-crop_preview.ppm",
    ROOT
    / "target/acceptance/preview-crop-reference/katana_sample_diagrams_md-preview-crop_reference.ppm",
    ROOT
    / "target/acceptance/preview-crop-reference/katana_sample_diagrams_md-preview-crop_preview.ppm",
    ROOT / "target/acceptance/kdv-storybook-acceptance-artifacts.sha256",
    ROOT / "target/acceptance/kdv-storybook-live-acceptance-artifacts.sha256",
    ROOT / "target/acceptance/kdv-storybook-live-acceptance.log",
)
REQUIRED_LIVE_ACCEPTANCE_ARTIFACT_PATHS = (
    ROOT / "target/acceptance/kdv-storybook-live-interactive.png",
    ROOT / "target/acceptance/kdv-storybook-live-light-toggle.png",
)
REQUIRED_ACCEPTANCE_SOURCE_ARTIFACT_PATHS = (
    ROOT / "target/kdv-storybook-window-hover-smoke.png",
    ROOT / "target/kdv-storybook-window-sidebar-smoke.png",
    ROOT / "target/kdv-storybook-window-sidebar-smoke-file-hover.png",
    ROOT / "target/kdv-storybook-window-sidebar-smoke-file-click.png",
    ROOT / "target/kdv-storybook-window-sidebar-smoke-settings-hover.png",
    ROOT / "target/kdv-storybook-window-sidebar-smoke-settings-click.png",
    ROOT / "target/kdv-storybook-window-sidebar-narrow-smoke.png",
    ROOT / "target/kdv-storybook-window-sidebar-narrow-smoke-file-hover.png",
    ROOT / "target/kdv-storybook-window-sidebar-narrow-smoke-file-click.png",
    ROOT / "target/kdv-storybook-window-sidebar-narrow-smoke-settings-hover.png",
    ROOT / "target/kdv-storybook-window-sidebar-narrow-smoke-settings-click.png",
    ROOT / "target/kdv-storybook-window-sidebar-large-smoke.png",
    ROOT / "target/kdv-storybook-window-sidebar-large-smoke-file-hover.png",
    ROOT / "target/kdv-storybook-window-sidebar-large-smoke-file-click.png",
    ROOT / "target/kdv-storybook-window-sidebar-large-smoke-settings-hover.png",
    ROOT / "target/kdv-storybook-window-sidebar-large-smoke-settings-click.png",
    ROOT / "target/kdv-storybook-window-html-margin-smoke.png",
    ROOT / "target/kdv-storybook-window-hover-wide-smoke.png",
    ROOT / "target/kdv-storybook-window-diagram-smoke.png",
    ROOT / "target/kdv-storybook-window-diagram-smoke-hover-zoom-in.png",
    ROOT / "target/kdv-storybook-window-diagram-smoke-zoom-in.png",
    ROOT / "target/kdv-storybook-window-diagram-smoke-hover-pan-right.png",
    ROOT / "target/kdv-storybook-window-diagram-smoke-pan-right.png",
    ROOT / "target/kdv-storybook-window-diagram-smoke-hover-pan-down.png",
    ROOT / "target/kdv-storybook-window-diagram-smoke-pan-down.png",
    ROOT / "target/kdv-storybook-window-diagram-smoke-hover-reset-view.png",
    ROOT / "target/kdv-storybook-window-diagram-smoke-reset-view.png",
    ROOT / "target/kdv-storybook-window-diagram-smoke-hover-pan-left.png",
    ROOT / "target/kdv-storybook-window-diagram-smoke-pan-left.png",
    ROOT / "target/kdv-storybook-window-diagram-smoke-hover-pan-up.png",
    ROOT / "target/kdv-storybook-window-diagram-smoke-pan-up.png",
    ROOT / "target/kdv-storybook-window-diagram-smoke-hover-zoom-out.png",
    ROOT / "target/kdv-storybook-window-diagram-smoke-zoom-out.png",
    ROOT / "target/kdv-storybook-window-diagram-smoke-hover-trackpad-help.png",
    ROOT / "target/kdv-storybook-window-diagram-smoke-trackpad-help.png",
    ROOT / "target/kdv-storybook-window-diagram-smoke-hover-fullscreen.png",
    ROOT / "target/kdv-storybook-window-diagram-smoke-fullscreen.png",
    ROOT / "target/kdv-storybook-window-drawio-diagram-smoke.png",
    ROOT / "target/kdv-storybook-window-drawio-diagram-smoke-hover-zoom-in.png",
    ROOT / "target/kdv-storybook-window-drawio-diagram-smoke-zoom-in.png",
    ROOT / "target/kdv-storybook-window-drawio-diagram-smoke-hover-pan-right.png",
    ROOT / "target/kdv-storybook-window-drawio-diagram-smoke-pan-right.png",
    ROOT / "target/kdv-storybook-window-drawio-diagram-smoke-hover-pan-down.png",
    ROOT / "target/kdv-storybook-window-drawio-diagram-smoke-pan-down.png",
    ROOT / "target/kdv-storybook-window-drawio-diagram-smoke-hover-reset-view.png",
    ROOT / "target/kdv-storybook-window-drawio-diagram-smoke-reset-view.png",
    ROOT / "target/kdv-storybook-window-drawio-diagram-smoke-hover-pan-left.png",
    ROOT / "target/kdv-storybook-window-drawio-diagram-smoke-pan-left.png",
    ROOT / "target/kdv-storybook-window-drawio-diagram-smoke-hover-pan-up.png",
    ROOT / "target/kdv-storybook-window-drawio-diagram-smoke-pan-up.png",
    ROOT / "target/kdv-storybook-window-drawio-diagram-smoke-hover-zoom-out.png",
    ROOT / "target/kdv-storybook-window-drawio-diagram-smoke-zoom-out.png",
    ROOT / "target/kdv-storybook-window-drawio-diagram-smoke-hover-trackpad-help.png",
    ROOT / "target/kdv-storybook-window-drawio-diagram-smoke-trackpad-help.png",
    ROOT / "target/kdv-storybook-window-drawio-diagram-smoke-hover-fullscreen.png",
    ROOT / "target/kdv-storybook-window-drawio-diagram-smoke-fullscreen.png",
    ROOT / "target/kdv-storybook-window-footnote-smoke.png",
    ROOT / "target/kdv-storybook-window-footnote-smoke-reference.png",
    ROOT / "target/kdv-storybook-window-footnote-smoke-definition.png",
    ROOT / "target/kdv-storybook-window-table-smoke.png",
    ROOT / "target/kdv-storybook-window-code-copy-smoke.png",
    ROOT / "target/kdv-storybook-window-code-copy-smoke-hover.png",
    ROOT / "target/kdv-storybook-window-code-copy-smoke-copied.png",
    ROOT / "target/kdv-storybook-window-selection-smoke.png",
    ROOT / "target/kdv-storybook-window-slideshow-smoke.png",
    ROOT / "target/kdv-storybook-window-slideshow-smoke-mode.png",
    ROOT / "target/kdv-storybook-window-slideshow-smoke-next.png",
    ROOT / "target/kdv-storybook-window-slideshow-smoke-previous.png",
    ROOT / "target/kdv-storybook-window-slideshow-smoke-close.png",
)
REQUIRED_ACCEPTANCE_SOURCE_CODE_PATHS = (
    ROOT / "Cargo.toml",
    ROOT / "Cargo.lock",
    ROOT / "Justfile",
    ROOT / "scripts/release/assert-viewer-recovery-dod.py",
    ROOT / "scripts/release/generate-storybook-acceptance-artifact.sh",
    ROOT / "scripts/release/generate-storybook-live-acceptance-artifact.sh",
    ROOT
    / "crates/katana-document-viewer/src/preview_runtime/storybook_score_gate_tests.rs",
    ROOT / "crates/katana-document-viewer/src/viewer/image_surface.rs",
    ROOT / "crates/katana-document-viewer/src/viewer/image_surface_factory.rs",
    ROOT / "crates/katana-document-viewer/src/viewer/image_surface_tests.rs",
    ROOT
    / "crates/katana-document-viewer/src/viewer/node_plan/builder_surface_height_test_support.rs",
    ROOT
    / "crates/katana-document-viewer/src/viewer/node_plan/builder_surface_height_tests.rs",
    ROOT / "crates/katana-document-viewer/src/viewer/node_plan/builder_media_height.rs",
    ROOT / "crates/katana-document-viewer/src/viewer/node_plan/builder_media_asset_height.rs",
    ROOT / "tools/kdv-storybook/src/args.rs",
    ROOT / "tools/kdv-storybook/src/main.rs",
    ROOT / "tools/kdv-storybook/src/frame_score_preview_crop_tests.rs",
    ROOT / "tools/kdv-storybook/src/frame_score_visual_tests.rs",
    ROOT / "tools/kdv-storybook/src/frame_surface_parity_tests.rs",
    ROOT / "tools/kdv-storybook/src/frame_performance_tests.rs",
    ROOT / "tools/kdv-storybook/src/preview_build_methods.rs",
    ROOT / "tools/kdv-storybook/src/window_command/tests/diagram.rs",
    ROOT / "tools/kdv-storybook/src/window/scroll_lazy_scene_tests.rs",
    ROOT / "tools/kdv-storybook/src/window_asset_job.rs",
    ROOT / "tools/kdv-storybook/src/window_asset_job_tests.rs",
    ROOT / "tools/kdv-storybook/src/window_loop.rs",
    ROOT / "tools/kdv-storybook/src/window_loop_tests.rs",
    ROOT / "tools/kdv-storybook/src/window_mouse.rs",
    ROOT / "tools/kdv-storybook/src/preview_theme_bridge.rs",
    ROOT / "tools/kdv-storybook/src/window_tests.rs",
    ROOT / "tools/kdv-storybook/src/document_viewer.rs",
    ROOT / "tools/kdv-storybook/src/test_assert.rs",
    ROOT / "tools/kdv-storybook/src/document_viewer/media_control_icons.rs",
    ROOT / "tools/kdv-storybook/src/document_viewer/node_factory_media_controls.rs",
    ROOT / "tools/kdv-storybook/src/document_viewer/node_factory_media_display_tests.rs",
    ROOT / "tools/kdv-storybook/src/document_viewer/node_factory.rs",
    ROOT / "tools/kdv-storybook/src/document_viewer/node_factory_media_geometry.rs",
    ROOT / "tools/kdv-storybook/src/document_viewer/node_factory_media_impl.rs",
    ROOT / "tools/kdv-storybook/src/document_viewer/node_factory_media_impl_tests.rs",
    ROOT / "tools/kdv-storybook/src/document_viewer/node_factory_media_fixture.rs",
    ROOT / "tools/kdv-storybook/src/document_viewer/node_factory_media_frame_tests.rs",
    ROOT / "tools/kdv-storybook/src/document_viewer/node_factory_media_frame_row_tests.rs",
    ROOT / "tools/kdv-storybook/src/document_viewer/node_factory_media_control_tests.rs",
    ROOT / "tools/kdv-storybook/src/document_viewer/config_tests.rs",
)
REQUIRED_ACCEPTANCE_SOURCE_CODE_ROOTS = (
    ROOT / "crates/katana-document-viewer/src",
    ROOT / "crates/katana-document-viewer/tests",
    ROOT / "crates/kdv-linter/src",
    ROOT / "tools/kdv-storybook/src",
)
REQUIRED_ACCEPTANCE_REFERENCE_ARTIFACT_SOURCE_PATHS = (
    ROOT / "assets/reference/katana/preview_crops/sample-top.png",
    ROOT / "assets/reference/katana/preview_crops/sample-diagrams-top.png",
)
REQUIRED_ACCEPTANCE_SOURCE_PNG_MIN_DIMENSION = 100
REQUIRED_ACCEPTANCE_SOURCE_PNG_ARTIFACTS = tuple(
    (
        path,
        REQUIRED_ACCEPTANCE_SOURCE_PNG_MIN_DIMENSION,
        REQUIRED_ACCEPTANCE_SOURCE_PNG_MIN_DIMENSION,
    )
    for path in REQUIRED_ACCEPTANCE_SOURCE_ARTIFACT_PATHS
)
REQUIRED_ACCEPTANCE_PNG_ARTIFACTS = (
    (
        ROOT / "target/acceptance/kdv-storybook-acceptance-contact-sheet.png",
        900,
        1200,
    ),
    (
        ROOT / "target/acceptance/kdv-storybook-text-regression-crops.png",
        700,
        900,
    ),
    (
        ROOT / "target/acceptance/kdv-storybook-katana-reference-comparison.png",
        2000,
        900,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/diagram-control-icons.png",
        100,
        140,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/title-body.png",
        740,
        180,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/language-link.png",
        640,
        100,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/html-margin-center.png",
        720,
        390,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/direct-html-margin-left.png",
        740,
        260,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/hover-highlight.png",
        740,
        180,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/wide-title-link-html.png",
        1480,
        460,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/table-section.png",
        740,
        780,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/reference-comparison/sample-top-reference.png",
        700,
        300,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/reference-comparison/sample-top-candidate.png",
        700,
        300,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/reference-comparison/sample-top-diff.png",
        700,
        300,
    ),
    (
        ROOT
        / "target/acceptance/text-regression-crops/reference-comparison/sample-diagrams-reference.png",
        700,
        500,
    ),
    (
        ROOT
        / "target/acceptance/text-regression-crops/reference-comparison/sample-diagrams-candidate.png",
        700,
        500,
    ),
    (
        ROOT
        / "target/acceptance/text-regression-crops/reference-comparison/sample-diagrams-diff.png",
        700,
        500,
    ),
    (
        ROOT / "target/acceptance/kdv-storybook-live-interactive.png",
        1000,
        700,
    ),
    (
        ROOT / "target/acceptance/kdv-storybook-live-light-toggle.png",
        1000,
        700,
    ),
) + REQUIRED_ACCEPTANCE_SOURCE_PNG_ARTIFACTS
REQUIRED_ACCEPTANCE_PPM_ARTIFACTS = (
    (
        ROOT / "target/acceptance/preview-crop-reference/katana_sample_md-preview-crop_reference.ppm",
        1280,
        2400,
    ),
    (
        ROOT / "target/acceptance/preview-crop-reference/katana_sample_md-preview-crop_preview.ppm",
        1280,
        2400,
    ),
    (
        ROOT
        / "target/acceptance/preview-crop-reference/katana_sample_diagrams_md-preview-crop_reference.ppm",
        1280,
        2400,
    ),
    (
        ROOT
        / "target/acceptance/preview-crop-reference/katana_sample_diagrams_md-preview-crop_preview.ppm",
        1280,
        2400,
    ),
)
REQUIRED_ACCEPTANCE_VISUAL_METRICS = (
    (
        ROOT / "target/acceptance/text-regression-crops/reference-comparison/sample-top-reference.png",
        300,
        0.75,
        0.98,
        0.10,
        0.35,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/reference-comparison/sample-top-candidate.png",
        300,
        0.75,
        0.98,
        0.10,
        0.35,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/reference-comparison/sample-top-diff.png",
        60,
        0.02,
        0.35,
        0.01,
        0.20,
    ),
    (
        ROOT
        / "target/acceptance/text-regression-crops/reference-comparison/sample-diagrams-reference.png",
        120,
        0.05,
        0.35,
        0.01,
        0.15,
    ),
    (
        ROOT
        / "target/acceptance/text-regression-crops/reference-comparison/sample-diagrams-candidate.png",
        120,
        0.05,
        0.35,
        0.01,
        0.15,
    ),
    (
        ROOT
        / "target/acceptance/text-regression-crops/reference-comparison/sample-diagrams-diff.png",
        60,
        0.02,
        0.35,
        0.01,
        0.20,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/diagram-control-icons.png",
        30,
        0.04,
        0.35,
        0.03,
        0.25,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/table-section.png",
        300,
        0.05,
        0.35,
        0.01,
        0.15,
    ),
)
REQUIRED_ACCEPTANCE_REFERENCE_PAIR_DIFF_METRICS = (
    (
        ROOT / "target/acceptance/text-regression-crops/reference-comparison/sample-top-reference.png",
        ROOT / "target/acceptance/text-regression-crops/reference-comparison/sample-top-candidate.png",
        "sample top text/link reference pair",
        0.035,
        0.130,
        0.160,
    ),
    (
        ROOT
        / "target/acceptance/text-regression-crops/reference-comparison/sample-diagrams-reference.png",
        ROOT
        / "target/acceptance/text-regression-crops/reference-comparison/sample-diagrams-candidate.png",
        "sample diagrams SVG reference pair",
        0.015,
        0.080,
        0.100,
    ),
)
REQUIRED_ACCEPTANCE_REFERENCE_TEXT_CONTRAST_METRICS = (
    (
        ROOT / "target/acceptance/text-regression-crops/reference-comparison/sample-top-reference.png",
        ROOT / "target/acceptance/text-regression-crops/reference-comparison/sample-top-candidate.png",
        "sample top text/link dark text contrast",
        0.45,
        18_000,
        17_000,
        0.93,
    ),
)
REQUIRED_ACCEPTANCE_CROP_CONTENT_METRICS = (
    (
        ROOT / "target/acceptance/text-regression-crops/title-body.png",
        300,
        2500,
        0,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/language-link.png",
        80,
        300,
        100,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/html-margin-center.png",
        80,
        5000,
        0,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/direct-html-margin-left.png",
        150,
        1500,
        0,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/hover-highlight.png",
        300,
        2500,
        0,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/wide-title-link-html.png",
        300,
        7500,
        100,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/table-section.png",
        300,
        5000,
        0,
    ),
)
REQUIRED_ACCEPTANCE_CROP_CHANGED_PIXELS = (
    (
        ROOT / "target/acceptance/text-regression-crops/title-body.png",
        ROOT / "target/acceptance/text-regression-crops/hover-highlight.png",
        250,
    ),
)
REQUIRED_ACCEPTANCE_HOVER_HIGHLIGHT_BANDS = (
    (
        ROOT / "target/acceptance/text-regression-crops/title-body.png",
        ROOT / "target/acceptance/text-regression-crops/hover-highlight.png",
        20,
        740,
        30,
        82,
        680,
        730,
        35,
        45,
        30_000,
        32_000,
    ),
)
REQUIRED_ACCEPTANCE_TITLE_BODY_TEXT_BANDS = (
    (
        ROOT / "target/acceptance/text-regression-crops/title-body.png",
        "title heading",
        24,
        34,
        45,
        55,
        380,
        480,
        22,
        30,
        2_100,
        2_800,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/title-body.png",
        "body first line",
        24,
        34,
        104,
        112,
        620,
        740,
        14,
        20,
        1_750,
        2_700,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/title-body.png",
        "body second line",
        24,
        34,
        127,
        135,
        560,
        740,
        14,
        20,
        1_750,
        2_400,
    ),
)
REQUIRED_ACCEPTANCE_SIDEBAR_SELECTED_ROW_BANDS = (
    (
        ROOT / "target/kdv-storybook-window-sidebar-smoke.png",
        38,
        42,
        90,
        112,
        440,
        460,
        20,
        24,
        8_500,
        10_000,
    ),
    (
        ROOT / "target/kdv-storybook-window-sidebar-narrow-smoke.png",
        38,
        42,
        90,
        112,
        440,
        460,
        20,
        24,
        8_500,
        10_000,
    ),
    (
        ROOT / "target/kdv-storybook-window-sidebar-large-smoke.png",
        38,
        42,
        90,
        112,
        440,
        460,
        20,
        24,
        8_500,
        10_000,
    ),
)
REQUIRED_ACCEPTANCE_TABLE_SECTION_BANDS = (
    ("5. Tables heading", 0, 40),
    ("5.1 Basic Table heading", 50, 90),
    ("basic table header row", 105, 145),
    ("basic table bottom row", 315, 350),
    ("5.2 Table with Alignment heading", 385, 420),
    ("alignment table first row", 440, 470),
    ("alignment table bottom row", 545, 575),
    ("5.3 Single Row Table heading", 610, 645),
)
REQUIRED_ACCEPTANCE_TABLE_SECTION_ROW_COUNTS = (
    ("basic table rows", 100, 370, 5, 5),
    ("alignment table rows", 430, 590, 3, 3),
)
REQUIRED_ACCEPTANCE_TABLE_GRID_COMPONENTS = (
    (
        ROOT / "target/acceptance/text-regression-crops/table-section.png",
        "basic table grid",
        0,
        4,
        98,
        102,
        722,
        726,
        250,
        275,
        5_300,
        5_600,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/table-section.png",
        "alignment table grid",
        0,
        4,
        428,
        434,
        722,
        726,
        150,
        165,
        3_300,
        3_700,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/table-section.png",
        "single-row table grid",
        0,
        4,
        654,
        660,
        722,
        726,
        100,
        110,
        2_200,
        2_500,
    ),
)
REQUIRED_ACCEPTANCE_DIAGRAM_CONTROL_ICON_CELLS = (
    ("pan-up", 28, 0, 26, 20, 20, 120),
    ("zoom-in", 58, 0, 28, 20, 45, 160),
    ("pan-left", 0, 24, 22, 28, 20, 120),
    ("reset-view", 26, 22, 30, 30, 80, 250),
    ("pan-right", 62, 24, 22, 28, 20, 120),
    ("trackpad-help", 0, 52, 22, 38, 40, 260),
    ("pan-down", 28, 56, 26, 24, 20, 120),
    ("zoom-out", 58, 54, 30, 28, 45, 180),
)
REQUIRED_ACCEPTANCE_DIAGRAM_CONTROL_STRIP_REGIONS = (
    (
        ROOT / "target/kdv-storybook-window-diagram-smoke-hover-reset-view.png",
        "Mermaid top-right control strip",
        1160,
        145,
        110,
        150,
    ),
    (
        ROOT / "target/kdv-storybook-window-drawio-diagram-smoke-hover-reset-view.png",
        "Draw.io top-right control strip",
        1160,
        145,
        110,
        150,
    ),
)
REQUIRED_ACCEPTANCE_DIAGRAM_CONTROL_STRIP_CELL_THRESHOLDS = {
    command: (min_pixels, max_pixels)
    for command, _, _, _, _, min_pixels, max_pixels in (
        REQUIRED_ACCEPTANCE_DIAGRAM_CONTROL_ICON_CELLS
    )
} | {
    "trackpad-help": (40, 260),
}
REQUIRED_ACCEPTANCE_HTML_CENTER_TEXT_BANDS = (
    (
        ROOT / "target/acceptance/text-regression-crops/html-margin-center.png",
        "centered HTML heading",
        170,
        200,
        380,
        410,
        150,
        190,
        700,
        1_050,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/html-margin-center.png",
        "centered HTML paragraph",
        305,
        335,
        375,
        435,
        500,
        590,
        1_400,
        2_100,
    ),
)
REQUIRED_ACCEPTANCE_LINK_UNDERLINE_BANDS = (
    (
        ROOT / "target/acceptance/text-regression-crops/language-link.png",
        300,
        370,
        0,
        20,
        35,
        60,
        140,
        340,
        35,
    ),
    (
        ROOT / "target/acceptance/text-regression-crops/wide-title-link-html.png",
        760,
        850,
        125,
        150,
        35,
        60,
        140,
        380,
        35,
    ),
)
REQUIRED_ACCEPTANCE_REFERENCE_CONTENT_METRICS = (
    (
        ROOT
        / "target/acceptance/text-regression-crops/reference-comparison/sample-top-reference.png",
        100,
        0,
    ),
    (
        ROOT
        / "target/acceptance/text-regression-crops/reference-comparison/sample-top-candidate.png",
        100,
        0,
    ),
    (
        ROOT
        / "target/acceptance/text-regression-crops/reference-comparison/sample-diagrams-reference.png",
        0,
        1000,
    ),
    (
        ROOT
        / "target/acceptance/text-regression-crops/reference-comparison/sample-diagrams-candidate.png",
        0,
        1000,
    ),
)
REQUIRED_ACCEPTANCE_REFERENCE_EDGE_RATIO_METRICS = (
    (
        ROOT
        / "target/acceptance/preview-crop-reference/katana_sample_diagrams_md-preview-crop_reference.ppm",
        ROOT
        / "target/acceptance/preview-crop-reference/katana_sample_diagrams_md-preview-crop_preview.ppm",
        "sample diagrams full-resolution SVG edge ratio",
        "1280x920+0+250",
        0.95,
    ),
    (
        ROOT
        / "target/acceptance/preview-crop-reference/katana_sample_diagrams_md-preview-crop_reference.ppm",
        ROOT
        / "target/acceptance/preview-crop-reference/katana_sample_diagrams_md-preview-crop_preview.ppm",
        "sample diagrams flowchart SVG edge ratio",
        "500x500+300+300",
        0.95,
    ),
    (
        ROOT
        / "target/acceptance/preview-crop-reference/katana_sample_diagrams_md-preview-crop_reference.ppm",
        ROOT
        / "target/acceptance/preview-crop-reference/katana_sample_diagrams_md-preview-crop_preview.ppm",
        "sample diagrams sequence SVG edge ratio",
        "900x480+190+1180",
        0.95,
    ),
)
REQUIRED_LIVE_ACCEPTANCE_THEME_SWITCH = (
    ROOT / "target/acceptance/kdv-storybook-live-interactive.png",
    ROOT / "target/acceptance/kdv-storybook-live-light-toggle.png",
    100_000,
    100_000,
    50_000,
)
REQUIRED_LIVE_ACCEPTANCE_WINDOW_SCREENSHOT_SIZE = (
    (ROOT / "target/acceptance/kdv-storybook-live-interactive.png", 2560, 1864),
    (ROOT / "target/acceptance/kdv-storybook-live-light-toggle.png", 2560, 1864),
)
REQUIRED_LIVE_ACCEPTANCE_INTERACTIVE_CONTENT = (
    ROOT / "target/acceptance/kdv-storybook-live-interactive.png",
    "live interactive initial content",
    "2560x1760+0+100",
    20_000,
    128,
)
REQUIRED_LIVE_ACCEPTANCE_LIGHT_TEXT_CONTRAST = (
    ROOT / "target/acceptance/kdv-storybook-live-light-toggle.png",
    "live light viewer content",
    "1500x1500+1480+320",
    1_500_000,
    20_000,
    0.90,
)
REQUIRED_ACCEPTANCE_LOG_FORBIDDEN_PATTERNS = (
    re.compile(r"asset job result send failed"),
    re.compile(r"sending on a closed channel"),
    re.compile(r"thread '.*' panicked"),
    re.compile(r"panicked at "),
)
REQUIRED_ACCEPTANCE_LOG_MARKERS = (
    "storybook live acceptance headless artifact ready",
    "storybook live acceptance interactive content ready",
    "storybook live acceptance clicked dark toggle",
    "storybook live acceptance theme switch verified",
)
REQUIRED_SCROLL_PERFORMANCE_VALUES = {
    "scenario": "large_loaded_diagram_wheel_present",
    "fixture": "katana/sample_drawio_performance.md",
    "full_preview_redraw_fallback_count": "0",
}
INCOMPLETE_MARKERS = [
    "DoD全体は未完了",
    "v0.2.0 goal 全体はまだ完了扱いにしない",
    "未完了とする",
]
NATIVE_FULLSCREEN_LEDGER_FILES = (
    USER_FEEDBACK,
    REMAINING_PLAN,
    STORYBOOK_USER_ACCEPTANCE,
)
NATIVE_FULLSCREEN_STALE_LEDGER_PATTERNS = (
    re.compile(r"native fullscreen host へ同期"),
    re.compile(r"(?:OS|macOS|live OS) window(?: backend)? に反映するだけ"),
    re.compile(r"native fullscreen sync の修正"),
    re.compile(r"host boundary として実装した"),
    re.compile(r"toggleFullScreen:` を送る"),
)
NATIVE_FULLSCREEN_LEDGER_ALLOW_MARKERS = (
    "historical",
    "superseded",
    "撤回",
    "反する",
    "禁止",
    "削除",
    "除去",
    "戻した",
    "削除済み",
    "変換しない",
    "呼ばない",
    "反映しない",
    "直接実行しない",
    "no longer",
    "removed",
    "without OS window side effects",
    "must not",
)
HEADLESS_LIVE_ACCEPTANCE_CURRENT_REQUIRED_TOKENS = (
    "headless live-acceptance artifact",
    "KUC 実 UI tree",
    "Dark toggle typed action",
    "human acceptance の代替ではない",
)
HEADLESS_LIVE_ACCEPTANCE_CURRENT_STALE_PATTERNS = (
    re.compile(r"live OS artifact"),
    re.compile(r"CoreGraphics"),
    re.compile(r"screencapture"),
)


def main() -> int:
    if len(sys.argv) > 1 and sys.argv[1] == "--self-test":
        return self_test()

    feedback = USER_FEEDBACK.read_text(encoding="utf-8")
    remaining = REMAINING_PLAN.read_text(encoding="utf-8")
    design = (CHANGE / "design.md").read_text(encoding="utf-8")
    kuc_context_menu_adr = KUC_CONTEXT_MENU_ADR.read_text(encoding="utf-8")
    kuc_interaction_target_adr = KUC_INTERACTION_TARGET_ADR.read_text(
        encoding="utf-8"
    )
    kuc_document_viewer_harness_adr = KUC_DOCUMENT_VIEWER_HARNESS_ADR.read_text(
        encoding="utf-8"
    )
    acceptance = (
        STORYBOOK_USER_ACCEPTANCE.read_text(encoding="utf-8")
        if STORYBOOK_USER_ACCEPTANCE.exists()
        else ""
    )
    open_feedback_items = open_checklist_items(feedback)
    open_remaining_items = open_checklist_items(remaining)
    incomplete = [
        marker
        for marker in INCOMPLETE_MARKERS
        if marker in remaining or marker in feedback
    ]
    acceptance_errors = storybook_acceptance_errors(acceptance)
    acceptance_ledger_errors = pending_acceptance_required_open_feedback_errors(
        acceptance,
        feedback,
    )
    kuc_blocker_errors = kuc_blocker_ledger_errors(
        feedback,
        remaining,
        design,
        kuc_context_menu_adr,
        kuc_interaction_target_adr,
    )
    kuc_interaction_completion_errors = kuc_interaction_target_completion_errors(
        remaining,
    )
    kuc_interaction_dependency_errors = kuc_interaction_target_dependency_errors(
        remaining,
        (ROOT / "Cargo.toml").read_text(encoding="utf-8"),
        (ROOT / "Cargo.lock").read_text(encoding="utf-8"),
        (ROOT / "tools/kdv-storybook/src/mouse_host_action.rs").read_text(
            encoding="utf-8"
        ),
    )
    handoff_ledger_errors = handoff_canonical_feedback_path_errors()
    document_viewer_harness_errors = document_viewer_harness_ownership_errors(
        remaining,
        design,
        kuc_document_viewer_harness_adr,
        handoff_feedback_ledger_text(),
    )
    kuc_cargo_dependency_gate_errors = kuc_cargo_dependency_errors(
        (ROOT / "Cargo.toml").read_text(encoding="utf-8"),
        (ROOT / "Cargo.lock").read_text(encoding="utf-8"),
        (ROOT / "tools/kdv-storybook/src/main.rs").read_text(encoding="utf-8"),
    )
    native_fullscreen_ledger_errors = native_fullscreen_ledger_contradiction_errors()
    headless_live_acceptance_errors = headless_live_acceptance_contract_errors(
        acceptance
    )
    source_integrity_errors = acceptance_source_integrity_errors()
    local_runtime_patch_errors = local_runtime_patch_overlay_errors()
    artifact_file_errors = acceptance_artifact_file_errors(
        acceptance_evidence(acceptance).get("confirmed_at", "").strip(),
        include_source_integrity=False,
    )
    plantuml_ci_errors = plantuml_ci_runtime_errors(
        CI_WORKFLOW.read_text(encoding="utf-8"),
        PREFLIGHT_WORKFLOW.read_text(encoding="utf-8"),
    )
    release_workflow_errors = release_workflow_acceptance_runtime_errors(
        RELEASE_WORKFLOW.read_text(encoding="utf-8")
    )
    release_test_entrypoint_errors = release_test_entrypoint_isolation_errors(
        (ROOT / "Justfile").read_text(encoding="utf-8")
    )

    if (
        open_feedback_items
        or open_remaining_items
        or incomplete
        or acceptance_errors
        or acceptance_ledger_errors
        or kuc_blocker_errors
        or kuc_interaction_completion_errors
        or kuc_interaction_dependency_errors
        or handoff_ledger_errors
        or document_viewer_harness_errors
        or kuc_cargo_dependency_gate_errors
        or native_fullscreen_ledger_errors
        or headless_live_acceptance_errors
        or source_integrity_errors
        or local_runtime_patch_errors
        or artifact_file_errors
        or plantuml_ci_errors
        or release_workflow_errors
        or release_test_entrypoint_errors
    ):
        print(
            "release DoD is not satisfied for the current KDV release target.",
            file=sys.stderr,
        )
        for error in acceptance_errors:
            print(f"storybook user acceptance: {error}", file=sys.stderr)
        for error in acceptance_ledger_errors:
            print(f"storybook user acceptance: {error}", file=sys.stderr)
        for error in kuc_blocker_errors:
            print(f"kuc blocker ledger: {error}", file=sys.stderr)
        for error in kuc_interaction_completion_errors:
            print(f"kuc blocker ledger: {error}", file=sys.stderr)
        for error in kuc_interaction_dependency_errors:
            print(f"kuc blocker ledger: {error}", file=sys.stderr)
        for error in handoff_ledger_errors:
            print(f"handoff feedback ledger: {error}", file=sys.stderr)
        for error in document_viewer_harness_errors:
            print(f"document_viewer harness ownership: {error}", file=sys.stderr)
        for error in kuc_cargo_dependency_gate_errors:
            print(f"kuc cargo dependency: {error}", file=sys.stderr)
        for error in native_fullscreen_ledger_errors:
            print(f"native fullscreen ledger: {error}", file=sys.stderr)
        for error in headless_live_acceptance_errors:
            print(f"storybook live acceptance contract: {error}", file=sys.stderr)
        for error in source_integrity_errors:
            print(f"release source integrity: {error}", file=sys.stderr)
        for error in local_runtime_patch_errors:
            print(f"release dependency overlay: {error}", file=sys.stderr)
        for error in artifact_file_errors:
            print(f"release acceptance artifact: {error}", file=sys.stderr)
        for error in plantuml_ci_errors:
            print(f"release PlantUML runtime: {error}", file=sys.stderr)
        for error in release_workflow_errors:
            print(f"release workflow runtime: {error}", file=sys.stderr)
        for error in release_test_entrypoint_errors:
            print(f"release test entrypoint: {error}", file=sys.stderr)
        if open_feedback_items:
            print_open_checklist_items(
                "open user-feedback item(s)", open_feedback_items
            )
        if open_remaining_items:
            print_open_checklist_items(
                "open remaining-plan item(s)", open_remaining_items
            )
        if incomplete:
            print("incomplete DoD markers remain:", file=sys.stderr)
            for marker in incomplete:
                print(f"- {marker}", file=sys.stderr)
        print(
            "Resolve or intentionally reclassify these items in "
            "user-feedback-todo.md and remaining-plan.md before release.",
            file=sys.stderr,
        )
        return 1

    print("release DoD check: ok")
    return 0


def self_test() -> int:
    pending = acceptance_document("pending", checked=True, evidence=True)
    accepted_partial = acceptance_document("accepted", checked=False, evidence=True)
    accepted_missing_evidence = acceptance_document("accepted", checked=True, evidence=False)
    accepted_complete = acceptance_document("accepted", checked=True, evidence=True)
    accepted_missing_required_check = accepted_complete.replace(
        "- [x] just storybook\n",
        "",
    )
    accepted_check_mentions_item_after_negation = accepted_complete.replace(
        "- [x] just storybook\n",
        "- [x] not just storybook\n",
    )
    just_storybook_matrix_row = (
        f"| just storybook | {acceptance_matrix_evidence_text('just storybook')} "
        "| human confirmed |\n"
    )
    accepted_missing_matrix_row = accepted_complete.replace(
        just_storybook_matrix_row,
        "",
    )
    accepted_matrix_mentions_item_outside_first_column = accepted_complete.replace(
        just_storybook_matrix_row,
        "| unrelated row | target/release/kdv-storybook --interactive --frames 0; "
        "storybook-entrypoint-check; just storybook gate | human confirmed |\n",
    )
    accepted_with_weak_matrix_evidence = accepted_complete.replace(
        acceptance_matrix_evidence_text("just storybook"),
        "automated gate",
        1,
    )
    accepted_missing_artifact_evidence = accepted_complete.replace(
        "- artifact: target/acceptance/kdv-storybook-acceptance-artifacts.sha256\n",
        "",
    )
    accepted_missing_live_artifact = accepted_complete.replace(
        "- artifact: target/acceptance/kdv-storybook-live-acceptance-artifacts.sha256\n",
        "",
    )
    accepted_with_pending_matrix = accepted_complete.replace(
        "human confirmed",
        "pending user real-machine confirmation",
        1,
    )
    accepted_with_weak_matrix = accepted_complete.replace(
        "human confirmed",
        "looks okay",
        1,
    )
    accepted_bad_timestamp = accepted_complete.replace(
        "- confirmed_at: 2026-06-18T00:00:00+09:00",
        "- confirmed_at: June 18",
    )
    accepted_future_timestamp = accepted_complete.replace(
        "- confirmed_at: 2026-06-18T00:00:00+09:00",
        "- confirmed_at: 2099-01-01T00:00:00+00:00",
    )
    accepted_placeholder_reviewer = accepted_complete.replace(
        "- confirmed_by: hiroyuki_furuno",
        "- confirmed_by: human reviewer",
    )

    cases = [
        ("pending status", pending, True),
        ("accepted with incomplete checklist", accepted_partial, True),
        ("accepted without required checklist row", accepted_missing_required_check, True),
        (
            "accepted with required checklist text after negation",
            accepted_check_mentions_item_after_negation,
            True,
        ),
        ("accepted without required matrix row", accepted_missing_matrix_row, True),
        (
            "accepted with required matrix text outside first column",
            accepted_matrix_mentions_item_outside_first_column,
            True,
        ),
        ("accepted with weak matrix evidence", accepted_with_weak_matrix_evidence, True),
        ("accepted without evidence", accepted_missing_evidence, True),
        ("accepted without artifact evidence", accepted_missing_artifact_evidence, True),
        ("accepted without live artifact evidence", accepted_missing_live_artifact, True),
        ("accepted with pending matrix", accepted_with_pending_matrix, True),
        ("accepted with weak matrix", accepted_with_weak_matrix, True),
        ("accepted with invalid timestamp", accepted_bad_timestamp, True),
        ("accepted with future timestamp", accepted_future_timestamp, True),
        ("accepted with placeholder reviewer", accepted_placeholder_reviewer, True),
        ("accepted complete", accepted_complete, False),
    ]
    failures: list[str] = []
    checklist = "- [ ] open release blocker\n- [/] partial item\n- [x] done item\n"
    if open_checklist_items(checklist) != ["- [ ] open release blocker"]:
        failures.append("open checklist scanner must only return unchecked items")
    pending_feedback = (
        "- [ ] UF-040: pending\n"
        "- [ ] UF-042: pending\n"
    )
    closed_feedback = (
        "- [x] UF-040: closed too early\n"
        "- [/] UF-042: partial only\n"
    )
    if pending_acceptance_required_open_feedback_errors(pending, pending_feedback):
        failures.append(
            "pending acceptance ledger scanner must allow open UF-040/UF-042"
        )
    if not pending_acceptance_required_open_feedback_errors(pending, closed_feedback):
        failures.append(
            "pending acceptance ledger scanner must reject closed UF-040/UF-042"
        )
    if pending_acceptance_required_open_feedback_errors(
        accepted_complete,
        closed_feedback,
    ):
        failures.append(
            "accepted acceptance ledger scanner must not require open UF-040/UF-042"
        )
    kuc_blocker_feedback = (
        pending_feedback
        + "- [ ] UF-043: pending context menu blocker\n"
        + f"- note: {KUC_ISSUE_7_URL}\n"
        + f"- note: {KUC_ISSUE_8_URL}\n"
    )
    kuc_blocker_remaining = (
        "- [ ] 2026-06-23 追補23: KUC ContextMenu blocker\n"
        f"  - KUC issue: {KUC_ISSUE_7_URL}\n"
        "- [ ] 2026-06-23 追補26: KUC interaction target blocker\n"
        f"  - KUC issue: {KUC_ISSUE_8_URL}\n"
        "- [/] 2026-06-23 追補30: document_viewer harness ownership\n"
        f"  - KDV command: {KDV_DOCUMENT_VIEWER_REQUIRED_COMMAND}\n"
    )
    kuc_blocker_design = (
        f"KUC issue #8 {KUC_ISSUE_8_URL} "
        + " ".join(KUC_INTERACTION_TARGET_REQUIRED_TOKENS)
    )
    kuc_blocker_adr = f"KUC issue #7 {KUC_ISSUE_7_URL}"
    kuc_interaction_adr = (
        f"KUC issue #8 {KUC_ISSUE_8_URL} "
        + " ".join(KUC_INTERACTION_TARGET_REQUIRED_TOKENS)
    )
    document_viewer_harness_adr = (
        "KDV owns the KatanA document_viewer harness. "
        "katana-document-viewer uses katana-ui-core-storybook as generic KUC "
        "component support. "
        f"`{KDV_DOCUMENT_VIEWER_REQUIRED_COMMAND}` is required. "
        f"`{KUC_DOCUMENT_VIEWER_REJECTED_COMMAND}` returned 0 passed and is "
        "not release proof."
    )
    document_viewer_handoff = (
        "KDV:\n"
        f"{KDV_DOCUMENT_VIEWER_REQUIRED_COMMAND} -- --test-threads=1\n"
        "KUC:\n"
        "cargo test -p katana-ui-core --locked settings -- --test-threads=1\n"
    )
    if kuc_blocker_ledger_errors(
        kuc_blocker_feedback,
        kuc_blocker_remaining,
        kuc_blocker_design,
        kuc_blocker_adr,
        kuc_interaction_adr,
    ):
        failures.append("KUC blocker ledger scanner must allow current blocker mapping")
    if not kuc_blocker_ledger_errors(
        kuc_blocker_feedback.replace("- [ ] UF-043", "- [/] UF-043"),
        kuc_blocker_remaining,
        kuc_blocker_design,
        kuc_blocker_adr,
        kuc_interaction_adr,
    ):
        failures.append("KUC blocker ledger scanner must reject closed UF-043")
    if not kuc_blocker_ledger_errors(
        kuc_blocker_feedback,
        kuc_blocker_remaining.replace(KUC_ISSUE_8_URL, ""),
        kuc_blocker_design,
        kuc_blocker_adr,
        kuc_interaction_adr,
    ):
        failures.append("KUC blocker ledger scanner must reject missing KUC #8 issue")
    if not kuc_blocker_ledger_errors(
        kuc_blocker_feedback,
        kuc_blocker_remaining,
        kuc_blocker_design.replace("typed action injection", ""),
        kuc_blocker_adr,
        kuc_interaction_adr,
    ):
        failures.append("KUC blocker ledger scanner must reject missing KUC #8 scope")
    if not kuc_blocker_ledger_errors(
        kuc_blocker_feedback,
        kuc_blocker_remaining,
        kuc_blocker_design,
        kuc_blocker_adr,
        kuc_interaction_adr.replace(KUC_ISSUE_8_URL, ""),
    ):
        failures.append("KUC blocker ledger scanner must reject missing KUC #8 ADR")
    if kuc_interaction_target_completion_errors(kuc_blocker_remaining):
        failures.append("KUC #8 completion scanner must allow open 追補26 blocker")
    closed_kuc_interaction_without_evidence = kuc_blocker_remaining.replace(
        "- [ ] 2026-06-23 追補26",
        "- [/] 2026-06-23 追補26",
    )
    if not kuc_interaction_target_completion_errors(
        closed_kuc_interaction_without_evidence
    ):
        failures.append(
            "KUC #8 completion scanner must reject closed 追補26 without evidence"
        )
    closed_kuc_interaction_with_evidence = (
        closed_kuc_interaction_without_evidence
        + "\nKUC #8 completion evidence:\n"
        + f"- issue: {KUC_ISSUE_8_URL}\n"
        + "- scope: TreeView / FileTree / SettingsList / Toggle / Button / "
        "link-like span / media control\n"
        + "- bridge: host_action_hits and interaction_for_hit\n"
        + "- API: UiTreeInteractionTarget and "
        "UiTreeSurfaceHost::interaction_target_for_hits_at\n"
        + "- KUC: cargo test -p katana-ui-core-storybook --locked "
        "ui_tree_interaction_surface\n"
        + "- KUC: cargo test -p katana-ui-core-storybook --locked "
        "ui_tree_storybook_host\n"
        + "- KUC: cargo test -p katana-ui-core --locked settings\n"
        + "- KUC: cargo test -p katana-ui-core --locked file_tree\n"
        + "- KDV: cargo test -p kdv-storybook --locked "
        "no_reintroduced_manual_storybook_action_contracts\n"
    )
    if kuc_interaction_target_completion_errors(closed_kuc_interaction_with_evidence):
        failures.append(
            "KUC #8 completion scanner must allow closed 追補26 with evidence"
        )
    stale_interaction_toml = "\n".join(
        f'{name} = {{ git = "{KUC_CARGO_GIT_URL}", '
        f'tag = "{KUC_INTERACTION_TARGET_STALE_CARGO_TAG}" }}'
        for name in KUC_CARGO_DEPENDENCY_NAMES
    )
    stale_interaction_lock = "\n".join(
        (
            f'name = "{name}"\n'
            f'source = "{KUC_INTERACTION_TARGET_STALE_CARGO_LOCK_SOURCE}"'
        )
        for name in KUC_CARGO_DEPENDENCY_NAMES
    )
    updated_interaction_toml = stale_interaction_toml.replace(
        KUC_INTERACTION_TARGET_STALE_CARGO_TAG,
        "v0.1.4",
    )
    updated_interaction_lock = stale_interaction_lock.replace(
        KUC_INTERACTION_TARGET_STALE_CARGO_LOCK_SOURCE,
        "git+https://github.com/HiroyukiFuruno/katana-ui-core.git"
        "?tag=v0.1.4#0123456789abcdef",
    )
    updated_mouse_host_action = (
        "use katana_ui_core_storybook::UiTreeInteractionTarget;\n"
        "UiTreeSurfaceHost::interaction_target_for_hits_at(&hits, &node_hits, x, y)\n"
        ".map(|target: UiTreeInteractionTarget| target.hover_node_id())\n"
    )
    if kuc_interaction_target_dependency_errors(
        kuc_blocker_remaining,
        stale_interaction_toml,
        stale_interaction_lock,
        "",
    ):
        failures.append("KUC #8 dependency scanner must allow open 追補26 blocker")
    if not kuc_interaction_target_dependency_errors(
        closed_kuc_interaction_with_evidence,
        stale_interaction_toml,
        stale_interaction_lock,
        "",
    ):
        failures.append(
            "KUC #8 dependency scanner must reject closed 追補26 on v0.1.1"
        )
    if kuc_interaction_target_dependency_errors(
        closed_kuc_interaction_with_evidence,
        updated_interaction_toml,
        updated_interaction_lock,
        updated_mouse_host_action,
    ):
        failures.append(
            "KUC #8 dependency scanner must allow closed 追補26 after KDV consumes "
            "the interaction target API"
        )
    if document_viewer_harness_ownership_errors(
        kuc_blocker_remaining,
        kuc_blocker_design + f" {KDV_DOCUMENT_VIEWER_REQUIRED_COMMAND}",
        document_viewer_harness_adr,
        document_viewer_handoff,
    ):
        failures.append("document_viewer harness scanner must allow current mapping")
    if not document_viewer_harness_ownership_errors(
        kuc_blocker_remaining,
        kuc_blocker_design + f" {KDV_DOCUMENT_VIEWER_REQUIRED_COMMAND}",
        document_viewer_harness_adr.replace(KDV_DOCUMENT_VIEWER_REQUIRED_COMMAND, ""),
        document_viewer_handoff,
    ):
        failures.append("document_viewer harness scanner must reject missing KDV command")
    if not document_viewer_harness_ownership_errors(
        kuc_blocker_remaining,
        kuc_blocker_design + f" {KDV_DOCUMENT_VIEWER_REQUIRED_COMMAND}",
        document_viewer_harness_adr,
        document_viewer_handoff
        + f"{KUC_DOCUMENT_VIEWER_REJECTED_COMMAND} -- --test-threads=1\n",
    ):
        failures.append(
            "document_viewer harness scanner must reject stale KUC document_viewer command"
        )
    valid_kuc_cargo_toml = (
        'katana-ui-core = { version = "=0.3.7", features = ["raster-host"] }'
    )
    valid_kuc_cargo_lock = (
        "[[package]]\n"
        'name = "katana-ui-core"\n'
        'version = "0.3.7"\n'
        'source = "registry+https://github.com/rust-lang/crates.io-index"'
    )
    if kuc_cargo_dependency_errors(
        valid_kuc_cargo_toml,
        valid_kuc_cargo_lock,
        "mod document_viewer;\nmod test_assert;\n",
    ):
        failures.append("KUC Cargo dependency scanner must allow exact registry v0.3.7 deps")
    if not kuc_cargo_dependency_errors(
        valid_kuc_cargo_toml.replace('version = "=0.3.7"', 'version = "=0.3.6"'),
        valid_kuc_cargo_lock,
        "mod document_viewer;\n",
    ):
        failures.append("KUC Cargo dependency scanner must reject stale Cargo.toml version")
    if not kuc_cargo_dependency_errors(
        valid_kuc_cargo_toml,
        valid_kuc_cargo_lock.replace('version = "0.3.7"', 'version = "0.3.6"'),
        "mod document_viewer;\n",
    ):
        failures.append("KUC Cargo dependency scanner must reject stale Cargo.lock version")
    if not kuc_cargo_dependency_errors(
        valid_kuc_cargo_toml.replace(
            'version = "=0.3.7", features',
            'git = "https://github.com/HiroyukiFuruno/katana-ui-core.git", version = "=0.3.7", features',
        ),
        valid_kuc_cargo_lock,
        "mod document_viewer;\n",
    ):
        failures.append("KUC Cargo dependency scanner must reject a KUC git override")
    if not kuc_cargo_dependency_errors(
        valid_kuc_cargo_toml
        + '\nkatana-ui-core-storybook = { package = "katana-ui-core", version = "=0.3.7", features = ["raster-host"] }',
        valid_kuc_cargo_lock,
        "mod document_viewer;\n",
    ):
        failures.append("KUC Cargo dependency scanner must reject a second Storybook alias")
    if not kuc_cargo_dependency_errors(
        valid_kuc_cargo_toml,
        valid_kuc_cargo_lock,
        '#[path = "../../../../katana-ui-core/crates/katana-ui-core-storybook/src/document_viewer.rs"]\nmod document_viewer;\n',
    ):
        failures.append("KUC Cargo dependency scanner must reject sibling source include")
    local_runtime_patch = """[patch.crates-io]
katana-render-runtime = { path = "/tmp/katana-render-runtime" }
"""
    if not local_runtime_patch_overlay_errors_from_toml(local_runtime_patch):
        failures.append("local runtime patch scanner must reject a path overlay")
    if local_runtime_patch_overlay_errors_from_toml(""):
        failures.append("local runtime patch scanner must allow an absent overlay")
    stale_handoff = f"- {STALE_ROOT_FEEDBACK_LEDGER_PATH}\n"
    if not handoff_canonical_feedback_path_errors_from_markdown(
        "self-test.md", stale_handoff
    ):
        failures.append(
            "handoff feedback ledger scanner must reject stale root user-feedback path"
        )
    canonical_handoff = f"- {CANONICAL_FEEDBACK_LEDGER_RELATIVE_PATH}\n"
    if handoff_canonical_feedback_path_errors_from_markdown(
        "self-test.md", canonical_handoff
    ):
        failures.append(
            "handoff feedback ledger scanner must allow canonical OpenSpec user-feedback path"
        )
    canonical_absolute_handoff = f"- {CANONICAL_FEEDBACK_LEDGER_PATH}\n"
    if handoff_canonical_feedback_path_errors_from_markdown(
        "self-test.md", canonical_absolute_handoff
    ):
        failures.append(
            "handoff feedback ledger scanner must allow canonical absolute user-feedback path"
        )
    complete_manifest = acceptance_artifact_manifest_text()
    missing_manifest = complete_manifest.replace(
        "target/acceptance/kdv-storybook-text-regression-crops.png",
        "target/acceptance/missing-text-regression-crops.png",
    )
    missing_source_manifest = complete_manifest.replace(
        "target/kdv-storybook-window-html-margin-smoke.png",
        "target/missing-html-margin-smoke.png",
    )
    missing_live_manifest = complete_manifest.replace(
        "target/acceptance/kdv-storybook-live-light-toggle.png",
        "target/acceptance/missing-live-light-toggle.png",
    )
    if missing_acceptance_artifact_manifest_rows(complete_manifest):
        failures.append("complete acceptance artifact manifest must satisfy required rows")
    if "target/missing-source.rs" not in missing_acceptance_source_code_files_from_paths(
        (ROOT / "target/missing-source.rs",)
    ):
        failures.append("source file scanner must detect missing required source file")
    untracked_labels = untracked_acceptance_source_code_files_from_labels(
        ("tracked.rs", "untracked.rs"),
        ("tracked.rs",),
    )
    if untracked_labels != ["untracked.rs"]:
        failures.append("source tracking scanner must detect untracked required source file")
    integrity_errors = acceptance_source_integrity_errors_from_labels(
        missing_labels=("missing.rs",),
        untracked_labels=("untracked.rs",),
    )
    if not any("missing.rs" in error for error in integrity_errors) or not any(
        "untracked.rs" in error for error in integrity_errors
    ):
        failures.append("source integrity scanner must report missing and untracked files")
    missing_rows = missing_acceptance_artifact_manifest_rows(missing_manifest)
    if missing_rows != ["target/acceptance/kdv-storybook-text-regression-crops.png"]:
        failures.append("manifest row scanner must detect missing text regression crop")
    missing_source_rows = missing_acceptance_artifact_manifest_rows(missing_source_manifest)
    if missing_source_rows != ["target/kdv-storybook-window-html-margin-smoke.png"]:
        failures.append("manifest row scanner must detect missing source screenshot rows")
    missing_live_rows = missing_acceptance_artifact_manifest_rows(missing_live_manifest)
    if missing_live_rows != ["target/acceptance/kdv-storybook-live-light-toggle.png"]:
        failures.append("manifest row scanner must detect missing live acceptance rows")
    mismatched_manifest = complete_manifest.replace("0" * 64, "1" * 64, 1)
    mismatch_errors = acceptance_artifact_manifest_errors_from_text(
        mismatched_manifest,
        expected_digests={
            "target/acceptance/kdv-storybook-acceptance-contact-sheet.png": "0" * 64,
            "target/acceptance/kdv-storybook-text-regression-crops.png": "0" * 64,
        },
    )
    if not any("checksum mismatch" in error for error in mismatch_errors):
        failures.append("manifest checksum scanner must detect mismatched artifact digests")
    stale_acceptance_errors = acceptance_artifact_freshness_errors_from_mtimes(
        "2026-06-18T00:00:00+09:00",
        {
            "target/acceptance/kdv-storybook-acceptance-contact-sheet.png": datetime(
                2026, 6, 18, 0, 0, 1, tzinfo=timezone.utc
            ).timestamp(),
        },
    )
    if not any("confirmed_at is older" in error for error in stale_acceptance_errors):
        failures.append("acceptance artifact freshness scanner must reject stale confirmation")
    stale_source_errors = acceptance_artifact_freshness_errors_from_mtimes(
        "2026-06-18T00:00:00+09:00",
        {
            "crates/katana-document-viewer/src/preview_runtime/storybook_score_gate_tests.rs": datetime(
                2026, 6, 18, 0, 0, 1, tzinfo=timezone.utc
            ).timestamp(),
        },
    )
    if not any("source file" in error for error in stale_source_errors):
        failures.append("acceptance freshness scanner must reject stale source changes")
    current_source_freshness_errors = (
        acceptance_artifact_source_freshness_errors_from_mtimes(
            artifact_mtimes={
                "target/acceptance/kdv-storybook-acceptance-contact-sheet.png": datetime(
                    2026, 6, 18, 0, 0, 2, tzinfo=timezone.utc
                ).timestamp(),
            },
            source_mtimes={
                "scripts/release/assert-viewer-recovery-dod.py": datetime(
                    2026, 6, 18, 0, 0, 1, tzinfo=timezone.utc
                ).timestamp(),
            },
        )
    )
    if current_source_freshness_errors:
        failures.append("acceptance source freshness scanner must accept current artifacts")
    stale_source_freshness_errors = (
        acceptance_artifact_source_freshness_errors_from_mtimes(
            artifact_mtimes={
                "target/acceptance/kdv-storybook-acceptance-contact-sheet.png": datetime(
                    2026, 6, 18, 0, 0, 1, tzinfo=timezone.utc
                ).timestamp(),
            },
            source_mtimes={
                "scripts/release/assert-viewer-recovery-dod.py": datetime(
                    2026, 6, 18, 0, 0, 2, tzinfo=timezone.utc
                ).timestamp(),
            },
        )
    )
    if not any("older than required source file" in error for error in stale_source_freshness_errors):
        failures.append("acceptance source freshness scanner must reject stale artifacts")
    future_error = confirmed_at_future_error(
        "2099-01-01T00:00:00+00:00",
        now=datetime(2026, 6, 18, 0, 0, 0, tzinfo=timezone.utc),
    )
    if not future_error:
        failures.append("acceptance timestamp scanner must reject future confirmation")
    png_error = png_dimension_error(
        b"\x89PNG\r\n\x1a\n" + b"\x00\x00\x00\rIHDR" + struct.pack(">II", 10, 20),
        min_width=100,
        min_height=100,
        label="target/acceptance/tiny.png",
    )
    if not png_error or "too small" not in png_error:
        failures.append("acceptance artifact PNG scanner must reject tiny review images")
    invalid_png_error = png_dimension_error(
        b"not a png",
        min_width=100,
        min_height=100,
        label="target/acceptance/not-png.png",
    )
    if not invalid_png_error or "not a PNG" not in invalid_png_error:
        failures.append("acceptance artifact PNG scanner must reject invalid PNG files")
    invalid_ihdr_error = png_dimension_error(
        b"\x89PNG\r\n\x1a\n" + b"\x00\x00\x00\rJUNK" + struct.pack(">II", 200, 200),
        min_width=100,
        min_height=100,
        label="target/acceptance/not-ihdr.png",
    )
    if not invalid_ihdr_error or "missing IHDR" not in invalid_ihdr_error:
        failures.append("acceptance artifact PNG scanner must reject missing IHDR chunk")
    ppm_error = ppm_dimension_error(
        b"P6\n640 480\n255\n" + b"\0" * 32,
        expected_width=1280,
        expected_height=2400,
        label="target/acceptance/preview-crop-reference/broken.ppm",
    )
    if not ppm_error or "wrong dimensions" not in ppm_error:
        failures.append("acceptance artifact PPM scanner must reject wrong dimensions")
    ppm_error = ppm_dimension_error(
        b"P6\n1280 2400\n255\n" + b"\0" * 32,
        expected_width=1280,
        expected_height=2400,
        label="target/acceptance/preview-crop-reference/truncated.ppm",
    )
    if not ppm_error or "truncated" not in ppm_error:
        failures.append("acceptance artifact PPM scanner must reject truncated data")
    visual_metric_errors = acceptance_artifact_visual_metric_errors_from_values(
        {
            "target/acceptance/text-regression-crops/reference-comparison/sample-top-reference.png": (
                20,
                0.92,
                0.23,
            )
        },
        {
            "target/acceptance/text-regression-crops/reference-comparison/sample-top-reference.png": (
                300,
                0.75,
                0.98,
                0.10,
                0.35,
            )
        },
    )
    if not any("too few colors" in error for error in visual_metric_errors):
        failures.append("acceptance visual metric scanner must reject weak review crops")
    visual_metric_errors = acceptance_artifact_visual_metric_errors_from_values(
        {
            "target/acceptance/text-regression-crops/reference-comparison/sample-top-reference.png": (
                320,
                0.99,
                0.23,
            )
        },
        {
            "target/acceptance/text-regression-crops/reference-comparison/sample-top-reference.png": (
                300,
                0.75,
                0.98,
                0.10,
                0.35,
            )
        },
    )
    if not any("mean out of range" in error for error in visual_metric_errors):
        failures.append("acceptance visual metric scanner must reject blank-like crops")
    reference_pair_diff_errors = (
        acceptance_artifact_reference_pair_diff_errors_from_values(
            {
                "sample top text/link reference pair": (
                    0.080,
                    0.090,
                    0.120,
                )
            },
            {
                "sample top text/link reference pair": (
                    0.035,
                    0.130,
                    0.160,
                )
            },
        )
    )
    if not any("diff mean too high" in error for error in reference_pair_diff_errors):
        failures.append("acceptance reference pair scanner must reject high mean diff")
    reference_pair_diff_errors = (
        acceptance_artifact_reference_pair_diff_errors_from_values(
            {
                "sample diagrams SVG reference pair": (
                    0.010,
                    0.060,
                    0.140,
                )
            },
            {
                "sample diagrams SVG reference pair": (
                    0.015,
                    0.080,
                    0.100,
                )
            },
        )
    )
    if not any("changed ratio too high" in error for error in reference_pair_diff_errors):
        failures.append(
            "acceptance reference pair scanner must reject high changed ratio"
        )
    reference_text_contrast_errors = (
        acceptance_artifact_reference_text_contrast_errors_from_values(
            {
                "sample top text/link dark text contrast": (18_437, 17_566, 0.953),
            },
            {
                "sample top text/link dark text contrast": (18_000, 17_000, 0.930),
            },
        )
    )
    if reference_text_contrast_errors:
        failures.append(
            "acceptance reference text contrast scanner must accept expected dark text ratio"
        )
    reference_text_contrast_errors = (
        acceptance_artifact_reference_text_contrast_errors_from_values(
            {
                "sample top text/link dark text contrast": (18_437, 16_200, 0.879),
            },
            {
                "sample top text/link dark text contrast": (18_000, 17_000, 0.930),
            },
        )
    )
    if not any(
        "candidate dark text pixels" in error
        for error in reference_text_contrast_errors
    ):
        failures.append(
            "acceptance reference text contrast scanner must reject faint candidate text"
        )
    if not any("contrast ratio too low" in error for error in reference_text_contrast_errors):
        failures.append(
            "acceptance reference text contrast scanner must reject low dark text ratio"
        )
    crop_content_errors = acceptance_artifact_crop_content_errors_from_values(
        {
            "target/acceptance/text-regression-crops/language-link.png": (
                90,
                350.0,
                20.0,
            )
        },
        {
            "target/acceptance/text-regression-crops/language-link.png": (
                80,
                300.0,
                100.0,
            )
        },
    )
    if not any("link-blue pixels" in error for error in crop_content_errors):
        failures.append("acceptance crop content scanner must reject missing link underline")
    link_underline_errors = acceptance_artifact_link_underline_errors_from_values(
        {
            "target/acceptance/text-regression-crops/language-link.png": (
                312,
                354,
                0,
                11,
                202,
                43,
            ),
            "target/acceptance/text-regression-crops/wide-title-link-html.png": (
                786,
                828,
                130,
                143,
                225,
                43,
            ),
        },
        {
            str(path.relative_to(ROOT)): (
                min_x,
                max_x,
                min_y,
                max_y,
                min_width,
                max_width,
                min_area,
                max_area,
                min_bottom_pixels,
            )
            for (
                path,
                min_x,
                max_x,
                min_y,
                max_y,
                min_width,
                max_width,
                min_area,
                max_area,
                min_bottom_pixels,
            ) in REQUIRED_ACCEPTANCE_LINK_UNDERLINE_BANDS
        },
    )
    if link_underline_errors:
        failures.append("acceptance link underline scanner must accept expected geometry")
    link_underline_errors = acceptance_artifact_link_underline_errors_from_values(
        {
            "target/acceptance/text-regression-crops/language-link.png": (
                312,
                520,
                0,
                11,
                520,
                210,
            ),
        },
        {
            str(path.relative_to(ROOT)): (
                min_x,
                max_x,
                min_y,
                max_y,
                min_width,
                max_width,
                min_area,
                max_area,
                min_bottom_pixels,
            )
            for (
                path,
                min_x,
                max_x,
                min_y,
                max_y,
                min_width,
                max_width,
                min_area,
                max_area,
                min_bottom_pixels,
            ) in REQUIRED_ACCEPTANCE_LINK_UNDERLINE_BANDS
        },
    )
    if not any("wrong link underline width" in error for error in link_underline_errors):
        failures.append("acceptance link underline scanner must reject full-row underline")
    link_underline_errors = acceptance_artifact_link_underline_errors_from_values(
        {
            "target/acceptance/text-regression-crops/language-link.png": (
                312,
                354,
                0,
                10,
                159,
                7,
            ),
        },
        {
            str(path.relative_to(ROOT)): (
                min_x,
                max_x,
                min_y,
                max_y,
                min_width,
                max_width,
                min_area,
                max_area,
                min_bottom_pixels,
            )
            for (
                path,
                min_x,
                max_x,
                min_y,
                max_y,
                min_width,
                max_width,
                min_area,
                max_area,
                min_bottom_pixels,
            ) in REQUIRED_ACCEPTANCE_LINK_UNDERLINE_BANDS
        },
    )
    if not any("too few underline pixels" in error for error in link_underline_errors):
        failures.append("acceptance link underline scanner must reject missing underline")
    hover_highlight_errors = acceptance_artifact_hover_highlight_errors_from_values(
        {
            "target/acceptance/text-regression-crops/title-body.png -> "
            "target/acceptance/text-regression-crops/hover-highlight.png": (
                28,
                739,
                39,
                81,
                30_616,
            ),
        },
        {
            f"{before.relative_to(ROOT)} -> {after.relative_to(ROOT)}": (
                min_x,
                max_x,
                min_y,
                max_y,
                min_width,
                max_width,
                min_height,
                max_height,
                min_area,
                max_area,
            )
            for (
                before,
                after,
                min_x,
                max_x,
                min_y,
                max_y,
                min_width,
                max_width,
                min_height,
                max_height,
                min_area,
                max_area,
            ) in REQUIRED_ACCEPTANCE_HOVER_HIGHLIGHT_BANDS
        },
    )
    if hover_highlight_errors:
        failures.append("acceptance hover highlight scanner must accept expected geometry")
    hover_highlight_errors = acceptance_artifact_hover_highlight_errors_from_values(
        {
            "target/acceptance/text-regression-crops/title-body.png -> "
            "target/acceptance/text-regression-crops/hover-highlight.png": (
                28,
                739,
                35,
                140,
                60_000,
            ),
        },
        {
            f"{before.relative_to(ROOT)} -> {after.relative_to(ROOT)}": (
                min_x,
                max_x,
                min_y,
                max_y,
                min_width,
                max_width,
                min_height,
                max_height,
                min_area,
                max_area,
            )
            for (
                before,
                after,
                min_x,
                max_x,
                min_y,
                max_y,
                min_width,
                max_width,
                min_height,
                max_height,
                min_area,
                max_area,
            ) in REQUIRED_ACCEPTANCE_HOVER_HIGHLIGHT_BANDS
        },
    )
    if not any("wrong hover highlight y position" in error for error in hover_highlight_errors):
        failures.append("acceptance hover highlight scanner must reject body-covering highlight")
    hover_highlight_errors = acceptance_artifact_hover_highlight_errors_from_values(
        {
            "target/acceptance/text-regression-crops/title-body.png -> "
            "target/acceptance/text-regression-crops/hover-highlight.png": (
                28,
                200,
                35,
                74,
                8_000,
            ),
        },
        {
            f"{before.relative_to(ROOT)} -> {after.relative_to(ROOT)}": (
                min_x,
                max_x,
                min_y,
                max_y,
                min_width,
                max_width,
                min_height,
                max_height,
                min_area,
                max_area,
            )
            for (
                before,
                after,
                min_x,
                max_x,
                min_y,
                max_y,
                min_width,
                max_width,
                min_height,
                max_height,
                min_area,
                max_area,
            ) in REQUIRED_ACCEPTANCE_HOVER_HIGHLIGHT_BANDS
        },
    )
    if not any("wrong hover highlight width" in error for error in hover_highlight_errors):
        failures.append("acceptance hover highlight scanner must reject narrow highlight")
    title_body_thresholds = {
        f"{path.relative_to(ROOT)}:{name}": (
            min_x,
            max_x,
            min_y,
            max_y,
            min_width,
            max_width,
            min_height,
            max_height,
            min_area,
            max_area,
        )
        for (
            path,
            name,
            min_x,
            max_x,
            min_y,
            max_y,
            min_width,
            max_width,
            min_height,
            max_height,
            min_area,
            max_area,
        ) in REQUIRED_ACCEPTANCE_TITLE_BODY_TEXT_BANDS
    }
    title_body_errors = acceptance_artifact_title_body_text_errors_from_values(
        {
            "target/acceptance/text-regression-crops/title-body.png:title heading": (
                29,
                418,
                52,
                74,
                2_232,
            ),
            "target/acceptance/text-regression-crops/title-body.png:body first line": (
                28,
                668,
                106,
                119,
                2_045,
            ),
            "target/acceptance/text-regression-crops/title-body.png:body second line": (
                29,
                608,
                129,
                142,
                1_889,
            ),
        },
        title_body_thresholds,
    )
    if title_body_errors:
        failures.append("acceptance title/body scanner must accept expected text bands")
    title_body_errors = acceptance_artifact_title_body_text_errors_from_values(
        {
            "target/acceptance/text-regression-crops/title-body.png:title heading": (
                29,
                418,
                50,
                65,
                2_232,
            ),
            "target/acceptance/text-regression-crops/title-body.png:body first line": (
                28,
                668,
                106,
                119,
                2_045,
            ),
            "target/acceptance/text-regression-crops/title-body.png:body second line": (
                29,
                608,
                129,
                142,
                1_889,
            ),
        },
        title_body_thresholds,
    )
    if not any("wrong title/body text height" in error for error in title_body_errors):
        failures.append("acceptance title/body scanner must reject crushed title text")
    title_body_errors = acceptance_artifact_title_body_text_errors_from_values(
        {
            "target/acceptance/text-regression-crops/title-body.png:title heading": (
                29,
                418,
                52,
                74,
                2_232,
            ),
            "target/acceptance/text-regression-crops/title-body.png:body first line": (
                28,
                520,
                106,
                119,
                1_700,
            ),
            "target/acceptance/text-regression-crops/title-body.png:body second line": (
                29,
                608,
                129,
                142,
                1_889,
            ),
        },
        title_body_thresholds,
    )
    if not any("wrong title/body text width" in error for error in title_body_errors):
        failures.append("acceptance title/body scanner must reject clipped body text")
    sidebar_selected_row_errors = acceptance_artifact_sidebar_selected_row_errors_from_values(
        {
            "target/kdv-storybook-window-sidebar-smoke.png": (
                40,
                491,
                96,
                117,
                9_324,
            ),
            "target/kdv-storybook-window-sidebar-narrow-smoke.png": (
                40,
                491,
                96,
                117,
                9_324,
            ),
            "target/kdv-storybook-window-sidebar-large-smoke.png": (
                40,
                491,
                96,
                117,
                9_324,
            ),
        },
        {
            str(path.relative_to(ROOT)): (
                min_x,
                max_x,
                min_y,
                max_y,
                min_width,
                max_width,
                min_height,
                max_height,
                min_area,
                max_area,
            )
            for (
                path,
                min_x,
                max_x,
                min_y,
                max_y,
                min_width,
                max_width,
                min_height,
                max_height,
                min_area,
                max_area,
            ) in REQUIRED_ACCEPTANCE_SIDEBAR_SELECTED_ROW_BANDS
        },
    )
    if sidebar_selected_row_errors:
        failures.append("acceptance sidebar selected-row scanner must accept expected geometry")
    sidebar_selected_row_errors = acceptance_artifact_sidebar_selected_row_errors_from_values(
        {
            "target/kdv-storybook-window-sidebar-smoke.png": (
                0,
                491,
                96,
                117,
                9_324,
            ),
        },
        {
            str(path.relative_to(ROOT)): (
                min_x,
                max_x,
                min_y,
                max_y,
                min_width,
                max_width,
                min_height,
                max_height,
                min_area,
                max_area,
            )
            for (
                path,
                min_x,
                max_x,
                min_y,
                max_y,
                min_width,
                max_width,
                min_height,
                max_height,
                min_area,
                max_area,
            ) in REQUIRED_ACCEPTANCE_SIDEBAR_SELECTED_ROW_BANDS
        },
    )
    if not any("wrong sidebar selected-row x position" in error for error in sidebar_selected_row_errors):
        failures.append("acceptance sidebar selected-row scanner must reject activity rail overlap")
    sidebar_selected_row_errors = acceptance_artifact_sidebar_selected_row_errors_from_values(
        {
            "target/kdv-storybook-window-sidebar-smoke.png": (
                40,
                180,
                96,
                117,
                4_000,
            ),
        },
        {
            str(path.relative_to(ROOT)): (
                min_x,
                max_x,
                min_y,
                max_y,
                min_width,
                max_width,
                min_height,
                max_height,
                min_area,
                max_area,
            )
            for (
                path,
                min_x,
                max_x,
                min_y,
                max_y,
                min_width,
                max_width,
                min_height,
                max_height,
                min_area,
                max_area,
            ) in REQUIRED_ACCEPTANCE_SIDEBAR_SELECTED_ROW_BANDS
        },
    )
    if not any("wrong sidebar selected-row width" in error for error in sidebar_selected_row_errors):
        failures.append("acceptance sidebar selected-row scanner must reject clipped row width")
    table_grid_thresholds = {
        f"{path.relative_to(ROOT)}:{name}": (
            min_x,
            max_x,
            min_y,
            max_y,
            min_width,
            max_width,
            min_height,
            max_height,
            min_area,
            max_area,
        )
        for (
            path,
            name,
            min_x,
            max_x,
            min_y,
            max_y,
            min_width,
            max_width,
            min_height,
            max_height,
            min_area,
            max_area,
        ) in REQUIRED_ACCEPTANCE_TABLE_GRID_COMPONENTS
    }
    table_grid_errors = acceptance_artifact_table_grid_errors_from_values(
        {
            "target/acceptance/text-regression-crops/table-section.png:basic table grid": (
                2,
                725,
                100,
                359,
                5_360,
            ),
            "target/acceptance/text-regression-crops/table-section.png:alignment table grid": (
                2,
                725,
                430,
                585,
                3_504,
            ),
            "target/acceptance/text-regression-crops/table-section.png:single-row table grid": (
                2,
                725,
                656,
                759,
                2_374,
            ),
        },
        table_grid_thresholds,
    )
    if table_grid_errors:
        failures.append("acceptance table grid scanner must accept expected grid geometry")
    table_grid_errors = acceptance_artifact_table_grid_errors_from_values(
        {
            "target/acceptance/text-regression-crops/table-section.png:basic table grid": (
                8,
                500,
                100,
                410,
                3_100,
            ),
            "target/acceptance/text-regression-crops/table-section.png:alignment table grid": (
                2,
                725,
                430,
                585,
                3_504,
            ),
            "target/acceptance/text-regression-crops/table-section.png:single-row table grid": (
                2,
                725,
                656,
                759,
                2_374,
            ),
        },
        table_grid_thresholds,
    )
    if not any("wrong table grid width" in error for error in table_grid_errors):
        failures.append("acceptance table grid scanner must reject clipped columns")
    table_grid_errors = acceptance_artifact_table_grid_errors_from_values(
        {
            "target/acceptance/text-regression-crops/table-section.png:basic table grid": (
                2,
                725,
                130,
                359,
                5_360,
            ),
            "target/acceptance/text-regression-crops/table-section.png:alignment table grid": (
                2,
                725,
                430,
                585,
                3_504,
            ),
            "target/acceptance/text-regression-crops/table-section.png:single-row table grid": (
                2,
                725,
                656,
                759,
                2_374,
            ),
        },
        table_grid_thresholds,
    )
    if not any("wrong table grid y position" in error for error in table_grid_errors):
        failures.append("acceptance table grid scanner must reject shifted grid")
    reference_content_errors = acceptance_artifact_reference_content_errors_from_values(
        {
            "target/acceptance/text-regression-crops/reference-comparison/sample-top-reference.png": (
                40.0,
                10_000.0,
            ),
            "target/acceptance/text-regression-crops/reference-comparison/sample-diagrams-reference.png": (
                0.0,
                250.0,
            ),
        },
        {
            "target/acceptance/text-regression-crops/reference-comparison/sample-top-reference.png": (
                100.0,
                0.0,
            ),
            "target/acceptance/text-regression-crops/reference-comparison/sample-diagrams-reference.png": (
                0.0,
                1000.0,
            ),
        },
    )
    if not any("reference crop" in error and "link-blue" in error for error in reference_content_errors):
        failures.append(
            "acceptance reference content scanner must reject missing link-blue pixels"
        )
    if not any("SVG/diagram edge pixels" in error for error in reference_content_errors):
        failures.append(
            "acceptance reference content scanner must reject missing diagram edges"
        )
    edge_ratio_errors = acceptance_artifact_reference_edge_ratio_errors_from_values(
        {"sample diagrams full-resolution SVG edge ratio": (10_000.0, 9_000.0)},
        {"sample diagrams full-resolution SVG edge ratio": 0.95},
    )
    if not any("edge ratio too low" in error for error in edge_ratio_errors):
        failures.append(
            "acceptance reference edge ratio scanner must reject blurry diagram candidate"
        )
    live_theme_errors = acceptance_artifact_live_theme_errors_from_values(
        (10.0, 5.0, 3.0),
        (100_000.0, 100_000.0, 50_000.0),
        "target/acceptance/kdv-storybook-live-interactive.png -> "
        "target/acceptance/kdv-storybook-live-light-toggle.png",
    )
    if not any("did not switch to light theme" in error for error in live_theme_errors):
        failures.append("acceptance live theme scanner must reject inert light toggle")
    live_window_size_errors = acceptance_artifact_live_window_size_errors_from_values(
        {
            "target/acceptance/kdv-storybook-live-light-toggle.png": (4032, 2624),
        },
        {
            "target/acceptance/kdv-storybook-live-light-toggle.png": (2560, 1864),
        },
    )
    if not any("expected headless canvas size" in error for error in live_window_size_errors):
        failures.append(
            "acceptance live canvas size scanner must reject unexpected dimensions"
        )
    live_interactive_content_errors = (
        acceptance_artifact_live_interactive_content_errors_from_values(
            (0.0, 72.0),
            (20_000.0, 128.0),
            "target/acceptance/kdv-storybook-live-interactive.png:"
            "live interactive initial content",
        )
    )
    if not any(
        "bright content pixels" in error for error in live_interactive_content_errors
    ):
        failures.append(
            "acceptance live interactive content scanner must reject blank captures"
        )
    live_light_text_errors = acceptance_artifact_live_light_text_errors_from_values(
        (1_800_000.0, 5_000.0, 0.95),
        (1_500_000.0, 20_000.0, 0.90),
        "target/acceptance/kdv-storybook-live-light-toggle.png:live light viewer content",
    )
    if not any("dark text pixels" in error for error in live_light_text_errors):
        failures.append(
            "acceptance live light text scanner must reject low-contrast light text"
        )
    live_light_background_errors = acceptance_artifact_live_light_text_errors_from_values(
        (10_000.0, 30_000.0, 0.60),
        (1_500_000.0, 20_000.0, 0.90),
        "target/acceptance/kdv-storybook-live-light-toggle.png:live light viewer content",
    )
    if not any(
        "bright background pixels" in error for error in live_light_background_errors
    ):
        failures.append(
            "acceptance live light text scanner must reject dark-theme viewer background"
        )
    log_errors = acceptance_artifact_log_errors_from_text(
        "target/acceptance/kdv-storybook-live-acceptance.log",
        "\n".join(
            [
                "storybook live acceptance headless artifact ready width=1280 height=932 scale=2.0",
                "storybook live acceptance interactive content ready source=headless",
                "storybook live acceptance clicked dark toggle at x=1 y=2",
                "storybook live acceptance theme switch verified: changed_pixels=1",
                "[kdv-storybook] asset job result send failed: sending on a closed channel",
            ]
        ),
    )
    if not any("asset job result send failed" in error for error in log_errors):
        failures.append("acceptance log scanner must reject closed channel noise")
    empty_log_errors = acceptance_artifact_log_errors_from_text(
        "target/acceptance/kdv-storybook-live-acceptance.log",
        "",
    )
    if not any("missing required marker" in error for error in empty_log_errors):
        failures.append("acceptance log scanner must reject empty live logs")
    scroll_performance_errors = acceptance_artifact_scroll_performance_errors_from_text(
        "target/acceptance/kdv-storybook-scroll-performance.txt",
        "\n".join(
            [
                "scenario=large_loaded_diagram_wheel_present",
                "fixture=katana/sample_drawio_performance.md",
                "frame_count=8",
                "budget_ms=130.000",
                "elapsed_ms=140.000",
                "full_preview_redraw_fallback_count=1",
            ]
        ),
    )
    if not any("full preview redraw fallback" in error for error in scroll_performance_errors):
        failures.append(
            "acceptance scroll performance scanner must reject full redraw fallback"
        )
    if not any("exceeded budget" in error for error in scroll_performance_errors):
        failures.append("acceptance scroll performance scanner must reject slow frames")
    crop_changed_errors = acceptance_artifact_changed_pixels_errors_from_values(
        {
            (
                "target/acceptance/text-regression-crops/title-body.png",
                "target/acceptance/text-regression-crops/hover-highlight.png",
            ): 12.0
        },
        {
            (
                "target/acceptance/text-regression-crops/title-body.png",
                "target/acceptance/text-regression-crops/hover-highlight.png",
            ): 250.0
        },
    )
    if not any("changed pixels" in error for error in crop_changed_errors):
        failures.append("acceptance crop change scanner must reject inert hover crop")
    margin_errors = acceptance_artifact_direct_margin_left_errors_from_bands(
        [
            CropBand(30, 300, 24, 44, 900),
            CropBand(29, 145, 70, 82, 300),
            CropBand(108, 282, 113, 126, 520),
            CropBand(149, 322, 156, 170, 620),
        ],
        "target/acceptance/text-regression-crops/direct-html-margin-left.png",
    )
    if margin_errors:
        failures.append("acceptance margin-left scanner must accept expected bands")
    margin_errors = acceptance_artifact_direct_margin_left_errors_from_bands(
        [
            CropBand(30, 300, 24, 44, 900),
            CropBand(29, 145, 70, 82, 300),
            CropBand(55, 220, 113, 126, 520),
            CropBand(149, 322, 156, 170, 620),
        ],
        "target/acceptance/text-regression-crops/direct-html-margin-left.png",
    )
    if not any("80px link" in error for error in margin_errors):
        failures.append("acceptance margin-left scanner must reject wrong link offset")
    table_errors = acceptance_artifact_table_section_errors_from_bands(
        [
            CropBand(10, 158, 10, 28, 687),
            CropBand(9, 145, 63, 77, 589),
            CropBand(25, 540, 120, 128, 350),
            CropBand(25, 558, 170, 183, 457),
            CropBand(25, 598, 222, 235, 563),
            CropBand(25, 576, 274, 287, 521),
            CropBand(25, 564, 326, 337, 418),
            CropBand(3, 228, 393, 409, 981),
            CropBand(19, 530, 450, 461, 284),
            CropBand(18, 522, 503, 510, 210),
            CropBand(19, 538, 554, 565, 386),
            CropBand(3, 192, 619, 635, 831),
        ],
        "target/acceptance/text-regression-crops/table-section.png",
    )
    if table_errors:
        failures.append("acceptance table scanner must accept expected section bands")
    table_errors = acceptance_artifact_table_section_errors_from_bands(
        [
            CropBand(10, 158, 10, 28, 687),
            CropBand(9, 145, 63, 77, 589),
            CropBand(25, 540, 120, 128, 350),
            CropBand(25, 558, 170, 183, 457),
            CropBand(25, 598, 222, 235, 563),
            CropBand(25, 576, 274, 287, 521),
            CropBand(9, 234, 360, 376, 981),
            CropBand(25, 564, 326, 337, 418),
            CropBand(19, 530, 450, 461, 284),
            CropBand(18, 522, 503, 510, 210),
            CropBand(19, 538, 554, 565, 386),
            CropBand(3, 192, 619, 635, 831),
        ],
        "target/acceptance/text-regression-crops/table-section.png",
    )
    if not any("Table with Alignment heading" in error for error in table_errors):
        failures.append("acceptance table scanner must reject heading/table overlap")
    table_errors = acceptance_artifact_table_section_errors_from_bands(
        [
            CropBand(10, 158, 10, 28, 687),
            CropBand(9, 145, 63, 77, 589),
            CropBand(25, 540, 120, 128, 350),
            CropBand(25, 558, 170, 183, 457),
            CropBand(25, 598, 222, 235, 563),
            CropBand(25, 576, 274, 287, 521),
            CropBand(25, 564, 326, 337, 418),
            CropBand(25, 300, 348, 359, 220),
            CropBand(3, 228, 393, 409, 981),
            CropBand(19, 530, 450, 461, 284),
            CropBand(18, 522, 503, 510, 210),
            CropBand(19, 538, 554, 565, 386),
            CropBand(3, 192, 619, 635, 831),
        ],
        "target/acceptance/text-regression-crops/table-section.png",
    )
    if not any("too many table rows" in error for error in table_errors):
        failures.append("acceptance table scanner must reject wrapped table text rows")
    icon_grid_errors = acceptance_artifact_diagram_control_icon_grid_errors_from_values(
        {
            "pan-up": 30.0,
            "zoom-in": 75.0,
            "pan-left": 32.0,
            "reset-view": 139.0,
            "pan-right": 32.0,
            "trackpad-help": 163.0,
            "pan-down": 32.0,
            "zoom-out": 79.0,
        },
        {
            command: (min_pixels, max_pixels)
            for command, _, _, _, _, min_pixels, max_pixels in (
                REQUIRED_ACCEPTANCE_DIAGRAM_CONTROL_ICON_CELLS
            )
        },
        "target/acceptance/text-regression-crops/diagram-control-icons.png",
    )
    if icon_grid_errors:
        failures.append("acceptance diagram icon scanner must accept expected glyph cells")
    icon_grid_errors = acceptance_artifact_diagram_control_icon_grid_errors_from_values(
        {
            "pan-up": 8.0,
            "zoom-in": 75.0,
            "pan-left": 32.0,
            "reset-view": 139.0,
            "pan-right": 32.0,
            "trackpad-help": 163.0,
            "pan-down": 32.0,
            "zoom-out": 79.0,
        },
        {
            command: (min_pixels, max_pixels)
            for command, _, _, _, _, min_pixels, max_pixels in (
                REQUIRED_ACCEPTANCE_DIAGRAM_CONTROL_ICON_CELLS
            )
        },
        "target/acceptance/text-regression-crops/diagram-control-icons.png",
    )
    if not any("too few bright glyph pixels" in error for error in icon_grid_errors):
        failures.append("acceptance diagram icon scanner must reject missing glyphs")
    icon_grid_errors = acceptance_artifact_diagram_control_icon_grid_errors_from_values(
        {
            "pan-up": 30.0,
            "zoom-in": 75.0,
            "pan-left": 32.0,
            "reset-view": 600.0,
            "pan-right": 32.0,
            "trackpad-help": 163.0,
            "pan-down": 32.0,
            "zoom-out": 79.0,
        },
        {
            command: (min_pixels, max_pixels)
            for command, _, _, _, _, min_pixels, max_pixels in (
                REQUIRED_ACCEPTANCE_DIAGRAM_CONTROL_ICON_CELLS
            )
        },
        "target/acceptance/text-regression-crops/diagram-control-icons.png",
    )
    if not any("too many bright glyph pixels" in error for error in icon_grid_errors):
        failures.append("acceptance diagram icon scanner must reject blocky square glyphs")
    control_strip_errors = (
        acceptance_artifact_diagram_control_strip_errors_from_values(
            {
                "target/kdv-storybook-window-diagram-smoke-hover-reset-view.png:Mermaid top-right control strip": {
                    "pan-up": 20.0,
                    "zoom-in": 63.0,
                    "pan-left": 20.0,
                    "reset-view": 86.0,
                    "pan-right": 48.0,
                    "trackpad-help": 127.0,
                    "pan-down": 38.0,
                    "zoom-out": 74.0,
                },
            },
            {
                "target/kdv-storybook-window-diagram-smoke-hover-reset-view.png:Mermaid top-right control strip": REQUIRED_ACCEPTANCE_DIAGRAM_CONTROL_STRIP_CELL_THRESHOLDS,
            },
        )
    )
    if control_strip_errors:
        failures.append(
            "acceptance diagram control strip scanner must accept expected right-edge strip"
        )
    control_strip_errors = (
        acceptance_artifact_diagram_control_strip_errors_from_values(
            {
                "target/kdv-storybook-window-diagram-smoke-hover-reset-view.png:Mermaid top-right control strip": {
                    "pan-up": 0.0,
                    "zoom-in": 0.0,
                    "pan-left": 0.0,
                    "reset-view": 0.0,
                    "pan-right": 0.0,
                    "trackpad-help": 0.0,
                    "pan-down": 0.0,
                    "zoom-out": 0.0,
                },
            },
            {
                "target/kdv-storybook-window-diagram-smoke-hover-reset-view.png:Mermaid top-right control strip": REQUIRED_ACCEPTANCE_DIAGRAM_CONTROL_STRIP_CELL_THRESHOLDS,
            },
        )
    )
    if not any("too few right-edge control pixels" in error for error in control_strip_errors):
        failures.append(
            "acceptance diagram control strip scanner must reject missing right-edge controls"
        )
    if not any("`reset-view`" in error for error in control_strip_errors):
        failures.append(
            "acceptance diagram control strip scanner must reject incomplete controls"
        )
    html_center_errors = acceptance_artifact_html_center_text_errors_from_values(
        {
            "target/acceptance/text-regression-crops/html-margin-center.png:centered HTML heading": (
                311,
                477,
                174,
                193,
                833,
            ),
            "target/acceptance/text-regression-crops/html-margin-center.png:centered HTML paragraph": (
                111,
                669,
                313,
                326,
                1776,
            ),
        },
        {
            "target/acceptance/text-regression-crops/html-margin-center.png:centered HTML heading": (
                380,
                410,
                150,
                190,
                700,
                1_050,
            ),
            "target/acceptance/text-regression-crops/html-margin-center.png:centered HTML paragraph": (
                375,
                435,
                520,
                590,
                1_500,
                2_100,
            ),
        },
    )
    if html_center_errors:
        failures.append("acceptance html center scanner must accept centered text")
    html_center_errors = acceptance_artifact_html_center_text_errors_from_values(
        {
            "target/acceptance/text-regression-crops/html-margin-center.png:centered HTML heading": (
                31,
                197,
                174,
                193,
                833,
            ),
            "target/acceptance/text-regression-crops/html-margin-center.png:centered HTML paragraph": (
                31,
                589,
                313,
                326,
                1776,
            ),
        },
        {
            "target/acceptance/text-regression-crops/html-margin-center.png:centered HTML heading": (
                380,
                410,
                150,
                190,
                700,
                1_050,
            ),
            "target/acceptance/text-regression-crops/html-margin-center.png:centered HTML paragraph": (
                375,
                435,
                520,
                590,
                1_500,
                2_100,
            ),
        },
    )
    if not any("wrong centered text position" in error for error in html_center_errors):
        failures.append("acceptance html center scanner must reject left-aligned HTML")
    aggregated_visual_metric_errors = acceptance_artifact_file_errors_from_parts(
        missing_files=[],
        source_integrity_errors=[],
        manifest_errors=[],
        png_errors=[],
        ppm_errors=[
            "acceptance artifact PPM has wrong dimensions: "
            "target/acceptance/preview-crop-reference/broken.ppm is 640x480, "
            "expected 1280x2400."
        ],
        visual_metric_errors=[
            "acceptance artifact visual metric has too few colors: "
            "target/acceptance/broken.png has 1, expected at least 30."
        ],
        crop_content_errors=[
            "acceptance artifact crop content has too few link-blue pixels: "
            "target/acceptance/text-regression-crops/language-link.png has 20, "
            "expected at least 100."
        ],
        live_theme_errors=[
            "acceptance live artifact did not switch to light theme: "
            "changed_pixels=10 bright_delta=5 dark_delta=3."
        ],
        log_errors=[
            "acceptance log contains forbidden runtime message: "
            "target/acceptance/kdv-storybook-live-acceptance.log matched "
            "`asset job result send failed`."
        ],
        scroll_performance_errors=[
            "acceptance scroll performance exceeded budget: "
            "target/acceptance/kdv-storybook-scroll-performance.txt elapsed_ms=140.000 "
            "budget_ms=130.000."
        ],
        source_freshness_errors=[
            "acceptance artifact is older than required source file(s): "
            "newest source scripts/release/assert-viewer-recovery-dod.py; "
            "stale artifact(s) target/acceptance/kdv-storybook-acceptance-contact-sheet.png."
        ],
        freshness_errors=[],
    )
    if not any("visual metric" in error for error in aggregated_visual_metric_errors):
        failures.append(
            "accepted artifact file scanner must include visual metric errors"
        )
    if not any("crop content" in error for error in aggregated_visual_metric_errors):
        failures.append(
            "accepted artifact file scanner must include crop content errors"
        )
    if not any("live artifact" in error for error in aggregated_visual_metric_errors):
        failures.append("accepted artifact file scanner must include live theme errors")
    if not any("PPM" in error for error in aggregated_visual_metric_errors):
        failures.append("accepted artifact file scanner must include PPM errors")
    if not any("forbidden runtime message" in error for error in aggregated_visual_metric_errors):
        failures.append("accepted artifact file scanner must include log errors")
    if not any("scroll performance" in error for error in aggregated_visual_metric_errors):
        failures.append(
            "accepted artifact file scanner must include scroll performance errors"
        )
    if not any("older than required source file" in error for error in aggregated_visual_metric_errors):
        failures.append(
            "accepted artifact file scanner must include source freshness errors"
        )
    for name, document, should_error in cases:
        has_error = bool(storybook_acceptance_errors(document, verify_artifact_files=False))
        if has_error != should_error:
            failures.append(
                f"{name}: expected error={should_error}, got error={has_error}"
            )
    current_acceptance_errors = storybook_acceptance_errors(
        current_acceptance_document_as_accepted_for_contract_check(),
        verify_artifact_files=False,
    )
    if current_acceptance_errors:
        failures.append(
            "current storybook-user-acceptance.md must be internally consistent when "
            "mechanically marked accepted: "
            + "; ".join(current_acceptance_errors[:5])
        )
    stale_native_fullscreen_ledger = (
        "- [/] 現行対応: Storybook host は fullscreen event を "
        "live OS window に反映するだけ。\n"
    )
    if not native_fullscreen_ledger_contradiction_errors_from_markdown(
        "self-test.md", stale_native_fullscreen_ledger
    ):
        failures.append(
            "native fullscreen ledger scanner must reject current OS-window sync claims"
        )
    historical_native_fullscreen_ledger = (
        "- [/] 2026-06-22 historical / superseded: fullscreen event を "
        "native fullscreen host へ同期した案は撤回済み。\n"
    )
    if native_fullscreen_ledger_contradiction_errors_from_markdown(
        "self-test.md", historical_native_fullscreen_ledger
    ):
        failures.append(
            "native fullscreen ledger scanner must allow historical superseded notes"
        )
    current_native_fullscreen_ledger_errors = native_fullscreen_ledger_contradiction_errors()
    if current_native_fullscreen_ledger_errors:
        failures.append(
            "current ledger must not claim native fullscreen sync as current behavior: "
            + "; ".join(current_native_fullscreen_ledger_errors[:5])
        )
    headless_current_section = """# Storybook User Acceptance

status: pending

Accepted release の更新条件:

- `/opt/homebrew/bin/rtk just storybook-release-acceptance-artifacts` を再実行し、静的 screenshot review artifact と headless live-acceptance artifact を一括で最新化する。
- `/opt/homebrew/bin/rtk just storybook-live-acceptance-artifact` を再実行し、KUC 実 UI tree 由来の Dark toggle typed action、dark/light screenshot、checksum manifest を headless に最新化する。

## Acceptance Procedure

1. live acceptance artifact は KUC 実 UI tree 由来の Dark toggle hit/action/state/rerender を headless に検証する補助証跡であり、human acceptance の代替ではない。

## Evidence

- 2026-06-20 historical: live OS artifact used CoreGraphics and screencapture.
"""
    if headless_live_acceptance_contract_errors(headless_current_section):
        failures.append(
            "headless live acceptance scanner must allow historical live OS notes "
            "outside the current acceptance section"
        )
    stale_headless_current_section = headless_current_section.replace(
        "headless live-acceptance artifact",
        "live OS artifact",
        1,
    )
    if not headless_live_acceptance_contract_errors(stale_headless_current_section):
        failures.append(
            "headless live acceptance scanner must reject current live OS artifact wording"
        )
    current_headless_live_acceptance_errors = headless_live_acceptance_contract_errors(
        STORYBOOK_USER_ACCEPTANCE.read_text(encoding="utf-8")
        if STORYBOOK_USER_ACCEPTANCE.exists()
        else ""
    )
    if current_headless_live_acceptance_errors:
        failures.append(
            "current storybook-user-acceptance.md must describe headless live "
            "acceptance as the current automated aid: "
            + "; ".join(current_headless_live_acceptance_errors[:5])
        )
    valid_ci_runtime = """
env:
  JAVA_TOOL_OPTIONS: -Xss16m -Djava.awt.headless=true -Djdk.lang.processReaperUseDefaultStackSize=true
steps:
  - name: Install Graphviz (Ubuntu)
    run: |
      sudo apt-get install -y graphviz
      sudo ln -sf /usr/bin/dot /opt/local/bin/dot
      echo "GRAPHVIZ_DOT=/opt/local/bin/dot" >> "$GITHUB_ENV"
  - name: Install Graphviz (macOS)
    run: |
      brew install graphviz
      echo "GRAPHVIZ_DOT=/opt/local/bin/dot" >> "$GITHUB_ENV"
  - name: Install Graphviz (Windows)
    run: choco install graphviz --no-progress -y
  - name: Run tests
    run: |
      cargo test --workspace --locked --exclude kdv-storybook
      cargo test -p kdv-storybook --locked -- --test-threads=1 --skip katana_intro_text_keeps_readable_frame_band_heights --skip katana_language_link_underline_reaches_frame_pixels --skip direct_html_margin_left_fixture_reaches_frame_pixels --skip storybook_score_visual_ --skip storybook_sample_ --skip storybook_preview_crop_score_ --skip storybook_score_export_surface_excludes_overlay_controls --skip mouse_click_uses_external_scroll_for_scroll_independent_scene
      cargo test -p kdv-storybook --locked mouse_click_uses_external_scroll_for_scroll_independent_scene -- --test-threads=1
"""
    valid_preflight_runtime = """
env:
  JAVA_TOOL_OPTIONS: -Xss16m -Djava.awt.headless=true -Djdk.lang.processReaperUseDefaultStackSize=true
steps:
  - name: Install acceptance artifact dependencies
    run: |
      sudo apt-get install -y graphviz imagemagick xvfb xclip
      sudo ln -sf /usr/bin/dot /opt/local/bin/dot
      if ! command -v magick >/dev/null 2>&1; then
        sudo tee /usr/local/bin/magick >/dev/null
        exec convert "$@"
      fi
      echo "GRAPHVIZ_DOT=/opt/local/bin/dot" >> "$GITHUB_ENV"
  - name: Release check
    run: |
      xvfb-run -a just storybook-release-acceptance-artifacts
      xvfb-run -a just VERSION="${VERSION}" release-check
"""
    if plantuml_ci_runtime_errors(valid_ci_runtime, valid_preflight_runtime):
        failures.append("PlantUML CI runtime scanner must allow complete workflow pins")
    if not plantuml_ci_runtime_errors(
        valid_ci_runtime.replace("-Xss16m ", ""), valid_preflight_runtime
    ):
        failures.append("PlantUML CI runtime scanner must reject missing JVM stack pin")
    if not plantuml_ci_runtime_errors(
        valid_ci_runtime.replace(
            " -Djdk.lang.processReaperUseDefaultStackSize=true", ""
        ),
        valid_preflight_runtime,
    ):
        failures.append(
            "PlantUML CI runtime scanner must reject missing process reaper stack pin"
        )
    if not plantuml_ci_runtime_errors(
        valid_ci_runtime, valid_preflight_runtime.replace("GRAPHVIZ_DOT", "GRAPHVIZDOT")
    ):
        failures.append("PlantUML CI runtime scanner must reject missing preflight Graphviz env")
    if not plantuml_ci_runtime_errors(
        valid_ci_runtime,
        valid_preflight_runtime.replace("/usr/local/bin/magick", "/usr/local/bin/not-magick"),
    ):
        failures.append(
            "PlantUML CI runtime scanner must reject missing ImageMagick magick shim"
        )
    valid_release_runtime = """
env:
  JAVA_TOOL_OPTIONS: -Xss16m -Djava.awt.headless=true -Djdk.lang.processReaperUseDefaultStackSize=true
steps:
  - name: Install acceptance artifact dependencies
    run: |
      sudo apt-get install -y graphviz imagemagick xvfb xclip
      sudo ln -sf /usr/bin/dot /opt/local/bin/dot
      if ! command -v magick >/dev/null 2>&1; then
        sudo tee /usr/local/bin/magick >/dev/null
        exec convert "$@"
      fi
      echo "GRAPHVIZ_DOT=/opt/local/bin/dot" >> "$GITHUB_ENV"
  - name: Release verify
    run: |
      xvfb-run -a just storybook-release-acceptance-artifacts
      xvfb-run -a just VERSION="${VERSION}" release-verify
"""
    if release_workflow_acceptance_runtime_errors(valid_release_runtime):
        failures.append("Release workflow runtime scanner must allow complete artifact runtime")
    if not release_workflow_acceptance_runtime_errors(
        valid_release_runtime.replace("storybook-release-acceptance-artifacts", "storybook")
    ):
        failures.append("Release workflow runtime scanner must reject missing artifact generation")
    valid_just_test = """
test:
    {{CARGO}} test --workspace --all-targets --all-features --locked --exclude kdv-storybook
    {{CARGO}} test -p kdv-storybook --locked -- --test-threads=1 --skip katana_intro_text_keeps_readable_frame_band_heights --skip storybook_score_visual_ --skip mouse_click_uses_external_scroll_for_scroll_independent_scene
    {{CARGO}} test -p kdv-storybook --locked mouse_click_uses_external_scroll_for_scroll_independent_scene -- --test-threads=1
"""
    if release_test_entrypoint_isolation_errors(valid_just_test):
        failures.append("release test entrypoint scanner must allow isolated kdv-storybook tests")
    stale_just_test = """
test:
    {{CARGO}} test --workspace --all-targets --all-features --locked
"""
    if not release_test_entrypoint_isolation_errors(stale_just_test):
        failures.append(
            "release test entrypoint scanner must reject generic kdv-storybook workspace tests"
        )
    if failures:
        print("release DoD self-test failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1
    print("release DoD self-test: ok")
    return 0


def open_checklist_items(markdown: str) -> list[str]:
    return OPEN_CHECKLIST_RE.findall(markdown)


def pending_acceptance_required_open_feedback_errors(
    acceptance: str, feedback: str
) -> list[str]:
    if acceptance_status(acceptance) == "accepted":
        return []
    open_ids = open_feedback_item_ids(feedback)
    missing_ids = [
        feedback_id
        for feedback_id in PENDING_ACCEPTANCE_REQUIRED_OPEN_FEEDBACK_IDS
        if feedback_id not in open_ids
    ]
    if not missing_ids:
        return []
    return [
        "pending storybook acceptance must keep these user-feedback items open: "
        + ", ".join(missing_ids)
        + "."
    ]


def kuc_blocker_ledger_errors(
    feedback: str,
    remaining: str,
    design: str,
    kuc_context_menu_adr: str,
    kuc_interaction_target_adr: str,
) -> list[str]:
    errors: list[str] = []
    open_ids = open_feedback_item_ids(feedback)
    open_remaining = open_checklist_items(remaining)
    context_menu_blocker_open = any("追補23" in item for item in open_remaining)
    interaction_blocker_open = any("追補26" in item for item in open_remaining)

    if context_menu_blocker_open and "UF-043" not in open_ids:
        errors.append(
            "KUC #7 ContextMenu blocker is open, so UF-043 must remain open."
        )
    for label, text in (
        ("remaining-plan.md", remaining),
        ("user-feedback-todo.md", feedback),
        ("ContextMenu ADR", kuc_context_menu_adr),
    ):
        if KUC_ISSUE_7_URL not in text:
            errors.append(f"{label} must reference KUC issue #7: {KUC_ISSUE_7_URL}.")
    if interaction_blocker_open:
        for label, text in (
            ("remaining-plan.md", remaining),
            ("user-feedback-todo.md", feedback),
            ("design.md", design),
            ("InteractionTarget ADR", kuc_interaction_target_adr),
        ):
            if KUC_ISSUE_8_URL not in text:
                errors.append(
                    f"{label} must reference KUC issue #8: {KUC_ISSUE_8_URL}."
                )
        for label, text in (
            ("design.md", design),
            ("InteractionTarget ADR", kuc_interaction_target_adr),
        ):
            missing_tokens = [
                token
                for token in KUC_INTERACTION_TARGET_REQUIRED_TOKENS
                if token not in text
            ]
            if missing_tokens:
                errors.append(
                    f"{label} must keep KUC #8 interaction target scope tokens: "
                    + ", ".join(missing_tokens)
                    + "."
                )

    return errors


def kuc_interaction_target_completion_errors(remaining: str) -> list[str]:
    open_remaining = open_checklist_items(remaining)
    if any("追補26" in item for item in open_remaining):
        return []
    if "追補26" not in remaining:
        return [
            "remaining-plan.md must keep 追補26 open or document KUC #8 completion evidence."
        ]
    missing_tokens = [
        token
        for token in KUC_INTERACTION_TARGET_COMPLETION_REQUIRED_TOKENS
        if token not in remaining
    ]
    if not missing_tokens:
        return []
    return [
        "closed 追補26 must include KUC #8 completion evidence tokens: "
        + ", ".join(missing_tokens)
        + "."
    ]


def kuc_interaction_target_dependency_errors(
    remaining: str,
    cargo_toml: str,
    cargo_lock: str,
    mouse_host_action_rs: str,
) -> list[str]:
    open_remaining = open_checklist_items(remaining)
    if any("追補26" in item for item in open_remaining):
        return []
    if "追補26" not in remaining:
        return []

    errors: list[str] = []
    stale_dependency = (
        f'tag = "{KUC_INTERACTION_TARGET_STALE_CARGO_TAG}"'
    )
    if stale_dependency in cargo_toml:
        errors.append(
            "closed 追補26 must update KUC Cargo dependency beyond "
            f"{KUC_INTERACTION_TARGET_STALE_CARGO_TAG}; Cargo.toml still "
            f"contains {stale_dependency}."
        )
    if KUC_INTERACTION_TARGET_STALE_CARGO_LOCK_SOURCE in cargo_lock:
        errors.append(
            "closed 追補26 must update Cargo.lock to a KUC revision that "
            "contains UiTreeInteractionTarget."
        )
    missing_tokens = [
        token
        for token in KUC_INTERACTION_TARGET_KDV_REQUIRED_TOKENS
        if token not in mouse_host_action_rs
    ]
    if missing_tokens:
        errors.append(
            "closed 追補26 must make KDV consume KUC interaction target API in "
            "tools/kdv-storybook/src/mouse_host_action.rs: "
            + ", ".join(missing_tokens)
            + "."
        )
    return errors


def document_viewer_harness_ownership_errors(
    remaining: str,
    design: str,
    kuc_document_viewer_harness_adr: str,
    handoff_text: str,
) -> list[str]:
    errors: list[str] = []
    if "追補30" not in remaining:
        errors.append("remaining-plan.md must document the 追補30 ownership decision.")
    for token in KUC_DOCUMENT_VIEWER_OWNERSHIP_REQUIRED_TOKENS:
        if token not in kuc_document_viewer_harness_adr:
            errors.append(
                "DocumentViewer harness ADR must keep ownership token: "
                f"{token}."
            )
    if KDV_DOCUMENT_VIEWER_REQUIRED_COMMAND not in design:
        errors.append(
            "design.md must name the KDV document_viewer regression command: "
            f"{KDV_DOCUMENT_VIEWER_REQUIRED_COMMAND}."
        )
    if KDV_DOCUMENT_VIEWER_REQUIRED_COMMAND not in remaining:
        errors.append(
            "remaining-plan.md must name the KDV document_viewer regression command: "
            f"{KDV_DOCUMENT_VIEWER_REQUIRED_COMMAND}."
        )
    if KDV_DOCUMENT_VIEWER_REQUIRED_COMMAND not in handoff_text:
        errors.append(
            "handoff instructions must include the KDV document_viewer regression command: "
            f"{KDV_DOCUMENT_VIEWER_REQUIRED_COMMAND}."
        )
    if KUC_DOCUMENT_VIEWER_REJECTED_COMMAND in handoff_text:
        errors.append(
            "handoff instructions must not require the KUC Storybook document_viewer "
            f"filter command because it is not release proof: "
            f"{KUC_DOCUMENT_VIEWER_REJECTED_COMMAND}."
        )
    return errors


def kuc_cargo_dependency_errors(
    cargo_toml: str,
    cargo_lock: str,
    storybook_main_rs: str,
) -> list[str]:
    errors: list[str] = []
    if re.search(r"^\s*katana-ui-core-storybook\s*=", cargo_toml, re.MULTILINE):
        errors.append(
            "Cargo.toml must resolve KUC through one katana-ui-core registry dependency."
        )
    for marker in KUC_CARGO_FORBIDDEN_SIBLING_MARKERS:
        if marker in cargo_toml:
            errors.append(
                "Cargo.toml must not use sibling katana-ui-core path dependency: "
                f"{marker}."
            )
        if marker in storybook_main_rs:
            errors.append(
                "kdv-storybook main.rs must not include sibling KUC source by path: "
                f"{marker}."
            )
    for name in KUC_CARGO_DEPENDENCY_NAMES:
        dependency_lines = [
            line
            for line in cargo_toml.splitlines()
            if re.match(rf"^\s*{re.escape(name)}\s*=", line)
        ]
        if not dependency_lines:
            errors.append(
                "Cargo.toml must declare the KUC registry dependency: "
                f"{name} v{KUC_REGISTRY_VERSION}."
            )
        for line in dependency_lines:
            if re.search(r"\b(?:git|path)\s*=", line):
                errors.append(
                    f"Cargo.toml must not override registry {name} with git/path: {line.strip()}."
                )
            if not re.search(
                rf"version\s*=\s*\"=?{re.escape(KUC_REGISTRY_VERSION)}\"",
                line,
            ):
                errors.append(
                    f"Cargo.toml must pin {name} to registry v{KUC_REGISTRY_VERSION}."
                )
            if "raster-host" not in line:
                errors.append(
                    f"Cargo.toml must enable KUC {name}'s raster-host feature."
                )
        expected_lock_block = (
            "[[package]]\n"
            f'name = "{name}"\n'
            f'version = "{KUC_REGISTRY_VERSION}"\n'
            f'source = "{KUC_REGISTRY_SOURCE}"'
        )
        if expected_lock_block not in cargo_lock:
            errors.append(
                "Cargo.lock must pin KUC to the exact registry artifact for "
                f"{name} v{KUC_REGISTRY_VERSION}."
            )
        if re.search(
            rf'(?ms)^name = "{re.escape(name)}"\nversion = "[^"]+"\nsource = "(?:git\+|path\+)',
            cargo_lock,
        ):
            errors.append(f"Cargo.lock must not resolve {name} from git/path.")
    if "katana-ui-core-storybook" in cargo_lock:
        errors.append(
            "Cargo.lock must not contain the obsolete katana-ui-core-storybook package."
        )
    return errors


def open_feedback_item_ids(feedback: str) -> set[str]:
    ids: set[str] = set()
    for item in open_checklist_items(feedback):
        match = re.match(r"^- \[ \] (?P<id>UF-\d+):", item)
        if match:
            ids.add(match.group("id"))
    return ids


def handoff_feedback_ledger_text() -> str:
    chunks: list[str] = []
    for path in HANDOFF_FEEDBACK_LEDGER_FILES:
        if path.exists():
            chunks.append(path.read_text(encoding="utf-8"))
    return "\n".join(chunks)


def handoff_canonical_feedback_path_errors() -> list[str]:
    errors: list[str] = []
    for path in HANDOFF_FEEDBACK_LEDGER_FILES:
        label = path.relative_to(ROOT).as_posix()
        if not path.exists():
            errors.append(f"{label} is missing.")
            continue
        errors.extend(
            handoff_canonical_feedback_path_errors_from_markdown(
                label,
                path.read_text(encoding="utf-8"),
            )
        )
    return errors


def handoff_canonical_feedback_path_errors_from_markdown(
    label: str, markdown: str
) -> list[str]:
    errors: list[str] = []
    has_canonical_feedback_path = (
        CANONICAL_FEEDBACK_LEDGER_RELATIVE_PATH in markdown
        or CANONICAL_FEEDBACK_LEDGER_PATH in markdown
    )
    if not has_canonical_feedback_path:
        errors.append(
            f"{label} must reference canonical feedback ledger "
            f"{CANONICAL_FEEDBACK_LEDGER_RELATIVE_PATH}."
        )
    stale_root_feedback_paths = [
        match.group("path")
        for match in re.finditer(
            r"(?P<path>(?:[A-Za-z0-9_./~:-]+/)?user-feedback-todo\.md)",
            markdown,
        )
        if CANONICAL_FEEDBACK_LEDGER_RELATIVE_PATH not in match.group("path")
    ]
    if STALE_ROOT_FEEDBACK_LEDGER_PATH in markdown or stale_root_feedback_paths:
        errors.append(
            f"{label} must not reference stale root feedback ledger "
            f"{STALE_ROOT_FEEDBACK_LEDGER_PATH}."
        )
    return errors


def print_open_checklist_items(label: str, items: list[str]) -> None:
    print(f"{label}: {len(items)}", file=sys.stderr)
    for item in items:
        print(f"- {item.removeprefix('- ')}", file=sys.stderr)


def native_fullscreen_ledger_contradiction_errors() -> list[str]:
    errors: list[str] = []
    for path in NATIVE_FULLSCREEN_LEDGER_FILES:
        if not path.exists():
            continue
        errors.extend(
            native_fullscreen_ledger_contradiction_errors_from_markdown(
                path.relative_to(ROOT).as_posix(), path.read_text(encoding="utf-8")
            )
        )
    return errors


def native_fullscreen_ledger_contradiction_errors_from_markdown(
    label: str, markdown: str
) -> list[str]:
    errors: list[str] = []
    for index, line in enumerate(markdown.splitlines(), start=1):
        if native_fullscreen_ledger_line_is_allowed(line):
            continue
        for pattern in NATIVE_FULLSCREEN_STALE_LEDGER_PATTERNS:
            if pattern.search(line):
                errors.append(
                    f"{label}:{index} claims diagram fullscreen uses native OS fullscreen: "
                    f"{line.strip()}"
                )
                break
    return errors


def native_fullscreen_ledger_line_is_allowed(line: str) -> bool:
    return any(marker in line for marker in NATIVE_FULLSCREEN_LEDGER_ALLOW_MARKERS)


def headless_live_acceptance_contract_errors(acceptance: str) -> list[str]:
    current_section = acceptance.split("## Evidence", 1)[0]
    errors = [
        f"storybook-user-acceptance.md current section missing `{token}`."
        for token in HEADLESS_LIVE_ACCEPTANCE_CURRENT_REQUIRED_TOKENS
        if token not in current_section
    ]
    for pattern in HEADLESS_LIVE_ACCEPTANCE_CURRENT_STALE_PATTERNS:
        if pattern.search(current_section):
            errors.append(
                "storybook-user-acceptance.md current section must describe "
                f"headless live-acceptance, not `{pattern.pattern}`."
            )
    return errors


def acceptance_document(status: str, checked: bool, evidence: bool) -> str:
    marker = "x" if checked else " "
    checklist = "\n".join(
        f"- [{marker}] {required}" for required in REQUIRED_ACCEPTANCE_CHECKS
    )
    confirmed_by = "hiroyuki_furuno" if evidence else ""
    confirmed_at = "2026-06-18T00:00:00+09:00" if evidence else ""
    matrix = "\n".join(
        f"| {required} | {acceptance_matrix_evidence_text(required)} | human confirmed |"
        for required in REQUIRED_ACCEPTANCE_CHECKS
    )
    artifact_notes = "\n".join(
        f"- artifact: {required}" for required in REQUIRED_ACCEPTANCE_ARTIFACTS
    )
    human_acceptance_note = (
        "- human acceptance: hiroyuki_furuno confirmed KatanA parity by operating "
        "just storybook as an interactive viewer on the real machine."
        if evidence
        else ""
    )
    return f"""# Storybook User Acceptance

status: {status}

## Acceptance Checklist

{checklist}

## Evidence

- confirmed_by: {confirmed_by}
- confirmed_at: {confirmed_at}
{human_acceptance_note}
{artifact_notes}

## Automated Evidence Matrix

| Acceptance item | Automated evidence | Human acceptance status |
| --- | --- | --- |
{matrix}
"""


def current_acceptance_document_as_accepted_for_contract_check() -> str:
    acceptance = STORYBOOK_USER_ACCEPTANCE.read_text()
    acceptance = re.sub(
        r"^\s*status\s*:\s*[a-zA-Z_-]+\s*$",
        "status: accepted",
        acceptance,
        count=1,
        flags=re.MULTILINE,
    )
    acceptance = acceptance.replace("- [ ] ", "- [x] ")
    acceptance = re.sub(
        r"^- confirmed_by:.*$",
        "- confirmed_by: hiroyuki_furuno",
        acceptance,
        count=1,
        flags=re.MULTILINE,
    )
    acceptance = re.sub(
        r"^- confirmed_at:.*$",
        "- confirmed_at: 2026-06-18T00:00:00+09:00",
        acceptance,
        count=1,
        flags=re.MULTILINE,
    )
    if not any(
        HUMAN_ACCEPTANCE_NOTE_RE.match(line)
        for line in section_text(acceptance, "## Evidence").splitlines()
    ):
        acceptance = acceptance.replace(
            "- confirmed_at: 2026-06-18T00:00:00+09:00",
            "- confirmed_at: 2026-06-18T00:00:00+09:00\n"
            "- human acceptance: hiroyuki_furuno confirmed KatanA parity "
            "by operating just storybook as an interactive viewer on the real machine.",
            1,
        )
    for pending_status in (
        "pending user real-machine confirmation",
        "pending user visual confirmation",
        "pending user interaction confirmation",
        "pending user performance confirmation",
        "pending user acceptance confirmation",
    ):
        acceptance = acceptance.replace(pending_status, "confirmed by hiroyuki_furuno")
    return acceptance


def acceptance_artifact_manifest_text() -> str:
    return "\n".join(
        f"{'0' * 64}  {path.relative_to(ROOT)}"
        for path in required_acceptance_manifest_paths()
        if path.name
        not in {
            "kdv-storybook-acceptance-artifacts.sha256",
            "kdv-storybook-live-acceptance-artifacts.sha256",
        }
    )


def acceptance_status(acceptance: str) -> str:
    for line in acceptance.splitlines():
        match = ACCEPTANCE_STATUS_RE.match(line)
        if match:
            return match.group("status")
    return ""


def storybook_acceptance_errors(
    acceptance: str, *, verify_artifact_files: bool = True
) -> list[str]:
    if acceptance_status(acceptance) != "accepted":
        return [
            f"{STORYBOOK_USER_ACCEPTANCE.relative_to(ROOT)} must contain "
            "`status: accepted` after user confirms `just storybook`."
        ]
    errors: list[str] = []
    checklist_items = acceptance_checklist_items(acceptance)
    if not checklist_items:
        errors.append("acceptance checklist has no checkable item.")
    incomplete_count = sum(1 for status, _ in checklist_items if status.lower() != "x")
    if incomplete_count:
        errors.append(f"acceptance checklist still has {incomplete_count} incomplete item(s).")
    missing_checks = missing_acceptance_checks(checklist_items)
    if missing_checks:
        errors.append(f"acceptance checklist missing required item(s): {', '.join(missing_checks)}.")
    evidence = acceptance_evidence(acceptance)
    missing_fields = [
        field for field in REQUIRED_EVIDENCE_FIELDS if not evidence.get(field, "").strip()
    ]
    if missing_fields:
        errors.append(f"acceptance evidence missing: {', '.join(missing_fields)}.")
    confirmed_by = evidence.get("confirmed_by", "").strip()
    if confirmed_by.lower() in CONFIRMED_BY_PLACEHOLDERS:
        errors.append("acceptance evidence confirmed_by must name the human reviewer.")
    confirmed_at = evidence.get("confirmed_at", "").strip()
    if confirmed_at and not CONFIRMED_AT_RE.fullmatch(confirmed_at):
        errors.append(
            "acceptance evidence confirmed_at must be an ISO-8601 timestamp with timezone."
        )
    future_error = confirmed_at_future_error(confirmed_at)
    if future_error:
        errors.append(future_error)
    human_note_errors = human_acceptance_note_errors(acceptance)
    if human_note_errors:
        errors.extend(human_note_errors)
    missing_matrix = missing_acceptance_matrix_rows(acceptance)
    if missing_matrix:
        errors.append(
            "acceptance evidence matrix missing required row(s): "
            + ", ".join(missing_matrix)
            + "."
        )
    pending_matrix = pending_acceptance_matrix_rows(acceptance)
    if pending_matrix:
        errors.append(
            "acceptance evidence matrix still has pending human confirmation row(s): "
            + ", ".join(pending_matrix)
            + "."
        )
    weak_matrix = weak_acceptance_matrix_rows(acceptance)
    if weak_matrix:
        errors.append(
            "acceptance evidence matrix row(s) must include accepted or confirmed human status: "
            + ", ".join(weak_matrix)
            + "."
        )
    weak_evidence_rows = weak_acceptance_matrix_evidence_rows(acceptance)
    if weak_evidence_rows:
        errors.append(
            "acceptance evidence matrix row(s) must include required automated evidence token(s): "
            + ", ".join(weak_evidence_rows)
            + "."
        )
    missing_artifacts = missing_acceptance_artifacts(acceptance)
    if missing_artifacts:
        errors.append(
            "acceptance evidence missing required artifact(s): "
            + ", ".join(missing_artifacts)
            + "."
        )
    if verify_artifact_files:
        errors.extend(acceptance_artifact_file_errors(confirmed_at))
    return errors


def acceptance_evidence(acceptance: str) -> dict[str, str]:
    evidence: dict[str, str] = {}
    for line in acceptance.splitlines():
        match = EVIDENCE_FIELD_RE.match(line)
        if match:
            evidence[match.group("field")] = match.group("value")
    return evidence


def human_acceptance_note_errors(acceptance: str) -> list[str]:
    evidence_section = section_text(acceptance, "## Evidence")
    notes = [
        match.group("note").strip()
        for line in evidence_section.splitlines()
        if (match := HUMAN_ACCEPTANCE_NOTE_RE.match(line))
    ]
    if not notes:
        return [
            "acceptance evidence missing human acceptance note; add "
            "`- human acceptance: ...` describing the real-machine `just storybook` check."
        ]
    valid_notes = [
        note
        for note in notes
        if human_acceptance_note_is_valid(note)
    ]
    if valid_notes:
        return []
    return [
        "acceptance evidence human acceptance note must mention "
        "`just storybook`, `KatanA`, and `interactive`, and must not be an automated/headless-only note."
    ]


def human_acceptance_note_is_valid(note: str) -> bool:
    lowered = note.lower()
    return all(
        token.lower() in lowered for token in HUMAN_ACCEPTANCE_NOTE_REQUIRED_TOKENS
    ) and not any(marker.lower() in lowered for marker in HUMAN_ACCEPTANCE_NOTE_FORBIDDEN_MARKERS)


def acceptance_checklist_items(acceptance: str) -> list[tuple[str, str]]:
    checklist = section_text(acceptance, "## Acceptance Checklist")
    return [
        (match.group("status"), match.group("label"))
        for match in CHECKLIST_ITEM_RE.finditer(checklist)
    ]


def missing_acceptance_checks(checklist_items: list[tuple[str, str]]) -> list[str]:
    labels = [label for _, label in checklist_items]
    return [
        required
        for required in REQUIRED_ACCEPTANCE_CHECKS
        if not any(label_starts_with_required_check(label, required) for label in labels)
    ]


def label_starts_with_required_check(label: str, required: str) -> bool:
    normalized_label = label.strip().lower().strip("`")
    normalized_required = required.strip().lower().strip("`")
    return normalized_label.startswith(normalized_required)


def missing_acceptance_matrix_rows(acceptance: str) -> list[str]:
    item_cells = acceptance_matrix_item_cells(acceptance)
    return [
        required
        for required in REQUIRED_ACCEPTANCE_CHECKS
        if not any(required.lower() in item for item in item_cells)
    ]


def weak_acceptance_matrix_evidence_rows(acceptance: str) -> list[str]:
    rows = acceptance_matrix_rows_by_item(acceptance)
    weak_rows: list[str] = []
    for required, required_tokens in REQUIRED_ACCEPTANCE_MATRIX_EVIDENCE_TOKENS:
        row = next(
            (
                cells
                for item, cells in rows.items()
                if label_starts_with_required_check(item, required)
            ),
            None,
        )
        if row is None or len(row) < 2:
            continue
        evidence = row[1].lower()
        missing_tokens = [
            token for token in required_tokens if token.lower() not in evidence
        ]
        if missing_tokens:
            weak_rows.append(f"{required} missing {', '.join(missing_tokens)}")
    return weak_rows


def acceptance_matrix_rows_by_item(acceptance: str) -> dict[str, list[str]]:
    matrix = section_text(acceptance, REQUIRED_MATRIX_HEADING)
    rows: dict[str, list[str]] = {}
    for line in matrix.splitlines():
        cells = acceptance_matrix_cells(line)
        if cells:
            rows[cells[0]] = cells
    return rows


def acceptance_matrix_item_cells(acceptance: str) -> list[str]:
    matrix = section_text(acceptance, REQUIRED_MATRIX_HEADING)
    return [
        cells[0]
        for line in matrix.splitlines()
        if (cells := acceptance_matrix_cells(line))
    ]


def acceptance_matrix_evidence_text(required: str) -> str:
    for check, tokens in REQUIRED_ACCEPTANCE_MATRIX_EVIDENCE_TOKENS:
        if check == required:
            return "; ".join(tokens)
    return "automated gate"


def pending_acceptance_matrix_rows(acceptance: str) -> list[str]:
    matrix = section_text(acceptance, REQUIRED_MATRIX_HEADING)
    pending_rows: list[str] = []
    for line in matrix.splitlines():
        cells = acceptance_matrix_cells(line)
        if len(cells) >= 3 and "pending" in cells[2]:
            pending_rows.append(cells[0])
    return pending_rows


def weak_acceptance_matrix_rows(acceptance: str) -> list[str]:
    matrix = section_text(acceptance, REQUIRED_MATRIX_HEADING)
    weak_rows: list[str] = []
    for line in matrix.splitlines():
        cells = acceptance_matrix_cells(line)
        if len(cells) >= 3 and not (
            "accepted" in cells[2] or "confirmed" in cells[2]
        ):
            weak_rows.append(cells[0])
    return weak_rows


def acceptance_matrix_cells(line: str) -> list[str]:
    if not line.startswith("|") or line.startswith("| ---"):
        return []
    cells = [cell.strip().lower() for cell in line.strip("|").split("|")]
    if cells and cells[0] == "acceptance item":
        return []
    return cells


def missing_acceptance_artifacts(acceptance: str) -> list[str]:
    return [
        required
        for required in REQUIRED_ACCEPTANCE_ARTIFACTS
        if required not in acceptance
    ]


def missing_acceptance_artifact_files() -> list[str]:
    return [
        str(path.relative_to(ROOT))
        for path in required_acceptance_file_paths()
        if not path.is_file() or path.stat().st_size == 0
    ]


def acceptance_artifact_file_errors(
    confirmed_at: str, *, include_source_integrity: bool = True
) -> list[str]:
    return acceptance_artifact_file_errors_from_parts(
        missing_files=missing_acceptance_artifact_files(),
        source_integrity_errors=(
            acceptance_source_integrity_errors() if include_source_integrity else []
        ),
        manifest_errors=acceptance_artifact_manifest_errors(),
        png_errors=acceptance_artifact_png_errors(),
        ppm_errors=acceptance_artifact_ppm_errors(),
        visual_metric_errors=acceptance_artifact_visual_metric_errors(),
        crop_content_errors=acceptance_artifact_crop_content_errors(),
        live_theme_errors=acceptance_artifact_live_theme_errors(),
        log_errors=acceptance_artifact_log_errors(),
        scroll_performance_errors=acceptance_artifact_scroll_performance_errors(),
        source_freshness_errors=acceptance_artifact_source_freshness_errors(),
        freshness_errors=acceptance_artifact_freshness_errors(confirmed_at),
    )


def acceptance_artifact_file_errors_from_parts(
    *,
    missing_files: list[str],
    source_integrity_errors: list[str],
    manifest_errors: list[str],
    png_errors: list[str],
    ppm_errors: list[str],
    visual_metric_errors: list[str],
    crop_content_errors: list[str],
    live_theme_errors: list[str],
    log_errors: list[str],
    scroll_performance_errors: list[str],
    source_freshness_errors: list[str],
    freshness_errors: list[str],
) -> list[str]:
    errors: list[str] = []
    if missing_files:
        errors.append(
            "acceptance artifact file missing or empty: "
            + ", ".join(missing_files)
            + "."
        )
    errors.extend(source_integrity_errors)
    errors.extend(manifest_errors)
    errors.extend(png_errors)
    errors.extend(ppm_errors)
    errors.extend(visual_metric_errors)
    errors.extend(crop_content_errors)
    errors.extend(live_theme_errors)
    errors.extend(log_errors)
    errors.extend(scroll_performance_errors)
    errors.extend(source_freshness_errors)
    errors.extend(freshness_errors)
    return errors


def acceptance_source_integrity_errors() -> list[str]:
    return acceptance_source_integrity_errors_from_labels(
        missing_labels=tuple(missing_acceptance_source_code_files()),
        untracked_labels=tuple(untracked_acceptance_source_code_files()),
    )


def acceptance_source_integrity_errors_from_labels(
    *, missing_labels: tuple[str, ...], untracked_labels: tuple[str, ...]
) -> list[str]:
    errors: list[str] = []
    if missing_labels:
        errors.append(
            "acceptance source file missing: "
            + ", ".join(missing_labels)
            + "."
        )
    if untracked_labels:
        errors.append(
            "acceptance source file is not tracked by git: "
            + ", ".join(untracked_labels)
            + "."
        )
    return errors


def local_runtime_patch_overlay_errors() -> list[str]:
    if not LOCAL_CARGO_CONFIG.exists():
        return []
    return local_runtime_patch_overlay_errors_from_toml(
        LOCAL_CARGO_CONFIG.read_text(encoding="utf-8")
    )


def local_runtime_patch_overlay_errors_from_toml(config_toml: str) -> list[str]:
    if not config_toml.strip():
        return []
    try:
        config = tomllib.loads(config_toml)
    except tomllib.TOMLDecodeError:
        return [".cargo/config.toml is not valid TOML."]
    patches = config.get("patch")
    if not isinstance(patches, dict):
        return []
    crates_io = patches.get("crates-io")
    if not isinstance(crates_io, dict):
        return []
    runtime = crates_io.get("katana-render-runtime")
    if not isinstance(runtime, dict) or not isinstance(runtime.get("path"), str):
        return []
    return [
        "local katana-render-runtime path patch is not allowed in .cargo/config.toml."
    ]


def plantuml_ci_runtime_errors(ci_workflow: str, preflight_workflow: str) -> list[str]:
    errors: list[str] = []
    requirements = (
        (
            "CI workflow",
            ci_workflow,
            (
                "JAVA_TOOL_OPTIONS",
                "-Xss16m",
                "-Djava.awt.headless=true",
                "-Djdk.lang.processReaperUseDefaultStackSize=true",
                "Install Graphviz (Ubuntu)",
                "apt-get install -y graphviz",
                "/opt/local/bin/dot",
                "GRAPHVIZ_DOT",
                "Install Graphviz (macOS)",
                "brew install graphviz",
                "Install Graphviz (Windows)",
                "choco install graphviz",
                "cargo test --workspace --locked --exclude kdv-storybook",
                "cargo test -p kdv-storybook --locked -- --test-threads=1 --skip katana_intro_text_keeps_readable_frame_band_heights",
                "--skip storybook_score_visual_",
                "--skip mouse_click_uses_external_scroll_for_scroll_independent_scene",
                "cargo test -p kdv-storybook --locked mouse_click_uses_external_scroll_for_scroll_independent_scene -- --test-threads=1",
            ),
        ),
        (
            "release preflight workflow",
            preflight_workflow,
            (
                "JAVA_TOOL_OPTIONS",
                "-Xss16m",
                "-Djava.awt.headless=true",
                "-Djdk.lang.processReaperUseDefaultStackSize=true",
                "apt-get install -y graphviz imagemagick xvfb xclip",
                "command -v magick",
                "/usr/local/bin/magick",
                "exec convert",
                "/opt/local/bin/dot",
                "GRAPHVIZ_DOT",
                "storybook-release-acceptance-artifacts",
                "release-check",
            ),
        ),
    )
    for label, text, tokens in requirements:
        missing = [token for token in tokens if token not in text]
        if missing:
            errors.append(
                f"{label} missing PlantUML/Graphviz runtime pin(s): "
                + ", ".join(missing)
                + "."
            )
    return errors


def release_workflow_acceptance_runtime_errors(release_workflow: str) -> list[str]:
    required = (
        "JAVA_TOOL_OPTIONS",
        "-Xss16m",
        "-Djava.awt.headless=true",
        "-Djdk.lang.processReaperUseDefaultStackSize=true",
        "Install acceptance artifact dependencies",
        "apt-get install -y graphviz imagemagick xvfb xclip",
        "command -v magick",
        "/usr/local/bin/magick",
        "exec convert",
        "/opt/local/bin/dot",
        "GRAPHVIZ_DOT",
        "xvfb-run -a just storybook-release-acceptance-artifacts",
        "xvfb-run -a just VERSION=",
        "release-verify",
    )
    missing = [token for token in required if token not in release_workflow]
    if not missing:
        return []
    return [
        "Release workflow must generate acceptance artifacts under xvfb before "
        "release-verify: missing " + ", ".join(missing) + "."
    ]


def release_test_entrypoint_isolation_errors(justfile: str) -> list[str]:
    errors: list[str] = []
    recipe = just_recipe_body(justfile, "test")
    if recipe is None:
        return ["Justfile test recipe is missing."]
    required = (
        "{{CARGO}} test --workspace --all-targets --all-features --locked --exclude kdv-storybook",
        "{{CARGO}} test -p kdv-storybook --locked -- --test-threads=1 --skip katana_intro_text_keeps_readable_frame_band_heights",
        "--skip storybook_score_visual_",
        "--skip mouse_click_uses_external_scroll_for_scroll_independent_scene",
        "{{CARGO}} test -p kdv-storybook --locked mouse_click_uses_external_scroll_for_scroll_independent_scene -- --test-threads=1",
    )
    missing = [token for token in required if token not in recipe]
    if missing:
        errors.append(
            "Justfile test recipe does not isolate platform-dependent kdv-storybook "
            "visual tests: missing " + ", ".join(missing) + "."
        )
    stale_lines = (
        "{{CARGO}} test --workspace --all-targets --all-features --locked",
        "cargo test --workspace --all-targets --all-features --locked",
    )
    for line in recipe.splitlines():
        if line.strip() in stale_lines:
            errors.append(
                "Justfile test recipe still runs kdv-storybook visual/html tests in "
                "the generic workspace test process."
            )
            break
    return errors


def just_recipe_body(justfile: str, recipe: str) -> str | None:
    header = f"{recipe}:"
    start = justfile.find(header)
    if start < 0:
        return None
    tail = justfile[start:]
    endings = [index for index in (tail.find("\n\n"), tail.find("\r\n\r\n")) if index >= 0]
    if not endings:
        return tail
    return tail[: min(endings)]


def missing_acceptance_source_code_files() -> list[str]:
    return missing_acceptance_source_code_files_from_paths(
        required_acceptance_source_integrity_paths()
    )


def missing_acceptance_source_code_files_from_paths(
    paths: tuple[pathlib.Path, ...]
) -> list[str]:
    return [
        path_label(path)
        for path in paths
        if not path.is_file() or path.stat().st_size == 0
    ]


def untracked_acceptance_source_code_files() -> list[str]:
    return untracked_acceptance_source_code_files_from_paths(
        tuple(
            path
            for path in required_acceptance_source_integrity_paths()
            if path.is_file() and path.stat().st_size > 0
        )
    )


def untracked_acceptance_source_code_files_from_paths(
    paths: tuple[pathlib.Path, ...]
) -> list[str]:
    paths_by_repo: dict[pathlib.Path, list[pathlib.Path]] = {}
    untracked: list[str] = []
    for path in paths:
        repo_root = source_code_repo_root(path)
        if repo_root is None:
            untracked.append(path_label(path))
            continue
        paths_by_repo.setdefault(repo_root, []).append(path)

    tracked_labels: set[str] = set()
    for repo_root, repo_paths in paths_by_repo.items():
        tracked_labels.update(git_tracked_source_file_labels(repo_root, tuple(repo_paths)))

    for path in paths:
        label = path_label(path)
        if label not in tracked_labels and label not in untracked:
            untracked.append(label)
    return untracked


def required_acceptance_source_integrity_paths() -> tuple[pathlib.Path, ...]:
    return (
        REQUIRED_ACCEPTANCE_SOURCE_CODE_PATHS
        + required_acceptance_source_root_file_paths()
        + REQUIRED_ACCEPTANCE_REFERENCE_ARTIFACT_SOURCE_PATHS
    )


def required_acceptance_source_root_file_paths() -> tuple[pathlib.Path, ...]:
    paths: list[pathlib.Path] = []
    for root in REQUIRED_ACCEPTANCE_SOURCE_CODE_ROOTS:
        if not root.is_dir():
            continue
        paths.extend(path for path in root.rglob("*.rs") if path.is_file())
    return tuple(sorted(set(paths)))


def git_tracks_source_file(path: pathlib.Path) -> bool:
    repo_root = source_code_repo_root(path)
    if repo_root is None:
        return False
    relative = path.relative_to(repo_root)
    result = subprocess.run(
        ["git", "-C", str(repo_root), "ls-files", "--error-unmatch", str(relative)],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    return result.returncode == 0


def git_tracked_source_file_labels(
    repo_root: pathlib.Path, paths: tuple[pathlib.Path, ...]
) -> tuple[str, ...]:
    relative_paths = tuple(
        sorted({str(path.relative_to(repo_root)) for path in paths})
    )
    if not relative_paths:
        return ()
    result = subprocess.run(
        ["git", "-C", str(repo_root), "ls-files", "-z", "--", *relative_paths],
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    if result.returncode != 0:
        return ()
    tracked: list[str] = []
    for raw_path in result.stdout.split(b"\0"):
        if not raw_path:
            continue
        tracked.append(path_label(repo_root / os.fsdecode(raw_path)))
    return tuple(tracked)


def source_code_repo_root(path: pathlib.Path) -> pathlib.Path | None:
    try:
        path.relative_to(ROOT)
    except ValueError:
        return None
    return ROOT
    return None


def untracked_acceptance_source_code_files_from_labels(
    required_labels: tuple[str, ...], tracked_labels: tuple[str, ...]
) -> list[str]:
    tracked = set(tracked_labels)
    return [label for label in required_labels if label not in tracked]


def acceptance_artifact_png_errors() -> list[str]:
    errors: list[str] = []
    for path, min_width, min_height in REQUIRED_ACCEPTANCE_PNG_ARTIFACTS:
        if not path.is_file() or path.stat().st_size == 0:
            continue
        label = str(path.relative_to(ROOT))
        error = png_dimension_error(
            path.read_bytes(),
            min_width=min_width,
            min_height=min_height,
            label=label,
        )
        if error:
            errors.append(error)
    return errors


def acceptance_artifact_ppm_errors() -> list[str]:
    errors: list[str] = []
    for path, expected_width, expected_height in REQUIRED_ACCEPTANCE_PPM_ARTIFACTS:
        if not path.is_file() or path.stat().st_size == 0:
            continue
        label = str(path.relative_to(ROOT))
        error = ppm_dimension_error(
            path.read_bytes(),
            expected_width=expected_width,
            expected_height=expected_height,
            label=label,
        )
        if error:
            errors.append(error)
    return errors


def acceptance_artifact_visual_metric_errors() -> list[str]:
    magick = magick_binary()
    if magick is None:
        return ["acceptance artifact visual metric check requires ImageMagick `magick`."]
    observed: dict[str, tuple[int, float, float]] = {}
    for path, *_ in REQUIRED_ACCEPTANCE_VISUAL_METRICS:
        if not path.is_file() or path.stat().st_size == 0:
            continue
        label = str(path.relative_to(ROOT))
        result = subprocess.run(
            [
                magick,
                "identify",
                "-format",
                "%k %[fx:mean] %[fx:standard_deviation]",
                str(path),
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=False,
        )
        if result.returncode != 0:
            observed[label] = (0, 0.0, 0.0)
            continue
        parts = result.stdout.strip().split()
        if len(parts) != 3:
            observed[label] = (0, 0.0, 0.0)
            continue
        try:
            observed[label] = (int(parts[0]), float(parts[1]), float(parts[2]))
        except ValueError:
            observed[label] = (0, 0.0, 0.0)
    thresholds = {
        str(path.relative_to(ROOT)): (
            min_colors,
            min_mean,
            max_mean,
            min_standard_deviation,
            max_standard_deviation,
        )
        for (
            path,
            min_colors,
            min_mean,
            max_mean,
            min_standard_deviation,
            max_standard_deviation,
        ) in REQUIRED_ACCEPTANCE_VISUAL_METRICS
    }
    return acceptance_artifact_visual_metric_errors_from_values(
        observed, thresholds
    )


def magick_binary() -> str | None:
    if configured := os.environ.get("MAGICK"):
        return configured
    if found := shutil.which("magick"):
        return found
    fallback = pathlib.Path("/opt/homebrew/bin/magick")
    if fallback.is_file():
        return str(fallback)
    return None


def acceptance_artifact_visual_metric_errors_from_values(
    observed: dict[str, tuple[int, float, float]],
    thresholds: dict[str, tuple[int, float, float, float, float]],
) -> list[str]:
    errors: list[str] = []
    for label, (
        min_colors,
        min_mean,
        max_mean,
        min_standard_deviation,
        max_standard_deviation,
    ) in thresholds.items():
        if label not in observed:
            continue
        colors, mean, standard_deviation = observed[label]
        if colors < min_colors:
            errors.append(
                "acceptance artifact visual metric has too few colors: "
                f"{label} has {colors}, expected at least {min_colors}."
            )
        if mean < min_mean or mean > max_mean:
            errors.append(
                "acceptance artifact visual metric mean out of range: "
                f"{label} has {mean:.6f}, expected {min_mean:.6f}..{max_mean:.6f}."
            )
        if (
            standard_deviation < min_standard_deviation
            or standard_deviation > max_standard_deviation
        ):
            errors.append(
                "acceptance artifact visual metric standard deviation out of range: "
                f"{label} has {standard_deviation:.6f}, expected "
                f"{min_standard_deviation:.6f}..{max_standard_deviation:.6f}."
            )
    return errors


def acceptance_artifact_crop_content_errors() -> list[str]:
    magick = magick_binary()
    if magick is None:
        return ["acceptance artifact crop content check requires ImageMagick `magick`."]
    observed = {
        str(path.relative_to(ROOT)): acceptance_artifact_crop_content_values(
            magick, path
        )
        for path, *_ in REQUIRED_ACCEPTANCE_CROP_CONTENT_METRICS
        if path.is_file() and path.stat().st_size > 0
    }
    thresholds = {
        str(path.relative_to(ROOT)): (min_colors, min_bright_pixels, min_blue_pixels)
        for path, min_colors, min_bright_pixels, min_blue_pixels in (
            REQUIRED_ACCEPTANCE_CROP_CONTENT_METRICS
        )
    }
    changed_observed = {
        (str(before.relative_to(ROOT)), str(after.relative_to(ROOT))): (
            acceptance_artifact_changed_pixels(magick, before, after)
        )
        for before, after, _ in REQUIRED_ACCEPTANCE_CROP_CHANGED_PIXELS
        if before.is_file()
        and before.stat().st_size > 0
        and after.is_file()
        and after.stat().st_size > 0
    }
    changed_thresholds = {
        (str(before.relative_to(ROOT)), str(after.relative_to(ROOT))): min_changed
        for before, after, min_changed in REQUIRED_ACCEPTANCE_CROP_CHANGED_PIXELS
    }
    return acceptance_artifact_crop_content_errors_from_values(
        observed, thresholds
    ) + acceptance_artifact_changed_pixels_errors_from_values(
        changed_observed, changed_thresholds
    ) + acceptance_artifact_reference_pair_diff_errors(
        magick
    ) + acceptance_artifact_reference_text_contrast_errors(
        magick
    ) + acceptance_artifact_reference_content_errors(
        magick
    ) + acceptance_artifact_reference_edge_ratio_errors(
        magick
    ) + acceptance_artifact_live_window_size_errors(
        magick
    ) + acceptance_artifact_live_interactive_content_errors(
        magick
    ) + acceptance_artifact_live_light_text_errors(
        magick
    ) + acceptance_artifact_title_body_text_errors(
        magick
    ) + acceptance_artifact_direct_margin_left_errors(
        magick,
        ROOT / "target/acceptance/text-regression-crops/direct-html-margin-left.png",
    ) + acceptance_artifact_table_section_errors(
        magick,
        ROOT / "target/acceptance/text-regression-crops/table-section.png",
    ) + acceptance_artifact_table_grid_errors(
        magick
    ) + acceptance_artifact_diagram_control_icon_grid_errors(
        magick,
        ROOT / "target/acceptance/text-regression-crops/diagram-control-icons.png",
    ) + acceptance_artifact_diagram_control_strip_errors(
        magick
    ) + acceptance_artifact_html_center_text_errors(
        magick
    ) + acceptance_artifact_link_underline_errors(
        magick
    ) + acceptance_artifact_hover_highlight_errors(
        magick
    ) + acceptance_artifact_sidebar_selected_row_errors(
        magick
    )


def acceptance_artifact_crop_content_values(
    magick: str, path: pathlib.Path
) -> tuple[int, float, float]:
    colors = magick_float(
        magick, [str(path), "-format", "%k", "info:"]
    )
    bright_pixels = magick_float(
        magick,
        [
            str(path),
            "-alpha",
            "off",
            "-colorspace",
            "Gray",
            "-threshold",
            "35%",
            "-format",
            "%[fx:mean*w*h]",
            "info:",
        ],
    )
    blue_pixels = magick_float(
        magick,
        [
            str(path),
            "-alpha",
            "off",
            "-fx",
            "(b > r + 0.08 && b > g + 0.02 && b > 0.35) ? 1 : 0",
            "-format",
            "%[fx:mean*w*h]",
            "info:",
        ],
    )
    return (int(colors), bright_pixels, blue_pixels)


def acceptance_artifact_reference_pair_diff_errors(magick: str) -> list[str]:
    observed: dict[str, tuple[float, float, float]] = {}
    for reference, candidate, label, *_ in REQUIRED_ACCEPTANCE_REFERENCE_PAIR_DIFF_METRICS:
        if (
            reference.is_file()
            and reference.stat().st_size > 0
            and candidate.is_file()
            and candidate.stat().st_size > 0
        ):
            observed[label] = acceptance_artifact_reference_pair_diff_values(
                magick, reference, candidate
            )
    thresholds = {
        label: (max_mean, max_standard_deviation, max_changed_ratio)
        for (
            _reference,
            _candidate,
            label,
            max_mean,
            max_standard_deviation,
            max_changed_ratio,
        ) in REQUIRED_ACCEPTANCE_REFERENCE_PAIR_DIFF_METRICS
    }
    return acceptance_artifact_reference_pair_diff_errors_from_values(
        observed, thresholds
    )


def acceptance_artifact_reference_text_contrast_errors(magick: str) -> list[str]:
    observed: dict[str, tuple[int, int, float]] = {}
    for (
        reference,
        candidate,
        label,
        dark_threshold,
        *_,
    ) in REQUIRED_ACCEPTANCE_REFERENCE_TEXT_CONTRAST_METRICS:
        if (
            reference.is_file()
            and reference.stat().st_size > 0
            and candidate.is_file()
            and candidate.stat().st_size > 0
        ):
            observed[label] = acceptance_artifact_reference_text_contrast_values(
                magick, reference, candidate, dark_threshold
            )
    thresholds = {
        label: (min_reference_dark_pixels, min_candidate_dark_pixels, min_dark_ratio)
        for (
            _reference,
            _candidate,
            label,
            _dark_threshold,
            min_reference_dark_pixels,
            min_candidate_dark_pixels,
            min_dark_ratio,
        ) in REQUIRED_ACCEPTANCE_REFERENCE_TEXT_CONTRAST_METRICS
    }
    return acceptance_artifact_reference_text_contrast_errors_from_values(
        observed, thresholds
    )


def acceptance_artifact_reference_text_contrast_values(
    magick: str,
    reference: pathlib.Path,
    candidate: pathlib.Path,
    dark_threshold: float,
) -> tuple[int, int, float]:
    reference_dark_pixels = acceptance_artifact_dark_pixel_count(
        magick, reference, dark_threshold
    )
    candidate_dark_pixels = acceptance_artifact_dark_pixel_count(
        magick, candidate, dark_threshold
    )
    if reference_dark_pixels <= 0:
        return (reference_dark_pixels, candidate_dark_pixels, 0.0)
    return (
        reference_dark_pixels,
        candidate_dark_pixels,
        candidate_dark_pixels / reference_dark_pixels,
    )


def acceptance_artifact_dark_pixel_count(
    magick: str, path: pathlib.Path, dark_threshold: float
) -> int:
    return int(
        magick_float(
            magick,
            [
                str(path),
                "-alpha",
                "off",
                "-colorspace",
                "Gray",
                "-fx",
                f"u<{dark_threshold}?1:0",
                "-format",
                "%[fx:mean*w*h]",
                "info:",
            ],
        )
    )


def acceptance_artifact_reference_text_contrast_errors_from_values(
    observed: dict[str, tuple[int, int, float]],
    thresholds: dict[str, tuple[int, int, float]],
) -> list[str]:
    errors: list[str] = []
    for label, (
        min_reference_dark_pixels,
        min_candidate_dark_pixels,
        min_dark_ratio,
    ) in thresholds.items():
        if label not in observed:
            continue
        reference_dark_pixels, candidate_dark_pixels, dark_ratio = observed[label]
        if reference_dark_pixels < min_reference_dark_pixels:
            errors.append(
                "acceptance artifact reference text contrast has too few reference dark text pixels: "
                f"{label} reference_dark_pixels={reference_dark_pixels}, "
                f"expected at least {min_reference_dark_pixels}."
            )
        if candidate_dark_pixels < min_candidate_dark_pixels:
            errors.append(
                "acceptance artifact reference text contrast has too few candidate dark text pixels: "
                f"{label} candidate_dark_pixels={candidate_dark_pixels}, "
                f"expected at least {min_candidate_dark_pixels}."
            )
        if dark_ratio < min_dark_ratio:
            errors.append(
                "acceptance artifact reference text contrast ratio too low: "
                f"{label} dark_ratio={dark_ratio:.3f}, "
                f"expected at least {min_dark_ratio:.3f}."
            )
    return errors


def acceptance_artifact_reference_pair_diff_values(
    magick: str, reference: pathlib.Path, candidate: pathlib.Path
) -> tuple[float, float, float]:
    if magick_identify_size(magick, reference) != magick_identify_size(magick, candidate):
        return (1.0, 1.0, 1.0)
    mean = magick_float(
        magick,
        [
            str(reference),
            str(candidate),
            "-alpha",
            "off",
            "-compose",
            "difference",
            "-composite",
            "-colorspace",
            "Gray",
            "-format",
            "%[fx:mean]",
            "info:",
        ],
    )
    standard_deviation = magick_float(
        magick,
        [
            str(reference),
            str(candidate),
            "-alpha",
            "off",
            "-compose",
            "difference",
            "-composite",
            "-colorspace",
            "Gray",
            "-format",
            "%[fx:standard_deviation]",
            "info:",
        ],
    )
    changed_ratio = magick_float(
        magick,
        [
            str(reference),
            str(candidate),
            "-alpha",
            "off",
            "-compose",
            "difference",
            "-composite",
            "-colorspace",
            "Gray",
            "-threshold",
            "6%",
            "-format",
            "%[fx:mean]",
            "info:",
        ],
    )
    return (mean, standard_deviation, changed_ratio)


def acceptance_artifact_reference_pair_diff_errors_from_values(
    observed: dict[str, tuple[float, float, float]],
    thresholds: dict[str, tuple[float, float, float]],
) -> list[str]:
    errors: list[str] = []
    for label, (
        max_mean,
        max_standard_deviation,
        max_changed_ratio,
    ) in thresholds.items():
        if label not in observed:
            continue
        mean, standard_deviation, changed_ratio = observed[label]
        if mean > max_mean:
            errors.append(
                "acceptance artifact reference pair diff mean too high: "
                f"{label} has {mean:.6f}, expected at most {max_mean:.6f}."
            )
        if standard_deviation > max_standard_deviation:
            errors.append(
                "acceptance artifact reference pair diff standard deviation too high: "
                f"{label} has {standard_deviation:.6f}, expected at most "
                f"{max_standard_deviation:.6f}."
            )
        if changed_ratio > max_changed_ratio:
            errors.append(
                "acceptance artifact reference pair changed ratio too high: "
                f"{label} has {changed_ratio:.6f}, expected at most "
                f"{max_changed_ratio:.6f}."
            )
    return errors


def acceptance_artifact_changed_pixels(
    magick: str, before: pathlib.Path, after: pathlib.Path
) -> float:
    result = subprocess.run(
        [magick, "compare", "-metric", "AE", str(before), str(after), "null:"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    output = (result.stderr or result.stdout).strip().split()
    if not output:
        return 0.0
    try:
        return float(output[0])
    except ValueError:
        return 0.0


def acceptance_artifact_reference_content_errors(magick: str) -> list[str]:
    observed = {
        str(path.relative_to(ROOT)): acceptance_artifact_reference_content_values(
            magick, path
        )
        for path, *_ in REQUIRED_ACCEPTANCE_REFERENCE_CONTENT_METRICS
        if path.is_file() and path.stat().st_size > 0
    }
    thresholds = {
        str(path.relative_to(ROOT)): (min_blue_pixels, min_edge_pixels)
        for path, min_blue_pixels, min_edge_pixels in (
            REQUIRED_ACCEPTANCE_REFERENCE_CONTENT_METRICS
        )
    }
    return acceptance_artifact_reference_content_errors_from_values(
        observed, thresholds
    )


def acceptance_artifact_reference_edge_ratio_errors(magick: str) -> list[str]:
    observed: dict[str, tuple[float, float]] = {}
    for reference, candidate, label, crop_geometry, _min_ratio in (
        REQUIRED_ACCEPTANCE_REFERENCE_EDGE_RATIO_METRICS
    ):
        if (
            reference.is_file()
            and reference.stat().st_size > 0
            and candidate.is_file()
            and candidate.stat().st_size > 0
        ):
            observed[label] = (
                acceptance_artifact_reference_edge_pixels(
                    magick, reference, crop_geometry
                ),
                acceptance_artifact_reference_edge_pixels(
                    magick, candidate, crop_geometry
                ),
            )
    thresholds = {
        label: min_ratio
        for _reference, _candidate, label, _crop_geometry, min_ratio in (
            REQUIRED_ACCEPTANCE_REFERENCE_EDGE_RATIO_METRICS
        )
    }
    return acceptance_artifact_reference_edge_ratio_errors_from_values(
        observed, thresholds
    )


def acceptance_artifact_reference_content_values(
    magick: str, path: pathlib.Path
) -> tuple[float, float]:
    blue_pixels = magick_float(
        magick,
        [
            str(path),
            "-alpha",
            "off",
            "-fx",
            "(b > r + 0.08 && b > g + 0.02 && b > 0.35) ? 1 : 0",
            "-format",
            "%[fx:mean*w*h]",
            "info:",
        ],
    )
    edge_pixels = magick_float(
        magick,
        [
            str(path),
            "-alpha",
            "off",
            "-edge",
            "1",
            "-colorspace",
            "Gray",
            "-threshold",
            "10%",
            "-format",
            "%[fx:mean*w*h]",
            "info:",
        ],
    )
    return (blue_pixels, edge_pixels)


def acceptance_artifact_reference_edge_pixels(
    magick: str, path: pathlib.Path, crop_geometry: str
) -> float:
    args = [str(path)]
    if crop_geometry:
        args.extend(["-crop", crop_geometry, "+repage"])
    args.extend(
        [
            "-alpha",
            "off",
            "-edge",
            "1",
            "-colorspace",
            "Gray",
            "-threshold",
            "10%",
            "-format",
            "%[fx:mean*w*h]",
            "info:",
        ]
    )
    return magick_float(magick, args)


def magick_float(magick: str, args: list[str]) -> float:
    result = subprocess.run(
        [magick, *args],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return 0.0
    try:
        return float(result.stdout.strip().split()[0])
    except (IndexError, ValueError):
        return 0.0


def acceptance_artifact_reference_content_errors_from_values(
    observed: dict[str, tuple[float, float]],
    thresholds: dict[str, tuple[float, float]],
) -> list[str]:
    errors: list[str] = []
    for label, (min_blue_pixels, min_edge_pixels) in thresholds.items():
        if label not in observed:
            continue
        blue_pixels, edge_pixels = observed[label]
        if blue_pixels < min_blue_pixels:
            errors.append(
                "acceptance artifact reference crop has too few link-blue pixels: "
                f"{label} has {blue_pixels:.0f}, expected at least "
                f"{min_blue_pixels:.0f}."
            )
        if edge_pixels < min_edge_pixels:
            errors.append(
                "acceptance artifact reference crop has too few SVG/diagram edge pixels: "
                f"{label} has {edge_pixels:.0f}, expected at least "
                f"{min_edge_pixels:.0f}."
            )
    return errors


def acceptance_artifact_reference_edge_ratio_errors_from_values(
    observed: dict[str, tuple[float, float]], thresholds: dict[str, float]
) -> list[str]:
    errors: list[str] = []
    for label, min_ratio in thresholds.items():
        if label not in observed:
            continue
        reference_edges, candidate_edges = observed[label]
        if reference_edges <= 0:
            errors.append(
                "acceptance artifact reference crop has invalid SVG/diagram edge baseline: "
                f"{label} reference_edges={reference_edges:.0f}."
            )
            continue
        ratio = candidate_edges / reference_edges
        if ratio < min_ratio:
            errors.append(
                "acceptance artifact reference crop SVG/diagram edge ratio too low: "
                f"{label} candidate_edges={candidate_edges:.0f} "
                f"reference_edges={reference_edges:.0f} ratio={ratio:.3f}, "
                f"expected at least {min_ratio:.3f}."
            )
    return errors


def acceptance_artifact_live_theme_errors() -> list[str]:
    magick = magick_binary()
    if magick is None:
        return ["acceptance live artifact theme switch check requires ImageMagick `magick`."]
    (
        before,
        after,
        min_changed_pixels,
        min_bright_delta,
        min_dark_delta,
    ) = REQUIRED_LIVE_ACCEPTANCE_THEME_SWITCH
    if (
        not before.is_file()
        or before.stat().st_size == 0
        or not after.is_file()
        or after.stat().st_size == 0
    ):
        return []
    label = f"{path_label(before)} -> {path_label(after)}"
    values = acceptance_artifact_live_theme_values(magick, before, after)
    return acceptance_artifact_live_theme_errors_from_values(
        values,
        (min_changed_pixels, min_bright_delta, min_dark_delta),
        label,
    )


def acceptance_artifact_live_light_text_errors(magick: str) -> list[str]:
    (
        path,
        label,
        crop_geometry,
        min_bright_background_pixels,
        min_dark_text_pixels,
        min_mean_luminance,
    ) = REQUIRED_LIVE_ACCEPTANCE_LIGHT_TEXT_CONTRAST
    if not path.is_file() or path.stat().st_size == 0:
        return []
    observed = acceptance_artifact_live_light_text_values(magick, path, crop_geometry)
    return acceptance_artifact_live_light_text_errors_from_values(
        observed,
        (
            min_bright_background_pixels,
            min_dark_text_pixels,
            min_mean_luminance,
        ),
        f"{path_label(path)}:{label}",
    )


def acceptance_artifact_live_window_size_errors(magick: str) -> list[str]:
    observed: dict[str, tuple[int, int]] = {}
    expected: dict[str, tuple[int, int]] = {}
    for path, expected_width, expected_height in REQUIRED_LIVE_ACCEPTANCE_WINDOW_SCREENSHOT_SIZE:
        if not path.is_file() or path.stat().st_size == 0:
            continue
        size = magick_identify_size(magick, path)
        if size is not None:
            observed[str(path.relative_to(ROOT))] = size
        expected[str(path.relative_to(ROOT))] = (expected_width, expected_height)
    return acceptance_artifact_live_window_size_errors_from_values(observed, expected)


def acceptance_artifact_live_window_size_errors_from_values(
    observed: dict[str, tuple[int, int]],
    expected: dict[str, tuple[int, int]],
) -> list[str]:
    errors: list[str] = []
    for label, expected_size in expected.items():
        actual_size = observed.get(label)
        if actual_size is None:
            continue
        if actual_size != expected_size:
            errors.append(
                "acceptance live artifact must use the expected headless canvas size: "
                f"{label} is {actual_size[0]}x{actual_size[1]}, expected "
                f"{expected_size[0]}x{expected_size[1]}."
            )
    return errors


def acceptance_artifact_live_interactive_content_errors(magick: str) -> list[str]:
    (
        path,
        label,
        crop_geometry,
        min_bright_content_pixels,
        min_unique_colors,
    ) = REQUIRED_LIVE_ACCEPTANCE_INTERACTIVE_CONTENT
    if not path.is_file() or path.stat().st_size == 0:
        return []
    observed = acceptance_artifact_live_interactive_content_values(
        magick, path, crop_geometry
    )
    return acceptance_artifact_live_interactive_content_errors_from_values(
        observed,
        (min_bright_content_pixels, min_unique_colors),
        f"{path_label(path)}:{label}",
    )


def acceptance_artifact_live_interactive_content_values(
    magick: str, path: pathlib.Path, crop_geometry: str
) -> tuple[float, float]:
    bright_content_pixels = magick_float(
        magick,
        [
            str(path),
            "-crop",
            crop_geometry,
            "-alpha",
            "off",
            "-fx",
            "((0.299*r + 0.587*g + 0.114*b) > 0.550) ? 1 : 0",
            "-format",
            "%[fx:mean*w*h]",
            "info:",
        ],
    )
    unique_colors = magick_float(
        magick,
        [
            str(path),
            "-crop",
            crop_geometry,
            "-alpha",
            "off",
            "-format",
            "%k",
            "info:",
        ],
    )
    return (bright_content_pixels, unique_colors)


def acceptance_artifact_live_light_text_values(
    magick: str, path: pathlib.Path, crop_geometry: str
) -> tuple[float, float, float]:
    bright_background_pixels = magick_float(
        magick,
        [
            str(path),
            "-crop",
            crop_geometry,
            "-alpha",
            "off",
            "-fx",
            "((0.299*r + 0.587*g + 0.114*b) > 0.940) ? 1 : 0",
            "-format",
            "%[fx:mean*w*h]",
            "info:",
        ],
    )
    dark_text_pixels = magick_float(
        magick,
        [
            str(path),
            "-crop",
            crop_geometry,
            "-alpha",
            "off",
            "-fx",
            "((0.299*r + 0.587*g + 0.114*b) < 0.300) ? 1 : 0",
            "-format",
            "%[fx:mean*w*h]",
            "info:",
        ],
    )
    mean_luminance = magick_float(
        magick,
        [
            str(path),
            "-crop",
            crop_geometry,
            "-colorspace",
            "Gray",
            "-format",
            "%[fx:mean]",
            "info:",
        ],
    )
    return (bright_background_pixels, dark_text_pixels, mean_luminance)


def acceptance_artifact_live_theme_values(
    magick: str, before: pathlib.Path, after: pathlib.Path
) -> tuple[float, float, float]:
    before_size = magick_identify_size(magick, before)
    after_size = magick_identify_size(magick, after)
    if before_size is None or after_size is None or before_size != after_size:
        return (0.0, 0.0, 0.0)
    changed_pixels = magick_float(
        magick,
        [
            str(before),
            str(after),
            "-alpha",
            "off",
            "-compose",
            "difference",
            "-composite",
            "-fx",
            "((r + g + b) > 0.118) ? 1 : 0",
            "-format",
            "%[fx:mean*w*h]",
            "info:",
        ],
    )
    before_bright = acceptance_artifact_luminance_count(magick, before, ">", 0.902)
    after_bright = acceptance_artifact_luminance_count(magick, after, ">", 0.902)
    before_dark = acceptance_artifact_luminance_count(magick, before, "<", 0.157)
    after_dark = acceptance_artifact_luminance_count(magick, after, "<", 0.157)
    return (changed_pixels, after_bright - before_bright, before_dark - after_dark)


def magick_identify_size(magick: str, path: pathlib.Path) -> tuple[int, int] | None:
    result = subprocess.run(
        [magick, "identify", "-format", "%w %h", str(path)],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return None
    parts = result.stdout.strip().split()
    if len(parts) != 2:
        return None
    try:
        return (int(parts[0]), int(parts[1]))
    except ValueError:
        return None


def acceptance_artifact_luminance_count(
    magick: str, path: pathlib.Path, operator: str, threshold: float
) -> float:
    return magick_float(
        magick,
        [
            str(path),
            "-alpha",
            "off",
            "-fx",
            f"((0.299*r + 0.587*g + 0.114*b) {operator} {threshold:.3f}) ? 1 : 0",
            "-format",
            "%[fx:mean*w*h]",
            "info:",
        ],
    )


def acceptance_artifact_live_theme_errors_from_values(
    observed: tuple[float, float, float],
    thresholds: tuple[float, float, float],
    label: str,
) -> list[str]:
    changed_pixels, bright_delta, dark_delta = observed
    min_changed_pixels, min_bright_delta, min_dark_delta = thresholds
    if (
        changed_pixels >= min_changed_pixels
        and bright_delta >= min_bright_delta
        and dark_delta >= min_dark_delta
    ):
        return []
    return [
        "acceptance live artifact did not switch to light theme: "
        f"{label} changed_pixels={changed_pixels:.0f}, "
        f"bright_delta={bright_delta:.0f}, dark_delta={dark_delta:.0f}, "
        f"expected changed_pixels>={min_changed_pixels:.0f}, "
        f"bright_delta>={min_bright_delta:.0f}, dark_delta>={min_dark_delta:.0f}."
    ]


def acceptance_artifact_live_interactive_content_errors_from_values(
    observed: tuple[float, float],
    thresholds: tuple[float, float],
    label: str,
) -> list[str]:
    bright_content_pixels, unique_colors = observed
    min_bright_content_pixels, min_unique_colors = thresholds
    errors: list[str] = []
    if bright_content_pixels < min_bright_content_pixels:
        errors.append(
            "acceptance live interactive artifact has too few bright content pixels: "
            f"{label} has {bright_content_pixels:.0f}, expected at least "
            f"{min_bright_content_pixels:.0f}."
        )
    if unique_colors < min_unique_colors:
        errors.append(
            "acceptance live interactive artifact has too few unique colors: "
            f"{label} has {unique_colors:.0f}, expected at least "
            f"{min_unique_colors:.0f}."
        )
    return errors


def acceptance_artifact_live_light_text_errors_from_values(
    observed: tuple[float, float, float],
    thresholds: tuple[float, float, float],
    label: str,
) -> list[str]:
    bright_background_pixels, dark_text_pixels, mean_luminance = observed
    (
        min_bright_background_pixels,
        min_dark_text_pixels,
        min_mean_luminance,
    ) = thresholds
    errors: list[str] = []
    if bright_background_pixels < min_bright_background_pixels:
        errors.append(
            "acceptance live light artifact has too few bright background pixels: "
            f"{label} has {bright_background_pixels:.0f}, expected at least "
            f"{min_bright_background_pixels:.0f}."
        )
    if dark_text_pixels < min_dark_text_pixels:
        errors.append(
            "acceptance live light artifact has too few dark text pixels: "
            f"{label} has {dark_text_pixels:.0f}, expected at least "
            f"{min_dark_text_pixels:.0f}."
        )
    if mean_luminance < min_mean_luminance:
        errors.append(
            "acceptance live light artifact viewer crop is not light enough: "
            f"{label} mean_luminance={mean_luminance:.3f}, expected at least "
            f"{min_mean_luminance:.3f}."
        )
    return errors


def acceptance_artifact_log_errors() -> list[str]:
    log = ROOT / "target/acceptance/kdv-storybook-live-acceptance.log"
    if not log.is_file() or log.stat().st_size == 0:
        return [
            "acceptance log is missing or empty: "
            "target/acceptance/kdv-storybook-live-acceptance.log."
        ]
    return acceptance_artifact_log_errors_from_text(
        str(log.relative_to(ROOT)),
        log.read_text(encoding="utf-8", errors="replace"),
    )


def acceptance_artifact_log_errors_from_text(label: str, text: str) -> list[str]:
    errors: list[str] = []
    for marker in REQUIRED_ACCEPTANCE_LOG_MARKERS:
        if marker not in text:
            errors.append(
                "acceptance log is missing required marker: "
                f"{label} missing `{marker}`."
            )
    for pattern in REQUIRED_ACCEPTANCE_LOG_FORBIDDEN_PATTERNS:
        if pattern.search(text):
            errors.append(
                "acceptance log contains forbidden runtime message: "
                f"{label} matched `{pattern.pattern}`."
            )
    return errors


def acceptance_artifact_scroll_performance_errors() -> list[str]:
    artifact = ROOT / "target/acceptance/kdv-storybook-scroll-performance.txt"
    if not artifact.is_file() or artifact.stat().st_size == 0:
        return []
    return acceptance_artifact_scroll_performance_errors_from_text(
        str(artifact.relative_to(ROOT)),
        artifact.read_text(encoding="utf-8", errors="replace"),
    )


def acceptance_artifact_scroll_performance_errors_from_text(
    label: str, text: str
) -> list[str]:
    values: dict[str, str] = {}
    for line in text.splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        values[key.strip()] = value.strip()

    errors: list[str] = []
    for key, expected in REQUIRED_SCROLL_PERFORMANCE_VALUES.items():
        actual = values.get(key)
        if actual != expected:
            if key == "full_preview_redraw_fallback_count":
                errors.append(
                    "acceptance scroll performance has full preview redraw fallback: "
                    f"{label} {key}={actual!r}, expected {expected!r}."
                )
            else:
                errors.append(
                    "acceptance scroll performance has unexpected value: "
                    f"{label} {key}={actual!r}, expected {expected!r}."
                )

    try:
        elapsed_ms = float(values["elapsed_ms"])
        budget_ms = float(values["budget_ms"])
    except (KeyError, ValueError) as error:
        errors.append(
            f"acceptance scroll performance is missing numeric budget data: {label} {error}."
        )
        return errors

    if elapsed_ms > budget_ms:
        errors.append(
            "acceptance scroll performance exceeded budget: "
            f"{label} elapsed_ms={elapsed_ms:.3f} budget_ms={budget_ms:.3f}."
        )
    return errors


def acceptance_artifact_crop_content_errors_from_values(
    observed: dict[str, tuple[int, float, float]],
    thresholds: dict[str, tuple[int, float, float]],
) -> list[str]:
    errors: list[str] = []
    for label, (min_colors, min_bright_pixels, min_blue_pixels) in thresholds.items():
        if label not in observed:
            continue
        colors, bright_pixels, blue_pixels = observed[label]
        if colors < min_colors:
            errors.append(
                "acceptance artifact crop content has too few unique colors: "
                f"{label} has {colors}, expected at least {min_colors}."
            )
        if bright_pixels < min_bright_pixels:
            errors.append(
                "acceptance artifact crop content has too few bright text pixels: "
                f"{label} has {bright_pixels:.0f}, expected at least "
                f"{min_bright_pixels:.0f}."
            )
        if blue_pixels < min_blue_pixels:
            errors.append(
                "acceptance artifact crop content has too few link-blue pixels: "
                f"{label} has {blue_pixels:.0f}, expected at least "
                f"{min_blue_pixels:.0f}."
            )
    return errors


def acceptance_artifact_changed_pixels_errors_from_values(
    observed: dict[tuple[str, str], float],
    thresholds: dict[tuple[str, str], float],
) -> list[str]:
    errors: list[str] = []
    for pair, min_changed_pixels in thresholds.items():
        if pair not in observed:
            continue
        changed_pixels = observed[pair]
        if changed_pixels < min_changed_pixels:
            before, after = pair
            errors.append(
                "acceptance artifact crop content has too few changed pixels: "
                f"{before} -> {after} has {changed_pixels:.0f}, expected at least "
                f"{min_changed_pixels:.0f}."
            )
    return errors


class CropBand:
    def __init__(
        self, min_x: int, max_x: int, min_y: int, max_y: int, area: int
    ) -> None:
        self.min_x = min_x
        self.max_x = max_x
        self.min_y = min_y
        self.max_y = max_y
        self.area = area


def acceptance_artifact_direct_margin_left_errors(
    magick: str, path: pathlib.Path
) -> list[str]:
    if not path.is_file() or path.stat().st_size == 0:
        return []
    result = subprocess.run(
        [
            magick,
            str(path),
            "-alpha",
            "off",
            "-colorspace",
            "Gray",
            "-threshold",
            "35%",
            "-define",
            "connected-components:verbose=true",
            "-connected-components",
            "8",
            "null:",
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return [
            "acceptance artifact direct margin-left crop could not be analyzed: "
            f"{path_label(path)}."
        ]
    components = connected_component_bands_from_text(result.stdout)
    bands = merge_connected_component_bands(components)
    return acceptance_artifact_direct_margin_left_errors_from_bands(
        bands, path_label(path)
    )


def connected_component_bands_from_text(text: str) -> list[CropBand]:
    bands: list[CropBand] = []
    pattern = re.compile(
        r"^\s*\d+:\s+(?P<w>\d+)x(?P<h>\d+)\+(?P<x>\d+)\+(?P<y>\d+)"
        r"\s+[^ ]+\s+(?P<area>\d+)\s+(?:gray\(255\)|srgb\(255,255,255\))"
    )
    for line in text.splitlines():
        match = pattern.match(line)
        if not match:
            continue
        width = int(match.group("w"))
        height = int(match.group("h"))
        x = int(match.group("x"))
        y = int(match.group("y"))
        area = int(match.group("area"))
        if area < 5 or width == 0 or height == 0:
            continue
        bands.append(CropBand(x, x + width - 1, y, y + height - 1, area))
    return bands


def merge_connected_component_bands(components: list[CropBand]) -> list[CropBand]:
    merged: list[CropBand] = []
    for component in sorted(components, key=lambda band: (band.min_y, band.min_x)):
        if not merged or component.min_y > merged[-1].max_y + 5:
            merged.append(
                CropBand(
                    component.min_x,
                    component.max_x,
                    component.min_y,
                    component.max_y,
                    component.area,
                )
            )
            continue
        current = merged[-1]
        current.min_x = min(current.min_x, component.min_x)
        current.max_x = max(current.max_x, component.max_x)
        current.min_y = min(current.min_y, component.min_y)
        current.max_y = max(current.max_y, component.max_y)
        current.area += component.area
    return [
        band
        for band in merged
        if band.area >= 100 and band.max_x - band.min_x >= 20
    ]


def acceptance_artifact_title_body_text_errors(magick: str) -> list[str]:
    observed_by_path: dict[pathlib.Path, list[CropBand]] = {}
    for path, *_ in REQUIRED_ACCEPTANCE_TITLE_BODY_TEXT_BANDS:
        if path not in observed_by_path:
            observed_by_path[path] = acceptance_artifact_text_bands(magick, path)
    observed: dict[str, tuple[int, int, int, int, int]] = {}
    for (
        path,
        name,
        _min_x,
        _max_x,
        min_y,
        max_y,
        _min_width,
        _max_width,
        _min_height,
        _max_height,
        _min_area,
        _max_area,
    ) in REQUIRED_ACCEPTANCE_TITLE_BODY_TEXT_BANDS:
        label = f"{path.relative_to(ROOT)}:{name}"
        band = first_band_starting_in_y_range(
            observed_by_path.get(path, []), min_y, max_y
        )
        if band is not None:
            observed[label] = (
                band.min_x,
                band.max_x,
                band.min_y,
                band.max_y,
                band.area,
            )
    thresholds = {
        f"{path.relative_to(ROOT)}:{name}": (
            min_x,
            max_x,
            min_y,
            max_y,
            min_width,
            max_width,
            min_height,
            max_height,
            min_area,
            max_area,
        )
        for (
            path,
            name,
            min_x,
            max_x,
            min_y,
            max_y,
            min_width,
            max_width,
            min_height,
            max_height,
            min_area,
            max_area,
        ) in REQUIRED_ACCEPTANCE_TITLE_BODY_TEXT_BANDS
    }
    return acceptance_artifact_title_body_text_errors_from_values(
        observed, thresholds
    )


def acceptance_artifact_text_bands(
    magick: str, path: pathlib.Path
) -> list[CropBand]:
    if not path.is_file() or path.stat().st_size == 0:
        return []
    result = subprocess.run(
        [
            magick,
            str(path),
            "-alpha",
            "off",
            "-colorspace",
            "Gray",
            "-threshold",
            "35%",
            "-define",
            "connected-components:verbose=true",
            "-connected-components",
            "8",
            "null:",
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return []
    return merge_connected_component_bands(
        connected_component_bands_from_text(result.stdout)
    )


def acceptance_artifact_title_body_text_errors_from_values(
    observed: dict[str, tuple[int, int, int, int, int]],
    thresholds: dict[str, tuple[int, int, int, int, int, int, int, int, int, int]],
) -> list[str]:
    errors: list[str] = []
    for label, (
        min_x,
        max_x,
        min_y,
        max_y,
        min_width,
        max_width,
        min_height,
        max_height,
        min_area,
        max_area,
    ) in thresholds.items():
        if label not in observed:
            errors.append(
                "acceptance artifact title/body crop missing expected text band: "
                f"{label}."
            )
            continue
        observed_min_x, observed_max_x, observed_min_y, observed_max_y, area = (
            observed[label]
        )
        width = observed_max_x - observed_min_x + 1
        height = observed_max_y - observed_min_y + 1
        if observed_min_x < min_x or observed_min_x > max_x:
            errors.append(
                "acceptance artifact title/body crop has wrong title/body text x position: "
                f"{label} x={observed_min_x}, expected {min_x}..{max_x}."
            )
        if observed_min_y < min_y or observed_min_y > max_y:
            errors.append(
                "acceptance artifact title/body crop has wrong title/body text y position: "
                f"{label} y={observed_min_y}, expected {min_y}..{max_y}."
            )
        if width < min_width or width > max_width:
            errors.append(
                "acceptance artifact title/body crop has wrong title/body text width: "
                f"{label} width={width}, expected {min_width}..{max_width}."
            )
        if height < min_height or height > max_height:
            errors.append(
                "acceptance artifact title/body crop has wrong title/body text height: "
                f"{label} height={height}, expected {min_height}..{max_height}."
            )
        if area < min_area or area > max_area:
            errors.append(
                "acceptance artifact title/body crop has wrong title/body text area: "
                f"{label} area={area}, expected {min_area}..{max_area}."
            )
    return errors


def acceptance_artifact_direct_margin_left_errors_from_bands(
    bands: list[CropBand], label: str
) -> list[str]:
    if len(bands) < 4:
        return [
            "acceptance artifact direct margin-left crop missing expected text bands: "
            f"{label} has {len(bands)}, expected at least 4."
        ]
    baseline = bands[1]
    indented_link = bands[2]
    indented_text = bands[3]
    link_delta = indented_link.min_x - baseline.min_x
    text_delta = indented_text.min_x - baseline.min_x
    text_over_link_delta = indented_text.min_x - indented_link.min_x
    errors: list[str] = []
    if link_delta < 75 or link_delta > 85:
        errors.append(
            "acceptance artifact direct margin-left crop has wrong 80px link offset: "
            f"{label} baseline_x={baseline.min_x} link_x={indented_link.min_x} "
            f"delta={link_delta}, expected 75..85."
        )
    if text_delta < 115 or text_delta > 125:
        errors.append(
            "acceptance artifact direct margin-left crop has wrong 120px text offset: "
            f"{label} baseline_x={baseline.min_x} text_x={indented_text.min_x} "
            f"delta={text_delta}, expected 115..125."
        )
    if text_over_link_delta < 35 or text_over_link_delta > 45:
        errors.append(
            "acceptance artifact direct margin-left crop has wrong link-to-text offset: "
            f"{label} link_x={indented_link.min_x} text_x={indented_text.min_x} "
            f"delta={text_over_link_delta}, expected 35..45."
        )
    return errors


def acceptance_artifact_table_section_errors(
    magick: str, path: pathlib.Path
) -> list[str]:
    if not path.is_file() or path.stat().st_size == 0:
        return []
    result = subprocess.run(
        [
            magick,
            str(path),
            "-alpha",
            "off",
            "-colorspace",
            "Gray",
            "-threshold",
            "35%",
            "-define",
            "connected-components:verbose=true",
            "-connected-components",
            "8",
            "null:",
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return [
            "acceptance artifact table-section crop could not be analyzed: "
            f"{path_label(path)}."
        ]
    components = connected_component_bands_from_text(result.stdout)
    bands = merge_connected_component_bands(components)
    return acceptance_artifact_table_section_errors_from_bands(
        bands, path_label(path)
    )


def acceptance_artifact_table_section_errors_from_bands(
    bands: list[CropBand], label: str
) -> list[str]:
    errors: list[str] = []
    sorted_bands = sorted(bands, key=lambda band: (band.min_y, band.min_x))
    if len(sorted_bands) < 10:
        errors.append(
            "acceptance artifact table-section crop missing expected text/table bands: "
            f"{label} has {len(sorted_bands)}, expected at least 10."
        )
        return errors
    located: dict[str, CropBand] = {}
    for name, min_y, max_y in REQUIRED_ACCEPTANCE_TABLE_SECTION_BANDS:
        band = first_band_in_y_range(sorted_bands, min_y, max_y)
        if band is None:
            errors.append(
                "acceptance artifact table-section crop missing expected band: "
                f"{label} missing `{name}` in y={min_y}..{max_y}."
            )
            continue
        located[name] = band
    if errors:
        return errors
    for name, min_y, max_y, min_count, max_count in (
        REQUIRED_ACCEPTANCE_TABLE_SECTION_ROW_COUNTS
    ):
        row_count = len(bands_in_y_range(sorted_bands, min_y, max_y))
        if row_count < min_count:
            errors.append(
                "acceptance artifact table-section crop has too few table rows: "
                f"{label} {name} has {row_count}, expected at least {min_count}."
            )
        if row_count > max_count:
            errors.append(
                "acceptance artifact table-section crop has too many table rows: "
                f"{label} {name} has {row_count}, expected at most {max_count}; "
                "this usually means table text wrapped unexpectedly."
            )
    basic_bottom = located["basic table bottom row"]
    alignment_heading = located["5.2 Table with Alignment heading"]
    alignment_first = located["alignment table first row"]
    alignment_bottom = located["alignment table bottom row"]
    single_heading = located["5.3 Single Row Table heading"]
    if alignment_heading.min_y - basic_bottom.max_y < 30:
        errors.append(
            "acceptance artifact table-section crop has overlapping 5.2 heading/table: "
            f"{label} basic_bottom_y={basic_bottom.max_y} "
            f"alignment_heading_y={alignment_heading.min_y}, expected gap >= 30."
        )
    if alignment_first.min_y - alignment_heading.max_y < 25:
        errors.append(
            "acceptance artifact table-section crop has overlapping alignment table: "
            f"{label} heading_bottom_y={alignment_heading.max_y} "
            f"table_first_y={alignment_first.min_y}, expected gap >= 25."
        )
    if single_heading.min_y - alignment_bottom.max_y < 45:
        errors.append(
            "acceptance artifact table-section crop has overlapping 5.3 heading/table: "
            f"{label} alignment_bottom_y={alignment_bottom.max_y} "
            f"single_heading_y={single_heading.min_y}, expected gap >= 45."
        )
    return errors


def acceptance_artifact_table_grid_errors(magick: str) -> list[str]:
    observed_by_path: dict[pathlib.Path, list[CropBand]] = {}
    for path, *_ in REQUIRED_ACCEPTANCE_TABLE_GRID_COMPONENTS:
        if path not in observed_by_path:
            observed_by_path[path] = acceptance_artifact_table_grid_components(magick, path)
    observed: dict[str, tuple[int, int, int, int, int]] = {}
    for (
        path,
        name,
        _min_x,
        _max_x,
        min_y,
        max_y,
        _min_width,
        _max_width,
        _min_height,
        _max_height,
        _min_area,
        _max_area,
    ) in REQUIRED_ACCEPTANCE_TABLE_GRID_COMPONENTS:
        label = f"{path.relative_to(ROOT)}:{name}"
        component = table_grid_component_in_y_range(
            observed_by_path.get(path, []), min_y, max_y
        )
        if component is not None:
            observed[label] = (
                component.min_x,
                component.max_x,
                component.min_y,
                component.max_y,
                component.area,
            )
    thresholds = {
        f"{path.relative_to(ROOT)}:{name}": (
            min_x,
            max_x,
            min_y,
            max_y,
            min_width,
            max_width,
            min_height,
            max_height,
            min_area,
            max_area,
        )
        for (
            path,
            name,
            min_x,
            max_x,
            min_y,
            max_y,
            min_width,
            max_width,
            min_height,
            max_height,
            min_area,
            max_area,
        ) in REQUIRED_ACCEPTANCE_TABLE_GRID_COMPONENTS
    }
    return acceptance_artifact_table_grid_errors_from_values(observed, thresholds)


def acceptance_artifact_table_grid_components(
    magick: str, path: pathlib.Path
) -> list[CropBand]:
    if not path.is_file() or path.stat().st_size == 0:
        return []
    result = subprocess.run(
        [
            magick,
            str(path),
            "-alpha",
            "off",
            "-fx",
            "((r>0.22)&&(r<0.25)&&(abs(r-g)<0.002)&&(abs(g-b)<0.002))?1:0",
            "-define",
            "connected-components:verbose=true",
            "-connected-components",
            "8",
            "null:",
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return []
    return [
        band
        for band in connected_component_bands_from_text(result.stdout)
        if band.max_x - band.min_x + 1 >= 700
    ]


def table_grid_component_in_y_range(
    components: list[CropBand], min_y: int, max_y: int
) -> CropBand | None:
    candidates = [
        component
        for component in components
        if component.min_y >= min_y and component.min_y <= max_y
    ]
    return max(candidates, key=lambda component: component.area, default=None)


def acceptance_artifact_table_grid_errors_from_values(
    observed: dict[str, tuple[int, int, int, int, int]],
    thresholds: dict[str, tuple[int, int, int, int, int, int, int, int, int, int]],
) -> list[str]:
    errors: list[str] = []
    for label, (
        min_x,
        max_x,
        min_y,
        max_y,
        min_width,
        max_width,
        min_height,
        max_height,
        min_area,
        max_area,
    ) in thresholds.items():
        if label not in observed:
            errors.append(
                "acceptance artifact table-section crop missing expected table grid: "
                f"{label}."
            )
            continue
        observed_min_x, observed_max_x, observed_min_y, observed_max_y, area = (
            observed[label]
        )
        width = observed_max_x - observed_min_x + 1
        height = observed_max_y - observed_min_y + 1
        if observed_min_x < min_x or observed_min_x > max_x:
            errors.append(
                "acceptance artifact table-section crop has wrong table grid x position: "
                f"{label} x={observed_min_x}, expected {min_x}..{max_x}."
            )
        if observed_min_y < min_y or observed_min_y > max_y:
            errors.append(
                "acceptance artifact table-section crop has wrong table grid y position: "
                f"{label} y={observed_min_y}, expected {min_y}..{max_y}."
            )
        if width < min_width or width > max_width:
            errors.append(
                "acceptance artifact table-section crop has wrong table grid width: "
                f"{label} width={width}, expected {min_width}..{max_width}."
            )
        if height < min_height or height > max_height:
            errors.append(
                "acceptance artifact table-section crop has wrong table grid height: "
                f"{label} height={height}, expected {min_height}..{max_height}."
            )
        if area < min_area or area > max_area:
            errors.append(
                "acceptance artifact table-section crop has wrong table grid area: "
                f"{label} area={area}, expected {min_area}..{max_area}."
            )
    return errors


def first_band_in_y_range(
    bands: list[CropBand], min_y: int, max_y: int
) -> CropBand | None:
    for band in bands:
        if band.min_y >= min_y and band.max_y <= max_y:
            return band
    return None


def first_band_starting_in_y_range(
    bands: list[CropBand], min_y: int, max_y: int
) -> CropBand | None:
    for band in bands:
        if band.min_y >= min_y and band.min_y <= max_y:
            return band
    return None


def bands_in_y_range(
    bands: list[CropBand], min_y: int, max_y: int
) -> list[CropBand]:
    return [
        band
        for band in bands
        if band.min_y >= min_y and band.max_y <= max_y
    ]


def acceptance_artifact_sidebar_selected_row_errors(magick: str) -> list[str]:
    observed = {
        str(path.relative_to(ROOT)): (
            acceptance_artifact_sidebar_selected_row_values(magick, path)
        )
        for path, *_ in REQUIRED_ACCEPTANCE_SIDEBAR_SELECTED_ROW_BANDS
        if path.is_file() and path.stat().st_size > 0
    }
    thresholds = {
        str(path.relative_to(ROOT)): (
            min_x,
            max_x,
            min_y,
            max_y,
            min_width,
            max_width,
            min_height,
            max_height,
            min_area,
            max_area,
        )
        for (
            path,
            min_x,
            max_x,
            min_y,
            max_y,
            min_width,
            max_width,
            min_height,
            max_height,
            min_area,
            max_area,
        ) in REQUIRED_ACCEPTANCE_SIDEBAR_SELECTED_ROW_BANDS
    }
    return acceptance_artifact_sidebar_selected_row_errors_from_values(
        observed, thresholds
    )


def acceptance_artifact_sidebar_selected_row_values(
    magick: str, path: pathlib.Path
) -> tuple[int, int, int, int, int]:
    result = subprocess.run(
        [
            magick,
            str(path),
            "-alpha",
            "off",
            "-fx",
            "((b>0.35)&&(g>0.20)&&(r<0.25))?1:0",
            "-define",
            "connected-components:verbose=true",
            "-connected-components",
            "8",
            "null:",
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return (0, 0, 0, 0, 0)
    bands = connected_component_bands_from_text(result.stdout)
    selected_row = max(bands, key=lambda band: band.area, default=None)
    if selected_row is None:
        return (0, 0, 0, 0, 0)
    return (
        selected_row.min_x,
        selected_row.max_x,
        selected_row.min_y,
        selected_row.max_y,
        selected_row.area,
    )


def acceptance_artifact_sidebar_selected_row_errors_from_values(
    observed: dict[str, tuple[int, int, int, int, int]],
    thresholds: dict[str, tuple[int, int, int, int, int, int, int, int, int, int]],
) -> list[str]:
    errors: list[str] = []
    for label, (
        min_x,
        max_x,
        min_y,
        max_y,
        min_width,
        max_width,
        min_height,
        max_height,
        min_area,
        max_area,
    ) in thresholds.items():
        if label not in observed:
            errors.append(
                "acceptance artifact sidebar selected-row missing expected artifact: "
                f"{label}."
            )
            continue
        observed_min_x, observed_max_x, observed_min_y, observed_max_y, area = (
            observed[label]
        )
        width = observed_max_x - observed_min_x + 1
        height = observed_max_y - observed_min_y + 1
        if observed_min_x < min_x or observed_min_x > max_x:
            errors.append(
                "acceptance artifact sidebar selected-row has wrong sidebar selected-row x position: "
                f"{label} x={observed_min_x}, expected {min_x}..{max_x}."
            )
        if observed_min_y < min_y or observed_min_y > max_y:
            errors.append(
                "acceptance artifact sidebar selected-row has wrong sidebar selected-row y position: "
                f"{label} y={observed_min_y}, expected {min_y}..{max_y}."
            )
        if width < min_width or width > max_width:
            errors.append(
                "acceptance artifact sidebar selected-row has wrong sidebar selected-row width: "
                f"{label} width={width}, expected {min_width}..{max_width}."
            )
        if height < min_height or height > max_height:
            errors.append(
                "acceptance artifact sidebar selected-row has wrong sidebar selected-row height: "
                f"{label} height={height}, expected {min_height}..{max_height}."
            )
        if area < min_area or area > max_area:
            errors.append(
                "acceptance artifact sidebar selected-row has wrong sidebar selected-row area: "
                f"{label} area={area}, expected {min_area}..{max_area}."
            )
    return errors


def acceptance_artifact_diagram_control_icon_grid_errors(
    magick: str, path: pathlib.Path
) -> list[str]:
    if not path.is_file() or path.stat().st_size == 0:
        return []
    observed: dict[str, float] = {}
    for command, x, y, width, height, _, _ in REQUIRED_ACCEPTANCE_DIAGRAM_CONTROL_ICON_CELLS:
        observed[command] = magick_float(
            magick,
            [
                str(path),
                "-crop",
                f"{width}x{height}+{x}+{y}",
                "+repage",
                "-alpha",
                "off",
                "-colorspace",
                "Gray",
                "-threshold",
                "35%",
                "-format",
                "%[fx:mean*w*h]",
                "info:",
            ],
        )
    thresholds = {
        command: (min_pixels, max_pixels)
        for command, _, _, _, _, min_pixels, max_pixels in (
            REQUIRED_ACCEPTANCE_DIAGRAM_CONTROL_ICON_CELLS
        )
    }
    return acceptance_artifact_diagram_control_icon_grid_errors_from_values(
        observed, thresholds, path_label(path)
    )


def acceptance_artifact_diagram_control_icon_grid_errors_from_values(
    observed: dict[str, float],
    thresholds: dict[str, tuple[int, int]],
    label: str,
) -> list[str]:
    errors: list[str] = []
    for command, (min_pixels, max_pixels) in thresholds.items():
        if command not in observed:
            errors.append(
                "acceptance artifact diagram control icon crop missing expected cell: "
                f"{label} missing `{command}`."
            )
            continue
        bright_pixels = observed[command]
        if bright_pixels < min_pixels:
            errors.append(
                "acceptance artifact diagram control icon crop has too few bright glyph pixels: "
                f"{label} `{command}` has {bright_pixels:.0f}, expected at least "
                f"{min_pixels}."
            )
        if bright_pixels > max_pixels:
            errors.append(
                "acceptance artifact diagram control icon crop has too many bright glyph pixels: "
                f"{label} `{command}` has {bright_pixels:.0f}, expected at most "
                f"{max_pixels}."
            )
    return errors


def acceptance_artifact_diagram_control_strip_errors(magick: str) -> list[str]:
    observed: dict[str, dict[str, float]] = {}
    thresholds: dict[str, dict[str, tuple[int, int]]] = {}
    for path, name, crop_x, crop_y, _crop_width, _crop_height in (
        REQUIRED_ACCEPTANCE_DIAGRAM_CONTROL_STRIP_REGIONS
    ):
        label = f"{path.relative_to(ROOT)}:{name}"
        thresholds[label] = REQUIRED_ACCEPTANCE_DIAGRAM_CONTROL_STRIP_CELL_THRESHOLDS
        if path.is_file() and path.stat().st_size > 0:
            observed[label] = acceptance_artifact_diagram_control_strip_cell_values(
                magick, path, crop_x, crop_y
            )
    return acceptance_artifact_diagram_control_strip_errors_from_values(
        observed, thresholds
    )


def acceptance_artifact_diagram_control_strip_cell_values(
    magick: str,
    path: pathlib.Path,
    crop_x: int,
    crop_y: int,
) -> dict[str, float]:
    observed: dict[str, float] = {}
    for command, x, y, width, height, _, _ in (
        REQUIRED_ACCEPTANCE_DIAGRAM_CONTROL_ICON_CELLS
    ):
        observed[command] = magick_float(
            magick,
            [
                str(path),
                "-crop",
                f"{width}x{height}+{crop_x + x}+{crop_y + y}",
                "+repage",
                "-alpha",
                "off",
                "-colorspace",
                "Gray",
                "-threshold",
                "35%",
                "-format",
                "%[fx:mean*w*h]",
                "info:",
            ],
        )
    return observed


def acceptance_artifact_diagram_control_strip_errors_from_values(
    observed: dict[str, dict[str, float]],
    thresholds: dict[str, dict[str, tuple[int, int]]],
) -> list[str]:
    errors: list[str] = []
    for label, expected_cells in thresholds.items():
        if label not in observed:
            errors.append(
                "acceptance artifact diagram control strip missing expected right-edge cells: "
                f"{label}."
            )
            continue
        actual_cells = observed[label]
        for command, (min_pixels, max_pixels) in expected_cells.items():
            if command not in actual_cells:
                errors.append(
                    "acceptance artifact diagram control strip missing expected control cell: "
                    f"{label} `{command}`."
                )
                continue
            bright_pixels = actual_cells[command]
            if bright_pixels < min_pixels:
                errors.append(
                    "acceptance artifact diagram control strip has too few right-edge control pixels: "
                    f"{label} `{command}` has {bright_pixels:.0f}, expected at least "
                    f"{min_pixels}."
                )
            if bright_pixels > max_pixels:
                errors.append(
                    "acceptance artifact diagram control strip has too many right-edge control pixels: "
                    f"{label} `{command}` has {bright_pixels:.0f}, expected at most "
                    f"{max_pixels}."
                )
    return errors


def acceptance_artifact_html_center_text_errors(magick: str) -> list[str]:
    observed_by_path: dict[pathlib.Path, list[CropBand]] = {}
    for path, *_ in REQUIRED_ACCEPTANCE_HTML_CENTER_TEXT_BANDS:
        if path not in observed_by_path:
            observed_by_path[path] = acceptance_artifact_text_bands(magick, path)
    observed: dict[str, tuple[int, int, int, int, int]] = {}
    thresholds: dict[str, tuple[int, int, int, int, int, int]] = {}
    for (
        path,
        name,
        min_y,
        max_y,
        min_center_x,
        max_center_x,
        min_width,
        max_width,
        min_area,
        max_area,
    ) in REQUIRED_ACCEPTANCE_HTML_CENTER_TEXT_BANDS:
        label = f"{path.relative_to(ROOT)}:{name}"
        band = first_band_starting_in_y_range(
            observed_by_path.get(path, []), min_y, max_y
        )
        if band is not None:
            observed[label] = (
                band.min_x,
                band.max_x,
                band.min_y,
                band.max_y,
                band.area,
            )
        thresholds[label] = (
            min_center_x,
            max_center_x,
            min_width,
            max_width,
            min_area,
            max_area,
        )
    return acceptance_artifact_html_center_text_errors_from_values(
        observed, thresholds
    )


def acceptance_artifact_html_center_text_errors_from_values(
    observed: dict[str, tuple[int, int, int, int, int]],
    thresholds: dict[str, tuple[int, int, int, int, int, int]],
) -> list[str]:
    errors: list[str] = []
    for label, (
        min_center_x,
        max_center_x,
        min_width,
        max_width,
        min_area,
        max_area,
    ) in thresholds.items():
        if label not in observed:
            errors.append(
                "acceptance artifact html center crop missing expected centered text band: "
                f"{label}."
            )
            continue
        observed_min_x, observed_max_x, _observed_min_y, _observed_max_y, area = (
            observed[label]
        )
        width = observed_max_x - observed_min_x + 1
        center_x = (observed_min_x + observed_max_x) / 2
        if center_x < min_center_x or center_x > max_center_x:
            errors.append(
                "acceptance artifact html center crop has wrong centered text position: "
                f"{label} center_x={center_x:.1f}, expected "
                f"{min_center_x}..{max_center_x}."
            )
        if width < min_width or width > max_width:
            errors.append(
                "acceptance artifact html center crop has wrong centered text width: "
                f"{label} width={width}, expected {min_width}..{max_width}."
            )
        if area < min_area or area > max_area:
            errors.append(
                "acceptance artifact html center crop has wrong centered text area: "
                f"{label} area={area}, expected {min_area}..{max_area}."
            )
    return errors


def acceptance_artifact_link_underline_errors(magick: str) -> list[str]:
    observed = {
        str(path.relative_to(ROOT)): acceptance_artifact_link_underline_values(
            magick, path
        )
        for path, *_ in REQUIRED_ACCEPTANCE_LINK_UNDERLINE_BANDS
        if path.is_file() and path.stat().st_size > 0
    }
    thresholds = {
        str(path.relative_to(ROOT)): (
            min_x,
            max_x,
            min_y,
            max_y,
            min_width,
            max_width,
            min_area,
            max_area,
            min_bottom_pixels,
        )
        for (
            path,
            min_x,
            max_x,
            min_y,
            max_y,
            min_width,
            max_width,
            min_area,
            max_area,
            min_bottom_pixels,
        ) in REQUIRED_ACCEPTANCE_LINK_UNDERLINE_BANDS
    }
    return acceptance_artifact_link_underline_errors_from_values(observed, thresholds)


def acceptance_artifact_link_underline_values(
    magick: str, path: pathlib.Path
) -> tuple[int, int, int, int, int, float]:
    result = subprocess.run(
        [
            magick,
            str(path),
            "-alpha",
            "off",
            "-fx",
            "(b > r + 0.08 && b > g + 0.02 && b > 0.35) ? 1 : 0",
            "-define",
            "connected-components:verbose=true",
            "-connected-components",
            "8",
            "null:",
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return (0, 0, 0, 0, 0, 0.0)
    bands = connected_component_bands_from_text(result.stdout)
    link_band = max(bands, key=lambda band: band.area, default=None)
    if link_band is None:
        return (0, 0, 0, 0, 0, 0.0)
    width = link_band.max_x - link_band.min_x + 1
    bottom_pixels = magick_float(
        magick,
        [
            str(path),
            "-crop",
            f"{width}x1+{link_band.min_x}+{link_band.max_y}",
            "+repage",
            "-alpha",
            "off",
            "-fx",
            "(b > r + 0.08 && b > g + 0.02 && b > 0.35) ? 1 : 0",
            "-format",
            "%[fx:mean*w*h]",
            "info:",
        ],
    )
    return (
        link_band.min_x,
        link_band.max_x,
        link_band.min_y,
        link_band.max_y,
        link_band.area,
        bottom_pixels,
    )


def acceptance_artifact_link_underline_errors_from_values(
    observed: dict[str, tuple[int, int, int, int, int, float]],
    thresholds: dict[str, tuple[int, int, int, int, int, int, int, int, int]],
) -> list[str]:
    errors: list[str] = []
    for label, (
        min_x,
        max_x,
        min_y,
        max_y,
        min_width,
        max_width,
        min_area,
        max_area,
        min_bottom_pixels,
    ) in thresholds.items():
        if label not in observed:
            continue
        observed_min_x, observed_max_x, observed_min_y, observed_max_y, area, bottom_pixels = (
            observed[label]
        )
        width = observed_max_x - observed_min_x + 1
        if observed_min_x < min_x or observed_max_x > max_x:
            errors.append(
                "acceptance artifact link underline has wrong x position: "
                f"{label} x={observed_min_x}..{observed_max_x}, "
                f"expected inside {min_x}..{max_x}."
            )
        if observed_min_y < min_y or observed_max_y > max_y:
            errors.append(
                "acceptance artifact link underline has wrong y position: "
                f"{label} y={observed_min_y}..{observed_max_y}, "
                f"expected inside {min_y}..{max_y}."
            )
        if width < min_width or width > max_width:
            errors.append(
                "acceptance artifact link underline has wrong link underline width: "
                f"{label} width={width}, expected {min_width}..{max_width}."
            )
        if area < min_area or area > max_area:
            errors.append(
                "acceptance artifact link underline has wrong blue component area: "
                f"{label} area={area}, expected {min_area}..{max_area}."
            )
        if bottom_pixels < min_bottom_pixels:
            errors.append(
                "acceptance artifact link underline has too few underline pixels: "
                f"{label} bottom_pixels={bottom_pixels:.0f}, expected at least "
                f"{min_bottom_pixels}."
            )
    return errors


def acceptance_artifact_hover_highlight_errors(magick: str) -> list[str]:
    observed = {
        f"{before.relative_to(ROOT)} -> {after.relative_to(ROOT)}": (
            acceptance_artifact_hover_highlight_values(magick, before, after)
        )
        for before, after, *_ in REQUIRED_ACCEPTANCE_HOVER_HIGHLIGHT_BANDS
        if before.is_file()
        and before.stat().st_size > 0
        and after.is_file()
        and after.stat().st_size > 0
    }
    thresholds = {
        f"{before.relative_to(ROOT)} -> {after.relative_to(ROOT)}": (
            min_x,
            max_x,
            min_y,
            max_y,
            min_width,
            max_width,
            min_height,
            max_height,
            min_area,
            max_area,
        )
        for (
            before,
            after,
            min_x,
            max_x,
            min_y,
            max_y,
            min_width,
            max_width,
            min_height,
            max_height,
            min_area,
            max_area,
        ) in REQUIRED_ACCEPTANCE_HOVER_HIGHLIGHT_BANDS
    }
    return acceptance_artifact_hover_highlight_errors_from_values(
        observed, thresholds
    )


def acceptance_artifact_hover_highlight_values(
    magick: str, before: pathlib.Path, after: pathlib.Path
) -> tuple[int, int, int, int, int]:
    result = subprocess.run(
        [
            magick,
            str(after),
            str(before),
            "-alpha",
            "off",
            "-compose",
            "difference",
            "-composite",
            "-fx",
            "((r+g+b) > 0.03) ? 1 : 0",
            "-define",
            "connected-components:verbose=true",
            "-connected-components",
            "8",
            "null:",
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return (0, 0, 0, 0, 0)
    bands = connected_component_bands_from_text(result.stdout)
    highlight = max(bands, key=lambda band: band.area, default=None)
    if highlight is None:
        return (0, 0, 0, 0, 0)
    return (
        highlight.min_x,
        highlight.max_x,
        highlight.min_y,
        highlight.max_y,
        highlight.area,
    )


def acceptance_artifact_hover_highlight_errors_from_values(
    observed: dict[str, tuple[int, int, int, int, int]],
    thresholds: dict[str, tuple[int, int, int, int, int, int, int, int, int, int]],
) -> list[str]:
    errors: list[str] = []
    for label, (
        min_x,
        max_x,
        min_y,
        max_y,
        min_width,
        max_width,
        min_height,
        max_height,
        min_area,
        max_area,
    ) in thresholds.items():
        if label not in observed:
            continue
        observed_min_x, observed_max_x, observed_min_y, observed_max_y, area = (
            observed[label]
        )
        width = observed_max_x - observed_min_x + 1
        height = observed_max_y - observed_min_y + 1
        if observed_min_x < min_x or observed_max_x > max_x:
            errors.append(
                "acceptance artifact hover highlight has wrong hover highlight x position: "
                f"{label} x={observed_min_x}..{observed_max_x}, "
                f"expected inside {min_x}..{max_x}."
            )
        if observed_min_y < min_y or observed_max_y > max_y:
            errors.append(
                "acceptance artifact hover highlight has wrong hover highlight y position: "
                f"{label} y={observed_min_y}..{observed_max_y}, "
                f"expected inside {min_y}..{max_y}."
            )
        if width < min_width or width > max_width:
            errors.append(
                "acceptance artifact hover highlight has wrong hover highlight width: "
                f"{label} width={width}, expected {min_width}..{max_width}."
            )
        if height < min_height or height > max_height:
            errors.append(
                "acceptance artifact hover highlight has wrong hover highlight height: "
                f"{label} height={height}, expected {min_height}..{max_height}."
            )
        if area < min_area or area > max_area:
            errors.append(
                "acceptance artifact hover highlight has wrong hover highlight area: "
                f"{label} area={area}, expected {min_area}..{max_area}."
            )
    return errors


def acceptance_artifact_source_freshness_errors() -> list[str]:
    artifact_mtimes: dict[str, float] = {}
    for path in required_acceptance_file_paths():
        if path.is_file() and path.stat().st_size > 0:
            artifact_mtimes[path_label(path)] = path.stat().st_mtime
    source_mtimes: dict[str, float] = {}
    for path in required_acceptance_source_integrity_paths():
        if (
            path.is_file()
            and path.stat().st_size > 0
            and acceptance_source_path_requires_fresh_artifact(path)
        ):
            source_mtimes[path_label(path)] = path.stat().st_mtime
    return acceptance_artifact_source_freshness_errors_from_mtimes(
        artifact_mtimes=artifact_mtimes,
        source_mtimes=source_mtimes,
    )


def acceptance_source_path_requires_fresh_artifact(path: pathlib.Path) -> bool:
    repo = source_repo_root(path)
    if repo is None:
        return True
    try:
        relative_path = path.resolve().relative_to(repo.resolve())
    except ValueError:
        return True
    result = subprocess.run(
        ["git", "status", "--porcelain=v1", "--", str(relative_path)],
        cwd=repo,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
    )
    if result.returncode != 0:
        return True
    return bool(result.stdout.strip())


def source_repo_root(path: pathlib.Path) -> pathlib.Path | None:
    resolved = path.resolve()
    try:
        resolved.relative_to(ROOT.resolve())
    except ValueError:
        return None
    return ROOT


def acceptance_artifact_source_freshness_errors_from_mtimes(
    *, artifact_mtimes: dict[str, float], source_mtimes: dict[str, float]
) -> list[str]:
    if not artifact_mtimes or not source_mtimes:
        return []
    newest_source_mtime = max(source_mtimes.values())
    stale_artifacts = [
        path for path, mtime in artifact_mtimes.items() if mtime < newest_source_mtime
    ]
    if not stale_artifacts:
        return []
    newest_sources = [
        path for path, mtime in source_mtimes.items() if mtime == newest_source_mtime
    ]
    stale_preview = ", ".join(stale_artifacts[:5])
    if len(stale_artifacts) > 5:
        stale_preview += f", ... ({len(stale_artifacts)} total)"
    return [
        "acceptance artifact is older than required source file(s): "
        f"newest source {', '.join(newest_sources[:3])}; stale artifact(s) "
        f"{stale_preview}. Regenerate artifacts with "
        "`/opt/homebrew/bin/rtk just storybook-release-acceptance-artifacts`."
    ]


def acceptance_artifact_freshness_errors(confirmed_at: str) -> list[str]:
    path_mtimes: dict[str, float] = {}
    for path in required_acceptance_freshness_paths():
        if path.is_file() and path.stat().st_size > 0:
            path_mtimes[path_label(path)] = path.stat().st_mtime
    return acceptance_artifact_freshness_errors_from_mtimes(
        confirmed_at, path_mtimes
    )


def acceptance_artifact_freshness_errors_from_mtimes(
    confirmed_at: str, path_mtimes: dict[str, float]
) -> list[str]:
    confirmed_datetime = parse_confirmed_at(confirmed_at)
    if confirmed_datetime is None:
        return []
    newer_artifacts: list[str] = []
    for path, mtime in path_mtimes.items():
        modified_at = datetime.fromtimestamp(mtime, tz=timezone.utc)
        if modified_at > confirmed_datetime:
            newer_artifacts.append(path)
    if not newer_artifacts:
        return []
    return [
        "acceptance evidence confirmed_at is older than review artifact/source file(s): "
        + ", ".join(newer_artifacts)
        + ". Regenerate artifacts, review `just storybook`, then record acceptance time."
    ]


def parse_confirmed_at(value: str) -> datetime | None:
    if not CONFIRMED_AT_RE.fullmatch(value):
        return None
    normalized = value.replace("Z", "+00:00")
    return datetime.fromisoformat(normalized).astimezone(timezone.utc)


def confirmed_at_future_error(
    confirmed_at: str, *, now: datetime | None = None
) -> str | None:
    confirmed_datetime = parse_confirmed_at(confirmed_at)
    if confirmed_datetime is None:
        return None
    current = now or datetime.now(timezone.utc)
    if confirmed_datetime > current:
        return "acceptance evidence confirmed_at must not be in the future."
    return None


def png_dimension_error(
    data: bytes, *, min_width: int, min_height: int, label: str
) -> str | None:
    if len(data) < 24 or data[:8] != b"\x89PNG\r\n\x1a\n":
        return f"acceptance artifact is not a PNG: {label}."
    if data[12:16] != b"IHDR":
        return f"acceptance artifact PNG is missing IHDR chunk: {label}."
    width, height = struct.unpack(">II", data[16:24])
    if width < min_width or height < min_height:
        return (
            "acceptance artifact PNG is too small: "
            f"{label} is {width}x{height}, expected at least {min_width}x{min_height}."
        )
    return None


def ppm_dimension_error(
    data: bytes, *, expected_width: int, expected_height: int, label: str
) -> str | None:
    header = ppm_header(data)
    if header is None:
        return f"acceptance artifact is not a P6 PPM: {label}."
    magic, width, height, max_value, pixel_offset = header
    if magic != "P6":
        return f"acceptance artifact is not a P6 PPM: {label}."
    if width != expected_width or height != expected_height:
        return (
            "acceptance artifact PPM has wrong dimensions: "
            f"{label} is {width}x{height}, expected {expected_width}x{expected_height}."
        )
    if max_value != 255:
        return (
            "acceptance artifact PPM has unsupported max value: "
            f"{label} has {max_value}, expected 255."
        )
    expected_bytes = expected_width * expected_height * 3
    if len(data) - pixel_offset < expected_bytes:
        return (
            "acceptance artifact PPM pixel data is truncated: "
            f"{label} has {len(data) - pixel_offset} bytes, expected at least "
            f"{expected_bytes}."
        )
    return None


def ppm_header(data: bytes) -> tuple[str, int, int, int, int] | None:
    tokens: list[tuple[bytes, int]] = []
    index = 0
    while len(tokens) < 4 and index < len(data):
        while index < len(data) and data[index] in b" \t\r\n":
            index += 1
        if index < len(data) and data[index] == ord("#"):
            while index < len(data) and data[index] not in b"\r\n":
                index += 1
            continue
        start = index
        while index < len(data) and data[index] not in b" \t\r\n":
            index += 1
        if start == index:
            break
        tokens.append((data[start:index], index))
    if len(tokens) != 4:
        return None
    try:
        magic = tokens[0][0].decode("ascii")
        width = int(tokens[1][0])
        height = int(tokens[2][0])
        max_value = int(tokens[3][0])
    except (UnicodeDecodeError, ValueError):
        return None
    pixel_offset = tokens[3][1]
    if pixel_offset < len(data) and data[pixel_offset] in b" \t\r\n":
        pixel_offset += 1
    return (magic, width, height, max_value, pixel_offset)


def acceptance_artifact_manifest_errors() -> list[str]:
    manifest_texts: list[str] = []
    for manifest in (
        ROOT / "target/acceptance/kdv-storybook-acceptance-artifacts.sha256",
        ROOT / "target/acceptance/kdv-storybook-live-acceptance-artifacts.sha256",
    ):
        if manifest.is_file() and manifest.stat().st_size > 0:
            manifest_texts.append(manifest.read_text(encoding="utf-8"))
    if not manifest_texts:
        return []
    return acceptance_artifact_manifest_errors_from_text(
        "\n".join(manifest_texts),
        expected_digests=acceptance_artifact_expected_digests(),
    )


def acceptance_artifact_manifest_errors_from_text(
    manifest_text: str, *, expected_digests: dict[str, str]
) -> list[str]:
    manifest_rows = parse_acceptance_artifact_manifest(manifest_text)
    missing = [
        required
        for required in missing_acceptance_artifact_manifest_rows(manifest_text)
    ]
    errors: list[str] = []
    if missing:
        errors.append(
            "acceptance artifact manifest missing required artifact row(s): "
            + ", ".join(missing)
            + "."
        )
    malformed = [
        path
        for path, digest in manifest_rows.items()
        if path in expected_digests and not re.fullmatch(r"[0-9a-fA-F]{64}", digest)
    ]
    if malformed:
        errors.append(
            "acceptance artifact manifest has malformed checksum row(s): "
            + ", ".join(malformed)
            + "."
        )
    mismatched = [
        path
        for path, expected_digest in expected_digests.items()
        if path in manifest_rows and manifest_rows[path].lower() != expected_digest
    ]
    if mismatched:
        errors.append(
            "acceptance artifact manifest checksum mismatch for artifact(s): "
            + ", ".join(mismatched)
            + "."
        )
    return errors


def parse_acceptance_artifact_manifest(manifest_text: str) -> dict[str, str]:
    rows: dict[str, str] = {}
    for line in manifest_text.splitlines():
        parts = line.strip().split(None, 1)
        if len(parts) == 2:
            rows[parts[1]] = parts[0]
    return rows


def acceptance_artifact_expected_digests() -> dict[str, str]:
    digests: dict[str, str] = {}
    for path in required_acceptance_manifest_paths():
        if path.name == "kdv-storybook-acceptance-artifacts.sha256":
            continue
        if path.is_file() and path.stat().st_size > 0:
            relative = str(path.relative_to(ROOT))
            digests[relative] = hashlib.sha256(path.read_bytes()).hexdigest()
    return digests


def missing_acceptance_artifact_manifest_rows(manifest_text: str) -> list[str]:
    return [
        str(path.relative_to(ROOT))
        for path in required_acceptance_manifest_paths()
        if path.name
        not in {
            "kdv-storybook-acceptance-artifacts.sha256",
            "kdv-storybook-live-acceptance-artifacts.sha256",
        }
        and str(path.relative_to(ROOT)) not in manifest_text
    ]


def required_acceptance_file_paths() -> tuple[pathlib.Path, ...]:
    return (
        REQUIRED_ACCEPTANCE_ARTIFACT_PATHS
        + REQUIRED_LIVE_ACCEPTANCE_ARTIFACT_PATHS
        + REQUIRED_ACCEPTANCE_SOURCE_ARTIFACT_PATHS
    )


def required_acceptance_manifest_paths() -> tuple[pathlib.Path, ...]:
    return required_acceptance_file_paths()


def required_acceptance_freshness_paths() -> tuple[pathlib.Path, ...]:
    return required_acceptance_file_paths() + required_acceptance_source_integrity_paths()


def path_label(path: pathlib.Path) -> str:
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def section_text(markdown: str, heading: str) -> str:
    lines = markdown.splitlines()
    start = None
    for index, line in enumerate(lines):
        if line.strip() == heading:
            start = index + 1
            break
    if start is None:
        return ""
    end = len(lines)
    for index in range(start, len(lines)):
        if lines[index].startswith("## "):
            end = index
            break
    return "\n".join(lines[start:end])


if __name__ == "__main__":
    raise SystemExit(main())
