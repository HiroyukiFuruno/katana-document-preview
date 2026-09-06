#!/usr/bin/env python3
"""Verify the release contract declared by the active KDV OpenSpec target."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import tempfile
from pathlib import Path

from toml_compat import loads as toml_loads


VERSION_RE = re.compile(r"^v(?P<major>0|[1-9][0-9]*)\.(?P<minor>0|[1-9][0-9]*)\.(?P<patch>0|[1-9][0-9]*)$")
REGISTRY_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"
RELEASE_CONTRACT = "multi-format-viewer"
KRR_MIN_VERSION = (0, 4, 19)
# Cargoの裸のバージョン指定はcaret互換のため、KRRを完全固定しない。
KRR_DECLARED_VERSION = ".".join(map(str, KRR_MIN_VERSION))
KRR_VERSION_REQUIREMENT = "^0.4.19"
KRR_LOCK_VERSION_RE = re.compile(r"^(?P<major>[0-9]+)\.(?P<minor>[0-9]+)\.(?P<patch>[0-9]+)$")
V8_VERSION = "152.2.0"
V8_DECLARED_VERSION = f"={V8_VERSION}"
ADAPTER_SOURCES = (
    "crates/katana-document-viewer/src/browser_session.rs",
    "crates/katana-document-viewer/src/browser_session_command_coalescing.rs",
    "crates/katana-document-viewer/src/browser_session_command_queue.rs",
    "crates/katana-document-viewer/src/browser_session_state.rs",
    "crates/katana-document-viewer/src/browser_session_types.rs",
    "crates/katana-document-viewer/src/browser_session_worker.rs",
    "crates/katana-document-viewer/src/browser_session_worker_startup.rs",
)
FORBIDDEN_ADAPTER_MARKERS = (
    "html5ever",
    "markup5ever",
    "cssparser",
    "v8::",
    "HtmlParser",
    "HtmlRenderer",
    "HtmlBrowserProcess",
    "headless_chrome",
    "Chromium",
    "WebView",
    "KRR_CHROME_BIN",
)
SELECTED_ENGINES = {
    "hayro": ("hayro", "0.7.1"),
    "office2pdf": ("office2pdf-katana", "0.6.10"),
    "ironcalc": ("ironcalc", "0.8.3"),
}
LINUX_SANDBOX_DEPENDENCIES = {
    "libc": "0.2.189",
    "seccompiler": "0.5.0",
    "skarn-sandbox": "1.0.1",
}
KUC_VERSION = "0.3.7"
KUC_DECLARED_VERSION = f"={KUC_VERSION}"
MULTI_FORMAT_SOURCES = (
    "crates/katana-document-viewer/src/multi_format/artifact.rs",
    "crates/katana-document-viewer/src/multi_format/capability.rs",
    "crates/katana-document-viewer/src/multi_format/diagnostic.rs",
    "crates/katana-document-viewer/src/multi_format/office_preflight.rs",
    "crates/katana-document-viewer/src/multi_format/office_static_adapter.rs",
    "crates/katana-document-viewer/src/multi_format/office_worker_constraints.rs",
    "crates/katana-document-viewer/src/multi_format/office_worker_entrypoint.rs",
    "crates/katana-document-viewer/src/multi_format/office_worker_fonts.rs",
    "crates/katana-document-viewer/src/multi_format/office_worker_network_seccomp.rs",
    "crates/katana-document-viewer/src/multi_format/pdf_adapter.rs",
    "crates/katana-document-viewer/src/multi_format/spreadsheet_engine.rs",
    "crates/katana-document-viewer/src/multi_format/spreadsheet_worker_entrypoint.rs",
    "crates/katana-document-viewer/src/multi_format/spreadsheet_worker_parent.rs",
    "crates/katana-document-viewer/src/multi_format/spreadsheet_worker_spawn_windows.rs",
    "crates/katana-document-viewer/src/multi_format/spreadsheet_worker_spawn_windows_stderr.rs",
    "crates/katana-document-viewer/src/multi_format/windows_worker_executable.rs",
    "crates/katana-document-viewer/src/document_surface/mod.rs",
    "crates/katana-document-viewer/src/document_surface/frame.rs",
    "crates/katana-document-viewer/src/document_surface/page_surface.rs",
    "crates/katana-document-viewer/src/document_surface/spreadsheet_grid.rs",
)
MULTI_FORMAT_TESTS = (
    "crates/katana-document-viewer/tests/multi_format_office_preflight_contract.rs",
    "crates/katana-document-viewer/tests/multi_format_office_worker_contract.rs",
    "crates/katana-document-viewer/tests/multi_format_pdf_contract.rs",
    "crates/katana-document-viewer/tests/multi_format_source_contract.rs",
    "crates/katana-document-viewer/tests/multi_format_xlsx_contract.rs",
)
FORBIDDEN_ENGINE_PACKAGES = {
    "chromiumoxide",
    "headless_chrome",
    "pdfium-render",
    "web-view",
    "wry",
}
OFFICE_FONT_SOURCE_COMMIT = "2d85e20401920891efb7cd6272d6339685df2820"
OFFICE_FONT_HASHES = {
    "Carlito-Bold.ttf": "bb5d20f79b82599ec72983597437373a80f2d2085fa91fc144fd74e876a594db",
    "Carlito-BoldItalic.ttf": "b32928186c119599e03ca6a1ffc680fdcb7fac95772f4b95d989cf6cd3861517",
    "Carlito-Italic.ttf": "0b019225e58d702bfedcbd35c21696769f8ee115cb6343f84c2f240312450d1c",
    "Carlito-Regular.ttf": "f6418f708baede9789daef5d458c0f53d2a888af9820e8062934e504fedc6595",
    "NotoSansJP-VariableFont_wght.ttf": "c2f3b4d463500a2ddcd3849cded1fceeb9fd6d1c32e6cbecd568453ba50fc68f",
}


def parse_version(value: str) -> tuple[int, int, int]:
    match = VERSION_RE.fullmatch(value)
    if match is None:
        raise ValueError(f"invalid release version: {value}")
    return tuple(int(match.group(name)) for name in ("major", "minor", "patch"))


def release_contract(root: Path, target_version: str) -> str:
    target = parse_version(target_version)
    targets = json.loads((root / "openspec/release-targets.json").read_text(encoding="utf-8"))
    if targets.get("schema_version") != "kdv.release-targets.v1":
        raise ValueError("unsupported OpenSpec release target schema")
    current = targets.get("current")
    if not isinstance(current, dict):
        raise ValueError("current release target is required")
    minor_line = current.get("minor_line")
    contract = current.get("release_contract")
    if minor_line != f"{target[0]}.{target[1]}":
        raise ValueError(f"{target_version} is outside the declared KDV release line {minor_line}.x")
    if contract != RELEASE_CONTRACT:
        raise ValueError(f"unsupported KDV release contract: {contract}")
    return contract


def manifest_errors(manifest: str) -> list[str]:
    workspace = toml_loads(manifest)
    dependencies = workspace.get("workspace", {}).get("dependencies", {})
    errors: list[str] = []
    if dependency_version(dependencies.get("katana-render-runtime")) != KRR_DECLARED_VERSION:
        errors.append(
            "Cargo.toml must depend on "
            f"katana-render-runtime = \"{KRR_DECLARED_VERSION}\"."
        )
    v8 = dependencies.get("v8")
    if dependency_version(v8) != V8_DECLARED_VERSION:
        errors.append(f"Cargo.toml must pin v8 to {V8_DECLARED_VERSION}.")
    if not isinstance(v8, dict) or "simdutf" not in v8.get("features", []):
        errors.append("Cargo.toml must retain the v8 simdutf feature.")
    return errors


def dependency_version(declared: object) -> str | None:
    if isinstance(declared, str):
        return declared
    if not isinstance(declared, dict):
        return None
    if any(key in declared for key in ("path", "git")):
        return None
    version = declared.get("version")
    return version if isinstance(version, str) else None


def krr_lock_version_is_allowed(version: object) -> bool:
    if not isinstance(version, str):
        return False
    match = KRR_LOCK_VERSION_RE.fullmatch(version)
    if match is None:
        return False
    parsed = tuple(int(match.group(name)) for name in ("major", "minor", "patch"))
    return (
        parsed[0] == KRR_MIN_VERSION[0]
        and parsed[1] == KRR_MIN_VERSION[1]
        and parsed[2] >= KRR_MIN_VERSION[2]
    )


def lockfile_errors(lockfile: str) -> list[str]:
    lock = toml_loads(lockfile)
    packages = [
        package
        for package in lock.get("package", [])
        if package.get("name") == "katana-render-runtime"
    ]
    if len(packages) != 1:
        return ["Cargo.lock must contain exactly one katana-render-runtime package."]
    package = packages[0]
    errors: list[str] = []
    if not krr_lock_version_is_allowed(package.get("version")):
        errors.append(
            "katana-render-runtime must resolve a "
            f"{KRR_VERSION_REQUIREMENT}-compatible version from crates.io."
        )
    if package.get("source") != REGISTRY_SOURCE:
        errors.append("katana-render-runtime must resolve from crates.io, not a path or git override.")
    checksum = package.get("checksum")
    if not isinstance(checksum, str) or not re.fullmatch(r"[0-9a-f]{64}", checksum):
        errors.append("katana-render-runtime crates.io lock entry must include a SHA-256 checksum.")
    v8_packages = [package for package in lock.get("package", []) if package.get("name") == "v8"]
    if len(v8_packages) != 1:
        errors.append("Cargo.lock must contain exactly one v8 package.")
        return errors
    v8 = v8_packages[0]
    if v8.get("version") != V8_VERSION:
        errors.append(f"v8 must resolve exactly {V8_VERSION}.")
    if v8.get("source") != REGISTRY_SOURCE:
        errors.append("v8 must resolve from crates.io, not a path or git override.")
    v8_checksum = v8.get("checksum")
    if not isinstance(v8_checksum, str) or not re.fullmatch(r"[0-9a-f]{64}", v8_checksum):
        errors.append("v8 crates.io lock entry must include a SHA-256 checksum.")
    return errors


def multi_format_manifest_errors(root: Path, _target_version: str) -> list[str]:
    workspace = toml_loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    members = workspace.get("workspace", {}).get("members", [])
    dependencies = workspace.get("workspace", {}).get("dependencies", {})
    errors: list[str] = []
    if "crates/katana-document-viewer-kuc" in members or (
        root / "crates/katana-document-viewer-kuc"
    ).exists():
        errors.append("the cross-layer katana-document-viewer-kuc crate must not exist.")
    for name, (package, version) in SELECTED_ENGINES.items():
        declared = dependencies.get(name)
        if dependency_version(declared) != f"={version}":
            errors.append(f"Cargo.toml must pin {name} to ={version}.")
        if package != name and (
            not isinstance(declared, dict) or declared.get("package") != package
        ):
            errors.append(f"Cargo.toml must resolve {name} from package {package}.")
    for name, version in LINUX_SANDBOX_DEPENDENCIES.items():
        declared = dependencies.get(name)
        if dependency_version(declared) != f"={version}":
            errors.append(f"Cargo.toml must pin {name} to ={version}.")

    kuc = dependencies.get("katana-ui-core")
    if (
        not isinstance(kuc, dict)
        or dependency_version(kuc) != KUC_DECLARED_VERSION
        or "raster-host" not in kuc.get("features", [])
    ):
        errors.append(
            "workspace katana-ui-core must use the exact registry KUC "
            f"{KUC_DECLARED_VERSION} raster-host API."
        )
    if "katana-ui-core-storybook" in dependencies:
        errors.append(
            "workspace must resolve KUC through one katana-ui-core registry dependency, "
            "not a second Storybook alias."
        )

    core_manifest = toml_loads(
        (root / "crates/katana-document-viewer/Cargo.toml").read_text(encoding="utf-8")
    )
    core_dependencies = core_manifest.get("dependencies", {})
    core_kuc = core_dependencies.get("katana-ui-core")
    if dependency_version(core_kuc) != KUC_DECLARED_VERSION:
        errors.append(
            "KDV document surface must depend on the exact crates.io "
            f"katana-ui-core {KUC_DECLARED_VERSION}."
        )
    elif isinstance(core_kuc, dict) and any(key in core_kuc for key in ("path", "git", "optional")):
        errors.append("KDV document surface KUC dependency must be required and registry-only.")
    for dependency in ("eframe", "egui"):
        if dependency in core_dependencies:
            errors.append(f"KDV must remain UI-backend neutral and must not depend on {dependency}.")
    features = core_manifest.get("features", {})
    if "egui" in features:
        errors.append("KDV must not expose an egui feature.")
    return errors


def multi_format_lockfile_errors(lockfile: str) -> list[str]:
    packages = toml_loads(lockfile).get("package", [])
    errors: list[str] = []
    selected_packages = {
        package: version for package, version in SELECTED_ENGINES.values()
    }
    for name, version in {
        **selected_packages,
        **LINUX_SANDBOX_DEPENDENCIES,
    }.items():
        registry_matches = [
            package
            for package in packages
            if package.get("name") == name
            and package.get("version") == version
            and package.get("source") == REGISTRY_SOURCE
            and isinstance(package.get("checksum"), str)
            and re.fullmatch(r"[0-9a-f]{64}", package["checksum"])
        ]
        if not registry_matches:
            errors.append(f"Cargo.lock must contain crates.io {name} {version} with checksum.")
    kuc_packages = [
        package for package in packages if package.get("name") == "katana-ui-core"
    ]
    if len(kuc_packages) != 1:
        errors.append("Cargo.lock must contain exactly one katana-ui-core package.")
    else:
        kuc = kuc_packages[0]
        if (
            kuc.get("version") != KUC_VERSION
            or kuc.get("source") != REGISTRY_SOURCE
            or not isinstance(kuc.get("checksum"), str)
            or not re.fullmatch(r"[0-9a-f]{64}", kuc["checksum"])
        ):
            errors.append(
                f"Cargo.lock must contain crates.io katana-ui-core {KUC_VERSION} with checksum."
            )
    if any(package.get("name") == "katana-ui-core-storybook" for package in packages):
        errors.append(
            "Cargo.lock must not contain a separate katana-ui-core-storybook package."
        )
    forbidden = sorted(
        {
            package.get("name")
            for package in packages
            if package.get("name") in FORBIDDEN_ENGINE_PACKAGES
        }
    )
    if forbidden:
        errors.append("forbidden browser/PDF engine packages are locked: " + ", ".join(forbidden) + ".")
    return errors


def multi_format_source_errors(root: Path) -> list[str]:
    errors: list[str] = []
    for relative in (*MULTI_FORMAT_SOURCES, *MULTI_FORMAT_TESTS):
        if not (root / relative).is_file():
            errors.append(f"multi-format release source is missing: {relative}.")
    production = "\n".join(
        (root / relative).read_text(encoding="utf-8")
        for relative in MULTI_FORMAT_SOURCES
        if (root / relative).is_file()
    )
    for marker in ("Chromium", "WebView", "PDFium", "headless_chrome", "pdfium_render"):
        if marker in production:
            errors.append(f"multi-format production source must not own forbidden engine {marker}.")
    required = (
        "OfficePackagePreflight",
        "OfficeWorkerEntrypoint",
        "PdfViewerSession",
        "SpreadsheetViewerSession",
        "SpreadsheetWorkerEntrypoint",
        "SeccompFilter",
        "NetPolicy::Deny",
        "GenericGrid",
        "ImageSurface",
        "DocumentSurfaceFrame",
        "DocumentPageSurfaceFrame",
        "DocumentGridSurfaceFrame",
        "SpreadsheetGridSurface",
    )
    missing = [token for token in required if token not in production]
    if missing:
        errors.append("multi-format implementation is incomplete: " + ", ".join(missing) + ".")
    public_surface = (root / "crates/katana-document-viewer/src/lib.rs").read_text(
        encoding="utf-8"
    )
    if "katana_ui_core" in public_surface or "katana-document-viewer-kuc" in public_surface:
        errors.append("KDV public API must not expose KUC types or the forbidden cross-layer crate.")
    if "egui" in production:
        errors.append("KDV multi-format production source must remain egui-independent.")
    return errors


def office_font_contract_errors(root: Path) -> list[str]:
    font_root = root / "crates/katana-document-viewer/assets/fonts"
    font_workflow_path = "crates/katana-document-viewer/assets/fonts/**"
    errors: list[str] = []
    for name, expected in OFFICE_FONT_HASHES.items():
        path = font_root / name
        if not path.is_file():
            errors.append(f"deterministic Office font is missing: {name}.")
            continue
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != expected:
            errors.append(f"deterministic Office font checksum changed: {name}.")
    source = font_root / "SOURCE.md"
    source_text = source.read_text(encoding="utf-8") if source.is_file() else ""
    if OFFICE_FONT_SOURCE_COMMIT not in source_text:
        errors.append("Office font source commit is missing from SOURCE.md.")
    license_path = font_root / "OFL.txt"
    license_text = license_path.read_text(encoding="utf-8") if license_path.is_file() else ""
    if "SIL OPEN FONT LICENSE Version 1.1" not in license_text:
        errors.append("Office fallback font OFL 1.1 license is missing.")
    for workflow_name in ("test-and-build.yml", "release-preflight.yml"):
        workflow_path = root / ".github/workflows" / workflow_name
        workflow_text = (
            workflow_path.read_text(encoding="utf-8") if workflow_path.is_file() else ""
        )
        if font_workflow_path not in workflow_text:
            errors.append(
                f"{workflow_name} must run when deterministic Office fonts change."
            )
    contract = (
        root / "crates/katana-document-viewer/tests/multi_format_office_worker_contract.rs"
    ).read_text(encoding="utf-8")
    for token in (
        "paragraph_row_bands",
        "PPTX paragraphs must preserve the 21.6pt line advance",
        "each Japanese glyph must render as ink instead of tofu",
        "Japanese glyphs must not collapse to repeated tofu boxes",
    ):
        if token not in contract:
            errors.append(f"Office cross-platform pixel contract is missing: {token}.")
    return errors


def cargo_config_errors(config: Path) -> list[str]:
    if not config.exists():
        return []
    text = config.read_text(encoding="utf-8")
    if "katana-render-runtime" in text and "path" in text:
        return ["KDV release must not use a local katana-render-runtime path overlay."]
    return []


def adapter_source_errors(root: Path) -> list[str]:
    errors: list[str] = []
    for relative in ADAPTER_SOURCES:
        path = root / relative
        if not path.is_file():
            errors.append(f"browser-session adapter source is missing: {relative}.")
            continue
        source = path.read_text(encoding="utf-8")
        for marker in FORBIDDEN_ADAPTER_MARKERS:
            if marker in source:
                errors.append(f"browser-session adapter must not own {marker}: {relative}.")
    return errors


def integration_contract_errors(root: Path) -> list[str]:
    path = root / "crates/katana-document-viewer/tests/browser_session_adapter_contract.rs"
    if not path.is_file():
        return ["browser-session adapter integration contract is missing."]
    source = path.read_text(encoding="utf-8")
    required = (
        "public_adapter_forwards_in_process_runtime_commands",
        "adapter_boundary_does_not_reintroduce_html_semantics_or_an_external_browser",
        "burst_continuous_input_preserves_discrete_input_and_frame_progress",
        "HtmlBrowserSource::new",
        "adapter.navigate",
        "adapter.refresh_frame",
        "adapter.close",
    )
    missing = [token for token in required if token not in source]
    if not missing:
        return []
    return ["browser-session adapter integration contract is incomplete: " + ", ".join(missing) + "."]


def justfile_errors(justfile: str) -> list[str]:
    required = (
        'COVERAGE_MIN_LINES := "100"',
        'COVERAGE_MAX_UNCOVERED_LINES := "0"',
        "--fail-under-functions 100 --fail-under-lines {{COVERAGE_MIN_LINES}} --fail-uncovered-functions 0 --fail-uncovered-lines {{COVERAGE_MAX_UNCOVERED_LINES}}",
        "release-contract-check:",
        "verify-release-contract.py --target-version \"{{TAG}}\"",
        "{{CARGO}} test -p katana-document-viewer --test browser_session_adapter_contract --locked",
        "release-verify: release-contract-check check coverage",
        'COVERAGE_TARGET_PACKAGES := "-p katana-document-viewer"',
        "coverage-v8-refresh:",
        "{{CARGO}} clean -p v8 --target-dir target/llvm-cov-target",
        "coverage: coverage-v8-refresh",
        "coverage-missing: coverage-v8-refresh",
        "document-surface-boundary-check:",
        "scripts/document-surface-boundary-check.sh",
        "v8-runtime-check:",
        "verify-v8-runtime-singleton.py",
        "office-profiling-stage-check:",
        "verify-office-profiling-stages.py",
        "office-performance-harness-check:",
        "measure-office-first-frame.py --self-test",
        "office-fidelity-harness-check:",
        "measure-office-fidelity.py --self-test",
        "measure-office-fidelity.py --verify-record",
        "verify-registry-consumer-link.py --self-test",
    )
    missing = [token for token in required if token not in justfile]
    if not missing:
        return []
    return ["release contract recipes are incomplete: " + ", ".join(missing) + "."]


def staged_publish_errors(script: str) -> list[str]:
    ordered = (
        "cargo publish -p katana-document-viewer --locked",
        "wait_until_published katana-document-viewer",
        "verify-registry-consumer-link.py",
    )
    positions = [script.find(token) for token in ordered]
    publishes_adapter = "cargo publish -p katana-document-viewer-kuc" in script
    if (
        all(position >= 0 for position in positions)
        and positions == sorted(positions)
        and not publishes_adapter
    ):
        return []
    return [
        "publish script must publish only the KDV core crate, await its registry entry, "
        "and link a fresh registry consumer."
    ]


def registry_consumer_contract_errors(root: Path) -> list[str]:
    manifest_path = root / "tools/kdv-v8-registry-consumer/Cargo.toml"
    source_path = root / "tools/kdv-v8-registry-consumer/src/main.rs"
    verifier_path = root / "scripts/release/verify-registry-consumer-link.py"
    errors: list[str] = []
    if not manifest_path.is_file():
        errors.append("V8 registry consumer manifest is missing.")
    else:
        manifest = manifest_path.read_text(encoding="utf-8")
        if 'katana-document-viewer = "__KDV_VERSION__"' not in manifest:
            errors.append("V8 registry consumer must substitute an exact KDV version.")
        if "path" in manifest or "git" in manifest:
            errors.append("V8 registry consumer template must not contain path or git overrides.")
    if not source_path.is_file():
        errors.append("V8 registry consumer link source is missing.")
    elif "KrrMathRenderEngine::render_display_svg" not in source_path.read_text(encoding="utf-8"):
        errors.append("V8 registry consumer must link KDV's public KRR entrypoint.")
    if not verifier_path.is_file():
        errors.append("V8 registry consumer verifier is missing.")
    else:
        verifier = verifier_path.read_text(encoding="utf-8")
        for token in ('"metadata"', '"build"', '"tree"', REGISTRY_SOURCE):
            if token not in verifier:
                errors.append(f"V8 registry consumer verifier is missing {token}.")
    return errors


def release_workflow_errors(preflight: str, release: str) -> list[str]:
    workflows = {
        "release preflight": (preflight, "release-check"),
        "release workflow": (release, "release-verify"),
    }
    errors: list[str] = []
    for label, (workflow, required_recipe) in workflows.items():
        artifact_command = "xvfb-run -a just storybook-release-acceptance-artifacts"
        recipe_command = (
            'xvfb-run -a just VERSION="${{ steps.version.outputs.version }}" '
            f"{required_recipe}"
        )
        if recipe_command not in workflow:
            errors.append(f"{label} must run the KDV {required_recipe} recipe.")
        artifact_position = workflow.find(artifact_command)
        recipe_position = workflow.find(recipe_command)
        if artifact_position < 0 or recipe_position < 0 or artifact_position > recipe_position:
            errors.append(
                f"{label} must refresh static and live Storybook acceptance artifacts "
                f"before the KDV {required_recipe} recipe."
            )
        diagnostic_position = workflow.find(
            "name: Upload Storybook preview-crop diagnostics on failure"
        )
        diagnostic = workflow[diagnostic_position:] if diagnostic_position >= 0 else ""
        diagnostic_required = (
            "if: failure()",
            "uses: actions/upload-artifact@v4",
            "path: target/acceptance/preview-crop-reference",
            "if-no-files-found: warn",
        )
        if diagnostic_position <= recipe_position or any(
            marker not in diagnostic for marker in diagnostic_required
        ):
            errors.append(
                f"{label} must upload preview-crop diagnostics after a failed release gate."
            )
    return errors


def v8_cache_refresh_errors(workflows: dict[str, str]) -> list[str]:
    errors: list[str] = []
    for label, workflow in workflows.items():
        cache_position = workflow.find("uses: Swatinem/rust-cache@v2")
        refresh_position = workflow.find(
            "name: Refresh V8 link artifact after cache restore"
        )
        clean_position = workflow.find("run: cargo clean -p v8")
        if (
            cache_position == -1
            or refresh_position == -1
            or clean_position == -1
            or not cache_position < refresh_position <= clean_position
        ):
            errors.append(
                f"{label} must refresh V8 link artifacts after restoring the Rust cache."
            )
    return errors


def validate(root: Path, target_version: str) -> list[str]:
    try:
        contract = release_contract(root, target_version)
    except ValueError as error:
        return [str(error)]
    if contract != RELEASE_CONTRACT:
        return [f"unsupported KDV release contract: {contract}"]
    errors = manifest_errors((root / "Cargo.toml").read_text(encoding="utf-8"))
    errors.extend(lockfile_errors((root / "Cargo.lock").read_text(encoding="utf-8")))
    errors.extend(multi_format_manifest_errors(root, target_version))
    errors.extend(multi_format_lockfile_errors((root / "Cargo.lock").read_text(encoding="utf-8")))
    errors.extend(cargo_config_errors(root / ".cargo/config.toml"))
    errors.extend(adapter_source_errors(root))
    errors.extend(integration_contract_errors(root))
    errors.extend(multi_format_source_errors(root))
    errors.extend(office_font_contract_errors(root))
    errors.extend(registry_consumer_contract_errors(root))
    errors.extend(justfile_errors((root / "Justfile").read_text(encoding="utf-8")))
    errors.extend(
        staged_publish_errors(
            (root / "scripts/release/publish-crates.sh").read_text(encoding="utf-8")
        )
    )
    errors.extend(
        release_workflow_errors(
            (root / ".github/workflows/release-preflight.yml").read_text(encoding="utf-8"),
            (root / ".github/workflows/release.yml").read_text(encoding="utf-8"),
        )
    )
    errors.extend(
        v8_cache_refresh_errors(
            {
                "CI workflow": (root / ".github/workflows/test-and-build.yml").read_text(
                    encoding="utf-8"
                ),
                "release preflight": (
                    root / ".github/workflows/release-preflight.yml"
                ).read_text(encoding="utf-8"),
                "release workflow": (root / ".github/workflows/release.yml").read_text(
                    encoding="utf-8"
                ),
            }
        )
    )
    return errors


def self_test() -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        (root / "openspec").mkdir()
        (root / "openspec/release-targets.json").write_text(
            json.dumps(
                {
                    "schema_version": "kdv.release-targets.v1",
                    "current": {
                        "minor_line": "0.4",
                        "change": "adapter",
                        "release_contract": RELEASE_CONTRACT,
                    },
                    "deferred": [],
                }
            ),
            encoding="utf-8",
        )
        assert release_contract(root, "v0.4.0") == RELEASE_CONTRACT
        try:
            release_contract(root, "v0.5.0")
        except ValueError:
            pass
        else:
            raise AssertionError("release contract must reject another release line")
    valid_manifest = (
        "[workspace.dependencies]\n"
        f'katana-render-runtime = "{KRR_DECLARED_VERSION}"\n'
        f'v8 = {{ version = "{V8_DECLARED_VERSION}", features = ["simdutf"] }}\n'
    )
    assert not manifest_errors(valid_manifest)
    assert not manifest_errors(
        "[workspace.dependencies]\n"
        f'katana-render-runtime = {{ version = "{KRR_DECLARED_VERSION}" }}\n'
        f'v8 = {{ version = "{V8_DECLARED_VERSION}", features = ["simdutf"] }}\n'
    )
    assert manifest_errors(valid_manifest.replace(V8_DECLARED_VERSION, "=150.0.0"))
    assert manifest_errors(valid_manifest.replace('features = ["simdutf"]', "features = []"))
    assert manifest_errors(
        '[workspace.dependencies]\nkatana-render-runtime = { path = "../krr" }\n'
    )
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        (root / "crates/katana-document-viewer").mkdir(parents=True)
        selected_dependencies = "\n".join(
            (
                'hayro = "=0.7.1"',
                'office2pdf = { package = "office2pdf-katana", version = "=0.6.10" }',
                'ironcalc = "=0.8.3"',
                'libc = "=0.2.189"',
                'seccompiler = "=0.5.0"',
                'skarn-sandbox = "=1.0.1"',
                f'katana-ui-core = {{ version = "{KUC_DECLARED_VERSION}", features = ["raster-host"] }}',
            )
        )
        (root / "Cargo.toml").write_text(
            f"[workspace]\nmembers = []\n[workspace.dependencies]\n{selected_dependencies}\n",
            encoding="utf-8",
        )
        (root / "crates/katana-document-viewer/Cargo.toml").write_text(
            '[package]\nname = "test"\nversion = "0.0.0"\n'
            f'[dependencies]\nkatana-ui-core = "{KUC_DECLARED_VERSION}"\n',
            encoding="utf-8",
        )
        assert not multi_format_manifest_errors(root, "v0.5.2")
        stale_manifest = (root / "Cargo.toml").read_text(encoding="utf-8").replace(
            'office2pdf = { package = "office2pdf-katana", version = "=0.6.10" }',
            'office2pdf = { package = "office2pdf-katana", version = "=0.6.9" }',
        )
        (root / "Cargo.toml").write_text(stale_manifest, encoding="utf-8")
        assert multi_format_manifest_errors(root, "v0.5.2")
    registry_lock = """
