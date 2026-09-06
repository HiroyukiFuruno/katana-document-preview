use super::{StorybookScoreEvidence, StorybookScoreReport};

#[test]
fn storybook_score_gate_uses_minimum_category_score() {
    let report = StorybookScoreReport {
        visual_score: 100,
        semantic_score: 99,
        interaction_score: 94,
        performance_score: 100,
        evidence: passing_evidence(),
    };

    assert_eq!(94, report.final_score());
    assert!(!report.is_pass());
}

#[test]
fn storybook_score_gate_passes_only_when_all_categories_are_at_least_95() {
    let report = StorybookScoreReport {
        visual_score: 95,
        semantic_score: 95,
        interaction_score: 95,
        performance_score: 95,
        evidence: passing_evidence(),
    };

    assert_eq!(95, report.final_score());
    assert!(report.is_pass());
}

#[test]
fn storybook_score_gate_rejects_scores_without_reference_and_runtime_evidence() {
    let report = StorybookScoreReport {
        visual_score: 100,
        semantic_score: 100,
        interaction_score: 100,
        performance_score: 100,
        evidence: StorybookScoreEvidence::default(),
    };

    assert!(!report.is_pass());
    assert_eq!(
        vec![
            "visual:katana-reference",
            "semantic:export-reference",
            "interaction:runtime-actions",
            "interaction:os-clipboard",
            "performance:budget-gate",
        ],
        report.missing_evidence()
    );
}

#[test]
fn storybook_check_recipe_keeps_all_score_category_gates() -> Result<(), Box<dyn std::error::Error>>
{
    let justfile = std::fs::read_to_string(workspace_root()?.join("Justfile"))?;
    let storybook_check = recipe_body(&justfile, "storybook-check")?;
    for required in ["storybook-registry-consumer-suite", "storybook-score-check"] {
        assert!(
            storybook_check.contains(required),
            "storybook-check must include `{required}`"
        );
    }

    let score_check = recipe_body(&justfile, "storybook-score-check")?;
    for required in [
        "katana_reference_artifacts",
        "storybook_score_visual_uses_katana_preview_crop_reference",
        "storybook_sample_top_local_text_metrics_match_katana_reference",
        "storybook_score_visual_uses_katana_sample_diagrams_crop_reference",
        "storybook_sample_diagrams_local_svg_metrics_match_katana_reference",
        "storybook_score_visual_uses_katana_export_png_reference",
        "storybook_score_gate_keeps_interactive_controls_disabled_for_reference_comparison",
        "storybook_preview_crop_score_excludes_storybook_overlay_controls",
        "storybook_score_export_surface_excludes_overlay_controls",
        "storybook_score_audit",
        "fixture_score_matrix",
        "surface_equivalence",
        "storybook_frame_matches_export_surface_for_katana_viewer",
        "storybook_frame_matches_export_surface_for_katana_viewer -- --ignored --exact --test-threads=1",
        "storybook_frame_matches_export_surface_for_katana_viewer_diagrams -- --ignored --exact --test-threads=1",
        "storybook-entrypoint-check",
        "storybook-content-check-core",
        "storybook-emoji-check-core",
        "storybook-treeview-check-core",
        "storybook-interaction-check-core",
        "storybook-coordinate-contract-check-core",
        "storybook-hover-contract-check-core",
        "storybook-clipboard-smoke",
        "storybook-clipboard-keyboard-smoke",
        "storybook-clipboard-drag-smoke",
        "storybook-selection-contract-check-core",
        "storybook-selection-screenshot-smoke",
        "storybook-settings-contract-check-core",
        "storybook-window-smoke",
        "katana_intro_text_keeps_readable_frame_band_heights",
        "katana_language_link_underline_reaches_frame_pixels",
        "direct_html_margin_left_fixture_reaches_frame_pixels",
        "storybook-window-sidebar-screenshot-smoke",
        "storybook-window-sidebar-narrow-screenshot-smoke",
        "storybook-window-sidebar-large-screenshot-smoke",
        "storybook-window-hover-screenshot-smoke",
        "storybook-window-hover-wide-screenshot-smoke",
        "direct/html-margin-left.html",
        "storybook-window-footnote-screenshot-smoke",
        "storybook-window-selection-screenshot-smoke",
        "storybook-window-code-copy-screenshot-smoke",
        "storybook-kuc-visual-check-core",
        "storybook-performance-check-core",
        "storybook-media-control-clickability-check-full-core",
        "storybook-code-block-check-core",
        "storybook-diagram-load-check-core",
        "storybook-window-diagram-screenshot-smoke",
        "storybook-window-drawio-diagram-screenshot-smoke",
        "storybook-link-footnote-check-core",
        "storybook-slideshow-check-core",
        "storybook-slideshow-screenshot-smoke",
        "storybook-window-slideshow-screenshot-smoke",
        "storybook-search-check-core",
        "storybook-task-checkbox-check-core",
        "storybook-accordion-check-core",
        "storybook-toc-check-core",
        "storybook-unresolved-metadata-check-core",
        "storybook-scroll-resize-contract-check",
    ] {
        assert!(
            score_check.contains(required),
            "storybook-score-check must include `{required}`"
        );
    }
    Ok(())
}

#[test]
fn storybook_score_gate_keeps_interactive_controls_disabled_for_reference_comparison()
-> Result<(), Box<dyn std::error::Error>> {
    let sources = score_surface_sources(&workspace_root()?)?;

    assert_score_controls_disabled(&sources);
    assert_overlay_controls_rejected(&sources);
    Ok(())
}

#[test]
fn storybook_treeview_core_gate_keeps_window_hover_and_collapse_pixel_contracts()
-> Result<(), Box<dyn std::error::Error>> {
    let justfile = std::fs::read_to_string(workspace_root()?.join("Justfile"))?;
    let treeview_check = recipe_body(&justfile, "storybook-treeview-check-core")?;

    for required in [
        "rendered_file_tree_directory_hover_paints_kuc_row_background",
        "rendered_file_tree_directory_click_collapses_visible_rows_and_pixels",
    ] {
        assert!(
            treeview_check.contains(required),
            "storybook-treeview-check-core must include `{required}`"
        );
    }
    Ok(())
}

#[test]
fn release_verify_uses_the_active_target_release_contract() -> Result<(), Box<dyn std::error::Error>>
{
    let root = workspace_root()?;
    assert_active_release_contract(&root)?;
    assert_legacy_storybook_acceptance_contract(&root)?;
    Ok(())
}

fn assert_active_release_contract(
    root: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let justfile = std::fs::read_to_string(root.join("Justfile"))?;
    let release_verify = recipe_body(&justfile, "release-verify")?;
    let release_contract = recipe_body(&justfile, "release-contract-check")?;

    assert_release_contract_recipe(root, release_verify, release_contract);
    let release_contract_script =
        std::fs::read_to_string(root.join("scripts/release/verify-release-contract.py"))?;
    assert_contains_all(
        "multi-format release contract",
        &release_contract_script,
        &[
            "multi-format-viewer",
            "SELECTED_ENGINES",
            "hayro",
            "office2pdf",
            "ironcalc",
            "FORBIDDEN_ENGINE_PACKAGES",
            "KUC_VERSION",
            "katana-render-runtime",
            "registry+https://github.com/rust-lang/crates.io-index",
            "browser_session_adapter_contract",
            "KRR_CHROME_BIN",
        ],
    );
    Ok(())
}

fn assert_legacy_storybook_acceptance_contract(
    root: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let justfile = std::fs::read_to_string(root.join("Justfile"))?;
    let release_dod = recipe_body(&justfile, "release-dod-check")?;
    assert_legacy_release_dod_recipe(root, release_dod);
    let release_dod_script =
        std::fs::read_to_string(root.join("scripts/release/assert-viewer-recovery-dod.py"))?;
    let acceptance_doc = std::fs::read_to_string(root.join(
        "openspec/changes/v0-2-0-markdown-viewer-kuc-integration/storybook-user-acceptance.md",
    ))?;
    assert_contains_all(
        "release DoD acceptance contract",
        &release_dod_script,
        RELEASE_DOD_REQUIRED_SNIPPETS,
    );
    assert_contains_all(
        "storybook user acceptance contract",
        &acceptance_doc,
        ACCEPTANCE_DOC_REQUIRED_SNIPPETS,
    );
    Ok(())
}

#[test]
fn ci_workflows_pin_linux_font_and_graphviz_runtime_for_katana_reference_scores()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let ci = std::fs::read_to_string(root.join(".github/workflows/test-and-build.yml"))?;
    let preflight = std::fs::read_to_string(root.join(".github/workflows/release-preflight.yml"))?;
    let release = std::fs::read_to_string(root.join(".github/workflows/release.yml"))?;
    assert_linux_emoji_install_contract(&root, &ci, &preflight, &release)?;

    assert_linux_runtime_requirements(
        "CI PlantUML runtime",
        &ci,
        CI_PLANTUML_RUNTIME_REQUIRED_SNIPPETS,
    );
    assert_linux_runtime_requirements(
        "release preflight PlantUML runtime",
        &preflight,
        PREFLIGHT_PLANTUML_RUNTIME_REQUIRED_SNIPPETS,
    );
    assert_linux_runtime_requirements(
        "Release workflow acceptance runtime",
        &release,
        RELEASE_WORKFLOW_RUNTIME_REQUIRED_SNIPPETS,
    );
    Ok(())
}

#[test]
fn just_test_recipe_isolates_platform_dependent_storybook_visual_tests()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let justfile = std::fs::read_to_string(root.join("Justfile"))?;
    let test_recipe = recipe_body(&justfile, "test")?;

    assert_contains_all(
        "Justfile test recipe",
        test_recipe,
        JUST_TEST_STORYBOOK_ISOLATION_REQUIRED_SNIPPETS,
    );
    assert!(
        !test_recipe.lines().any(|line| {
            line.trim() == "{{CARGO}} test --workspace --all-targets --all-features --locked"
        }),
        "Justfile test recipe must not run kdv-storybook platform-dependent tests in the generic workspace process"
    );
    Ok(())
}

