use std::path::{Path, PathBuf};

const REFERENCE_REQUIREMENTS: &[ReferenceRequirement] = &[
    ReferenceRequirement::new("assets/reference/katana/html", "html"),
    ReferenceRequirement::new("assets/reference/katana/html", "png"),
    ReferenceRequirement::new("assets/reference/katana/pdf", "pdf"),
    ReferenceRequirement::new("assets/reference/katana/pdf", "png"),
    ReferenceRequirement::new("assets/reference/katana/export_png", "png"),
    ReferenceRequirement::new("assets/reference/katana/preview_crops", "png"),
    ReferenceRequirement::new("assets/reference/katana/screenshots", "png"),
];

const VISUAL_PARITY_REFERENCE_FILES: &[&str] = &[
    "assets/reference/katana/preview_crops/sample-top.png",
    "assets/reference/katana/preview_crops/sample-diagrams-top.png",
    "assets/reference/katana/screenshots/direct-kdv-icon-svg.png",
    "assets/reference/katana/screenshots/drawio-basic-flow.png",
    "assets/reference/katana/screenshots/sample.png",
    "assets/reference/katana/screenshots/sample_html.png",
];

const VIEWER_EXPORT_PARITY_REFERENCE_FILES: &[&str] = &[
    "assets/reference/katana/html/sample.html",
    "assets/reference/katana/html/sample.png",
    "assets/reference/katana/html/sample_html.html",
    "assets/reference/katana/html/sample_html.png",
    "assets/reference/katana/pdf/sample.pdf",
    "assets/reference/katana/pdf/sample.png",
    "assets/reference/katana/pdf/sample_html.pdf",
    "assets/reference/katana/pdf/sample_html.png",
    "assets/reference/katana/export_png/sample.png",
    "assets/reference/katana/preview_crops/sample-top.png",
    "assets/reference/katana/preview_crops/sample-diagrams-top.png",
];

const PARITY_AUDIT_REQUIREMENTS: &[ParityAuditRequirement] = &[
    ParityAuditRequirement::new(
        "katana-viewer-markdown",
        "assets/reference/katana/preview_crops/sample-top.png",
        &[
            "storybook_score_visual_uses_katana_preview_crop_reference",
            "fixture_score_matrix",
            "storybook_frame_matches_export_surface_for_katana_viewer",
            "storybook-performance-check-core",
        ],
        &[
            ParityAuditCategory::Visual,
            ParityAuditCategory::Semantic,
            ParityAuditCategory::Performance,
        ],
    ),
    ParityAuditRequirement::new(
        "katana-viewer-window-markdown",
        "assets/reference/katana/screenshots/sample.png",
        &[
            "storybook-window-smoke",
            "storybook_score_visual_uses_katana_preview_crop_reference",
            "storybook_frame_matches_export_surface_for_katana_viewer",
            "storybook-performance-check-core",
        ],
        &[
            ParityAuditCategory::Visual,
            ParityAuditCategory::Interaction,
            ParityAuditCategory::Performance,
        ],
    ),
    ParityAuditRequirement::new(
        "katana-viewer-window-html",
        "assets/reference/katana/screenshots/sample_html.png",
        &[
            "storybook-window-smoke",
            "storybook_score_visual_uses_katana_export_png_reference",
            "storybook_frame_matches_export_surface_for_katana_viewer",
            "storybook-performance-check-core",
        ],
        &[
            ParityAuditCategory::Visual,
            ParityAuditCategory::Interaction,
            ParityAuditCategory::Performance,
        ],
    ),
    ParityAuditRequirement::new(
        "katana-viewer-diagrams",
        "assets/reference/katana/preview_crops/sample-diagrams-top.png",
        &[
            "storybook_score_visual_uses_katana_sample_diagrams_crop_reference",
            "storybook_preview_crop_score_excludes_storybook_overlay_controls",
            "storybook-window-diagram-screenshot-smoke",
            "storybook_frame_matches_export_surface_for_katana_viewer_diagrams",
            "storybook-performance-check-core",
        ],
        &[
            ParityAuditCategory::Visual,
            ParityAuditCategory::Interaction,
            ParityAuditCategory::Performance,
        ],
    ),
    ParityAuditRequirement::new(
        "katana-viewer-svg-screenshot",
        "assets/reference/katana/screenshots/direct-kdv-icon-svg.png",
        &[
            "storybook_score_visual_uses_katana_sample_diagrams_crop_reference",
            "storybook-window-diagram-screenshot-smoke",
        ],
        &[
            ParityAuditCategory::Visual,
            ParityAuditCategory::Interaction,
        ],
    ),
    ParityAuditRequirement::new(
        "katana-viewer-drawio-screenshot",
        "assets/reference/katana/screenshots/drawio-basic-flow.png",
        &[
            "storybook_score_visual_uses_katana_sample_diagrams_crop_reference",
            "storybook-window-drawio-diagram-screenshot-smoke",
        ],
        &[
            ParityAuditCategory::Visual,
            ParityAuditCategory::Interaction,
        ],
    ),
    ParityAuditRequirement::new(
        "katana-export-html",
        "assets/reference/katana/html/sample.html",
        &[
            "storybook_score_visual_uses_katana_export_png_reference",
            "fixture_score_matrix",
            "surface_equivalence",
            "storybook_frame_matches_export_surface_for_katana_viewer",
        ],
        &[ParityAuditCategory::Visual, ParityAuditCategory::Semantic],
    ),
    ParityAuditRequirement::new(
        "katana-export-html-png",
        "assets/reference/katana/html/sample.png",
        &[
            "storybook_score_visual_uses_katana_export_png_reference",
            "surface_equivalence",
        ],
        &[ParityAuditCategory::Visual, ParityAuditCategory::Semantic],
    ),
    ParityAuditRequirement::new(
        "katana-export-pdf",
        "assets/reference/katana/pdf/sample.pdf",
        &["fixture_score_matrix", "surface_equivalence"],
        &[ParityAuditCategory::Semantic, ParityAuditCategory::Visual],
    ),
    ParityAuditRequirement::new(
        "katana-export-pdf-png",
        "assets/reference/katana/pdf/sample.png",
        &["surface_equivalence"],
        &[ParityAuditCategory::Semantic, ParityAuditCategory::Visual],
    ),
];