version = 4

[[package]]
name = "katana-render-runtime"
version = "0.4.19"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0000000000000000000000000000000000000000000000000000000000000000"

[[package]]
name = "v8"
version = "152.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0000000000000000000000000000000000000000000000000000000000000000"
"""
    assert not lockfile_errors(registry_lock)
    assert lockfile_errors(registry_lock.replace('version = "0.4.19"', 'version = "0.4.18"'))
    assert not lockfile_errors(registry_lock.replace('version = "0.4.19"', 'version = "0.4.20"'))
    assert not lockfile_errors(registry_lock.replace('version = "0.4.19"', 'version = "0.4.99"'))
    assert lockfile_errors(registry_lock.replace('version = "0.4.19"', 'version = "0.5.0"'))
    duplicate_package = registry_lock.split("[[package]]", maxsplit=1)[1]
    assert lockfile_errors(registry_lock + "\n[[package]]" + duplicate_package)
    assert lockfile_errors(registry_lock.replace(REGISTRY_SOURCE, "path+file:///tmp/krr"))
    assert lockfile_errors(registry_lock.replace('version = "152.2.0"', 'version = "150.0.0"'))
    assert lockfile_errors(
        registry_lock.replace(
            "0000000000000000000000000000000000000000000000000000000000000000",
            "invalid",
        )
    )
    release_preflight = "\n".join(
        (
            "xvfb-run -a just storybook-release-acceptance-artifacts",
            'xvfb-run -a just VERSION="${{ steps.version.outputs.version }}" release-check',
            "name: Upload Storybook preview-crop diagnostics on failure",
            "if: failure()",
            "uses: actions/upload-artifact@v4",
            "path: target/acceptance/preview-crop-reference",
            "if-no-files-found: warn",
        )
    )
    release_workflow = "\n".join(
        (
            "xvfb-run -a just storybook-release-acceptance-artifacts",
            'xvfb-run -a just VERSION="${{ steps.version.outputs.version }}" release-verify',
            "name: Upload Storybook preview-crop diagnostics on failure",
            "if: failure()",
            "uses: actions/upload-artifact@v4",
            "path: target/acceptance/preview-crop-reference",
            "if-no-files-found: warn",
        )
    )
    assert not release_workflow_errors(release_preflight, release_workflow)
    assert release_workflow_errors(
        'xvfb-run -a just VERSION="${{ steps.version.outputs.version }}" release-check\n',
        release_workflow,
    )
    assert release_workflow_errors(
        "\n".join(reversed(release_preflight.splitlines())), release_workflow
    )
    v8_cache_workflow = "\n".join(
        (
            "uses: Swatinem/rust-cache@v2",
            "name: Refresh V8 link artifact after cache restore",
            "run: cargo clean -p v8",
        )
    )
    assert not v8_cache_refresh_errors(
        {
            "CI workflow": v8_cache_workflow,
            "release preflight": v8_cache_workflow,
            "release workflow": v8_cache_workflow,
        }
    )
    assert v8_cache_refresh_errors({"CI workflow": "uses: Swatinem/rust-cache@v2"})
    staged_publish = "\n".join(
        (
            "cargo publish -p katana-document-viewer --locked",
            "wait_until_published katana-document-viewer",
            "verify-registry-consumer-link.py",
        )
    )
    assert not staged_publish_errors(staged_publish)
    assert staged_publish_errors("wait_until_published katana-document-viewer")
    assert staged_publish_errors(
        staged_publish + "\ncargo publish -p katana-document-viewer-kuc --locked"
    )
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        (root / "tools/kdv-v8-registry-consumer/src").mkdir(parents=True)
        (root / "scripts/release").mkdir(parents=True)
        (root / "tools/kdv-v8-registry-consumer/Cargo.toml").write_text(
            '[dependencies]\nkatana-document-viewer = "__KDV_VERSION__"\n',
            encoding="utf-8",
        )
        (root / "tools/kdv-v8-registry-consumer/src/main.rs").write_text(
            "KrrMathRenderEngine::render_display_svg", encoding="utf-8"
        )
        (root / "scripts/release/verify-registry-consumer-link.py").write_text(
            '"metadata"\n"build"\n"tree"\n' + REGISTRY_SOURCE,
            encoding="utf-8",
        )
        assert not registry_consumer_contract_errors(root)
        (root / "tools/kdv-v8-registry-consumer/Cargo.toml").write_text(
            '[dependencies]\nkatana-document-viewer = { path = "../kdv" }\n',
            encoding="utf-8",
        )
        assert registry_consumer_contract_errors(root)
    selected_lock = "version = 4\n\n" + "\n\n".join(
        (
            "[[package]]\n"
            f'name = "{name}"\nversion = "{version}"\n'
            f'source = "{REGISTRY_SOURCE}"\n'
            f'checksum = "{"0" * 64}"'
        )
        for name, version in {
            **{package: version for package, version in SELECTED_ENGINES.values()},
            **LINUX_SANDBOX_DEPENDENCIES,
            "katana-ui-core": KUC_VERSION,
        }.items()
    )
    assert not multi_format_lockfile_errors(selected_lock)
    assert multi_format_lockfile_errors(
        selected_lock.replace('version = "0.6.10"', 'version = "0.6.9"', 1)
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--target-version")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        print("release contract self-test passed")
        return 0
    if args.target_version is None:
        parser.error("--target-version is required unless --self-test is used")
    root = Path(__file__).resolve().parents[2]
    errors = validate(root, args.target_version)
    if errors:
        for error in errors:
            print(f"release contract: {error}")
        return 1
    print(f"release contract passed: {RELEASE_CONTRACT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