#[test]
fn release_dod_tracks_every_generated_acceptance_source_artifact()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let acceptance_artifact_script = std::fs::read_to_string(
        root.join("scripts/release/generate-storybook-acceptance-artifact.sh"),
    )?;
    let release_dod_script =
        std::fs::read_to_string(root.join("scripts/release/assert-viewer-recovery-dod.py"))?;

    let generated = acceptance_script_required_source_artifacts(&acceptance_artifact_script);
    let required = release_dod_required_source_artifacts(&release_dod_script);
    let missing: Vec<_> = generated.difference(&required).cloned().collect();

    assert!(
        !generated.is_empty(),
        "acceptance artifact script must expose its source screenshot required list"
    );
    assert!(
        missing.is_empty(),
        "release DoD must require every source screenshot generated by storybook acceptance artifacts; missing: {missing:?}"
    );
    Ok(())
}

#[test]
fn storybook_release_acceptance_artifacts_runs_static_and_live_artifacts()
-> Result<(), Box<dyn std::error::Error>> {
    let justfile = std::fs::read_to_string(workspace_root()?.join("Justfile"))?;
    let recipe = recipe_body(&justfile, "storybook-release-acceptance-artifacts")?;

    for required in [
        "storybook-acceptance-artifact",
        "storybook-live-acceptance-artifact",
    ] {
        assert!(
            recipe.contains(required),
            "storybook-release-acceptance-artifacts must include `{required}`"
        );
    }

    Ok(())
}

#[test]
fn storybook_acceptance_artifact_is_reproducible_from_just()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let justfile = std::fs::read_to_string(root.join("Justfile"))?;
    let recipe = recipe_body(&justfile, "storybook-acceptance-artifact")?;

    assert_contains_all(
        "storybook-acceptance-artifact recipe",
        recipe,
        ACCEPTANCE_ARTIFACT_RECIPE_REQUIRED_SNIPPETS,
    );

    let script = std::fs::read_to_string(
        root.join("scripts/release/generate-storybook-acceptance-artifact.sh"),
    )?;
    assert_contains_all(
        "acceptance artifact script",
        &script,
        ACCEPTANCE_ARTIFACT_SCRIPT_REQUIRED_SNIPPETS,
    );
    assert!(
        !script.contains("-resize 330x450"),
        "diagram control review artifact must not upscale the native icon crop"
    );
    assert_hover_screenshot_artifact_contract(&root)?;

    Ok(())
}

#[test]
fn storybook_live_acceptance_artifact_is_reproducible_from_just()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let justfile = std::fs::read_to_string(root.join("Justfile"))?;
    let recipe = recipe_body(&justfile, "storybook-live-acceptance-artifact")?;

    assert_contains_all(
        "storybook-live-acceptance-artifact recipe",
        recipe,
        LIVE_ACCEPTANCE_RECIPE_REQUIRED_SNIPPETS,
    );

    let script = std::fs::read_to_string(
        root.join("scripts/release/generate-storybook-live-acceptance-artifact.sh"),
    )?;
    assert_contains_all(
        "live acceptance artifact script",
        &script,
        LIVE_ACCEPTANCE_SCRIPT_REQUIRED_SNIPPETS,
    );
    assert_live_acceptance_doc_contract(&root)?;

    Ok(())
}

#[test]
fn release_publish_depends_on_release_check() -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let justfile = std::fs::read_to_string(root.join("Justfile"))?;
    let publish_script = std::fs::read_to_string(root.join("scripts/release/publish-crates.sh"))?;
    let release_publish = recipe_body(&justfile, "release-publish")?;

    assert_release_publish_recipe(&root, release_publish);
    assert_publish_script_requires_clean_token(&publish_script);
    Ok(())
}

#[test]
fn release_publish_retries_registry_failures_from_the_immutable_tag()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let publish_script = std::fs::read_to_string(root.join("scripts/release/publish-crates.sh"))?;
    let recovery =
        std::fs::read_to_string(root.join(".github/workflows/release-publish-retry.yml"))?;

    assert_contains_all(
        "publish retry script",
        &publish_script,
        &[
            "PUBLISH_ATTEMPTS:-3",
            "PUBLISH_RETRY_DELAY_SECONDS:-10",
            "if is_published katana-document-viewer",
            "if cargo publish",
            "sleep \"${delay}\"",
        ],
    );
    assert_contains_all(
        "release publish recovery workflow",
        &recovery,
        &[
            "path: release-source",
            "ref: ${{ inputs.version }}",
            "git -C release-source rev-parse HEAD",
            "../release-tools/scripts/release/publish-crates.sh",
        ],
    );
    Ok(())
}

#[test]
fn release_scripts_do_not_depend_on_obsolete_preview_egui_package()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let obsolete_script = root.join("scripts/release/verify-internal-dependencies.sh");
    assert!(
        !obsolete_script.exists(),
        "obsolete release script must not check removed katana-document-preview-egui package"
    );

    for script in [
        "scripts/release/publish-crates.sh",
        "scripts/release/verify-version.sh",
        "scripts/release/verify-release-target.py",
        "scripts/release/assert-viewer-recovery-dod.py",
    ] {
        let content = std::fs::read_to_string(root.join(script))?;
        assert!(
            !content.contains("katana-document-preview-egui")
                && !content.contains("katana-document-preview ="),
            "{script} must not depend on obsolete katana-document-preview-egui release metadata"
        );
    }

    Ok(())
}

#[test]
fn document_surface_boundary_requires_the_registry_kuc_artifact()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let script = std::fs::read_to_string(root.join("scripts/document-surface-boundary-check.sh"))?;

    assert_registry_kuc_artifact_contract(&script);

    Ok(())
}

#[test]
fn document_surface_boundary_checks_kuc_optional_backends_from_the_resolved_graph()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let script = std::fs::read_to_string(root.join("scripts/document-surface-boundary-check.sh"))?;

    assert_kuc_optional_backend_contract(&script)?;

    Ok(())
}

fn assert_registry_kuc_artifact_contract(script: &str) {
    assert_contains_all(
        "document-surface-boundary-check.sh",
        script,
        &[
            "resolve_cargo_package_root",
            "metadata --locked --format-version 1",
            "kuc_core_source_root",
            "registry+",
            "0.3.7",
            "KUC_ROOT overrides are forbidden",
        ],
    );
    assert!(
        !script.contains("KUC_ROOT is missing"),
        "KUC boundary check must use cargo dependency source when the sibling KUC repo is absent"
    );
    assert!(
        !script.contains("kuc_storybook_source_root")
            && !script.contains("katana-ui-core-storybook --locked"),
        "KUC boundary check must not depend on the obsolete Git Storybook package"
    );
    assert!(
        script.contains("git|path") && script.contains("KUC_ROOT overrides are forbidden"),
        "KUC boundary check must fail closed for Git/path and local-root overrides"
    );
}

fn assert_kuc_optional_backend_contract(script: &str) -> Result<(), Box<dyn std::error::Error>> {
    assert_contains_all(
        "document-surface-boundary-check.sh",
        script,
        &[
            "resolve_cargo_package_tree katana-ui-core registry+ 0.3.7",
            "kuc_tree=",
            "check_kuc_core_semantic_boundary \"katana-ui-core\"",
            "check_embedded_kuc_source_boundary",
            "raster-host",
            "pub mod raster_host;",
        ],
    );
    let semantic_boundary_start = script
        .find("check_kuc_core_semantic_boundary() {")
        .ok_or_else(|| std::io::Error::other("KUC semantic boundary function must exist"))?;
    let embedded_boundary_start = script
        .find("check_embedded_kuc_source_boundary() {")
        .ok_or_else(|| std::io::Error::other("embedded KUC source boundary function must exist"))?;
    let semantic_boundary = &script[semantic_boundary_start..embedded_boundary_start];
    assert!(
        !semantic_boundary.contains("$source_pattern"),
        "registry KUC optional backends must be checked from the resolved dependency graph, not dormant source modules"
    );
    Ok(())
}

#[test]
fn storybook_kuc_smoke_uses_the_registry_resolved_storybook_consumer()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let script = std::fs::read_to_string(root.join("scripts/storybook-kuc-smoke.sh"))?;

    assert_contains_all(
        "storybook-kuc-smoke.sh",
        &script,
        &[
            "test -p kdv-storybook --locked document_viewer",
            "test -p katana-document-viewer --locked --lib document_surface",
            "test -p katana-document-viewer --locked asset_loader",
            "test -p katana-document-viewer --locked direct_",
        ],
    );
    assert!(
        !script.contains("../katana-ui-core")
            && !script.contains("katana-ui-core-storybook --locked"),
        "storybook smoke must exercise the registry-resolved KDV consumer, not a sibling KUC checkout"
    );

    Ok(())
}

#[test]
fn storybook_link_footnote_gate_keeps_real_pointer_jump_tests()
-> Result<(), Box<dyn std::error::Error>> {
    let justfile = std::fs::read_to_string(workspace_root()?.join("Justfile"))?;
    let link_footnote_check = recipe_body(&justfile, "storybook-link-footnote-check-core")?;

    for required in [
        "storybook_window_footnote_reference_click_jumps_to_definition",
        "storybook_window_footnote_backlink_click_jumps_to_reference",
        "storybook_window_sample_footnote",
    ] {
        assert!(
            link_footnote_check.contains(required),
            "storybook-link-footnote-check-core must include `{required}`"
        );
    }
    Ok(())
}

#[test]
fn storybook_slideshow_screenshot_smoke_keeps_close_stage_evidence()
-> Result<(), Box<dyn std::error::Error>> {
    let justfile = std::fs::read_to_string(workspace_root()?.join("Justfile"))?;
    let slideshow_smoke = recipe_body(&justfile, "storybook-slideshow-screenshot-smoke")?;
    let window_slideshow_smoke =
        recipe_body(&justfile, "storybook-window-slideshow-screenshot-smoke")?;

    assert!(
        slideshow_smoke.contains("target/kdv-storybook-slideshow-smoke-close.png"),
        "storybook-slideshow-screenshot-smoke must verify close stage screenshot"
    );
    assert!(
        window_slideshow_smoke.contains("target/kdv-storybook-window-slideshow-smoke-close.png"),
        "storybook-window-slideshow-screenshot-smoke must verify close stage screenshot"
    );
    Ok(())
}