const RUNTIME_PARITY_REQUIREMENTS: &[RuntimeParityRequirement] = &[
    RuntimeParityRequirement::new(
        "katana-viewer-link-footnote",
        "storybook-link-footnote-check-core",
        &[],
        &[
            ParityAuditCategory::Interaction,
            ParityAuditCategory::Semantic,
        ],
    ),
    RuntimeParityRequirement::new(
        "katana-viewer-search",
        "storybook-search-check-core",
        &[],
        &[
            ParityAuditCategory::Interaction,
            ParityAuditCategory::Semantic,
        ],
    ),
    RuntimeParityRequirement::new(
        "katana-viewer-slideshow",
        "storybook-slideshow-check-core",
        &[
            "storybook-slideshow-screenshot-smoke",
            "storybook-window-slideshow-screenshot-smoke",
            "storybook-performance-check-core",
        ],
        &[
            ParityAuditCategory::Visual,
            ParityAuditCategory::Interaction,
            ParityAuditCategory::Performance,
        ],
    ),
    RuntimeParityRequirement::new(
        "katana-viewer-selection-clipboard",
        "storybook-window-selection-screenshot-smoke",
        &[],
        &[
            ParityAuditCategory::Interaction,
            ParityAuditCategory::Semantic,
        ],
    ),
    RuntimeParityRequirement::new(
        "katana-viewer-sidebar-navigation",
        "storybook-window-sidebar-screenshot-smoke",
        &[
            "storybook-window-sidebar-narrow-screenshot-smoke",
            "storybook-window-sidebar-large-screenshot-smoke",
        ],
        &[
            ParityAuditCategory::Visual,
            ParityAuditCategory::Interaction,
        ],
    ),
    RuntimeParityRequirement::new(
        "katana-viewer-media-controls",
        "storybook-media-control-clickability-check-full-core",
        &[
            "storybook-window-diagram-screenshot-smoke",
            "storybook-window-drawio-diagram-screenshot-smoke",
        ],
        &[
            ParityAuditCategory::Visual,
            ParityAuditCategory::Interaction,
        ],
    ),
    RuntimeParityRequirement::new(
        "katana-viewer-settings-toggles",
        "storybook-settings-contract-check-core",
        &[
            "storybook-window-sidebar-screenshot-smoke",
            "storybook-performance-check-core",
        ],
        &[
            ParityAuditCategory::Visual,
            ParityAuditCategory::Interaction,
            ParityAuditCategory::Performance,
        ],
    ),
    RuntimeParityRequirement::new(
        "katana-viewer-task-checkbox",
        "storybook-task-checkbox-check-core",
        &[],
        &[
            ParityAuditCategory::Interaction,
            ParityAuditCategory::Semantic,
        ],
    ),
    RuntimeParityRequirement::new(
        "katana-viewer-accordion",
        "storybook-accordion-check-core",
        &[],
        &[
            ParityAuditCategory::Interaction,
            ParityAuditCategory::Semantic,
        ],
    ),
];

