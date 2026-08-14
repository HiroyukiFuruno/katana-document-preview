#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path


def dependency_closure(
    root_id: str,
    nodes: dict[str, dict[str, object]],
) -> set[str]:
    pending = [root_id]
    visited: set[str] = set()
    while pending:
        package_id = pending.pop()
        if package_id in visited:
            continue
        visited.add(package_id)
        node = nodes.get(package_id)
        if node is not None:
            pending.extend(str(dependency["pkg"]) for dependency in node["deps"])
    return visited


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("manifest", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("roots", nargs="+")
    args = parser.parse_args()

    result = subprocess.run(
        [
            "rtk",
            "cargo",
            "metadata",
            "--format-version",
            "1",
            "--manifest-path",
            str(args.manifest),
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    metadata = json.loads(result.stdout)
    packages = {package["id"]: package for package in metadata["packages"]}
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}

    audits: dict[str, object] = {}
    for root_name in args.roots:
        matching = [
            package for package in packages.values() if package["name"] == root_name
        ]
        if len(matching) != 1:
            raise SystemExit(
                f"expected one package named {root_name}, found {len(matching)}"
            )
        root = matching[0]
        closure = dependency_closure(root["id"], nodes)
        records = []
        for package_id in sorted(
            closure,
            key=lambda item: (
                packages[item]["name"],
                packages[item]["version"],
                item,
            ),
        ):
            package = packages[package_id]
            records.append(
                {
                    "name": package["name"],
                    "version": package["version"],
                    "license": package["license"],
                    "license_file": package["license_file"],
                    "source": package["source"],
                    "links": package["links"],
                    "rust_version": package["rust_version"],
                    "build_script": any(
                        target["kind"] == ["custom-build"]
                        for target in package["targets"]
                    ),
                }
            )
        audits[root_name] = {
            "package_count": len(records),
            "missing_license_count": sum(
                record["license"] is None and record["license_file"] is None
                for record in records
            ),
            "native_links": [
                {"name": record["name"], "links": record["links"]}
                for record in records
                if record["links"] is not None
            ],
            "build_scripts": [
                record["name"] for record in records if record["build_script"]
            ],
            "packages": records,
        }

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(audits, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