#[test]
fn storybook_kuc_visual_gate_keeps_presentation_downscale_quality_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let justfile = std::fs::read_to_string(workspace_root()?.join("Justfile"))?;
    let visual_check = recipe_body(&justfile, "storybook-kuc-visual-check-core")?;

    assert!(
        visual_check.contains("presentation"),
        "storybook-kuc-visual-check-core must include KUC presentation downscale quality tests"
    );
    Ok(())
}

#[test]
fn storybook_media_control_gate_keeps_diagram_min_height_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let justfile = std::fs::read_to_string(workspace_root()?.join("Justfile"))?;
    let media_check = recipe_body(&justfile, "storybook-media-control-clickability-check-core")?;
    let full_media_check = recipe_body(
        &justfile,
        "storybook-media-control-clickability-check-full-core",
    )?;
    let diagram_smoke = recipe_body(&justfile, "storybook-window-diagram-screenshot-smoke")?;
    let drawio_smoke = recipe_body(
        &justfile,
        "storybook-window-drawio-diagram-screenshot-smoke",
    )?;

    assert!(
        media_check.contains("diagram_controls_keep_katana_min_container_height_for_short_surface"),
        "storybook-media-control-clickability-check-core must include KUC diagram min-height tests"
    );
    assert!(
        media_check.contains("diagram_controls_use_katana_icon_preset_by_default"),
        "storybook-media-control-clickability-check-core must include KatanA diagram control icon preset tests"
    );
    for (recipe_name, recipe) in [
        (
            "storybook-media-control-clickability-check-core",
            &media_check,
        ),
        (
            "storybook-media-control-clickability-check-full-core",
            &full_media_check,
        ),
    ] {
        assert!(
            recipe.contains("every_diagram_control_uses_katana_icon_asset_source"),
            "{recipe_name} must include KatanA diagram control icon source tests"
        );
        assert!(
            recipe.contains("katana_stroke_icon_tints_white_stroke_to_requested_color"),
            "{recipe_name} must include KatanA stroke icon theme tint tests"
        );
        assert!(
            recipe.contains("kuc_default_diagram_control_icons_match_katana_asset_files"),
            "{recipe_name} must include exact KatanA asset file parity for KUC default diagram icons"
        );
        assert!(
            recipe.contains("diagram_control_icons_render_as_katana_glyphs_not_blocky_squares"),
            "{recipe_name} must include rendered KDV diagram icon pixel-shape tests"
        );
        assert!(
            recipe.contains("diagram_controls_follow_katana_top_and_grid_layout"),
            "{recipe_name} must include KatanA 28px / 2px diagram controller grid layout tests"
        );
    }
    assert!(
        media_check.contains("diagram_control_icons_can_be_overridden_from_host_config"),
        "storybook-media-control-clickability-check-core must include external media control icon override tests"
    );
    assert!(
        media_check
            .contains("storybook_window_diagram_controls_survive_continuous_kuc_hit_sequence_without_asset_reload"),
        "storybook-media-control-clickability-check-core must include continuous KUC hit diagram control tests"
    );
    assert!(
        media_check.contains(
            "storybook_window_fullscreen_diagram_overlay_controls_dispatch_from_kuc_hits"
        ),
        "storybook-media-control-clickability-check-core must include fullscreen overlay KUC hit diagram control tests"
    );
    for recipe in [diagram_smoke, drawio_smoke] {
        assert!(
            !recipe.contains("hover-copy-source") && !recipe.contains("copy-source.png"),
            "real window diagram smoke must match KatanA normal diagram controls without default copy-source overlay"
        );
    }
    Ok(())
}

struct ScoreSurfaceSources {
    preview_crop_tests: String,
    visual_tests: String,
    preview_build_methods: String,
    surface_parity_tests: String,
}

const SCORE_CONTROL_DISABLED_SNIPPETS: &[&str] = &[
    "image_controls_enabled: false",
    "diagram_controls_enabled: false",
    "code_controls_enabled: false",
];

const SCORE_OVERLAY_CONTROL_SNIPPETS: &[&str] = &[
    "copy-code",
    "copy-source",
    "fullscreen",
    "zoom-in",
    "reset-view",
];

const LINUX_EMOJI_FONT_PIN_REQUIRED_SNIPPETS: &[&str] =
    &[".github/scripts/install-linux-render-dependencies.sh"];

const LINUX_EMOJI_INSTALL_SCRIPT_REQUIRED_SNIPPETS: &[&str] = &[
    "emoji_version='2.047-0ubuntu0.24.04.1'",
    "emoji_package_url=\"https://archive.ubuntu.com/ubuntu/pool/main/f/fonts-noto-color-emoji/${emoji_package}_${emoji_version}_all.deb\"",
    "emoji_package_sha256='b102b62ce6a7313315223cff5e052bdf8fc0ad162c9e961ce0dc2202bc139ce2'",
    "emoji_font_sha256='93cdc4ee9aa40e2afceecc63da0ca05ec7aab4bec991ece51a6b52389f48a477'",
    "curl --fail --location --silent --show-error",
    "sudo dpkg --install",
    "fonts-noto-cjk graphviz imagemagick xvfb xclip",
    "KUC_PINNED_LINUX_EMOJI_SHA256=${emoji_font_sha256}",
];

const CI_PLANTUML_RUNTIME_REQUIRED_SNIPPETS: &[&str] = &[
    "JAVA_TOOL_OPTIONS",
    "-Xss16m",
    "-Djava.awt.headless=true",
    "-Djdk.lang.processReaperUseDefaultStackSize=true",
    "Install Graphviz (Ubuntu)",
    ".github/scripts/install-linux-render-dependencies.sh",
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
];

const JUST_TEST_STORYBOOK_ISOLATION_REQUIRED_SNIPPETS: &[&str] = &[
    "{{CARGO}} test --workspace --all-targets --all-features --locked --exclude kdv-storybook",
    "{{CARGO}} test -p kdv-storybook --locked -- --test-threads=1 --skip katana_intro_text_keeps_readable_frame_band_heights",
    "--skip storybook_score_visual_",
    "--skip mouse_click_uses_external_scroll_for_scroll_independent_scene",
    "{{CARGO}} test -p kdv-storybook --locked mouse_click_uses_external_scroll_for_scroll_independent_scene -- --test-threads=1",
];

const PREFLIGHT_PLANTUML_RUNTIME_REQUIRED_SNIPPETS: &[&str] = &[
    "JAVA_TOOL_OPTIONS",
    "-Xss16m",
    "-Djava.awt.headless=true",
    "-Djdk.lang.processReaperUseDefaultStackSize=true",
    ".github/scripts/install-linux-render-dependencies.sh",
    "command -v magick",
    "/usr/local/bin/magick",
    "exec convert",
    "/opt/local/bin/dot",
    "GRAPHVIZ_DOT",
    "release-check",
];

const RELEASE_WORKFLOW_RUNTIME_REQUIRED_SNIPPETS: &[&str] = &[
    "JAVA_TOOL_OPTIONS",
    "-Xss16m",
    "-Djava.awt.headless=true",
    "-Djdk.lang.processReaperUseDefaultStackSize=true",
    "Install diagram test dependencies",
    ".github/scripts/install-linux-render-dependencies.sh",
    "command -v magick",
    "/usr/local/bin/magick",
    "exec convert",
    "/opt/local/bin/dot",
    "GRAPHVIZ_DOT",
    "just VERSION=\"${{ steps.version.outputs.version }}\" release-verify",
];

