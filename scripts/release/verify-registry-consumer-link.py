#!/usr/bin/env python3
"""Build a fresh crates.io KDV consumer and verify its V8 link graph."""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import tempfile
from pathlib import Path

from toml_compat import loads as toml_loads


ROOT = Path(__file__).resolve().parents[2]
TEMPLATE = ROOT / "tools/kdv-v8-registry-consumer"
REGISTRY_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"
EXPECTED_V8_VERSION = "152.2.0"
EXPECTED_KRR_VERSION = "0.4.19"
VERSION_RE = re.compile(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$")
ROOT_V8_LINE = re.compile(r"^v8 v(?P<version>[^\s]+)", re.MULTILINE)


def run(command: list[str], *, cwd: Path, environment: dict[str, str]) -> str:
    merged_environment = os.environ.copy()
    merged_environment.update(environment)
    completed = subprocess.run(
        command,
        cwd=cwd,
        env=merged_environment,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    if completed.returncode != 0:
        raise RuntimeError(f"{' '.join(command)} failed:\n{completed.stdout}")
    return completed.stdout


def prepare_consumer(destination: Path, version: str) -> Path:
    consumer = destination / "consumer"
    shutil.copytree(TEMPLATE, consumer)
    manifest_path = consumer / "Cargo.toml"
    manifest = manifest_path.read_text(encoding="utf-8")
    placeholder = 'katana-document-viewer = "__KDV_VERSION__"'
    replacement = f'katana-document-viewer = "={version}"'
    if manifest.count(placeholder) != 1:
        raise RuntimeError("registry consumer template must contain one KDV version placeholder")
    manifest_path.write_text(manifest.replace(placeholder, replacement), encoding="utf-8")
    return consumer


def manifest_errors(manifest_path: Path, version: str) -> list[str]:
    manifest = toml_loads(manifest_path.read_text(encoding="utf-8"))
    dependencies = manifest.get("dependencies", {})
    declared = dependencies.get("katana-document-viewer")
    if declared != f"={version}":
        return ["registry consumer must use the exact published KDV version."]
    if "path" in manifest_path.read_text(encoding="utf-8") or "git" in manifest_path.read_text(
        encoding="utf-8"
    ):
        return ["registry consumer must not use path or git dependencies."]
    return []


def package_errors(metadata: dict[str, object], version: str) -> list[str]:
    packages = metadata.get("packages")
    if not isinstance(packages, list):
        return ["cargo metadata did not return packages."]
    resolve = metadata.get("resolve")
    if not isinstance(resolve, dict) or not isinstance(resolve.get("root"), str):
        return ["cargo metadata did not identify the registry consumer root."]
    root_id = resolve["root"]
    records = [package for package in packages if isinstance(package, dict)]
    if len(records) != len(packages):
        return ["cargo metadata returned an invalid package record."]
    errors = non_registry_source_errors(records, root_id)
    matches = [
        package
        for package in records
        if package.get("name") == "katana-document-viewer" and package.get("version") == version
    ]
    if len(matches) != 1:
        errors.append("fresh consumer did not resolve exactly one published KDV package.")
    elif matches[0].get("source") != REGISTRY_SOURCE:
        errors.append("fresh consumer KDV package must resolve from crates.io, not path or git.")
    errors.extend(exact_registry_package_errors(records, "v8", EXPECTED_V8_VERSION))
    errors.extend(
        exact_registry_package_errors(records, "katana-render-runtime", EXPECTED_KRR_VERSION)
    )
    return errors


def non_registry_source_errors(records: list[dict[str, object]], root_id: str) -> list[str]:
    errors: list[str] = []
    for package in records:
        if package.get("id") == root_id:
            continue
        if package.get("source") == REGISTRY_SOURCE:
            continue
        name = package.get("name")
        source = package.get("source")
        errors.append(
            "fresh consumer resolved a non-registry dependency: "
            f"{name!r} from {source!r}."
        )
    return errors


def exact_registry_package_errors(
    records: list[dict[str, object]], name: str, version: str
) -> list[str]:
    matches = [
        package
        for package in records
        if package.get("name") == name and package.get("version") == version
    ]
    if len(matches) != 1:
        return [f"fresh consumer must resolve exactly {name} {version}."]
    if matches[0].get("source") != REGISTRY_SOURCE:
        return [f"fresh consumer {name} {version} must resolve from crates.io."]
    return []


def tree_errors(tree: str, inverse: str) -> list[str]:
    errors: list[str] = []
    duplicates = ROOT_V8_LINE.findall(tree)
    if duplicates:
        errors.append("fresh consumer has duplicate V8 versions: " + ", ".join(duplicates))
    inverse_versions = ROOT_V8_LINE.findall(inverse)
    if inverse_versions != [EXPECTED_V8_VERSION]:
        errors.append(
            "fresh consumer must resolve exactly v8 "
            f"{EXPECTED_V8_VERSION}; got {inverse_versions!r}."
        )
    for package in ("katana-document-viewer", "katana-render-runtime"):
        if package not in inverse:
            errors.append(f"fresh consumer V8 graph must include {package}.")
    return errors


def validate(version: str) -> list[str]:
    if VERSION_RE.fullmatch(version) is None:
        return [f"invalid published KDV version: {version}"]
    with tempfile.TemporaryDirectory(prefix="kdv-v8-registry-consumer-") as directory:
        temporary = Path(directory)
        consumer = prepare_consumer(temporary, version)
        manifest_path = consumer / "Cargo.toml"
        errors = manifest_errors(manifest_path, version)
        if errors:
            return errors
        environment = {"CARGO_TARGET_DIR": str(temporary / "target")}
        run(["cargo", "generate-lockfile", "--manifest-path", str(manifest_path)], cwd=consumer, environment=environment)
        metadata = json.loads(
            run(
                [
                    "cargo",
                    "metadata",
                    "--manifest-path",
                    str(manifest_path),
                    "--locked",
                    "--format-version=1",
                ],
                cwd=consumer,
                environment=environment,
            )
        )
        errors.extend(package_errors(metadata, version))
        if errors:
            return errors
        run(
            ["cargo", "build", "--manifest-path", str(manifest_path), "--locked"],
            cwd=consumer,
            environment=environment,
        )
        duplicates = run(
            [
                "cargo",
                "tree",
                "--manifest-path",
                str(manifest_path),
                "--locked",
                "--edges",
                "normal",
                "-d",
            ],
            cwd=consumer,
            environment=environment,
        )
        inverse = run(
            [
                "cargo",
                "tree",
                "--manifest-path",
                str(manifest_path),
                "--locked",
                "--edges",
                "normal",
                "-i",
                "v8",
            ],
            cwd=consumer,
            environment=environment,
        )
        return tree_errors(duplicates, inverse)


def self_test() -> None:
    with tempfile.TemporaryDirectory(prefix="kdv-v8-registry-consumer-self-test-") as directory:
        manifest_path = prepare_consumer(Path(directory), "0.5.6") / "Cargo.toml"
        assert not manifest_errors(manifest_path, "0.5.6")
    metadata = registry_metadata()
    assert package_errors(metadata, "0.5.6") == []
    metadata["packages"][2]["source"] = "git+https://example.invalid/v8"
    assert package_errors(metadata, "0.5.6") == [
        "fresh consumer resolved a non-registry dependency: 'v8' from "
        "'git+https://example.invalid/v8'.",
        "fresh consumer v8 152.2.0 must resolve from crates.io.",
    ]
    missing_viewer = registry_metadata()
    missing_viewer["packages"] = [
        package
        for package in missing_viewer["packages"]
        if package["name"] != "katana-document-viewer"
    ]
    assert "fresh consumer did not resolve exactly one published KDV package." in package_errors(
        missing_viewer, "0.5.6"
    )
    path_krr = registry_metadata()
    path_krr["packages"][3]["source"] = None
    assert "fresh consumer resolved a non-registry dependency: 'katana-render-runtime' from None." in package_errors(
        path_krr, "0.5.6"
    )
    assert tree_errors(
        "katana-document-viewer v0.5.6\n└── v8 v152.2.0\n",
        "v8 v152.2.0\n├── katana-document-viewer v0.5.6\n└── katana-render-runtime v0.4.19\n",
    ) == []
    assert tree_errors(
        "v8 v150.0.0\n└── package\n",
        "v8 v150.0.0\n└── katana-document-viewer v0.5.6\n",
    )


def registry_metadata() -> dict[str, object]:
    root_id = "kdv-v8-registry-consumer 0.1.0 (path+file:///consumer)"
    return {
        "packages": [
            {
                "id": root_id,
                "name": "kdv-v8-registry-consumer",
                "version": "0.1.0",
                "source": None,
            },
            {
                "id": "katana-document-viewer 0.5.6 (registry)",
                "name": "katana-document-viewer",
                "version": "0.5.6",
                "source": REGISTRY_SOURCE,
            },
            {
                "id": "v8 152.2.0 (registry)",
                "name": "v8",
                "version": "152.2.0",
                "source": REGISTRY_SOURCE,
            },
            {
                "id": "katana-render-runtime 0.4.19 (registry)",
                "name": "katana-render-runtime",
                "version": "0.4.19",
                "source": REGISTRY_SOURCE,
            },
        ],
        "resolve": {"root": root_id},
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("version", nargs="?")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        print("registry consumer link self-test passed")
        return 0
    if args.version is None:
        parser.error("version is required unless --self-test is used")
    try:
        errors = validate(args.version)
    except RuntimeError as error:
        print(f"registry consumer link failed: {error}")
        return 1
    if errors:
        for error in errors:
            print(f"registry consumer link failed: {error}")
        return 1
    print(f"registry consumer link passed: katana-document-viewer {args.version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
