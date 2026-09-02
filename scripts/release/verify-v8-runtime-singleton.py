#!/usr/bin/env python3
"""Verify that KDV and KRR resolve and link exactly one V8 runtime."""

from __future__ import annotations

import argparse
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
EXPECTED_V8_VERSION = "152.2.0"
ROOT_V8_LINE = re.compile(r"^v8 v(?P<version>[^\s]+)", re.MULTILINE)


def run(command: list[str]) -> str:
    completed = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    if completed.returncode != 0:
        joined = " ".join(command)
        raise RuntimeError(f"{joined} failed:\n{completed.stdout}")
    return completed.stdout


def duplicate_v8_versions(tree: str) -> list[str]:
    """Read only duplicate-root sections; nested tree lines are not duplicates."""
    return ROOT_V8_LINE.findall(tree)


def inverse_v8_versions(tree: str) -> list[str]:
    return ROOT_V8_LINE.findall(tree)


def validation_errors() -> list[str]:
    duplicates = run(
        [
            "cargo",
            "tree",
            "-p",
            "katana-document-viewer",
            "--locked",
            "--edges",
            "normal",
            "-d",
        ]
    )
    duplicate_versions = duplicate_v8_versions(duplicates)
    errors: list[str] = []
    if duplicate_versions:
        errors.append(
            "cargo tree -d found duplicate v8 versions: "
            + ", ".join(duplicate_versions)
        )

    inverse = run(
        [
            "cargo",
            "tree",
            "-p",
            "katana-document-viewer",
            "--locked",
            "--edges",
            "normal",
            "-i",
            "v8",
        ]
    )
    versions = inverse_v8_versions(inverse)
    if versions != [EXPECTED_V8_VERSION]:
        errors.append(
            "KDV must resolve exactly v8 "
            f"{EXPECTED_V8_VERSION}; cargo tree -i v8 reported {versions!r}."
        )
    for package in ("katana-document-viewer", "katana-render-runtime"):
        if package not in inverse:
            errors.append(f"cargo tree -i v8 must include {package}.")
    if errors:
        return errors

    run(
        [
            "cargo",
            "test",
            "-p",
            "katana-document-viewer",
            "--test",
            "v8_runtime_link_contract",
            "--locked",
            "--",
            "--test-threads=1",
        ]
    )
    return []


def self_test() -> None:
    assert duplicate_v8_versions("katana-document-viewer v0.5.6\n└── v8 v152.2.0\n") == []
    assert duplicate_v8_versions("v8 v150.0.0\n└── package\n") == ["150.0.0"]
    assert inverse_v8_versions("v8 v152.2.0\n├── katana-document-viewer v0.5.6\n") == [
        EXPECTED_V8_VERSION
    ]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        print("V8 runtime singleton self-test passed")
        return 0
    try:
        errors = validation_errors()
    except RuntimeError as error:
        print(f"V8 runtime singleton check failed: {error}")
        return 1
    if errors:
        for error in errors:
            print(f"V8 runtime singleton check failed: {error}")
        return 1
    print(f"V8 runtime singleton check passed: v8 {EXPECTED_V8_VERSION}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