const RELEASE_DOD_REQUIRED_SNIPPETS: &[&str] = &[
    "storybook-user-acceptance.md",
    "status: accepted",
    "def acceptance_status",
    "ACCEPTANCE_STATUS_RE",
    "acceptance_status(acceptance) != \"accepted\"",
    "def storybook_acceptance_errors",
    "CHECKLIST_ITEM_RE",
    "def acceptance_checklist_items",
    "REQUIRED_ACCEPTANCE_CHECKS",
    "KatanA viewer / export HTML / export PDF",
    "def missing_acceptance_checks",
    "def label_starts_with_required_check",
    "status.lower() != \"x\"",
    "acceptance checklist still has",
    "REQUIRED_EVIDENCE_FIELDS",
    "acceptance evidence missing",
    "CONFIRMED_BY_PLACEHOLDERS",
    "confirmed_by must name the human reviewer",
    "HUMAN_ACCEPTANCE_NOTE_RE",
    "HUMAN_ACCEPTANCE_NOTE_REQUIRED_TOKENS",
    "HUMAN_ACCEPTANCE_NOTE_FORBIDDEN_MARKERS",
    "def human_acceptance_note_errors",
    "def human_acceptance_note_is_valid",
    "acceptance evidence missing human acceptance note",
    "automated/headless-only note",
    "CONFIRMED_AT_RE",
    "confirmed_at must be an ISO-8601 timestamp with timezone",
    "def confirmed_at_future_error",
    "confirmed_at must not be in the future",
    "REQUIRED_MATRIX_HEADING",
    "def missing_acceptance_matrix_rows",
    "def acceptance_matrix_item_cells",
    "acceptance evidence matrix missing required row",
    "def pending_acceptance_matrix_rows",
    "acceptance evidence matrix still has pending human confirmation row",
    "HEADLESS_LIVE_ACCEPTANCE_CURRENT_REQUIRED_TOKENS",
    "HEADLESS_LIVE_ACCEPTANCE_CURRENT_STALE_PATTERNS",
    "def headless_live_acceptance_contract_errors",
    "storybook-user-acceptance.md current section must describe ",
    "headless live-acceptance, not",
    "def weak_acceptance_matrix_rows",
    "def acceptance_matrix_cells",
    "must include accepted or confirmed human status",
    "REQUIRED_ACCEPTANCE_MATRIX_EVIDENCE_TOKENS",
    "def weak_acceptance_matrix_evidence_rows",
    "def acceptance_matrix_rows_by_item",
    "def acceptance_matrix_evidence_text",
    "must include required automated evidence token",
    "REQUIRED_ACCEPTANCE_ARTIFACTS",
    "def missing_acceptance_artifacts",
    "acceptance evidence missing required artifact",
    "REQUIRED_ACCEPTANCE_ARTIFACT_PATHS",
    "text-regression-crops/title-body.png",
    "text-regression-crops/language-link.png",
    "text-regression-crops/direct-html-margin-left.png",
    "text-regression-crops/hover-highlight.png",
    "text-regression-crops/wide-title-link-html.png",
    "text-regression-crops/table-section.png",
    "REQUIRED_ACCEPTANCE_SOURCE_ARTIFACT_PATHS",
    "def required_acceptance_file_paths",
    "def required_acceptance_manifest_paths",
    "kdv-storybook-window-hover-wide-smoke.png",
    "kdv-storybook-window-sidebar-smoke-file-hover.png",
    "kdv-storybook-window-sidebar-smoke-settings-click.png",
    "kdv-storybook-window-sidebar-narrow-smoke-file-click.png",
    "kdv-storybook-window-sidebar-large-smoke-settings-hover.png",
    "kdv-storybook-window-drawio-diagram-smoke.png",
    "kdv-storybook-window-drawio-diagram-smoke-hover-reset-view.png",
    "kdv-storybook-window-drawio-diagram-smoke-trackpad-help.png",
    "kdv-storybook-window-table-smoke.png",
    "kdv-storybook-window-code-copy-smoke-copied.png",
    "kdv-storybook-window-diagram-smoke-hover-zoom-in.png",
    "kdv-storybook-window-diagram-smoke-fullscreen.png",
    "kdv-storybook-window-slideshow-smoke-mode.png",
    "kdv-storybook-window-slideshow-smoke-next.png",
    "kdv-storybook-window-slideshow-smoke-previous.png",
    "kdv-storybook-window-slideshow-smoke-close.png",
    "kdv-storybook-window-footnote-smoke-reference.png",
    "kdv-storybook-window-footnote-smoke-definition.png",
    "REQUIRED_LIVE_ACCEPTANCE_ARTIFACT_PATHS",
    "kdv-storybook-live-interactive.png",
    "kdv-storybook-live-light-toggle.png",
    "kdv-storybook-live-acceptance.log",
    "kdv-storybook-live-acceptance-artifacts.sha256",
    "def missing_acceptance_artifact_files",
    "def acceptance_artifact_file_errors",
    "def acceptance_artifact_file_errors_from_parts",
    "artifact_file_errors = acceptance_artifact_file_errors",
    "include_source_integrity=False",
    "release acceptance artifact:",
    "acceptance artifact file missing or empty",
    "REQUIRED_ACCEPTANCE_PNG_ARTIFACTS",
    "text-regression-crops/diagram-control-icons.png",
    "100,\n        140,",
    "REQUIRED_ACCEPTANCE_SOURCE_PNG_ARTIFACTS",
    "REQUIRED_ACCEPTANCE_SOURCE_PNG_MIN_DIMENSION",
    "def acceptance_artifact_png_errors",
    "def png_dimension_error",
    "acceptance artifact PNG is too small",
    "acceptance artifact PNG is missing IHDR chunk",
    "REQUIRED_ACCEPTANCE_PPM_ARTIFACTS",
    "def acceptance_artifact_ppm_errors",
    "def ppm_dimension_error",
    "def ppm_header",
    "acceptance artifact PPM scanner must reject wrong dimensions",
    "acceptance artifact PPM scanner must reject truncated data",
    "acceptance artifact PPM has wrong dimensions",
    "REQUIRED_ACCEPTANCE_VISUAL_METRICS",
    "REQUIRED_ACCEPTANCE_REFERENCE_PAIR_DIFF_METRICS",
    "REQUIRED_ACCEPTANCE_REFERENCE_TEXT_CONTRAST_METRICS",
    "def acceptance_artifact_visual_metric_errors",
    "def acceptance_artifact_visual_metric_errors_from_values",
    "def magick_binary",
    "accepted artifact file scanner must include visual metric errors",
    "acceptance artifact visual metric has too few colors",
    "acceptance artifact visual metric mean out of range",
    "acceptance artifact visual metric standard deviation out of range",
    "def acceptance_artifact_reference_pair_diff_errors",
    "def acceptance_artifact_reference_pair_diff_values",
    "def acceptance_artifact_reference_pair_diff_errors_from_values",
    "acceptance reference pair scanner must reject high mean diff",
    "acceptance reference pair scanner must reject high changed ratio",
    "acceptance artifact reference pair diff mean too high",
    "acceptance artifact reference pair changed ratio too high",
    "def acceptance_artifact_reference_text_contrast_errors",
    "def acceptance_artifact_reference_text_contrast_values",
    "def acceptance_artifact_reference_text_contrast_errors_from_values",
    "acceptance reference text contrast scanner must reject faint candidate text",
    "acceptance reference text contrast scanner must reject low dark text ratio",
    "acceptance artifact reference text contrast has too few candidate dark text pixels",
    "acceptance artifact reference text contrast ratio too low",
    "REQUIRED_ACCEPTANCE_CROP_CONTENT_METRICS",
    "REQUIRED_ACCEPTANCE_CROP_CHANGED_PIXELS",
    "def acceptance_artifact_crop_content_errors",
    "def acceptance_artifact_crop_content_values",
    "def acceptance_artifact_changed_pixels",
    "def magick_float",
    "REQUIRED_ACCEPTANCE_REFERENCE_CONTENT_METRICS",
    "def acceptance_artifact_reference_content_errors",
    "def acceptance_artifact_reference_content_values",
    "def acceptance_artifact_reference_content_errors_from_values",
    "REQUIRED_LIVE_ACCEPTANCE_THEME_SWITCH",
    "def acceptance_artifact_live_theme_errors",
    "def acceptance_artifact_live_theme_values",
    "REQUIRED_LIVE_ACCEPTANCE_WINDOW_SCREENSHOT_SIZE",
    "def acceptance_artifact_live_window_size_errors",
    "def acceptance_artifact_live_window_size_errors_from_values",
    "REQUIRED_LIVE_ACCEPTANCE_INTERACTIVE_CONTENT",
    "def acceptance_artifact_live_interactive_content_errors",
    "def acceptance_artifact_live_interactive_content_values",
    "def acceptance_artifact_live_interactive_content_errors_from_values",
    "REQUIRED_LIVE_ACCEPTANCE_LIGHT_TEXT_CONTRAST",
    "def acceptance_artifact_live_light_text_errors",
    "def acceptance_artifact_live_light_text_values",
    "def magick_identify_size",
    "def acceptance_artifact_luminance_count",
    "def acceptance_artifact_live_theme_errors_from_values",
    "def acceptance_artifact_live_light_text_errors_from_values",
    "REQUIRED_ACCEPTANCE_LOG_FORBIDDEN_PATTERNS",
    "REQUIRED_ACCEPTANCE_LOG_MARKERS",
    "def acceptance_artifact_log_errors",
    "def acceptance_artifact_log_errors_from_text",
    "def acceptance_artifact_scroll_performance_errors",
    "def acceptance_artifact_scroll_performance_errors_from_text",
    "def acceptance_artifact_crop_content_errors_from_values",
    "def acceptance_artifact_changed_pixels_errors_from_values",
    "def acceptance_artifact_direct_margin_left_errors",
    "def connected_component_bands_from_text",
    "def merge_connected_component_bands",
    "def acceptance_artifact_direct_margin_left_errors_from_bands",
    "acceptance crop content scanner must reject missing link underline",
    "acceptance reference content scanner must reject missing link-blue pixels",
    "acceptance reference content scanner must reject missing diagram edges",
    "acceptance live theme scanner must reject inert light toggle",
    "acceptance live light text scanner must reject low-contrast light text",
    "acceptance live light text scanner must reject dark-theme viewer background",
    "acceptance log scanner must reject closed channel noise",
    "acceptance log scanner must reject empty live logs",
    "acceptance scroll performance scanner must reject full redraw fallback",
    "acceptance scroll performance scanner must reject slow frames",
    "accepted artifact file scanner must include scroll performance errors",
    "acceptance crop change scanner must reject inert hover crop",
    "acceptance margin-left scanner must reject wrong link offset",
    "REQUIRED_ACCEPTANCE_TABLE_SECTION_BANDS",
    "REQUIRED_ACCEPTANCE_TABLE_SECTION_ROW_COUNTS",
    "def acceptance_artifact_table_section_errors",
    "def acceptance_artifact_table_section_errors_from_bands",
    "acceptance table scanner must reject heading/table overlap",
    "acceptance table scanner must reject wrapped table text rows",
    "REQUIRED_ACCEPTANCE_TABLE_GRID_COMPONENTS",
    "def acceptance_artifact_table_grid_errors",
    "def acceptance_artifact_table_grid_components",
    "def acceptance_artifact_table_grid_errors_from_values",
    "acceptance table grid scanner must reject clipped columns",
    "acceptance table grid scanner must reject shifted grid",
    "REQUIRED_ACCEPTANCE_DIAGRAM_CONTROL_ICON_CELLS",
    "def acceptance_artifact_diagram_control_icon_grid_errors",
    "def acceptance_artifact_diagram_control_icon_grid_errors_from_values",
    "acceptance diagram icon scanner must reject blocky square glyphs",
    "REQUIRED_ACCEPTANCE_DIAGRAM_CONTROL_STRIP_REGIONS",
    "REQUIRED_ACCEPTANCE_DIAGRAM_CONTROL_STRIP_CELL_THRESHOLDS",
    "def acceptance_artifact_diagram_control_strip_errors",
    "def acceptance_artifact_diagram_control_strip_cell_values",
    "def acceptance_artifact_diagram_control_strip_errors_from_values",
    "acceptance diagram control strip scanner must reject missing right-edge controls",
    "acceptance artifact diagram control strip has too few right-edge control pixels",
    "REQUIRED_ACCEPTANCE_HTML_CENTER_TEXT_BANDS",
    "def acceptance_artifact_html_center_text_errors",
    "def acceptance_artifact_html_center_text_errors_from_values",
    "acceptance html center scanner must reject left-aligned HTML",
    "acceptance artifact html center crop has wrong centered text position",
    "REQUIRED_ACCEPTANCE_LINK_UNDERLINE_BANDS",
    "def acceptance_artifact_link_underline_errors",
    "def acceptance_artifact_link_underline_values",
    "def acceptance_artifact_link_underline_errors_from_values",
    "acceptance link underline scanner must reject full-row underline",
    "acceptance link underline scanner must reject missing underline",
    "REQUIRED_ACCEPTANCE_HOVER_HIGHLIGHT_BANDS",
    "def acceptance_artifact_hover_highlight_errors",
    "def acceptance_artifact_hover_highlight_values",
    "def acceptance_artifact_hover_highlight_errors_from_values",
    "acceptance hover highlight scanner must reject body-covering highlight",
    "acceptance hover highlight scanner must reject narrow highlight",
    "REQUIRED_ACCEPTANCE_TITLE_BODY_TEXT_BANDS",
    "def acceptance_artifact_title_body_text_errors",
    "def acceptance_artifact_text_bands",
    "def acceptance_artifact_title_body_text_errors_from_values",
    "def first_band_starting_in_y_range",
    "acceptance title/body scanner must reject crushed title text",
    "acceptance title/body scanner must reject clipped body text",
    "REQUIRED_ACCEPTANCE_SIDEBAR_SELECTED_ROW_BANDS",
    "def acceptance_artifact_sidebar_selected_row_errors",
    "def acceptance_artifact_sidebar_selected_row_values",
    "def acceptance_artifact_sidebar_selected_row_errors_from_values",
    "acceptance sidebar selected-row scanner must reject activity rail overlap",
    "acceptance sidebar selected-row scanner must reject clipped row width",
    "accepted artifact file scanner must include crop content errors",
    "acceptance artifact crop content has too few bright text pixels",
    "acceptance artifact crop content has too few link-blue pixels",
    "acceptance artifact reference crop has too few link-blue pixels",
    "acceptance artifact reference crop has too few SVG/diagram edge pixels",
    "REQUIRED_ACCEPTANCE_REFERENCE_EDGE_RATIO_METRICS",
    "def acceptance_artifact_reference_edge_ratio_errors",
    "def acceptance_artifact_reference_edge_pixels",
    "def acceptance_artifact_reference_edge_ratio_errors_from_values",
    "acceptance artifact reference crop SVG/diagram edge ratio too low",
    "acceptance live artifact did not switch to light theme",
    "acceptance live light artifact has too few dark text pixels",
    "acceptance live light artifact viewer crop is not light enough",
    "acceptance log contains forbidden runtime message",
    "acceptance log is missing required marker",
    "acceptance artifact crop content has too few changed pixels",
    "acceptance artifact direct margin-left crop has wrong 80px link offset",
    "acceptance artifact title/body crop has wrong title/body text height",
    "acceptance artifact title/body crop has wrong title/body text width",
    "acceptance artifact table-section crop has overlapping 5.2 heading/table",
    "acceptance artifact table-section crop has too many table rows",
    "acceptance artifact table-section crop has wrong table grid width",
    "acceptance artifact table-section crop has wrong table grid y position",
    "acceptance artifact diagram control icon crop has too many bright glyph pixels",
    "acceptance artifact link underline has wrong link underline width",
    "acceptance artifact link underline has too few underline pixels",
    "acceptance artifact hover highlight has wrong hover highlight y position",
    "acceptance artifact hover highlight has wrong hover highlight width",
    "acceptance artifact sidebar selected-row has wrong sidebar selected-row x position",
    "acceptance artifact sidebar selected-row has wrong sidebar selected-row width",
    "def acceptance_artifact_manifest_errors",
    "acceptance artifact manifest missing required artifact row",
    "def acceptance_artifact_manifest_errors_from_text",
    "def parse_acceptance_artifact_manifest",
    "def acceptance_artifact_expected_digests",
    "hashlib.sha256",
    "acceptance artifact manifest checksum mismatch",
    "def acceptance_artifact_source_freshness_errors",
    "def acceptance_artifact_source_freshness_errors_from_mtimes",
    "acceptance artifact is older than required source file",
    "acceptance source freshness scanner must reject stale artifacts",
    "def acceptance_artifact_freshness_errors",
    "acceptance evidence confirmed_at is older than review artifact/source file",
    "REQUIRED_ACCEPTANCE_REFERENCE_ARTIFACT_SOURCE_PATHS",
    "def required_acceptance_source_integrity_paths",
    "REQUIRED_ACCEPTANCE_SOURCE_CODE_PATHS",
    "REQUIRED_ACCEPTANCE_SOURCE_CODE_ROOTS",
    "crates/katana-document-viewer/src",
    "crates/katana-document-viewer/src/preview_runtime",
    "crates/katana-document-viewer/src/viewer",
    "crates/katana-document-viewer/tests",
    "crates/kdv-linter/src",
    "tools/kdv-storybook/src",
    "tools/kdv-storybook/src/window_command",
    "Cargo.toml",
    "Cargo.lock",
    "KUC_CARGO_GIT_URL",
    "KUC_REGISTRY_VERSION",
    "KUC_REGISTRY_SOURCE",
    "def kuc_cargo_dependency_errors",
    "def required_acceptance_source_root_file_paths",
    "root.rglob(\"*.rs\")",
    "def untracked_acceptance_source_code_files_from_paths",
    "def git_tracked_source_file_labels",
    "ls-files",
    "-z",
    "assets/reference/katana/preview_crops/sample-top.png",
    "assets/reference/katana/preview_crops/sample-diagrams-top.png",
    "src/viewer/image_surface.rs",
    "src/viewer/image_surface_factory.rs",
    "src/viewer/image_surface_tests.rs",
    "src/viewer/node_plan/builder_surface_height_test_support.rs",
    "src/viewer/node_plan/builder_surface_height_tests.rs",
    "tools/kdv-storybook/src/args.rs",
    "tools/kdv-storybook/src/main.rs",
    "tools/kdv-storybook/src/frame_score_preview_crop_tests.rs",
    "tools/kdv-storybook/src/frame_score_visual_tests.rs",
    "tools/kdv-storybook/src/frame_surface_parity_tests.rs",
    "tools/kdv-storybook/src/frame_performance_tests.rs",
    "tools/kdv-storybook/src/preview_build_methods.rs",
    "tools/kdv-storybook/src/window_command/tests/diagram.rs",
    "tools/kdv-storybook/src/window/scroll_lazy_scene_tests.rs",
    "tools/kdv-storybook/src/window_asset_job.rs",
    "tools/kdv-storybook/src/window_asset_job_tests.rs",
    "tools/kdv-storybook/src/window_loop.rs",
    "tools/kdv-storybook/src/window_loop_tests.rs",
    "tools/kdv-storybook/src/window_mouse.rs",
    "tools/kdv-storybook/src/preview_theme_bridge.rs",
    "tools/kdv-storybook/src/window_tests.rs",
    "node_factory_media_controls.rs",
    "node_factory_media_display_tests.rs",
    "node_factory_media_impl.rs",
    "node_factory_media_impl_tests.rs",
    "node_factory_media_frame_tests.rs",
    "def required_acceptance_freshness_paths",
    "def missing_acceptance_source_code_files",
    "acceptance source file missing",
    "def untracked_acceptance_source_code_files",
    "acceptance source file is not tracked by git",
    "def acceptance_source_integrity_errors",
    "release source integrity",
    "ls-files",
    "tools/kdv-storybook/src/document_viewer/media_control_icons.rs",
    "kuc cargo dependency:",
    "KUC Cargo dependency scanner must reject stale Cargo.toml version",
    "KUC Cargo dependency scanner must reject sibling source include",
    "datetime.fromisoformat",
    "just storybook-release-acceptance-artifacts",
    "just storybook-acceptance-artifact",
    "target/acceptance/kdv-storybook-text-regression-crops.png",
    "target/acceptance/kdv-storybook-acceptance-artifacts.sha256",
    "target/acceptance/kdv-storybook-live-acceptance-artifacts.sha256",
    "accepted_missing_live_artifact",
    "verify_artifact_files=False",
    "--self-test",
    "def self_test",
    "accepted_missing_required_check",
    "accepted_check_mentions_item_after_negation",
    "accepted_missing_matrix_row",
    "accepted_matrix_mentions_item_outside_first_column",
    "accepted_missing_artifact_evidence",
    "accepted_with_pending_matrix",
    "accepted_with_weak_matrix",
    "accepted_bad_timestamp",
    "accepted_future_timestamp",
    "accepted_placeholder_reviewer",
    "accepted_complete",
    "OPEN_CHECKLIST_RE",
    "def open_checklist_items",
    "def print_open_checklist_items",
    r"^- \[ \] .+",
    "PENDING_ACCEPTANCE_REQUIRED_OPEN_FEEDBACK_IDS",
    "UF-040",
    "UF-042",
    "KUC_ISSUE_7_URL",
    "KUC_ISSUE_8_URL",
    "KUC_INTERACTION_TARGET_REQUIRED_TOKENS",
    "KUC_INTERACTION_TARGET_COMPLETION_REQUIRED_TOKENS",
    "KUC_INTERACTION_TARGET_KDV_REQUIRED_TOKENS",
    "KUC_DOCUMENT_VIEWER_HARNESS_ADR",
    "KUC_DOCUMENT_VIEWER_REJECTED_COMMAND",
    "KDV_DOCUMENT_VIEWER_REQUIRED_COMMAND",
    "KUC_DOCUMENT_VIEWER_OWNERSHIP_REQUIRED_TOKENS",
    "def pending_acceptance_required_open_feedback_errors",
    "def kuc_blocker_ledger_errors",
    "def kuc_interaction_target_completion_errors",
    "def kuc_interaction_target_dependency_errors",
    "def document_viewer_harness_ownership_errors",
    "def handoff_feedback_ledger_text",
    "def open_feedback_item_ids",
    "pending storybook acceptance must keep these user-feedback items open",
    "pending acceptance ledger scanner must reject closed UF-040/UF-042",
    "kuc blocker ledger:",
    "KUC blocker ledger scanner must reject closed UF-043",
    "KUC blocker ledger scanner must reject missing KUC #8 issue",
    "KUC blocker ledger scanner must reject missing KUC #8 scope",
    "KUC #8 completion scanner must reject closed 追補26 without evidence",
    "KUC #8 dependency scanner must reject closed 追補26 on v0.1.1",
    "document_viewer harness ownership:",
    "document_viewer harness scanner must reject stale KUC document_viewer command",
    "HANDOFF_FEEDBACK_LEDGER_FILES",
    "CANONICAL_FEEDBACK_LEDGER_PATH",
    "STALE_ROOT_FEEDBACK_LEDGER_PATH",
    "def handoff_canonical_feedback_path_errors",
    "def handoff_canonical_feedback_path_errors_from_markdown",
    "handoff feedback ledger scanner must reject stale root user-feedback path",
    "handoff feedback ledger:",
    "open_remaining_items",
    "open remaining-plan item(s)",
    "NATIVE_FULLSCREEN_STALE_LEDGER_PATTERNS",
    "NATIVE_FULLSCREEN_LEDGER_ALLOW_MARKERS",
    "def native_fullscreen_ledger_contradiction_errors",
    "def native_fullscreen_ledger_contradiction_errors_from_markdown",
    "native fullscreen ledger scanner must reject current OS-window sync claims",
    "native fullscreen ledger scanner must allow historical superseded notes",
    "native fullscreen ledger:",
    "def release_test_entrypoint_isolation_errors",
    "def release_workflow_acceptance_runtime_errors",
    "def just_recipe_body",
    "release workflow runtime:",
    "release test entrypoint scanner must reject generic kdv-storybook workspace tests",
    "release test entrypoint:",
];