const OPENSPEC_REQUIREMENT_PARITY: &[OpenSpecRequirementParity] = &[
    OpenSpecRequirementParity::new(
        "KUC viewerでMarkdownを表示しなければならない",
        &["storybook-content-check-core", "storybook-score-check"],
    ),
    OpenSpecRequirementParity::new(
        "viewerはSPAではなく単一instanceとして扱われなければならない",
        &[
            "storybook-entrypoint-check",
            "storybook-window-smoke",
            "storybook-performance-check-core",
        ],
    ),
    OpenSpecRequirementParity::new(
        "viewerはviewport駆動でlayoutしなければならない",
        &[
            "storybook-scroll-resize-contract-check",
            "storybook-slideshow-check-core",
        ],
    ),
    OpenSpecRequirementParity::new(
        "external artifactは並列load可能なassetとして扱われなければならない",
        &[
            "storybook-diagram-load-check-core",
            "storybook-window-diagram-screenshot-smoke",
            "storybook-window-drawio-diagram-screenshot-smoke",
            "storybook-performance-check-core",
        ],
    ),
    OpenSpecRequirementParity::new(
        "Markdown viewer はDocument modeとSlideshow modeを切り替えられなければならない",
        &[
            "storybook-slideshow-check-core",
            "storybook-slideshow-screenshot-smoke",
            "storybook-window-slideshow-screenshot-smoke",
            "storybook-performance-check-core",
        ],
    ),
    OpenSpecRequirementParity::new(
        "hit-test metadataでKMM位置へ戻れなければならない",
        &[
            "storybook-interaction-check-core",
            "storybook-coordinate-contract-check-core",
        ],
    ),
    OpenSpecRequirementParity::new(
        "KMM AST由来の目次を表示しなければならない",
        &[
            "storybook-toc-check-core",
            "storybook-interaction-check-core",
        ],
    ),
    OpenSpecRequirementParity::new(
        "検索hitをhighlightしjumpできなければならない",
        &["storybook-search-check-core"],
    ),
    OpenSpecRequirementParity::new(
        "hover / selection / media controls をinteraction設定で制御しなければならない",
        &[
            "storybook-hover-contract-check-core",
            "storybook-selection-contract-check-core",
            "storybook-window-selection-screenshot-smoke",
            "storybook-media-control-clickability-check-full-core",
            "storybook-window-diagram-screenshot-smoke",
            "storybook-window-drawio-diagram-screenshot-smoke",
        ],
    ),
    OpenSpecRequirementParity::new(
        "表示テキストは選択とclipboard copyができなければならない",
        &[
            "storybook-selection-contract-check-core",
            "storybook-selection-screenshot-smoke",
            "storybook-window-selection-screenshot-smoke",
            "storybook-clipboard-smoke",
            "storybook-clipboard-keyboard-smoke",
            "storybook-clipboard-drag-smoke",
        ],
    ),
    OpenSpecRequirementParity::new(
        "unresolved metadataを本文から消してはならない",
        &["storybook-unresolved-metadata-check-core"],
    ),
];

#[test]
fn katana_reference_artifacts_are_fixed_in_kdv_assets() -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    for requirement in REFERENCE_REQUIREMENTS {
        requirement.assert_satisfied(&root)?;
    }
    Ok(())
}

