#!/usr/bin/env python3
"""Verify the DEBUG-only Office profiling stage contract."""

from __future__ import annotations

import argparse
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
REQUIRED_STAGES = {
    "crates/katana-document-viewer/src/multi_format/office_worker_parent.rs": (
        "office.transfer_to_worker",
        "office.conversion",
        "office.transfer_from_worker",
    ),
    "crates/katana-document-viewer/src/multi_format/office_worker_parent_preflight.rs": (
        "office.archive_intake",
        "office.package_parse",
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
    ),
    "crates/katana-document-viewer/src/multi_format/spreadsheet_worker_open.rs": (
        "spreadsheet.package_parse",
    ),
    "crates/katana-document-viewer/src/multi_format/document_session_spreadsheet.rs": (
        "spreadsheet.frame_publication",
    ),
}

WINDOWS_SPREADSHEET_SPAWN = "crates/katana-document-viewer/src/multi_format/spreadsheet_worker_spawn.rs"
WINDOWS_SPREADSHEET_PROCESS = "crates/katana-document-viewer/src/multi_format/spreadsheet_worker_spawn_windows.rs"
WINDOWS_SPREADSHEET_STDERR = (
    "crates/katana-document-viewer/src/multi_format/spreadsheet_worker_spawn_windows_stderr.rs"
)
WINDOWS_OFFICE_PROCESS = "crates/katana-document-viewer/src/multi_format/office_worker_process_windows.rs"


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
    errors.extend(windows_contract_errors(root))
    return errors


def windows_contract_errors(root: Path) -> list[str]:
    errors: list[str] = []
    spreadsheet = root / WINDOWS_SPREADSHEET_SPAWN
    if spreadsheet.is_file():
        source = spreadsheet.read_text(encoding="utf-8")
        windows_start = source.find("#[cfg(windows)]\n    pub(crate) fn spawn")
        non_windows_start = source.find("#[cfg(not(windows))]", windows_start)
        stage = 'DebugTrace::start("spreadsheet.worker_spawn")'
        stage_index = source.find(stage, windows_start)
        if windows_start == -1 or stage_index == -1 or (
            non_windows_start != -1 and stage_index >= non_windows_start
        ):
            errors.append("Windows spreadsheet spawn does not emit spreadsheet.worker_spawn")
    spreadsheet_process = root / WINDOWS_SPREADSHEET_PROCESS
    if spreadsheet_process.is_file():
        source = spreadsheet_process.read_text(encoding="utf-8")
        required = (
            "let debug_enabled = super::debug_trace::DebugTrace::enabled();",
            "env: Some(worker_environment_with_trace(workspace, debug_enabled)),",
            "stdio: StdioConfig::Pipe",
            "child.stderr.take()",
            "spawn_stderr_reader(stderr, debug_enabled)",
        )
        if any(marker not in source for marker in required):
            errors.append("Windows spreadsheet DEBUG trace does not preserve worker stderr spawn")
    else:
        errors.append("Windows spreadsheet stderr spawn source is missing")
    spreadsheet_stderr = root / WINDOWS_SPREADSHEET_STDERR
    if spreadsheet_stderr.is_file():
        source = spreadsheet_stderr.read_text(encoding="utf-8")
        required = (
            "forward_debug_stderr(&mut source)",
            "forward_stderr_chunks(source, |chunk| {",
            "std::io::stderr().lock()",
            "std::io::sink()",
        )
        if any(marker not in source for marker in required):
            errors.append("Windows spreadsheet DEBUG trace does not drain and forward worker stderr")
        long_lived_lock = (
            "let mut parent_stderr = std::io::stderr().lock();\n"
            "        forward_stderr(&mut source, &mut parent_stderr);"
        )
        if long_lived_lock in source:
            errors.append("Windows spreadsheet DEBUG stderr relay retains the parent lock until EOF")
    else:
        errors.append("Windows spreadsheet stderr relay source is missing")
    office = root / WINDOWS_OFFICE_PROCESS
    if office.is_file():
        source = office.read_text(encoding="utf-8")
        required = (
            "let debug_enabled = crate::multi_format::debug_trace::DebugTrace::enabled();",
            "stdio: worker_stdio_config(debug_enabled),",
            "rappct::StdioConfig::Inherit",
            "rappct::StdioConfig::Null",
        )
        if any(marker not in source for marker in required):
            errors.append("Windows Office DEBUG trace does not preserve worker stderr")
    return errors


def self_test() -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        for relative, stages in REQUIRED_STAGES.items():
            path = root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("\n".join(stages), encoding="utf-8")
        (root / WINDOWS_SPREADSHEET_SPAWN).write_text(
            "#[cfg(windows)]\n"
            "    pub(crate) fn spawn() { let _spawn = DebugTrace::start(\"spreadsheet.worker_spawn\"); }\n"
            "#[cfg(not(windows))]",
            encoding="utf-8",
        )
        (root / WINDOWS_SPREADSHEET_PROCESS).write_text(
            "let debug_enabled = super::debug_trace::DebugTrace::enabled();\n"
            "env: Some(worker_environment_with_trace(workspace, debug_enabled)),\n"
            "stdio: StdioConfig::Pipe\n"
            "child.stderr.take()\n"
            "spawn_stderr_reader(stderr, debug_enabled)",
            encoding="utf-8",
        )
        (root / WINDOWS_SPREADSHEET_STDERR).write_text(
            "forward_debug_stderr(&mut source)\n"
            "forward_stderr_chunks(source, |chunk| {\n"
            "std::io::stderr().lock()\n"
            "std::io::sink()",
            encoding="utf-8",
        )
        (root / WINDOWS_OFFICE_PROCESS).write_text(
            "office.worker_spawn\n"
            "let debug_enabled = crate::multi_format::debug_trace::DebugTrace::enabled();\n"
            "stdio: worker_stdio_config(debug_enabled),\n"
            "rappct::StdioConfig::Inherit\n"
            "rappct::StdioConfig::Null",
            encoding="utf-8",
        )
        assert stage_errors(root) == []
        (root / WINDOWS_SPREADSHEET_STDERR).write_text(
            "forward_debug_stderr(&mut source)\n"
            "forward_stderr_chunks(source, |chunk| {\n"
            "std::io::stderr().lock()\n"
            "std::io::sink()\n"
            "let mut parent_stderr = std::io::stderr().lock();\n"
            "        forward_stderr(&mut source, &mut parent_stderr);",
            encoding="utf-8",
        )
        assert any(
            "retains the parent lock until EOF" in error for error in stage_errors(root)
        )
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