#[test]
fn storybook_score_gate_keeps_diagram_scale_and_scroll_flake_contract_sources()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let document_viewer_root = root.join("tools/kdv-storybook/src/document_viewer");

    let release_dod_script =
        std::fs::read_to_string(root.join("scripts/release/assert-viewer-recovery-dod.py"))?;
    assert_contains_all(
        "release DoD diagram scale source integrity",
        &release_dod_script,
        &[
            "tools/kdv-storybook/src/window/scroll_lazy_scene_tests.rs",
            "node_factory_media_impl.rs",
            "node_factory_media_impl_tests.rs",
            "node_factory_media_fixture.rs",
            "builder_media_height.rs",
            "builder_media_asset_height.rs",
            "KUC_REGISTRY_SOURCE",
            "KUC_REGISTRY_VERSION",
            "def kuc_cargo_dependency_errors",
        ],
    );

    let cargo_toml = std::fs::read_to_string(root.join("Cargo.toml"))?;
    assert_contains_all(
        "KUC Cargo dependency is pinned through Cargo.toml",
        &cargo_toml,
        &["katana-ui-core = { version = \"=0.3.7\", features = [\"raster-host\"] }"],
    );
    assert!(
        !cargo_toml.contains("katana-ui-core-storybook"),
        "KDV Storybook must share the one registry-resolved KUC dependency"
    );

    let cargo_lock = std::fs::read_to_string(root.join("Cargo.lock"))?;
    assert_contains_all(
        "KUC Cargo dependency is pinned through Cargo.lock",
        &cargo_lock,
        &[
            "name = \"katana-ui-core\"",
            "source = \"registry+https://github.com/rust-lang/crates.io-index\"",
        ],
    );

    let image_surface = std::fs::read_to_string(
        root.join("crates/katana-document-viewer/src/viewer/image_surface.rs"),
    )?;
    assert_contains_all(
        "KDV diagram keeps fixed KatanA display scale",
        &image_surface,
        &[
            "pub const VIEWER_DIAGRAM_DISPLAY_SCALE: f32 = 1.0",
            "pub const VIEWER_DIAGRAM_DISPLAY_MAX_WIDTH: u32 = 1264",
        ],
    );

    let image_surface_factory = std::fs::read_to_string(
        root.join("crates/katana-document-viewer/src/viewer/image_surface_factory.rs"),
    )?;
    assert_contains_all(
        "KDV diagram fixed display scale keeps retina raster",
        &image_surface_factory,
        &[
            ".unwrap_or(Self::SVG_CONTENT_SCALE)",
            "fn fit_diagram_display_size",
            "fn diagram_display_scale",
            "fn diagram_display_max_width",
            "let preview_width = display_width * diagram_display_scale(display_width, max_width)",
            "let max_width = diagram_display_max_width(max_width) as f32",
            "crate::viewer::VIEWER_DIAGRAM_DISPLAY_SCALE",
        ],
    );

    let media_height = std::fs::read_to_string(
        root.join("crates/katana-document-viewer/src/viewer/node_plan/builder_media_height.rs"),
    )?;
    assert_contains_all(
        "KDV diagram interactive layout uses viewer width",
        &media_height,
        &[
            "fn diagram_layout_width",
            "ViewerHeightMode::InteractivePreview",
            "context.content_width.max(1)",
            "interactive_diagram_height_uses_viewer_row_width_without_upscaling",
        ],
    );

    let media_asset_height =
        std::fs::read_to_string(root.join(
            "crates/katana-document-viewer/src/viewer/node_plan/builder_media_asset_height.rs",
        ))?;
    assert_contains_all(
        "KDV diagram asset height uses viewer width",
        &media_asset_height,
        &[
            "fn diagram_max_width",
            "ViewerHeightMode::InteractivePreview => {",
            "content_width.min(VIEWER_DIAGRAM_DISPLAY_MAX_WIDTH)",
            "fn diagram_content_height",
            "ViewerImageSurfaceFactory::from_diagram_artifact",
            "interactive_diagram_height_uses_katana_reference_width_cap_for_large_viewports",
            "interactive_diagram_height_uses_viewer_width_for_wide_diagram",
        ],
    );

    let media_impl =
        std::fs::read_to_string(document_viewer_root.join("node_factory_media_impl.rs"))?;
    assert_contains_all(
        "KDV document_viewer diagram interactive keeps intrinsic size and export caps",
        &media_impl,
        &[
            "fn diagram_raster_max_width",
            "if self.export_surface",
            "self.max_media_width",
            "return capped_diagram_width(surface.display_width, DIAGRAM_EXPORT_MAX_WIDTH)",
            "fullscreen_diagram_width(",
        ],
    );

    let media_geometry =
        std::fs::read_to_string(document_viewer_root.join("node_factory_media_geometry.rs"))?;
    assert_contains_all(
        "KDV document_viewer diagram media geometry keeps KatanA fullscreen/export fit constants",
        &media_geometry,
        &[
            "pub(super) const DIAGRAM_EXPORT_MAX_WIDTH: u32 = 860",
            "const KATANA_FULLSCREEN_PADDING_PX: u32 = 40",
            "pub(super) fn fullscreen_diagram_width",
            "let height_scale = viewport_height",
            "let scale = width_scale.min(height_scale).min(1.0)",
            "saturating_sub(KATANA_FULLSCREEN_PADDING_PX.saturating_mul(2))",
        ],
    );

    let media_tests =
        std::fs::read_to_string(document_viewer_root.join("node_factory_media_impl_tests.rs"))?;
    assert_contains_all(
        "KDV document_viewer diagram interactive/export scale tests",
        &media_tests,
        &[
            "KATANA_VIEWER_ROW_MAX_WIDTH",
            "media_max_width_uses_katana_viewer_row_width_for_interactive_diagram",
            "interactive_diagram_keeps_intrinsic_body_width_while_row_stays_full_width",
            "image.props().common.width",
            "the full row width belongs to the wrapper",
            "media_max_width_keeps_export_surface_diagram_contract",
            "fullscreen_diagram_keeps_katana_original_size_without_upscaling",
            "fullscreen_diagram_fits_height_inside_katana_padded_viewport",
        ],
    );

    let node_factory = std::fs::read_to_string(document_viewer_root.join("node_factory.rs"))?;
    assert_contains_all(
        "KDV document_viewer media frame row wrapper keeps full-width KatanA container",
        &node_factory,
        &[
            "fn uses_media_row_wrapper",
            "matches!(node.kind, ViewerNodeKind::Diagram { .. })",
            "ui_node.kind() == UiNodeKind::ImageSurface",
            "UiVisualRole::MediaFrame | UiVisualRole::ExportMediaFrame",
            "fn media_row_wrapper",
            ".visual_role(visual_role)",
            ".width(width)",
        ],
    );

    let media_frame_tests =
        std::fs::read_to_string(document_viewer_root.join("node_factory_media_frame_tests.rs"))?;
    assert_contains_all(
        "KDV document_viewer media frame row wrapper tests",
        &media_frame_tests,
        &[
            "diagram_media_row_wrapper_uses_full_katana_row_width_even_without_extra_height",
            "the full-width row owns the media frame background",
            "the full-width row must not upscale the image body",
        ],
    );

    let media_fixture =
        std::fs::read_to_string(document_viewer_root.join("node_factory_media_fixture.rs"))?;
    assert_contains_all(
        "KDV document_viewer diagram interactive/export scale fixture constants",
        &media_fixture,
        &[
            "const KATANA_VIEWER_ROW_MAX_WIDTH: u32 = 1168",
            "const EXPORT_MEDIA_MAX_WIDTH: u32 = KATANA_VIEWER_ROW_MAX_WIDTH",
            "const DIAGRAM_MEDIA_MAX_WIDTH: u32 = 860",
        ],
    );

    let scroll_tests = std::fs::read_to_string(
        root.join("tools/kdv-storybook/src/window/scroll_lazy_scene_tests.rs"),
    )?;
    assert_contains_all(
        "KDV diagram scroll flake guard",
        &scroll_tests,
        &[
            "struct FastDiagramEngine",
            "PreviewBuilder::with_diagram_engine(Arc::new(FastDiagramEngine))",
            "loaded_diagram_wheel_scroll_uses_presented_band_redraw_without_full_fallback",
            "zoomed_loaded_diagram_wheel_scroll_uses_presented_band_redraw_without_full_fallback",
        ],
    );

    let diagram_window_tests = std::fs::read_to_string(
        root.join("tools/kdv-storybook/src/window_command/tests/diagram.rs"),
    )?;
    assert_contains_all(
        "KDV wide window diagram intrinsic body width guard",
        &diagram_window_tests,
        &[
            "wide_window_diagram_image_surface_keeps_fixed_scale_after_katana_fit_width",
            "KatanA fixed SVG display scale",
            "VIEWER_DIAGRAM_DISPLAY_MAX_WIDTH",
            "expected at least one KatanA-width diagram over 640px",
        ],
    );
    Ok(())
}