#[test]
fn katana_reference_artifacts_cover_svg_and_diagram_visual_parity()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    for relative_path in VISUAL_PARITY_REFERENCE_FILES {
        let path = root.join(relative_path);
        assert!(
            path.is_file(),
            "missing KatanA visual parity reference artifact: {}",
            path.display()
        );
    }
    Ok(())
}

#[test]
fn katana_reference_artifacts_cover_viewer_html_pdf_parity()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    for relative_path in VIEWER_EXPORT_PARITY_REFERENCE_FILES {
        let path = root.join(relative_path);
        assert!(
            path.is_file(),
            "missing KatanA viewer/export parity reference artifact: {}",
            path.display()
        );
    }
    Ok(())
}

#[test]
fn katana_reference_artifacts_have_expected_signatures_and_payloads()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    for relative_path in VIEWER_EXPORT_PARITY_REFERENCE_FILES {
        ReferenceArtifactPayload::new(relative_path).assert_valid(&root)?;
    }
    for relative_path in VISUAL_PARITY_REFERENCE_FILES {
        ReferenceArtifactPayload::new(relative_path).assert_valid(&root)?;
    }
    Ok(())
}

#[test]
fn katana_reference_artifacts_are_connected_to_requirement_parity_gates()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let justfile = std::fs::read_to_string(root.join("Justfile"))?;
    let storybook_check = recipe_body(&justfile, "storybook-check")?;
    let score_check = recipe_body(&justfile, "storybook-score-check")?;
    for requirement in PARITY_AUDIT_REQUIREMENTS {
        let artifact = root.join(requirement.reference_artifact);
        assert!(
            artifact.is_file(),
            "missing parity reference for {}: {}",
            requirement.name,
            artifact.display()
        );
        for gate in requirement.gates {
            assert!(
                score_check.contains(gate) || storybook_check.contains(gate),
                "KatanA parity requirement `{}` is not connected to gate `{gate}`",
                requirement.name
            );
        }
    }
    Ok(())
}

#[test]
fn katana_reference_artifacts_are_connected_to_score_check()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let justfile = std::fs::read_to_string(root.join("Justfile"))?;
    let score_check = recipe_body(&justfile, "storybook-score-check")?;
    for requirement in PARITY_AUDIT_REQUIREMENTS {
        for gate in requirement.gates {
            assert!(
                score_check.contains(gate),
                "KatanA parity requirement `{}` must be proven by storybook-score-check gate `{gate}`",
                requirement.name
            );
        }
    }
    Ok(())
}

#[test]
fn katana_runtime_parity_requirements_are_connected_to_storybook_check()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let justfile = std::fs::read_to_string(root.join("Justfile"))?;
    let storybook_check = recipe_body(&justfile, "storybook-check")?;
    let registry_consumer_suite = storybook_check.contains("storybook-registry-consumer-suite");
    for requirement in RUNTIME_PARITY_REQUIREMENTS {
        assert!(
            registry_consumer_suite || storybook_check.contains(requirement.gate),
            "KatanA runtime parity requirement `{}` is not connected to storybook-check gate `{}`",
            requirement.name,
            requirement.gate
        );
        for supporting_gate in requirement.supporting_gates {
            assert!(
                registry_consumer_suite || storybook_check.contains(supporting_gate),
                "KatanA runtime parity requirement `{}` is not connected to supporting gate `{supporting_gate}`",
                requirement.name
            );
        }
        assert!(
            !requirement.categories.is_empty(),
            "KatanA runtime parity requirement `{}` must declare score categories",
            requirement.name
        );
    }
    Ok(())
}

#[test]
fn katana_runtime_parity_requirements_are_connected_to_score_check()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let justfile = std::fs::read_to_string(root.join("Justfile"))?;
    let score_check = recipe_body(&justfile, "storybook-score-check")?;
    for requirement in RUNTIME_PARITY_REQUIREMENTS {
        assert!(
            score_check.contains(requirement.gate),
            "KatanA runtime parity requirement `{}` must be proven by storybook-score-check gate `{}`",
            requirement.name,
            requirement.gate
        );
        for supporting_gate in requirement.supporting_gates {
            assert!(
                score_check.contains(supporting_gate),
                "KatanA runtime parity requirement `{}` must be proven by storybook-score-check supporting gate `{supporting_gate}`",
                requirement.name
            );
        }
    }
    Ok(())
}

