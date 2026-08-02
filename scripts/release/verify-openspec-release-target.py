#!/usr/bin/env python3
"""Validate that a KDV release line has one active OpenSpec change."""

from __future__ import annotations

import argparse
import json
import re
import tempfile
from pathlib import Path

VERSION_PATTERN = re.compile(r"^v(?P<major>0|[1-9][0-9]*)\.(?P<minor>0|[1-9][0-9]*)\.(?P<patch>0|[1-9][0-9]*)$")
SUPPORTED_RELEASE_CONTRACTS = {"browser-session-adapter", "multi-format-viewer"}


def parse_version(value: str) -> tuple[int, int, int]:
    match = VERSION_PATTERN.fullmatch(value)
    if match is None:
        raise ValueError(f"invalid release version: {value}")
    return tuple(int(match.group(key)) for key in ("major", "minor", "patch"))


def validate(root: Path, target_version: str) -> str:
    target = parse_version(target_version)
    release_targets_path = root / "openspec/release-targets.json"
    release_targets = json.loads(release_targets_path.read_text(encoding="utf-8"))
    if release_targets.get("schema_version") != "kdv.release-targets.v1":
        raise ValueError("unsupported OpenSpec release target schema")

    current = release_targets.get("current")
    if not isinstance(current, dict):
        raise ValueError("current release target is required")
    minor_line = current.get("minor_line")
    change = current.get("change")
    release_contract = current.get("release_contract")
    if (
        not isinstance(minor_line, str)
        or not isinstance(change, str)
        or not isinstance(release_contract, str)
    ):
        raise ValueError("current release target must define minor_line, change, and release_contract")
    if release_contract not in SUPPORTED_RELEASE_CONTRACTS:
        raise ValueError(f"unsupported release contract: {release_contract}")
    if minor_line != f"{target[0]}.{target[1]}":
        raise ValueError(f"{target_version} is outside the declared KDV release line {minor_line}.x")
    if not (root / "openspec/changes" / change).is_dir():
        raise ValueError(f"current OpenSpec change is missing: {change}")

    deferred = release_targets.get("deferred")
    if not isinstance(deferred, list) or not deferred:
        raise ValueError("deferred OpenSpec changes are required")
    for item in deferred:
        if not isinstance(item, dict):
            raise ValueError("deferred OpenSpec entry must be an object")
        deferred_change = item.get("change")
        planned_version = item.get("planned_version")
        if not isinstance(deferred_change, str) or not isinstance(planned_version, str):
            raise ValueError("deferred OpenSpec entry must define change and planned_version")
        if deferred_change == change:
            raise ValueError("current OpenSpec change cannot also be deferred")
        if parse_version(planned_version) <= target:
            raise ValueError(f"deferred OpenSpec change is not after {target_version}: {deferred_change}")
        if not (root / "openspec/changes" / deferred_change).is_dir():
            raise ValueError(f"deferred OpenSpec change is missing: {deferred_change}")
    return change


def self_test() -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        changes = root / "openspec/changes"
        for name in ("viewer", "future"):
            (changes / name).mkdir(parents=True, exist_ok=True)
        targets = root / "openspec/release-targets.json"
        targets.parent.mkdir(parents=True, exist_ok=True)
        targets.write_text(
            json.dumps(
                {
                    "schema_version": "kdv.release-targets.v1",
                    "current": {
                        "minor_line": "0.4",
                        "change": "viewer",
                        "release_contract": "multi-format-viewer",
                    },
                    "deferred": [{"change": "future", "planned_version": "v0.5.0"}],
                }
            ),
            encoding="utf-8",
        )
        assert validate(root, "v0.4.0") == "viewer"
        assert validate(root, "v0.4.9") == "viewer"
        for invalid in ("v0.3.9", "v0.5.0", "0.4.0"):
            try:
                validate(root, invalid)
            except ValueError:
                continue
            raise AssertionError(f"expected {invalid} to be rejected")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--target-version")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        print("OpenSpec release target self-test passed")
        return
    if args.target_version is None:
        parser.error("--target-version is required unless --self-test is used")
    root = Path(__file__).resolve().parents[2]
    print(f"OpenSpec release target passed: {validate(root, args.target_version)}")


if __name__ == "__main__":
    main()