const ACCEPTANCE_DOC_REQUIRED_SNIPPETS: &[&str] = &[
    "Accepted release の更新条件",
    "confirmed_by",
    "human reviewer",
    "timezone 付き ISO-8601 timestamp",
    "未来時刻を書かない",
    "review artifact / source screenshot / headless live-acceptance artifact / source file の最終更新時刻以降",
    "source file が git 管理下にある",
    "Human acceptance status から `pending` を消し",
    "`accepted` または `confirmed`",
    "/opt/homebrew/bin/rtk just storybook-release-acceptance-artifacts",
    "/opt/homebrew/bin/rtk just storybook-acceptance-artifact",
    "/opt/homebrew/bin/rtk just storybook-live-acceptance-artifact",
    "/opt/homebrew/bin/rtk cargo test -p kdv-storybook --locked document_viewer -- --test-threads=1",
    "KDV owns the KatanA document_viewer harness",
    "## Acceptance Procedure",
    "smoke / screenshot / score gate の通過だけで accepted にしない",
    "KatanA viewer / export HTML / export PDF",
    "artifact 再生成と実機確認の後",
];

const ACCEPTANCE_ARTIFACT_RECIPE_REQUIRED_SNIPPETS: &[&str] = &[
    "storybook-window-sidebar-screenshot-smoke",
    "storybook-window-sidebar-narrow-screenshot-smoke",
    "storybook-window-sidebar-large-screenshot-smoke",
    "storybook-window-hover-screenshot-smoke",
    "storybook-window-hover-wide-screenshot-smoke",
    "storybook-window-footnote-screenshot-smoke",
    "storybook-window-table-screenshot-smoke",
    "storybook-window-code-copy-screenshot-smoke",
    "storybook-window-selection-screenshot-smoke",
    "storybook-window-diagram-screenshot-smoke",
    "storybook-window-drawio-diagram-screenshot-smoke",
    "storybook-window-slideshow-screenshot-smoke",
    "storybook-scroll-performance-artifact",
    "KDV_STORYBOOK_PREVIEW_CROP_DUMP_DIR=\"{{REPO_ROOT}}/target/acceptance/preview-crop-reference\"",
    "storybook_score_visual_uses_katana_",
    "scripts/release/generate-storybook-acceptance-artifact.sh",
];