#[test]
fn katana_runtime_parity_audit_declares_category_gates() {
    for requirement in RUNTIME_PARITY_REQUIREMENTS {
        for category in requirement.categories {
            assert!(
                requirement.has_gate_for_category(*category),
                "KatanA runtime parity requirement `{}` declares {:?} without a matching gate",
                requirement.name,
                category
            );
        }
    }
}

#[test]
fn katana_reference_artifacts_runtime_parity_audit_covers_reported_visual_and_performance_risks() {
    assert_runtime_requirement_categories(
        "katana-viewer-sidebar-navigation",
        &[
            ParityAuditCategory::Visual,
            ParityAuditCategory::Interaction,
        ],
    );
    assert_runtime_requirement_categories(
        "katana-viewer-media-controls",
        &[
            ParityAuditCategory::Visual,
            ParityAuditCategory::Interaction,
        ],
    );
    assert_runtime_requirement_categories(
        "katana-viewer-slideshow",
        &[
            ParityAuditCategory::Visual,
            ParityAuditCategory::Interaction,
            ParityAuditCategory::Performance,
        ],
    );
    assert_runtime_requirement_categories(
        "katana-viewer-settings-toggles",
        &[
            ParityAuditCategory::Visual,
            ParityAuditCategory::Interaction,
            ParityAuditCategory::Performance,
        ],
    );
}

fn assert_runtime_requirement_categories(name: &str, expected_categories: &[ParityAuditCategory]) {
    let requirement = RUNTIME_PARITY_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.name == name);
    assert!(
        requirement.is_some(),
        "missing runtime parity requirement `{name}`"
    );
    let Some(requirement) = requirement else {
        return;
    };
    for category in expected_categories {
        assert!(
            requirement.categories.contains(category),
            "KatanA runtime parity requirement `{}` must declare {:?}",
            requirement.name,
            category
        );
    }
}

#[test]
fn katana_reference_artifact_parity_audit_declares_score_categories() {
    for requirement in PARITY_AUDIT_REQUIREMENTS {
        assert!(
            !requirement.categories.is_empty(),
            "KatanA parity requirement `{}` must declare score categories",
            requirement.name
        );
        for category in requirement.categories {
            assert!(
                requirement.has_gate_for_category(*category),
                "KatanA parity requirement `{}` declares {:?} without a matching gate",
                requirement.name,
                category
            );
        }
    }

    for category in [
        ParityAuditCategory::Visual,
        ParityAuditCategory::Semantic,
        ParityAuditCategory::Interaction,
        ParityAuditCategory::Performance,
    ] {
        assert!(
            PARITY_AUDIT_REQUIREMENTS
                .iter()
                .any(|requirement| requirement.categories.contains(&category)),
            "KatanA parity audit must include {:?} evidence",
            category
        );
    }
}

#[test]
fn katana_reference_artifacts_are_all_explicitly_audited() -> Result<(), Box<dyn std::error::Error>>
{
    let root = workspace_root()?;
    let mut actual = reference_artifact_files(&root)?;
    let mut expected = audited_reference_artifact_files();
    actual.sort();
    expected.sort();
    actual.dedup();
    expected.dedup();

    assert_eq!(
        expected, actual,
        "all KatanA reference artifacts must be explicitly listed in viewer/export parity audit"
    );
    Ok(())
}

#[test]
fn openspec_requirements_are_connected_to_storybook_gates() -> Result<(), Box<dyn std::error::Error>>
{
    let root = workspace_root()?;
    let spec = std::fs::read_to_string(root.join(
        "openspec/changes/v0-2-0-markdown-viewer-kuc-integration/specs/markdown-viewer-kuc-integration/spec.md",
    ))?;
    let justfile = std::fs::read_to_string(root.join("Justfile"))?;
    let storybook_check = recipe_body(&justfile, "storybook-check")?;
    let registry_consumer_suite = storybook_check.contains("storybook-registry-consumer-suite");

    for requirement in openspec_requirement_names(&spec) {
        let parity = OPENSPEC_REQUIREMENT_PARITY
            .iter()
            .find(|parity| parity.requirement == requirement);
        assert!(
            parity.is_some(),
            "OpenSpec requirement `{requirement}` has no parity gate mapping"
        );
        let Some(parity) = parity else {
            continue;
        };
        for gate in parity.gates {
            assert!(
                justfile.contains(gate),
                "OpenSpec requirement `{}` references unknown gate `{gate}`",
                parity.requirement
            );
            assert!(
                registry_consumer_suite
                    || storybook_check.contains(gate)
                    || *gate == "storybook-check",
                "OpenSpec requirement `{}` gate `{gate}` is not connected to storybook-check",
                parity.requirement
            );
        }
    }
    Ok(())
}

