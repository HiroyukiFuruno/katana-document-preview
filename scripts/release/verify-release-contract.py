#!/usr/bin/env python3
"""Verify the release contract declared by the active KDV OpenSpec target."""

from __future__ import annotations

import argparse
import json
import re
import tempfile
import tomllib
from pathlib import Path


VERSION_RE = re.compile(r"^v(?P<major>0|[1-9][0-9]*)\.(?P<minor>0|[1-9][0-9]*)\.(?P<patch>0|[1-9][0-9]*)$")
REGISTRY_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"
RELEASE_CONTRACT = "multi-format-viewer"
KRR_MIN_VERSION = (0, 4, 14)
KRR_DECLARED_VERSION = ".".join(map(str, KRR_MIN_VERSION))
KRR_VERSION_REQUIREMENT = "^0.4.14"
KRR_LOCK_VERSION_RE = re.compile(r"^(?P<major>[0-9]+)\.(?P<minor>[0-9]+)\.(?P<patch>[0-9]+)$")
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
    "hayro": "0.7.1",
    "office2pdf": "0.6.5",
    "ironcalc": "0.8.3",
}
LINUX_SANDBOX_DEPENDENCIES = {
    "libc": "0.2.189",
    "seccompiler": "0.5.0",
    "skarn-sandbox": "1.0.1",
}
KUC_VERSION = "0.3.0"
MULTI_FORMAT_SOURCES = (
    "crates/katana-document-viewer/src/multi_format/artifact.rs",
    "crates/katana-document-viewer/src/multi_format/capability.rs",
    "crates/katana-document-viewer/src/multi_format/diagnostic.rs",
    "crates/katana-document-viewer/src/multi_format/office_preflight.rs",
    "crates/katana-document-viewer/src/multi_format/office_static_adapter.rs",
    "crates/katana-document-viewer/src/multi_format/office_worker_constraints.rs",
    "crates/katana-document-viewer/src/multi_format/office_worker_entrypoint.rs",
    "crates/katana-document-viewer/src/multi_format/office_worker_network_seccomp.rs",
    "crates/katana-document-viewer/src/multi_format/pdf_adapter.rs",
    "crates/katana-document-viewer/src/multi_format/spreadsheet_engine.rs",
    "crates/katana-document-viewer/src/multi_format/spreadsheet_worker_parent.rs",
    "crates/katana-document-viewer/src/multi_format/spreadsheet_worker_spawn_windows.rs",
    "crates/katana-document-viewer/src/multi_format/windows_worker_executable.rs",
    "crates/katana-document-viewer/src/document_surface/mod.rs",
    "crates/katana-document-viewer/src/document_surface/page_surface.rs",
    "crates/katana-document-viewer/src/document_surface/spreadsheet_grid.rs",
    "crates/katana-document-viewer/src/document_surface/host.rs",
    "crates/katana-document-viewer/src/document_surface/host/grid.rs",
    "crates/katana-document-viewer/src/document_surface/host/page.rs",
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
    workspace = tomllib.loads(manifest)
    dependencies = workspace.get("workspace", {}).get("dependencies", {})
    if dependency_version(dependencies.get("katana-render-runtime")) == KRR_DECLARED_VERSION:
        return []
    return [
        "Cargo.toml must depend on "
        f"katana-render-runtime = \"{KRR_DECLARED_VERSION}\"."
    ]


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
    return parsed >= KRR_MIN_VERSION and parsed[:2] == KRR_MIN_VERSION[:2]


def lockfile_errors(lockfile: str) -> list[str]:
    lock = tomllib.loads(lockfile)
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
    return errors


def multi_format_manifest_errors(root: Path, _target_version: str) -> list[str]:
    workspace = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    members = workspace.get("workspace", {}).get("members", [])
    dependencies = workspace.get("workspace", {}).get("dependencies", {})
    errors: list[str] = []
    if "crates/katana-document-viewer-kuc" in members or (
        root / "crates/katana-document-viewer-kuc"
    ).exists():
        errors.append("the cross-layer katana-document-viewer-kuc crate must not exist.")
    for name, version in {**SELECTED_ENGINES, **LINUX_SANDBOX_DEPENDENCIES}.items():
        declared = dependencies.get(name)
        if dependency_version(declared) != f"={version}":
            errors.append(f"Cargo.toml must pin {name} to ={version}.")

    kuc = dependencies.get("katana-ui-core")
    kuc_storybook = dependencies.get("katana-ui-core-storybook")
    expected_git = "https://github.com/HiroyukiFuruno/katana-ui-core.git"
    for name, declared in (
        ("katana-ui-core", kuc),
        ("katana-ui-core-storybook", kuc_storybook),
    ):
        if not isinstance(declared, dict) or declared.get("git") != expected_git or declared.get("tag") != "v0.3.0":
            errors.append(f"development-only {name} must resolve from KUC tag v0.3.0.")

    core_manifest = tomllib.loads(
        (root / "crates/katana-document-viewer/Cargo.toml").read_text(encoding="utf-8")
    )
    core_dependencies = core_manifest.get("dependencies", {})
    core_kuc = core_dependencies.get("katana-ui-core")
    if not isinstance(core_kuc, dict) or core_kuc.get("version") != KUC_VERSION:
        errors.append("KDV document surface must depend on crates.io katana-ui-core 0.3.0.")
    elif core_kuc.get("optional") is not True or any(
        key in core_kuc for key in ("path", "git")
    ):
        errors.append("KDV document surface KUC dependency must be optional and registry-only.")
    core_egui = core_dependencies.get("egui")
    if not isinstance(core_egui, dict) or core_egui.get("version") != "0.35":
        errors.append("KDV document surface must use egui 0.35 through its host feature.")
    elif core_egui.get("optional") is not True or any(
        key in core_egui for key in ("path", "git")
    ):
        errors.append("KDV egui host dependency must be optional and registry-only.")
    features = core_manifest.get("features", {})
    if features.get("egui") != ["dep:egui", "dep:katana-ui-core"]:
        errors.append("KDV must own KUC presentation behind its egui feature.")
    return errors


def multi_format_lockfile_errors(lockfile: str) -> list[str]:
    packages = tomllib.loads(lockfile).get("package", [])
    errors: list[str] = []
    for name, version in {
        **SELECTED_ENGINES,
        **LINUX_SANDBOX_DEPENDENCIES,
        "katana-ui-core": KUC_VERSION,
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
        "SeccompFilter",
        "NetPolicy::Deny",
        "GenericGrid",
        "ImageSurface",
        "DocumentSurfaceFrame",
        "DocumentSurfaceHost",
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
        "document-surface-boundary-check:",
        "scripts/document-surface-boundary-check.sh",
    )
    missing = [token for token in required if token not in justfile]
    if not missing:
        return []
    return ["release contract recipes are incomplete: " + ", ".join(missing) + "."]


def staged_publish_errors(script: str) -> list[str]:
    ordered = (
        "cargo publish -p katana-document-viewer --locked",
        "wait_until_published katana-document-viewer",
    )
    positions = [script.find(token) for token in ordered]
    publishes_adapter = "cargo publish -p katana-document-viewer-kuc" in script
    if (
        all(position >= 0 for position in positions)
        and positions == sorted(positions)
        and not publishes_adapter
    ):
        return []
    return ["publish script must publish only the KDV core crate and await its registry entry."]


def release_workflow_errors(preflight: str, release: str) -> list[str]:
    workflows = {
        "release preflight": (preflight, "release-check"),
        "release workflow": (release, "release-verify"),
    }
    errors: list[str] = []
    for label, (workflow, required_recipe) in workflows.items():
        if f'just VERSION="${{{{ steps.version.outputs.version }}}}" {required_recipe}' not in workflow:
            errors.append(f"{label} must run the KDV {required_recipe} recipe.")
        if "storybook-release-acceptance-artifacts" in workflow:
            errors.append(
                f"{label} must not make the legacy Storybook artifact a browser-session release gate."
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
    )
    assert not manifest_errors(valid_manifest)
    assert not manifest_errors(
        "[workspace.dependencies]\n"
        f'katana-render-runtime = {{ version = "{KRR_DECLARED_VERSION}" }}\n'
    )
    assert manifest_errors(
        '[workspace.dependencies]\nkatana-render-runtime = { path = "../krr" }\n'
    )
    registry_lock = """
version = 4

[[package]]
name = "katana-render-runtime"
version = "0.4.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0000000000000000000000000000000000000000000000000000000000000000"
"""
    assert not lockfile_errors(registry_lock)
    assert not lockfile_errors(registry_lock.replace('version = "0.4.14"', 'version = "0.4.15"'))
    assert lockfile_errors(registry_lock.replace('version = "0.4.14"', 'version = "0.4.13"'))
    assert lockfile_errors(registry_lock.replace('version = "0.4.14"', 'version = "0.5.0"'))
    duplicate_package = registry_lock.split("[[package]]", maxsplit=1)[1]
    assert lockfile_errors(registry_lock + "\n[[package]]" + duplicate_package)
    assert lockfile_errors(registry_lock.replace(REGISTRY_SOURCE, "path+file:///tmp/krr"))
    assert lockfile_errors(
        registry_lock.replace(
            "0000000000000000000000000000000000000000000000000000000000000000",
            "invalid",
        )
    )
    release_preflight = 'just VERSION="${{ steps.version.outputs.version }}" release-check\n'
    release_workflow = 'just VERSION="${{ steps.version.outputs.version }}" release-verify\n'
    assert not release_workflow_errors(release_preflight, release_workflow)
    assert release_workflow_errors(
        "storybook-release-acceptance-artifacts\n", release_workflow
    )
    staged_publish = "\n".join(
        (
            "cargo publish -p katana-document-viewer --locked",
            "wait_until_published katana-document-viewer",
        )
    )
    assert not staged_publish_errors(staged_publish)
    assert staged_publish_errors("wait_until_published katana-document-viewer")
    assert staged_publish_errors(
        staged_publish + "\ncargo publish -p katana-document-viewer-kuc --locked"
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
