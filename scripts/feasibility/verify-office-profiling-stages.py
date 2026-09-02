#!/usr/bin/env python3
"""Verify the DEBUG-only Office profiling stage contract."""

from __future__ import annotations

import argparse
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
REQUIRED_STAGES = {
    "crates/katana-document-viewer/src/multi_format/office_worker_parent.rs": (
        "office.archive_intake",
        "office.package_parse",
        "office.transfer_to_worker",
        "office.conversion",
        "office.transfer_from_worker",
    ),
    "crates/katana-document-viewer/src/multi_format/office_worker_process.rs": (
        "office.worker_spawn",
    ),
    "crates/katana-document-viewer/src/multi_format/office_worker_process_windows.rs": (
        "office.worker_spawn",
    ),
    "crates/katana-document-viewer/src/multi_format/office_worker_runtime.rs": (
        "office.runtime_init",
    ),
    "crates/katana-document-viewer/src/multi_format/office_worker_entrypoint.rs": (
        "office.parse_layout",
    ),
    "crates/katana-document-viewer/src/multi_format/office_static_adapter.rs": (
        "office.raster",
    ),
    "crates/katana-document-viewer/src/multi_format/document_session_paged.rs": (
        "office.frame_publication",
    ),
    "crates/katana-document-viewer/src/multi_format/spreadsheet_worker_spawn.rs": (
        "spreadsheet.worker_spawn",
    ),
    "crates/katana-document-viewer/src/multi_format/spreadsheet_worker_entrypoint.rs": (
        "spreadsheet.runtime_init",
        "spreadsheet.package_parse",
    ),
    "crates/katana-document-viewer/src/multi_format/document_session_spreadsheet.rs": (
        "spreadsheet.frame_publication",
    ),
}


def stage_errors(root: Path) -> list[str]:
    errors: list[str] = []
    for relative, stages in REQUIRED_STAGES.items():
        path = root / relative
        if not path.is_file():
            errors.append(f"profiling stage source is missing: {relative}")
            continue
        source = path.read_text(encoding="utf-8")
        for stage in stages:
            if stage not in source:
                errors.append(f"profiling stage is missing: {stage} ({relative})")
    return errors


def self_test() -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        for relative, stages in REQUIRED_STAGES.items():
            path = root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("\n".join(stages), encoding="utf-8")
        assert stage_errors(root) == []
        missing_path = root / next(iter(REQUIRED_STAGES))
        missing_path.write_text("", encoding="utf-8")
        assert stage_errors(root)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        print("office profiling stage self-test passed")
        return 0
    errors = stage_errors(ROOT)
    if errors:
        for error in errors:
            print(f"office profiling stage check failed: {error}")
        return 1
    print("office profiling stage check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