#[test]
fn openspec_requirements_are_connected_to_score_check() -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let spec = std::fs::read_to_string(root.join(
        "openspec/changes/v0-2-0-markdown-viewer-kuc-integration/specs/markdown-viewer-kuc-integration/spec.md",
    ))?;
    let justfile = std::fs::read_to_string(root.join("Justfile"))?;
    let score_check = recipe_body(&justfile, "storybook-score-check")?;

    for requirement in openspec_requirement_names(&spec) {
        let parity = OPENSPEC_REQUIREMENT_PARITY
            .iter()
            .find(|parity| parity.requirement == requirement);
        assert!(
            parity.is_some(),
            "OpenSpec requirement `{requirement}` has no parity gate mapping"
        );
        let Some(parity) = parity else {
            continue;
        };
        for gate in parity.gates {
            assert!(
                justfile.contains(gate),
                "OpenSpec requirement `{}` references unknown gate `{gate}`",
                parity.requirement
            );
            assert!(
                *gate == "storybook-score-check" || score_check.contains(gate),
                "OpenSpec requirement `{}` gate `{gate}` is not connected to storybook-score-check",
                parity.requirement
            );
        }
    }
    Ok(())
}

#[test]
fn openspec_runtime_parity_audit_covers_reported_visual_and_performance_risks() {
    assert_openspec_requirement_gates(
        "Markdown viewer はDocument modeとSlideshow modeを切り替えられなければならない",
        &[
            "storybook-slideshow-check-core",
            "storybook-slideshow-screenshot-smoke",
            "storybook-window-slideshow-screenshot-smoke",
            "storybook-performance-check-core",
        ],
    );
    assert_openspec_requirement_gates(
        "external artifactは並列load可能なassetとして扱われなければならない",
        &[
            "storybook-diagram-load-check-core",
            "storybook-window-diagram-screenshot-smoke",
            "storybook-window-drawio-diagram-screenshot-smoke",
            "storybook-performance-check-core",
        ],
    );
    assert_openspec_requirement_gates(
        "hover / selection / media controls をinteraction設定で制御しなければならない",
        &[
            "storybook-hover-contract-check-core",
            "storybook-selection-contract-check-core",
            "storybook-window-selection-screenshot-smoke",
            "storybook-media-control-clickability-check-full-core",
            "storybook-window-diagram-screenshot-smoke",
            "storybook-window-drawio-diagram-screenshot-smoke",
        ],
    );
    assert_openspec_requirement_gates(
        "表示テキストは選択とclipboard copyができなければならない",
        &[
            "storybook-selection-contract-check-core",
            "storybook-selection-screenshot-smoke",
            "storybook-window-selection-screenshot-smoke",
            "storybook-clipboard-smoke",
            "storybook-clipboard-keyboard-smoke",
            "storybook-clipboard-drag-smoke",
        ],
    );
}

fn assert_openspec_requirement_gates(requirement: &str, expected_gates: &[&str]) {
    let parity = OPENSPEC_REQUIREMENT_PARITY
        .iter()
        .find(|parity| parity.requirement == requirement);
    assert!(
        parity.is_some(),
        "OpenSpec requirement `{requirement}` has no parity mapping"
    );
    let Some(parity) = parity else {
        return;
    };
    for gate in expected_gates {
        assert!(
            parity.gates.contains(gate),
            "OpenSpec requirement `{requirement}` must include parity gate `{gate}`"
        );
    }
}

