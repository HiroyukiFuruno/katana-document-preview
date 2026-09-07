#!/usr/bin/env python3
"""Measure cold KDV Office first-frame runs without hiding missing evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import platform
import re
import statistics
import subprocess
import time
from pathlib import Path
from typing import Any


TRACE_PATTERN = re.compile(
    r"\[KDV_TRACE\]\s+stage=(?P<stage>[A-Za-z0-9_.-]+)"
    r"(?:\s+(?:session|source)=[^\s]+)*\s+elapsed_ms=(?P<elapsed>\d+)"
)
ACCEPTANCE_FIRST_FRAME_PATTERN = re.compile(
    r"\[KDV_ACCEPTANCE\]\s+stage=first_frame\s+elapsed_ms=(?P<elapsed>\d+)"
)
MACOS_RSS_PATTERN = re.compile(r"(?P<bytes>\d+)\s+maximum resident set size", re.IGNORECASE)
LINUX_RSS_PATTERN = re.compile(
    r"Maximum resident set size \(kbytes\):\s*(?P<kilobytes>\d+)", re.IGNORECASE
)
SHA256_PATTERN = re.compile(r"[0-9a-f]{64}\Z")
OFFICE_REQUIRED_STAGES = (
    "office.archive_intake",
    "office.package_parse",
    "office.worker_spawn",
    "office.runtime_init",
    "office.transfer_to_worker",
    "office.conversion",
    "office.parse_layout",
    "office.transfer_from_worker",
    "office.frame_publication",
    "office.raster",
)
ACCEPTANCE_TEST_NAME = "user_supplied_office_fixtures_open_through_the_unified_session"
BASELINE_TEST_NAME = "supplied_pptx_measurement_noop_baseline"


def percentile(samples: list[int], fraction: float) -> int:
    if not samples:
        raise ValueError("cannot calculate a percentile without samples")
    if not 0 < fraction <= 1:
        raise ValueError(f"percentile fraction must be in (0, 1], got {fraction}")
    ordered = sorted(samples)
    return ordered[math.ceil(len(ordered) * fraction) - 1]


def summarize(samples: list[int]) -> dict[str, int]:
    return {
        "minimum": min(samples),
        "p50": int(statistics.median(samples)),
        "p95": percentile(samples, 0.95),
        "maximum": max(samples),
    }


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def normalized_sha256(value: str) -> str:
    normalized = value.lower()
    if not SHA256_PATTERN.fullmatch(normalized):
        raise ValueError("expected SHA-256 must be 64 lowercase hexadecimal characters")
    return normalized


def parse_stage_elapsed(text: str) -> dict[str, int]:
    stages: dict[str, int] = {}
    for match in TRACE_PATTERN.finditer(text):
        stage = match.group("stage")
        stages[stage] = stages.get(stage, 0) + int(match.group("elapsed"))
    return stages


def parse_stage_occurrences(text: str) -> dict[str, int]:
    occurrences: dict[str, int] = {}
    for match in TRACE_PATTERN.finditer(text):
        stage = match.group("stage")
        occurrences[stage] = occurrences.get(stage, 0) + 1
    return occurrences


def parse_first_frame_elapsed_ms(text: str) -> list[int]:
    return [
        int(match.group("elapsed"))
        for match in ACCEPTANCE_FIRST_FRAME_PATTERN.finditer(text)
    ]


def parse_peak_rss_bytes(text: str) -> int | None:
    macos = [int(match.group("bytes")) for match in MACOS_RSS_PATTERN.finditer(text)]
    linux = [
        int(match.group("kilobytes")) * 1024
        for match in LINUX_RSS_PATTERN.finditer(text)
    ]
    values = macos + linux
    return max(values) if values else None


def time_prefix(time_binary: str) -> list[str]:
    if platform.system() == "Darwin":
        return [time_binary, "-l"]
    return [time_binary, "-v"]


def prepare_test_binary(cargo: str) -> tuple[Path, list[str]]:
    command = [
        cargo,
        "test",
        "-p",
        "katana-document-viewer",
        "--test",
        "multi_format_document_session_contract",
        "--locked",
        "--no-run",
        "--message-format=json",
    ]
    completed = subprocess.run(
        command,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if completed.returncode != 0:
        raise RuntimeError(
            "failed to prepare the Office acceptance test binary:\n"
            + completed.stderr[-4000:]
        )
    for line in completed.stdout.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        target = message.get("target")
        executable = message.get("executable")
        if (
            message.get("reason") == "compiler-artifact"
            and isinstance(target, dict)
            and target.get("name") == "multi_format_document_session_contract"
            and isinstance(executable, str)
        ):
            path = Path(executable)
            if path.is_file():
                return path, command
    raise RuntimeError("Cargo did not report the Office acceptance test executable")


def build_test_command(test_binary: Path, test_name: str) -> list[str]:
    return [
        str(test_binary),
        test_name,
        "--ignored",
        "--nocapture",
        "--test-threads=1",
    ]


def execute_measurement(
    fixture: Path | None,
    test_binary: Path,
    time_binary: str,
    test_name: str,
    required_stages: tuple[str, ...],
) -> dict[str, Any]:
    environment = os.environ.copy()
    environment["DEBUG"] = "true"
    if fixture is not None:
        environment.update(
            {
                "KDV_ACCEPTANCE_FIXTURE_DIR": str(fixture.parent),
                "KDV_ACCEPTANCE_FIXTURE_NAME": fixture.name,
            }
        )
    command = [*time_prefix(time_binary), *build_test_command(test_binary, test_name)]
    started = time.monotonic_ns()
    completed = subprocess.run(
        command,
        env=environment,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    elapsed_ms = (time.monotonic_ns() - started) // 1_000_000
    output = completed.stdout + completed.stderr
    if completed.returncode != 0:
        tail = output[-4000:]
        raise RuntimeError(
            "measurement command failed "
            f"with exit code {completed.returncode}:\n{tail}"
        )

    stage_elapsed_ms = parse_stage_elapsed(output)
    stage_occurrences = parse_stage_occurrences(output)
    missing_stages = [stage for stage in required_stages if stage not in stage_elapsed_ms]
    if missing_stages:
        raise RuntimeError(
            "KDV trace is incomplete; required stages missing: "
            + ", ".join(missing_stages)
        )
    if (
        test_name == ACCEPTANCE_TEST_NAME
        and stage_occurrences.get("office.conversion") != 1
    ):
        raise RuntimeError(
            "unchanged-source reuse failed: expected one office.conversion across "
            "the first, resized, and repeated frames"
        )
    first_frame_elapsed_ms: int | None = None
    if test_name == ACCEPTANCE_TEST_NAME:
        first_frame_samples = parse_first_frame_elapsed_ms(output)
        if len(first_frame_samples) != 1:
            raise RuntimeError(
                "first-frame trace is incomplete: expected one first_frame measurement"
            )
        first_frame_elapsed_ms = first_frame_samples[0]
    peak_rss_bytes = parse_peak_rss_bytes(output)
    if peak_rss_bytes is None:
        raise RuntimeError("peak RSS was not emitted by the selected time command")
    return {
        "elapsed_ms": elapsed_ms,
        "first_frame_elapsed_ms": first_frame_elapsed_ms,
        "peak_rss_bytes": peak_rss_bytes,
        "stage_elapsed_ms": stage_elapsed_ms,
        "stage_occurrences": stage_occurrences,
    }


def parser() -> argparse.ArgumentParser:
    argument_parser = argparse.ArgumentParser(
        description="Run separate cold KDV PPTX first-frame processes and summarize p50/p95/RSS."
    )
    argument_parser.add_argument("--fixture", type=Path)
    argument_parser.add_argument("--expected-sha256")
    argument_parser.add_argument("--iterations", type=int, default=5)
    argument_parser.add_argument("--baseline-iterations", type=int)
    argument_parser.add_argument("--output", type=Path)
    argument_parser.add_argument("--replace-output", action="store_true")
    argument_parser.add_argument("--cargo", default="cargo")
    argument_parser.add_argument("--time", dest="time_binary", default="/usr/bin/time")
    argument_parser.add_argument("--require-stage", action="append", default=[])
    argument_parser.add_argument("--self-test", action="store_true")
    return argument_parser


def self_test() -> None:
    assert percentile([10, 20, 30, 40, 50], 0.50) == 30
    assert percentile([10, 20, 30, 40, 50], 0.95) == 50
    assert summarize([10, 20, 30, 40, 50]) == {
        "minimum": 10,
        "p50": 30,
        "p95": 50,
        "maximum": 50,
    }
    assert normalized_sha256("A" * 64) == "a" * 64
    traces = parse_stage_elapsed(
        "[KDV_TRACE] stage=office.worker_spawn elapsed_ms=12\n"
        "[KDV_TRACE] stage=office.worker_spawn session=42 source=0123456789abcdef elapsed_ms=3\n"
        "[KDV_TRACE] stage=office.raster elapsed_ms=9\n"
    )
    assert traces == {"office.worker_spawn": 15, "office.raster": 9}
    assert parse_stage_occurrences(
        "[KDV_TRACE] stage=office.conversion source=0123456789abcdef session=42 elapsed_ms=12\n"
        "[KDV_TRACE] stage=office.conversion session=42 source=0123456789abcdef elapsed_ms=3\n"
    ) == {"office.conversion": 2}
    assert parse_first_frame_elapsed_ms(
        "[KDV_ACCEPTANCE] stage=first_frame elapsed_ms=123\n"
    ) == [123]
    assert parse_peak_rss_bytes("152862720 maximum resident set size") == 152862720
    assert parse_peak_rss_bytes("Maximum resident set size (kbytes): 2048") == 2_097_152
    assert build_test_command(Path("/tmp/office-test"), ACCEPTANCE_TEST_NAME)[0] == "/tmp/office-test"
    print("office first-frame measurement harness: self-test passed")


def main() -> None:
    arguments = parser().parse_args()
    if arguments.self_test:
        self_test()
        return
    if (
        arguments.fixture is None
        or arguments.expected_sha256 is None
        or arguments.output is None
    ):
        raise SystemExit(
            "--fixture, --expected-sha256, and --output are required unless --self-test is used"
        )
    if arguments.iterations < 2:
        raise SystemExit("--iterations must be at least 2 for a distribution")
    baseline_iterations = arguments.baseline_iterations or arguments.iterations
    if baseline_iterations < 2:
        raise SystemExit("--baseline-iterations must be at least 2 for a distribution")
    fixture = arguments.fixture.resolve()
    if not fixture.is_file() or fixture.suffix.lower() != ".pptx":
        raise SystemExit(f"--fixture must name an existing PPTX: {fixture}")
    try:
        expected_sha256 = normalized_sha256(arguments.expected_sha256)
    except ValueError as error:
        raise SystemExit(str(error)) from error
    fixture_sha256 = sha256(fixture)
    if fixture_sha256 != expected_sha256:
        raise SystemExit(
            "fixture SHA-256 does not match --expected-sha256: "
            f"expected {expected_sha256}, got {fixture_sha256}"
        )
    output = arguments.output.resolve()
    if output.exists() and not arguments.replace_output:
        raise SystemExit(f"refusing to overwrite existing evidence: {output}")
    required_stages = tuple(arguments.require_stage) or OFFICE_REQUIRED_STAGES
    test_binary, preparation_command = prepare_test_binary(arguments.cargo)

    baseline_measurements = [
        execute_measurement(
            None,
            test_binary,
            arguments.time_binary,
            BASELINE_TEST_NAME,
            (),
        )
        for _ in range(baseline_iterations)
    ]
    baseline_peak_rss_bytes = summarize(
        [measurement["peak_rss_bytes"] for measurement in baseline_measurements]
    )

    measurements = [
        execute_measurement(
            fixture,
            test_binary,
            arguments.time_binary,
            ACCEPTANCE_TEST_NAME,
            required_stages,
        )
        for _ in range(arguments.iterations)
    ]
    for measurement in measurements:
        measurement["peak_rss_delta_from_noop_bytes"] = (
            measurement["peak_rss_bytes"] - baseline_peak_rss_bytes["p50"]
        )
    stage_summary = {
        stage: summarize([measurement["stage_elapsed_ms"][stage] for measurement in measurements])
        for stage in required_stages
    }
    stage_occurrence_summary = {
        stage: summarize(
            [measurement["stage_occurrences"].get(stage, 0) for measurement in measurements]
        )
        for stage in required_stages
    }
    report = {
        "fixture": {
            "name": fixture.name,
            "sha256": fixture_sha256,
            "expected_sha256": expected_sha256,
            "bytes": fixture.stat().st_size,
        },
        "iterations": arguments.iterations,
        "cold_process": True,
        "preparation_command": preparation_command,
        "command": build_test_command(test_binary, ACCEPTANCE_TEST_NAME),
        "baseline": {
            "test_name": BASELINE_TEST_NAME,
            "iterations": baseline_iterations,
            "measurements": baseline_measurements,
            "summary": {
                "elapsed_ms": summarize(
                    [measurement["elapsed_ms"] for measurement in baseline_measurements]
                ),
                "peak_rss_bytes": baseline_peak_rss_bytes,
            },
        },
        "platform": platform.platform(),
        "measurements": measurements,
        "summary": {
            "elapsed_ms": summarize([measurement["elapsed_ms"] for measurement in measurements]),
            "first_frame_elapsed_ms": summarize(
                [measurement["first_frame_elapsed_ms"] for measurement in measurements]
            ),
            "peak_rss_bytes": summarize(
                [measurement["peak_rss_bytes"] for measurement in measurements]
            ),
            "peak_rss_delta_from_noop_bytes": summarize(
                [measurement["peak_rss_delta_from_noop_bytes"] for measurement in measurements]
            ),
            "stage_elapsed_ms": stage_summary,
            "stage_occurrences": stage_occurrence_summary,
        },
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"wrote cold first-frame evidence: {output}")


if __name__ == "__main__":
    main()
