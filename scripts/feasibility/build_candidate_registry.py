#!/usr/bin/env python3
"""Build a loopback sparse Cargo registry from one packaged crate.

This preflight diagnostic writes registry files only; it never starts a
server, publishes a crate, or changes Cargo's crates.io resolution.
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib
from pathlib import Path
from typing import Any, Callable


SCRIPT_DIR = Path(__file__).resolve().parent
PACKAGE_NAME_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_-]*$")
VERSION_RE = re.compile(r"^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$")
VERSION_REQUIREMENT_PART_RE = re.compile(
    r"\s*(?:\*|(?:\^|~|>=|<=|>|<|=)?\s*(?:0|[1-9][0-9]*)(?:\.(?:0|[1-9][0-9]*|\*))?(?:\.(?:0|[1-9][0-9]*|\*))?(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?)\s*"
)
LOOPBACK_URL_RE = re.compile(r"^http://127\.0\.0\.1:(\d+)$")
CRATES_IO_INDEX_URL = "https://github.com/rust-lang/crates.io-index"
MAX_CRATE_BYTES = 128 * 1024 * 1024
MAX_ARCHIVE_MEMBERS = 50_000
MAX_ARCHIVE_UNPACKED_BYTES = 512 * 1024 * 1024
MAX_MANIFEST_BYTES = 1024 * 1024


class RegistryError(ValueError):
    """An input cannot safely produce a Cargo registry artifact."""


def validate_name(value: str) -> str:
    if not isinstance(value, str) or not PACKAGE_NAME_RE.fullmatch(value) or value.lower() != value:
        raise RegistryError(f"unsafe package name: {value!r}")
    return value


def validate_version(value: str) -> str:
    if not isinstance(value, str) or not VERSION_RE.fullmatch(value):
        raise RegistryError(f"unsupported package version: {value!r}")
    return value


def validate_requirement(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value:
        raise RegistryError(f"{label} must be a non-empty Cargo version requirement")
    parts = value.split(",")
    if not all(VERSION_REQUIREMENT_PART_RE.fullmatch(part) for part in parts):
        raise RegistryError(f"{label} has an invalid Cargo version requirement: {value!r}")
    return value


def validate_boolean(value: Any, label: str, default: bool) -> bool:
    if value is None:
        return default
    if not isinstance(value, bool):
        raise RegistryError(f"{label} must be a boolean")
    return value


def validate_feature_list(value: Any, label: str) -> list[str]:
    if not isinstance(value, list) or not all(isinstance(item, str) and item for item in value):
        raise RegistryError(f"{label} must be a string list")
    return sorted(value)


def package_features(manifest: dict[str, Any]) -> dict[str, list[str]]:
    features = manifest.get("features", {})
    if not isinstance(features, dict):
        raise RegistryError("package features must be a TOML table")
    normalized: dict[str, list[str]] = {}
    for name, members in features.items():
        if not isinstance(name, str) or not name:
            raise RegistryError("package feature names must be non-empty strings")
        normalized[name] = validate_feature_list(members, f"package feature {name!r}")
    return normalized


def validate_loopback_url(value: str, port: int) -> str:
    match = LOOPBACK_URL_RE.fullmatch(value)
    if not 1 <= port <= 65535 or match is None or int(match.group(1)) != port:
        raise RegistryError(
            "base URL must be exactly http://127.0.0.1:<port> with the selected port"
        )
    return value


def index_path(name: str) -> Path:
    if len(name) == 1:
        return Path("1") / name
    if len(name) == 2:
        return Path("2") / name
    if len(name) == 3:
        return Path("3") / name[0] / name
    return Path(name[:2]) / name[2:4] / name


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def resolve_crate_path(path: Path) -> Path:
    if path.is_symlink():
        raise RegistryError(f"crate path must not be a symlink: {path}")
    resolved = path.resolve()
    if not resolved.is_file():
        raise RegistryError(f"crate path must be a regular file: {path}")
    return resolved


def validate_archive_member(member: tarfile.TarInfo) -> None:
    parts = Path(member.name).parts
    if not member.name or member.name.startswith("/") or any(part in ("", ".", "..") for part in parts):
        raise RegistryError(f"unsafe archive member path: {member.name!r}")
    if not (member.isfile() or member.isdir()):
        raise RegistryError(f"unsupported archive member type: {member.name!r}")


def read_manifest(crate: Path, requested_name: str | None, requested_version: str | None) -> tuple[str, str, dict[str, Any]]:
    if not crate.is_file() or crate.is_symlink():
        raise RegistryError(f"crate path must be a regular file: {crate}")
    if crate.stat().st_size > MAX_CRATE_BYTES:
        raise RegistryError(f"crate exceeds {MAX_CRATE_BYTES} byte input limit")
    try:
        with tarfile.open(crate, mode="r:gz") as archive:
            manifest_bytes: bytes | None = None
            member_count = 0
            unpacked_bytes = 0
            for member in archive:
                member_count += 1
                if member_count > MAX_ARCHIVE_MEMBERS:
                    raise RegistryError(f"crate exceeds {MAX_ARCHIVE_MEMBERS} archive member limit")
                validate_archive_member(member)
                if member.isfile():
                    unpacked_bytes += member.size
                    if unpacked_bytes > MAX_ARCHIVE_UNPACKED_BYTES:
                        raise RegistryError(
                            f"crate exceeds {MAX_ARCHIVE_UNPACKED_BYTES} byte unpacked limit"
                        )
                if member.name.count("/") == 1 and member.name.endswith("/Cargo.toml"):
                    if manifest_bytes is not None:
                        raise RegistryError("crate must contain exactly one normalized top-level Cargo.toml")
                    if member.size > MAX_MANIFEST_BYTES:
                        raise RegistryError(f"Cargo.toml exceeds {MAX_MANIFEST_BYTES} byte limit")
                    manifest_file = archive.extractfile(member)
                    if manifest_file is None:
                        raise RegistryError("could not read Cargo.toml from crate")
                    manifest_bytes = manifest_file.read(MAX_MANIFEST_BYTES + 1)
                    if len(manifest_bytes) > MAX_MANIFEST_BYTES:
                        raise RegistryError(f"Cargo.toml exceeds {MAX_MANIFEST_BYTES} byte limit")
            if manifest_bytes is None:
                raise RegistryError("crate must contain exactly one normalized top-level Cargo.toml")
            manifest = tomllib.loads(manifest_bytes.decode("utf-8"))
    except (UnicodeDecodeError, tarfile.TarError, tomllib.TOMLDecodeError) as error:
        raise RegistryError(f"invalid crate archive or normalized Cargo.toml: {error}") from error

    package = manifest.get("package")
    if not isinstance(package, dict) or not isinstance(package.get("name"), str) or not isinstance(package.get("version"), str):
        raise RegistryError("normalized Cargo.toml lacks package.name/package.version")
    package_name = validate_name(package["name"])
    package_version = validate_version(package["version"])
    name = validate_name(requested_name) if requested_name is not None else package_name
    version = validate_version(requested_version) if requested_version is not None else package_version
    if name != package_name or version != package_version:
        raise RegistryError("requested package identity does not match normalized Cargo.toml")
    return name, version, manifest


def dependency_entries(manifest: dict[str, Any]) -> list[dict[str, Any]]:
    entries: list[dict[str, Any]] = []

    def add_table(table: Any, kind: str, target: str | None = None) -> None:
        if table is None:
            return
        if not isinstance(table, dict):
            raise RegistryError(f"{kind} dependency table must be a TOML table")
        for name, specification in table.items():
            dependency_name = validate_name(name)
            if isinstance(specification, str):
                options: dict[str, Any] = {}
                requirement = specification
            elif isinstance(specification, dict):
                options = specification
                requirement = options.get("version")
            else:
                raise RegistryError(f"dependency {dependency_name!r} must be a string or TOML table")
            requirement = validate_requirement(requirement, f"dependency {dependency_name!r}")
            if any(key in options for key in ("path", "git", "registry", "registry-index")):
                raise RegistryError(f"dependency {dependency_name!r} uses a non-crates.io source")
            features = validate_feature_list(options.get("features", []), f"dependency {dependency_name!r} features")
            entry = {
                "name": dependency_name,
                "req": requirement,
                "features": features,
                "optional": validate_boolean(options.get("optional"), f"dependency {dependency_name!r} optional", False),
                "default_features": validate_boolean(
                    options.get("default-features"),
                    f"dependency {dependency_name!r} default-features",
                    True,
                ),
                "target": target,
                "kind": kind,
                "registry": CRATES_IO_INDEX_URL,
            }
            package_name = options.get("package")
            if package_name is not None:
                package_name = validate_name(package_name)
                entry["package"] = package_name
            entries.append(entry)

    for table_name, kind in (("dependencies", "normal"), ("dev-dependencies", "dev"), ("build-dependencies", "build")):
        add_table(manifest.get(table_name), kind)
    targets = manifest.get("target", {})
    if not isinstance(targets, dict):
        raise RegistryError("target dependency table must be a TOML table")
    for target, target_table in targets.items():
        if not isinstance(target, str) or not isinstance(target_table, dict):
            raise RegistryError("target dependency entries must be TOML tables")
        for table_name, kind in (("dependencies", "normal"), ("dev-dependencies", "dev"), ("build-dependencies", "build")):
            add_table(target_table.get(table_name), kind, target)
    return entries


def build_registry(crate: Path, output: Path, name: str | None, version: str | None, base_url: str) -> dict[str, Any]:
    package_name, package_version, manifest = read_manifest(crate, name, version)
    base_match = LOOPBACK_URL_RE.fullmatch(base_url)
    if base_match is None:
        raise RegistryError("base URL must be exactly http://127.0.0.1:<port>")
    validate_loopback_url(base_url, int(base_match.group(1)))
    if not output.is_absolute():
        raise RegistryError("output directory must be an absolute path")
    if output.is_symlink():
        raise RegistryError("output directory must not be a symlink")
    if output.exists():
        raise RegistryError("output directory must not already exist")
    if not output.parent.is_dir():
        raise RegistryError("output parent directory must already exist")
    digest = file_sha256(crate)
    config = {
        "dl": f"{base_url.rstrip('/')}/crates/{{crate}}/{{version}}/download",
        "api": f"{base_url.rstrip('/')}/api/v1",
    }
    metadata = {
        "name": package_name,
        "vers": package_version,
        "deps": dependency_entries(manifest),
        "cksum": digest,
        "features": package_features(manifest),
        "yanked": False,
    }
    package = manifest["package"]
    if package.get("links") is not None:
        if not isinstance(package["links"], str):
            raise RegistryError("package links must be a string")
        metadata["links"] = package["links"]
    if package.get("rust-version") is not None:
        if not isinstance(package["rust-version"], str):
            raise RegistryError("package rust-version must be a string")
        metadata["rust_version"] = package["rust-version"]

    temporary = Path(tempfile.mkdtemp(prefix=".candidate-registry-", dir=output.parent))
    try:
        index_dir = temporary / "index"
        index_file = index_dir / index_path(package_name)
        index_file.parent.mkdir(parents=True, exist_ok=True)
        (index_dir / "config.json").write_text(json.dumps(config, sort_keys=True) + "\n", encoding="utf-8")
        index_file.write_text(json.dumps(metadata, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8")
        download = temporary / "crates" / package_name / package_version / "download"
        download.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(crate, download)
        evidence = {
            "package": {"name": package_name, "version": package_version, "sha256": digest},
            "paths": {"index": str(index_file.relative_to(temporary)), "download": str(download.relative_to(temporary))},
            "urls": config,
        }
        (temporary / "candidate-registry-evidence.json").write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        temporary.rename(output)
        return evidence
    finally:
        if temporary.exists():
            shutil.rmtree(temporary)


def self_test() -> None:
    def check(condition: bool, message: str) -> None:
        if not condition:
            raise RegistryError(f"self-test failed: {message}")

    def expect_registry_error(action: Callable[[], Any], message: str) -> None:
        try:
            action()
        except RegistryError:
            return
        raise RegistryError(f"self-test failed: {message}")

    def write_crate(path: Path, members: list[tuple[str, bytes]]) -> None:
        with tarfile.open(path, mode="w:gz") as archive:
            for member_name, content in members:
                info = tarfile.TarInfo(member_name)
                info.size = len(content)
                archive.addfile(info, io.BytesIO(content))

    with tempfile.TemporaryDirectory(prefix="candidate-registry-", dir=SCRIPT_DIR) as temporary:
        root = Path(temporary)
        crate = root / "tiny-package-0.1.0.crate"
        content = b'[package]\nname = "tiny-package"\nversion = "0.1.0"\n\n[dependencies]\nepaint_default_fonts = "0.1"\nserde = "1"\n'
        write_crate(crate, [("tiny-package-0.1.0/Cargo.toml", content)])
        output = root / "registry"
        evidence = build_registry(crate, output, None, None, "http://127.0.0.1:39001")
        expected_index = output / "index" / index_path("tiny-package")
        check(Path(evidence["paths"]["index"]) == Path("index") / index_path("tiny-package"), "index path")
        check(expected_index.is_file(), "index file")
        metadata = json.loads(expected_index.read_text(encoding="utf-8"))
        check(metadata["cksum"] == file_sha256(crate), "checksum")
        check({dependency["registry"] for dependency in metadata["deps"]} == {CRATES_IO_INDEX_URL}, "crates.io dependency registry")
        check({dependency["name"] for dependency in metadata["deps"]} == {"epaint_default_fonts", "serde"}, "dependency names")
        config = json.loads((output / "index" / "config.json").read_text(encoding="utf-8"))
        check(config["dl"] == "http://127.0.0.1:39001/crates/{crate}/{version}/download", "static download URL template")
        download = output / "crates" / "tiny-package" / "0.1.0" / "download"
        check(download.is_file(), "download layout")
        check(download.relative_to(output).as_posix() == config["dl"].format(crate="tiny-package", version="0.1.0").removeprefix("http://127.0.0.1:39001/"), "download URL/layout match")
        check(download.read_bytes() == crate.read_bytes(), "download contents")
        check(json.loads((output / "candidate-registry-evidence.json").read_text(encoding="utf-8"))["package"]["sha256"] == file_sha256(crate), "evidence")
        link = root / "output-link"
        link.symlink_to(root / "missing-output", target_is_directory=True)
        try:
            build_registry(crate, link, None, None, "http://127.0.0.1:39001")
        except RegistryError:
            pass
        else:
            raise RegistryError("self-test failed: symlink output was accepted")
        crate_link = root / "crate-link"
        crate_link.symlink_to(crate)
        try:
            resolve_crate_path(crate_link)
        except RegistryError:
            pass
        else:
            raise RegistryError("self-test failed: symlink crate was accepted")
        bad_gzip = root / "bad-gzip.crate"
        bad_gzip.write_bytes(b"not a gzip archive")
        expect_registry_error(
            lambda: read_manifest(bad_gzip, None, None),
            "malformed gzip escaped RegistryError",
        )
        failed_cli = subprocess.run(
            [sys.executable, str(Path(__file__).resolve()), str(bad_gzip), str(root / "bad-registry")],
            capture_output=True,
            check=False,
            text=True,
        )
        check(failed_cli.returncode == 2, "malformed gzip CLI exit status")
        check("UnboundLocalError" not in failed_cli.stderr, "malformed gzip leaked implementation error")
        missing_manifest = root / "missing-manifest.crate"
        write_crate(missing_manifest, [("tiny-package-0.1.0/README.md", b"missing manifest")])
        expect_registry_error(
            lambda: read_manifest(missing_manifest, None, None),
            "missing manifest escaped RegistryError",
        )
        invalid_toml = root / "invalid-toml.crate"
        write_crate(invalid_toml, [("tiny-package-0.1.0/Cargo.toml", b"[package\n")])
        expect_registry_error(
            lambda: read_manifest(invalid_toml, None, None),
            "invalid TOML escaped RegistryError",
        )
        unsafe_member = root / "unsafe-member.crate"
        write_crate(unsafe_member, [("../Cargo.toml", content)])
        expect_registry_error(
            lambda: read_manifest(unsafe_member, None, None),
            "unsafe member escaped RegistryError",
        )
        unsafe_link = root / "unsafe-link.crate"
        with tarfile.open(unsafe_link, mode="w:gz") as archive:
            manifest_info = tarfile.TarInfo("tiny-package-0.1.0/Cargo.toml")
            manifest_info.size = len(content)
            archive.addfile(manifest_info, io.BytesIO(content))
            link_info = tarfile.TarInfo("tiny-package-0.1.0/link")
            link_info.type = tarfile.SYMTYPE
            link_info.linkname = "Cargo.toml"
            archive.addfile(link_info)
        expect_registry_error(
            lambda: read_manifest(unsafe_link, None, None),
            "archive symlink escaped RegistryError",
        )
        oversized_manifest = root / "oversized-manifest.crate"
        write_crate(
            oversized_manifest,
            [("tiny-package-0.1.0/Cargo.toml", b"#" * (MAX_MANIFEST_BYTES + 1))],
        )
        expect_registry_error(
            lambda: read_manifest(oversized_manifest, None, None),
            "oversized manifest was accepted",
        )
        expect_registry_error(
            lambda: dependency_entries({"dependencies": {"serde": {"version": "not a requirement"}}}),
            "invalid dependency requirement was accepted",
        )
        expect_registry_error(
            lambda: dependency_entries({"dependencies": {"serde": {"version": "1", "optional": "yes"}}}),
            "string dependency boolean was accepted",
        )
        expect_registry_error(
            lambda: dependency_entries({"dependencies": {"serde": {"version": "1", "default-features": 0}}}),
            "numeric dependency boolean was accepted",
        )
        expect_registry_error(
            lambda: dependency_entries({"dependencies": {"invalid name": "1"}}),
            "invalid dependency name was accepted",
        )
        expect_registry_error(
            lambda: dependency_entries({"dependencies": {"alias": {"package": "Uppercase", "version": "1"}}}),
            "invalid dependency package alias was accepted",
        )
        expect_registry_error(
            lambda: package_features({"features": {"invalid": "not-a-list"}}),
            "invalid package feature members were accepted",
        )
        non_registry_crate = root / "non-registry-dependency.crate"
        write_crate(
            non_registry_crate,
            [
                (
                    "tiny-package-0.1.0/Cargo.toml",
                    b'[package]\nname = "tiny-package"\nversion = "0.1.0"\n\n[dependencies]\nlocal = { path = "../local", version = "1" }\n',
                )
            ],
        )
        failed_output = root / "failed-registry"
        expect_registry_error(
            lambda: build_registry(non_registry_crate, failed_output, None, None, "http://127.0.0.1:39001"),
            "non-registry dependency was accepted",
        )
        check(not failed_output.exists(), "failed registry left partial output")
        for invalid_url in (
            "https://127.0.0.1:39001",
            "http://localhost:39001",
            "http://[::1]:39001",
            "http://192.0.2.1:39001",
            "http://127.0.0.1:39001/path",
            "http://127.0.0.1:39001?query",
            "http://127.0.0.1:39001#fragment",
        ):
            try:
                validate_loopback_url(invalid_url, 39001)
            except RegistryError:
                continue
            raise RegistryError(f"self-test failed: accepted URL {invalid_url!r}")
        check(validate_loopback_url("http://127.0.0.1:39001", 39001) == "http://127.0.0.1:39001", "loopback URL")
    print("self-test: PASS")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("crate", nargs="?", type=Path, help="path to one cargo package archive")
    parser.add_argument("output", nargs="?", type=Path, help="absolute output directory")
    parser.add_argument("--name", help="package name; otherwise read normalized Cargo.toml")
    parser.add_argument("--version", help="package version; otherwise read normalized Cargo.toml")
    parser.add_argument("--host", help="must be 127.0.0.1 when supplied")
    parser.add_argument("--port", type=int, default=39001, help="loopback server port used in generated URLs")
    parser.add_argument("--base-url", help="must be http://127.0.0.1:<port> when supplied")
    parser.add_argument("--self-test", action="store_true", help="build and verify a synthetic crate")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        if args.self_test:
            self_test()
            return 0
        if args.crate is None or args.output is None:
            raise RegistryError("crate and output are required unless --self-test is used")
        if not 1 <= args.port <= 65535:
            raise RegistryError(f"invalid port: {args.port}")
        if args.host is not None and args.host != "127.0.0.1":
            raise RegistryError("host must be exactly 127.0.0.1")
        base_url = args.base_url or f"http://127.0.0.1:{args.port}"
        validate_loopback_url(base_url, args.port)
        if args.output.is_symlink():
            raise RegistryError("output directory must not be a symlink")
        crate = resolve_crate_path(args.crate)
        evidence = build_registry(crate, args.output.resolve(), args.name, args.version, base_url)
        print(json.dumps(evidence, indent=2, sort_keys=True))
        return 0
    except (OSError, RegistryError, tarfile.TarError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