#[test]
fn toc_openspec_requirement_uses_dedicated_storybook_gate() {
    let parity = OPENSPEC_REQUIREMENT_PARITY
        .iter()
        .find(|parity| parity.requirement == "KMM AST由来の目次を表示しなければならない");
    assert!(parity.is_some(), "TOC OpenSpec requirement must be mapped");
    let Some(parity) = parity else {
        return;
    };

    assert!(
        parity.gates.contains(&"storybook-toc-check-core"),
        "TOC OpenSpec requirement must be connected to storybook-toc-check-core"
    );
}

fn workspace_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    match manifest_dir.parent().and_then(Path::parent) {
        Some(root) => Ok(root.to_path_buf()),
        None => Err(format!(
            "crate manifest must be below workspace crates directory: {}",
            manifest_dir.display()
        )
        .into()),
    }
}

struct ReferenceRequirement {
    directory: &'static str,
    extension: &'static str,
}

struct ParityAuditRequirement {
    name: &'static str,
    reference_artifact: &'static str,
    gates: &'static [&'static str],
    categories: &'static [ParityAuditCategory],
}

struct RuntimeParityRequirement {
    name: &'static str,
    gate: &'static str,
    supporting_gates: &'static [&'static str],
    categories: &'static [ParityAuditCategory],
}

struct OpenSpecRequirementParity {
    requirement: &'static str,
    gates: &'static [&'static str],
}

struct ReferenceArtifactPayload {
    relative_path: &'static str,
}

impl ReferenceArtifactPayload {
    const fn new(relative_path: &'static str) -> Self {
        Self { relative_path }
    }

    fn assert_valid(&self, root: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let path = root.join(self.relative_path);
        let bytes = std::fs::read(&path)?;
        assert!(
            bytes.len() >= self.minimum_size(),
            "KatanA reference artifact is too small to prove parity: {} bytes={} minimum={}",
            self.relative_path,
            bytes.len(),
            self.minimum_size()
        );
        match path.extension().and_then(std::ffi::OsStr::to_str) {
            Some("html") => self.assert_html(&bytes),
            Some("pdf") => self.assert_starts_with(&bytes, b"%PDF-", "PDF"),
            Some("png") => self.assert_starts_with(&bytes, b"\x89PNG\r\n\x1a\n", "PNG"),
            Some(extension) => {
                return Err(format!(
                    "unsupported KatanA reference artifact extension `{}`: {}",
                    extension, self.relative_path
                )
                .into());
            }
            None => {
                return Err(format!(
                    "KatanA reference artifact has no extension: {}",
                    self.relative_path
                )
                .into());
            }
        }
        Ok(())
    }

    fn minimum_size(&self) -> usize {
        match Path::new(self.relative_path)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
        {
            Some("html") => 512,
            Some("pdf") => 1024,
            Some("png") => 1024,
            _ => 1,
        }
    }

    fn assert_html(&self, bytes: &[u8]) {
        let text = std::str::from_utf8(bytes);
        assert!(
            text.is_ok(),
            "KatanA HTML reference artifact is not UTF-8: {}",
            self.relative_path
        );
        let Ok(text) = text else {
            return;
        };
        assert!(
            text.trim_start().starts_with("<!doctype html>"),
            "KatanA HTML reference artifact must start with doctype: {}",
            self.relative_path
        );
        assert!(
            text.contains("data-kdv-export-style"),
            "KatanA HTML reference artifact must contain export style payload: {}",
            self.relative_path
        );
    }

    fn assert_starts_with(&self, bytes: &[u8], signature: &[u8], label: &str) {
        assert!(
            bytes.starts_with(signature),
            "KatanA {label} reference artifact has invalid signature: {}",
            self.relative_path
        );
    }
}

impl OpenSpecRequirementParity {
    const fn new(requirement: &'static str, gates: &'static [&'static str]) -> Self {
        Self { requirement, gates }
    }
}

impl RuntimeParityRequirement {
    const fn new(
        name: &'static str,
        gate: &'static str,
        supporting_gates: &'static [&'static str],
        categories: &'static [ParityAuditCategory],
    ) -> Self {
        Self {
            name,
            gate,
            supporting_gates,
            categories,
        }
    }

    fn has_gate_for_category(&self, category: ParityAuditCategory) -> bool {
        gate_matches_category(self.gate, category)
            || self
                .supporting_gates
                .iter()
                .any(|gate| gate_matches_category(gate, category))
    }
}