const ACCEPTANCE_ARTIFACT_SCRIPT_REQUIRED_SNIPPETS: &[&str] = &[
    "kdv-storybook-window-hover-smoke.png",
    "kdv-storybook-window-sidebar-smoke.png",
    "kdv-storybook-window-sidebar-smoke-file-hover.png",
    "kdv-storybook-window-sidebar-smoke-settings-click.png",
    "kdv-storybook-window-sidebar-narrow-smoke.png",
    "kdv-storybook-window-sidebar-narrow-smoke-file-click.png",
    "kdv-storybook-window-sidebar-large-smoke.png",
    "kdv-storybook-window-sidebar-large-smoke-settings-hover.png",
    "kdv-storybook-window-html-margin-smoke.png",
    "kdv-storybook-window-hover-wide-smoke.png",
    "kdv-storybook-window-diagram-smoke.png",
    "kdv-storybook-window-diagram-smoke-hover-zoom-in.png",
    "kdv-storybook-window-diagram-smoke-fullscreen.png",
    "kdv-storybook-window-drawio-diagram-smoke.png",
    "kdv-storybook-window-drawio-diagram-smoke-hover-reset-view.png",
    "kdv-storybook-window-drawio-diagram-smoke-trackpad-help.png",
    "kdv-storybook-window-footnote-smoke.png",
    "kdv-storybook-window-footnote-smoke-reference.png",
    "kdv-storybook-window-footnote-smoke-definition.png",
    "kdv-storybook-window-table-smoke.png",
    "kdv-storybook-window-code-copy-smoke.png",
    "kdv-storybook-window-code-copy-smoke-hover.png",
    "kdv-storybook-window-code-copy-smoke-copied.png",
    "kdv-storybook-window-selection-smoke.png",
    "kdv-storybook-window-slideshow-smoke.png",
    "kdv-storybook-window-slideshow-smoke-mode.png",
    "kdv-storybook-window-slideshow-smoke-next.png",
    "kdv-storybook-window-slideshow-smoke-previous.png",
    "kdv-storybook-window-slideshow-smoke-close.png",
    "kdv-storybook-scroll-performance.txt",
    "full_preview_redraw_fallback_count=0",
    "target/acceptance/kdv-storybook-acceptance-contact-sheet.png",
    "target/acceptance/kdv-storybook-text-regression-crops.png",
    "target/acceptance/kdv-storybook-katana-reference-comparison.png",
    "target/acceptance/kdv-storybook-acceptance-artifacts.sha256",
    "preview-crop-reference/katana_sample_md-preview-crop_reference.ppm",
    "preview-crop-reference/katana_sample_md-preview-crop_preview.ppm",
    "preview-crop-reference/katana_sample_diagrams_md-preview-crop_reference.ppm",
    "preview-crop-reference/katana_sample_diagrams_md-preview-crop_preview.ppm",
    "title-body.png",
    "language-link.png",
    "640x100+590+235",
    "html-margin-center.png",
    "direct-html-margin-left.png",
    "740x260+500+75",
    "hover-highlight.png",
    "wide-title-link-html.png",
    "1480x460+500+80",
    "diagram-control-icons.png",
    "110x150+1160+145",
    "table-section.png",
    "740x780+526+80",
    r#""$crop_dir/title-body.png" \"#,
    r#""$crop_dir/language-link.png" \"#,
    r#""$crop_dir/html-margin-center.png" \"#,
    r#""$crop_dir/direct-html-margin-left.png" \"#,
    r#""$crop_dir/hover-highlight.png" \"#,
    r#""$crop_dir/wide-title-link-html.png" \"#,
    r#""$crop_dir/table-section.png" \"#,
    "review_dir",
    "language-link-review.png",
    "wide-title-link-html-review.png",
    "diagram-control-icons-review.png",
    "table-section-review.png",
    "reference-comparison",
    "sample-top-reference.png",
    "sample-top-candidate.png",
    "sample-top-diff.png",
    "sample-diagrams-reference.png",
    "sample-diagrams-candidate.png",
    "sample-diagrams-diff.png",
    "KatanA text / link reference",
    "KDV text / link candidate",
    "KatanA/KDV text diff heatmap",
    "KatanA SVG / diagram reference",
    "KDV SVG / diagram candidate",
    "KatanA/KDV SVG diff heatmap",
    "-compose difference -composite -auto-level",
    "require_png_size \"$review_dir/diagram-control-icons-review.png\" 740 180",
    "1280x520+0+0",
    "1280x920+0+250",
    "require_png_size \"$comparison_dir/sample-top-reference.png\" 740 320",
    "require_png_size \"$comparison_dir/sample-top-diff.png\" 740 320",
    "require_png_size \"$comparison_dir/sample-diagrams-candidate.png\" 740 560",
    "require_png_size \"$comparison_dir/sample-diagrams-diff.png\" 740 560",
    "-tile 4x4",
    "-tile 1x8",
    "-tile 3x2",
    "kdv-storybook-acceptance-contact-sheet-*.png",
    "require_png_size",
    "require_min_unique_colors",
    "require_min_bright_pixels",
    "require_min_blue_pixels",
    "require_min_changed_pixels",
    "require_min_edge_pixels",
    "expected_width",
    "expected_height",
    "shasum -a 256",
    "montage",
];

const LIVE_ACCEPTANCE_RECIPE_REQUIRED_SNIPPETS: &[&str] = &[
    "build --release --locked -p kdv-storybook",
    "scripts/release/generate-storybook-live-acceptance-artifact.sh",
];

const LIVE_ACCEPTANCE_SCRIPT_REQUIRED_SNIPPETS: &[&str] = &[
    "target/release/kdv-storybook",
    "--live-acceptance-artifact",
    "--light-screenshot-output",
    "target/acceptance/kdv-storybook-live-interactive.png",
    "target/acceptance/kdv-storybook-live-light-toggle.png",
    "target/acceptance/kdv-storybook-live-acceptance-artifacts.sha256",
    "STORYBOOK_LIVE_ACCEPTANCE_WIDTH",
    "STORYBOOK_LIVE_ACCEPTANCE_HEIGHT",
    "STORYBOOK_LIVE_ACCEPTANCE_EXPECTED_WIDTH",
    "STORYBOOK_LIVE_ACCEPTANCE_EXPECTED_HEIGHT",
    "STORYBOOK_LIVE_INTERACTIVE_MIN_BRIGHT_PIXELS",
    "STORYBOOK_LIVE_INTERACTIVE_MIN_UNIQUE_COLORS",
    "storybook live acceptance headless artifact ready",
    "storybook live acceptance interactive content ready",
    ">> \"$LOG_OUT\" 2>&1",
    "storybook live acceptance clicked dark toggle",
    "storybook live acceptance theme switch verified",
    "tee -a \"$LOG_OUT\"",
    "live acceptance theme switch verified",
    "changed_pixels",
    "bright_delta",
    "live acceptance light screenshot did not switch to light theme",
    "shasum -a 256",
];