fn openspec_requirement_names(spec: &str) -> impl Iterator<Item = &str> {
    spec.lines().filter_map(|line| {
        line.strip_prefix("### Requirement: ")
            .map(str::trim)
            .filter(|name| !name.is_empty())
    })
}

impl ParityAuditRequirement {
    const fn new(
        name: &'static str,
        reference_artifact: &'static str,
        gates: &'static [&'static str],
        categories: &'static [ParityAuditCategory],
    ) -> Self {
        Self {
            name,
            reference_artifact,
            gates,
            categories,
        }
    }

    fn has_gate_for_category(&self, category: ParityAuditCategory) -> bool {
        self.gates
            .iter()
            .any(|gate| gate_matches_category(gate, category))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParityAuditCategory {
    Visual,
    Semantic,
    Interaction,
    Performance,
}

fn gate_matches_category(gate: &str, category: ParityAuditCategory) -> bool {
    match category {
        ParityAuditCategory::Visual => {
            gate.contains("visual")
                || gate.contains("screenshot")
                || gate.contains("surface_equivalence")
                || gate.contains("frame_matches_export_surface")
        }
        ParityAuditCategory::Semantic => {
            gate.contains("fixture_score_matrix")
                || gate.contains("surface_equivalence")
                || gate.contains("frame_matches_export_surface")
                || gate.contains("link-footnote")
                || gate.contains("search")
                || gate.contains("selection")
                || gate.contains("task-checkbox")
                || gate.contains("accordion")
        }
        ParityAuditCategory::Interaction => {
            gate.contains("window")
                || gate.contains("interaction")
                || gate.contains("link-footnote")
                || gate.contains("search")
                || gate.contains("slideshow")
                || gate.contains("selection")
                || gate.contains("media-control")
                || gate.contains("settings")
                || gate.contains("task-checkbox")
                || gate.contains("accordion")
        }
        ParityAuditCategory::Performance => gate.contains("performance"),
    }
}

impl ReferenceRequirement {
    const fn new(directory: &'static str, extension: &'static str) -> Self {
        Self {
            directory,
            extension,
        }
    }

    fn assert_satisfied(&self, root: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let directory = root.join(self.directory);
        assert!(
            directory.is_dir(),
            "missing KatanA reference directory in KDV assets: {}",
            directory.display()
        );
        assert!(
            self.has_reference_artifact(&directory)?,
            "missing KatanA reference artifact in KDV assets: {}/*.{}",
            directory.display(),
            self.extension
        );
        Ok(())
    }

    fn has_reference_artifact(&self, directory: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        for entry in std::fs::read_dir(directory)? {
            if self.is_reference_artifact(&entry?.path()) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn is_reference_artifact(&self, path: &Path) -> bool {
        path.is_file()
            && path.file_name().and_then(|name| name.to_str()) != Some(".gitkeep")
            && path.extension().and_then(|extension| extension.to_str()) == Some(self.extension)
    }
}

fn reference_artifact_files(root: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    collect_reference_artifact_files(root, &root.join("assets/reference/katana"), &mut files)?;
    Ok(files)
}

fn collect_reference_artifact_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_reference_artifact_files(root, &path, files)?;
            continue;
        }
        if path.is_file() && is_audited_reference_artifact(&path) {
            files.push(relative_reference_path(root, &path)?);
        }
    }
    Ok(())
}

fn is_audited_reference_artifact(path: &Path) -> bool {
    matches!(
        path.extension().and_then(std::ffi::OsStr::to_str),
        Some("html" | "pdf" | "png")
    )
}

fn audited_reference_artifact_files() -> Vec<String> {
    let mut files = Vec::new();
    files.extend(
        VISUAL_PARITY_REFERENCE_FILES
            .iter()
            .map(|path| (*path).to_string()),
    );
    files.extend(
        VIEWER_EXPORT_PARITY_REFERENCE_FILES
            .iter()
            .map(|path| (*path).to_string()),
    );
    files.extend(
        PARITY_AUDIT_REQUIREMENTS
            .iter()
            .map(|requirement| requirement.reference_artifact.to_string()),
    );
    files
}

fn relative_reference_path(root: &Path, path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    Ok(path
        .strip_prefix(root)?
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/"))
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