fn passing_evidence() -> StorybookScoreEvidence {
    StorybookScoreEvidence {
        visual_katana_reference: true,
        semantic_export_reference: true,
        interaction_runtime_actions: true,
        interaction_os_clipboard: true,
        performance_budget_gate: true,
    }
}

fn score_surface_sources(
    root: &std::path::Path,
) -> Result<ScoreSurfaceSources, Box<dyn std::error::Error>> {
    Ok(ScoreSurfaceSources {
        preview_crop_tests: std::fs::read_to_string(
            root.join("tools/kdv-storybook/src/frame_score_preview_crop_tests.rs"),
        )?,
        visual_tests: std::fs::read_to_string(
            root.join("tools/kdv-storybook/src/frame_score_visual_tests.rs"),
        )?,
        preview_build_methods: std::fs::read_to_string(
            root.join("tools/kdv-storybook/src/preview_build_methods.rs"),
        )?,
        surface_parity_tests: std::fs::read_to_string(
            root.join("tools/kdv-storybook/src/frame_surface_parity_tests.rs"),
        )?,
    })
}

fn assert_score_controls_disabled(sources: &ScoreSurfaceSources) {
    for (label, source) in [
        ("preview crop score", sources.preview_crop_tests.as_str()),
        (
            "export score surface",
            sources.preview_build_methods.as_str(),
        ),
        ("surface parity", sources.surface_parity_tests.as_str()),
    ] {
        assert_contains_all(label, source, SCORE_CONTROL_DISABLED_SNIPPETS);
    }
}

fn assert_overlay_controls_rejected(sources: &ScoreSurfaceSources) {
    for (label, source) in [
        ("preview crop score", sources.preview_crop_tests.as_str()),
        ("export score surface", sources.visual_tests.as_str()),
    ] {
        assert!(
            source.contains("assert_no_overlay_controls"),
            "{label} must reject Storybook overlay controls in score surfaces"
        );
        assert_contains_all(label, source, SCORE_OVERLAY_CONTROL_SNIPPETS);
    }
}

fn assert_release_contract_recipe(
    root: &std::path::Path,
    release_verify: &str,
    release_contract: &str,
) {
    assert!(
        release_verify.contains("release-contract-check"),
        "release-verify must enforce the active target release contract before package/publish"
    );
    assert!(
        release_contract.contains("scripts/release/verify-release-contract.py")
            && release_contract.contains("--self-test")
            && release_contract.contains("browser_session_adapter_contract"),
        "release-contract-check must self-test and run the browser-session adapter contract"
    );
    assert!(
        root.join("scripts/release/verify-release-contract.py")
            .is_file(),
        "release contract script must exist"
    );
}

fn assert_legacy_release_dod_recipe(root: &std::path::Path, release_dod: &str) {
    assert!(
        release_dod.contains("scripts/release/assert-viewer-recovery-dod.py")
            && release_dod.contains("--self-test"),
        "legacy Storybook UI releases must retain the viewer recovery DoD"
    );
    assert!(
        root.join("scripts/release/assert-viewer-recovery-dod.py")
            .is_file(),
        "legacy viewer recovery DoD script must exist"
    );
}

fn acceptance_script_required_source_artifacts(source: &str) -> std::collections::BTreeSet<String> {
    let source = normalize_newlines(source);
    let Some((_, after_start)) = source.split_once("required=(\n") else {
        return std::collections::BTreeSet::new();
    };
    let Some((body, _)) = after_start.split_once("\n)") else {
        return std::collections::BTreeSet::new();
    };

    body.lines()
        .map(str::trim)
        .filter(|line| line.starts_with("target/"))
        .map(str::to_owned)
        .collect()
}

fn release_dod_required_source_artifacts(source: &str) -> std::collections::BTreeSet<String> {
    source
        .lines()
        .filter_map(quoted_python_path)
        .filter(|path| {
            path.starts_with("target/kdv-storybook-window-")
                || *path == "target/acceptance/kdv-storybook-scroll-performance.txt"
        })
        .map(str::to_owned)
        .collect()
}

fn quoted_python_path(line: &str) -> Option<&str> {
    let (_, after_first_quote) = line.split_once('"')?;
    let (path, _) = after_first_quote.split_once('"')?;
    Some(path)
}

fn assert_hover_screenshot_artifact_contract(
    root: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let window_loop = normalize_newlines(&std::fs::read_to_string(
        root.join("tools/kdv-storybook/src/window_loop.rs"),
    )?);
    assert!(
        window_loop.contains(
            "write_stage_canvas_png(\n            &self.args.screenshot_output,\n            \"hover\",\n            &hovered,"
        ) && window_loop.contains("write_canvas_png(&self.args.screenshot_output, &base)"),
        "hover screenshot smoke must write base to the primary artifact and hover to the stage artifact"
    );
    Ok(())
}

fn assert_live_acceptance_doc_contract(
    root: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let acceptance_doc = std::fs::read_to_string(root.join(
        "openspec/changes/v0-2-0-markdown-viewer-kuc-integration/storybook-user-acceptance.md",
    ))?;
    let current_acceptance_section = acceptance_doc
        .split("## Evidence")
        .next()
        .unwrap_or(acceptance_doc.as_str());
    assert!(
        acceptance_doc.contains("/opt/homebrew/bin/rtk just storybook-live-acceptance-artifact")
            && acceptance_doc.contains("headless live-acceptance artifact")
            && acceptance_doc.contains("KUC 実 UI tree")
            && acceptance_doc.contains("Dark toggle typed action")
            && acceptance_doc.contains("human acceptance の代替ではない"),
        "storybook-user-acceptance.md must document the live acceptance artifact aid"
    );
    assert!(
        !current_acceptance_section.contains("live OS artifact")
            && !current_acceptance_section.contains("CoreGraphics")
            && !current_acceptance_section.contains("screencapture"),
        "storybook-user-acceptance.md current contract must not describe OS-window live artifact capture as the current acceptance aid"
    );
    Ok(())
}

fn assert_release_publish_recipe(root: &std::path::Path, release_publish: &str) {
    assert!(
        release_publish.contains("release-check")
            && release_publish.contains("scripts/release/publish-crates.sh"),
        "release-publish must run release-check and use the checked publish script"
    );
    assert!(
        root.join("scripts/release/publish-crates.sh").is_file(),
        "release-publish script must exist"
    );
}

fn assert_publish_script_requires_clean_token(publish_script: &str) {
    assert!(
        publish_script.contains("--token") && publish_script.contains("${CARGO_REGISTRY_TOKEN}"),
        "publish script must require explicit CARGO_REGISTRY_TOKEN"
    );
    assert!(
        publish_script.contains("require_clean_worktree")
            && publish_script.contains("git diff --quiet")
            && publish_script.contains("git diff --cached --quiet")
            && !publish_script.contains("--allow-dirty"),
        "real release publish must reject dirty worktrees instead of using --allow-dirty"
    );
}

fn recipe_body<'a>(justfile: &'a str, recipe: &str) -> Result<&'a str, String> {
    let header = format!("{recipe}:");
    let start = justfile
        .find(&header)
        .ok_or_else(|| format!("recipe `{recipe}` not found"))?;
    let tail = &justfile[start..];
    let end = tail
        .find("\n\n")
        .or_else(|| tail.find("\r\n\r\n"))
        .ok_or_else(|| format!("recipe `{recipe}` body not terminated"))?;
    Ok(&tail[..end])
}

fn assert_contains_all(label: &str, haystack: &str, needles: &[&str]) {
    let haystack = normalize_newlines(haystack);
    for needle in needles {
        let needle = normalize_newlines(needle);
        assert!(
            haystack.contains(&needle),
            "{label} must include `{needle}`"
        );
    }
}

fn assert_linux_runtime_requirements(label: &str, workflow: &str, runtime_snippets: &[&str]) {
    assert_contains_all(label, workflow, runtime_snippets);
    assert_contains_all(
        &format!("{label} color emoji font pin"),
        workflow,
        LINUX_EMOJI_FONT_PIN_REQUIRED_SNIPPETS,
    );
}

fn assert_linux_emoji_install_contract(
    root: &std::path::Path,
    ci: &str,
    preflight: &str,
    release: &str,
) -> Result<(), std::io::Error> {
    let install_script =
        std::fs::read_to_string(root.join(".github/scripts/install-linux-render-dependencies.sh"))?;
    assert_contains_all(
        "Linux render dependency installer",
        &install_script,
        LINUX_EMOJI_INSTALL_SCRIPT_REQUIRED_SNIPPETS,
    );
    assert!(!install_script.contains("apt-get install -y fonts-noto-color-emoji"));
    assert_ci_ubuntu_contract(ci);
    assert_release_ubuntu_contract("release preflight Ubuntu runner", preflight);
    assert_release_ubuntu_contract("Release Ubuntu runner", release);
    Ok(())
}

fn assert_ci_ubuntu_contract(ci: &str) {
    assert_contains_all(
        "CI Ubuntu runner",
        ci,
        &[
            "os: [macos-latest, ubuntu-latest, windows-latest]",
            "ubuntu-24.04",
            "runs-on: ${{ matrix.os == 'ubuntu-latest' && 'ubuntu-24.04' || matrix.os }}",
            "Install cargo-llvm-cov\n        if: matrix.os == 'ubuntu-latest'",
            "Install Graphviz (Ubuntu)\n        if: matrix.os == 'ubuntu-latest'",
            "Run coverage\n        if: matrix.os == 'ubuntu-latest'",
        ],
    );
    assert!(!ci.contains("runs-on: ${{ matrix.os }}"));
}

fn assert_release_ubuntu_contract(label: &str, workflow: &str) {
    assert_contains_all(
        label,
        workflow,
        &[
            "ubuntu-24.04",
            "run: |\n          .github/scripts/install-linux-render-dependencies.sh",
        ],
    );
}

fn normalize_newlines(source: &str) -> String {
    source.replace("\r\n", "\n")
}

fn workspace_root() -> Result<std::path::PathBuf, std::io::Error> {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .map(std::path::Path::to_path_buf)
        .ok_or_else(|| std::io::Error::other("crate must live under workspace crates directory"))
}
