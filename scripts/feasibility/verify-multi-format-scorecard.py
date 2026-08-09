#!/usr/bin/env python3
"""Validate the fixed multi-format feasibility score and release gate."""

from __future__ import annotations

import argparse
import copy
import json
import tempfile
from pathlib import Path
from typing import Any

EXPECTED_WEIGHTS = {
    "visual_fidelity": 30,
    "format_coverage": 20,
    "security_isolation": 20,
    "performance": 10,
    "distribution": 10,
    "license": 10,
}
REQUIRED_FORMATS = {"pdf", "docx", "xlsx", "pptx"}
RELEASE_DECISIONS = {"recommended", "approved"}


def require_mapping(value: Any, field: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ValueError(f"{field} must be an object")
    return value


def validate_score(candidate_name: str, candidate: dict[str, Any], weights: dict[str, int]) -> None:
    score = candidate.get("score")
    if score is None:
        return
    score = require_mapping(score, f"candidate {candidate_name} score")
    expected_keys = set(weights) | {"total"}
    if set(score) != expected_keys:
        raise ValueError(f"candidate {candidate_name} score keys do not match fixed weights")

    calculated_total = 0
    for dimension, maximum in weights.items():
        value = score.get(dimension)
        if not isinstance(value, int) or isinstance(value, bool):
            raise ValueError(f"candidate {candidate_name} {dimension} score must be an integer")
        if value < 0 or value > maximum:
            raise ValueError(
                f"candidate {candidate_name} {dimension} score {value} is outside 0..{maximum}"
            )
        calculated_total += value

    if score.get("total") != calculated_total:
        raise ValueError(
            f"candidate {candidate_name} total {score.get('total')} does not equal {calculated_total}"
        )


def validate_decision(
    candidate_name: str,
    candidate: dict[str, Any],
    minimum_score: int,
) -> None:
    hard_gate = candidate.get("hard_gate")
    decision = candidate.get("decision")
    if not isinstance(hard_gate, str) or not hard_gate:
        raise ValueError(f"candidate {candidate_name} hard_gate is required")
    if not isinstance(decision, str) or not decision:
        raise ValueError(f"candidate {candidate_name} decision is required")

    score = candidate.get("score")
    total = score.get("total") if isinstance(score, dict) else None
    if decision in RELEASE_DECISIONS:
        if hard_gate != "pass":
            raise ValueError(
                f"release candidate {candidate_name} must pass every hard gate"
            )
        if not isinstance(total, int) or total < minimum_score:
            raise ValueError(
                f"release candidate {candidate_name} must score at least {minimum_score}"
            )
    if decision.startswith("recommended_after_"):
        if not hard_gate.startswith("conditional_"):
            raise ValueError(
                f"conditional recommendation {candidate_name} must have a conditional hard gate"
            )
        if not isinstance(total, int) or total < minimum_score:
            raise ValueError(
                f"conditional recommendation {candidate_name} must score at least {minimum_score}"
            )
    if decision.startswith("rejected") and hard_gate == "pass":
        raise ValueError(f"rejected candidate {candidate_name} cannot have passing hard gates")


def validate_selection(
    document: dict[str, Any],
    candidates: dict[str, Any],
    minimum_score: int,
    require_approved: bool,
) -> None:
    selection = require_mapping(document.get("proposed_selection"), "proposed_selection")
    status = selection.get("approval_status")
    if status not in {"pending", "approved"}:
        raise ValueError("proposed_selection approval_status must be pending or approved")
    if require_approved and status != "approved":
        raise ValueError("multi-format profile selection has not been explicitly approved")
    profiles = require_mapping(selection.get("profiles"), "proposed_selection profiles")
    if set(profiles) != REQUIRED_FORMATS:
        raise ValueError("proposed_selection must define exactly pdf, docx, xlsx, and pptx")

    for format_name, raw_profile in profiles.items():
        profile = require_mapping(raw_profile, f"proposed_selection profile {format_name}")
        candidate_name = profile.get("candidate")
        profile_name = profile.get("profile")
        if not isinstance(candidate_name, str) or candidate_name not in candidates:
            raise ValueError(f"selected {format_name} candidate does not exist: {candidate_name}")
        if not isinstance(profile_name, str) or not profile_name:
            raise ValueError(f"selected {format_name} profile name is required")

        if require_approved:
            candidate = require_mapping(candidates[candidate_name], f"candidate {candidate_name}")
            score = require_mapping(candidate.get("score"), f"candidate {candidate_name} score")
            if score.get("total", -1) < minimum_score:
                raise ValueError(
                    f"selected {format_name} candidate {candidate_name} is below {minimum_score}"
                )
            if candidate.get("hard_gate") != "pass":
                raise ValueError(
                    f"selected {format_name} candidate {candidate_name} has not passed every hard gate"
                )
            if candidate.get("decision") not in RELEASE_DECISIONS:
                raise ValueError(
                    f"selected {format_name} candidate {candidate_name} is not release-approved"
                )

def validate(document: dict[str, Any], require_approved: bool = False) -> None:
    if document.get("schema_version") != 1:
        raise ValueError("unsupported multi-format scorecard schema")

    acceptance = require_mapping(document.get("acceptance"), "acceptance")
    if acceptance != {
        "minimum_score": 80,
        "all_hard_gates_required": True,
        "threshold_relaxation_allowed": False,
    }:
        raise ValueError("multi-format acceptance contract was relaxed or changed")

    weights = require_mapping(document.get("weights"), "weights")
    if weights != EXPECTED_WEIGHTS or sum(weights.values()) != 100:
        raise ValueError("multi-format score weights must match the fixed 100-point rubric")

    candidates = require_mapping(document.get("candidates"), "candidates")
    if not candidates:
        raise ValueError("at least one multi-format candidate is required")
    minimum_score = acceptance["minimum_score"]
    for candidate_name, raw_candidate in candidates.items():
        candidate = require_mapping(raw_candidate, f"candidate {candidate_name}")
        validate_score(candidate_name, candidate, weights)
        validate_decision(candidate_name, candidate, minimum_score)

    validate_selection(document, candidates, minimum_score, require_approved)


def passing_candidate() -> dict[str, Any]:
    score = dict(EXPECTED_WEIGHTS)
    score["total"] = 100
    return {
        "score": score,
        "hard_gate": "pass",
        "decision": "recommended",
    }


def sample_document() -> dict[str, Any]:
    candidates = {format_name: passing_candidate() for format_name in REQUIRED_FORMATS}
    return {
        "schema_version": 1,
        "acceptance": {
            "minimum_score": 80,
            "all_hard_gates_required": True,
            "threshold_relaxation_allowed": False,
        },
        "weights": dict(EXPECTED_WEIGHTS),
        "proposed_selection": {
            "approval_status": "approved",
            "profiles": {
                format_name: {"candidate": format_name, "profile": "test-profile"}
                for format_name in REQUIRED_FORMATS
            },
        },
        "candidates": candidates,
    }


def expect_invalid(document: dict[str, Any], expected_message: str, require_approved: bool = False) -> None:
    try:
        validate(document, require_approved=require_approved)
    except ValueError as error:
        if expected_message not in str(error):
            raise AssertionError(f"expected {expected_message!r}, got {error!r}") from error
        return
    raise AssertionError(f"expected invalid scorecard containing {expected_message!r}")


def self_test() -> None:
    valid = sample_document()
    validate(valid)
    validate(valid, require_approved=True)

    relaxed = copy.deepcopy(valid)
    relaxed["acceptance"]["minimum_score"] = 79
    expect_invalid(relaxed, "acceptance contract")

    bad_total = copy.deepcopy(valid)
    bad_total["candidates"]["pdf"]["score"]["total"] = 99
    expect_invalid(bad_total, "does not equal")

    failing_recommendation = copy.deepcopy(valid)
    failing_recommendation["candidates"]["pptx"]["hard_gate"] = "fail_layout"
    expect_invalid(failing_recommendation, "must pass every hard gate")

    failing_approval = copy.deepcopy(valid)
    failing_approval["candidates"]["xlsx"]["decision"] = "approved"
    failing_approval["candidates"]["xlsx"]["hard_gate"] = "fail_security"
    expect_invalid(failing_approval, "must pass every hard gate")

    pending = copy.deepcopy(valid)
    pending["proposed_selection"]["approval_status"] = "pending"
    expect_invalid(pending, "not been explicitly approved", require_approved=True)

    selected_failure = copy.deepcopy(valid)
    selected_failure["candidates"]["xlsx"]["hard_gate"] = "conditional_security"
    selected_failure["candidates"]["xlsx"]["decision"] = "recommended_after_security_adapter"
    expect_invalid(selected_failure, "has not passed every hard gate", require_approved=True)

    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "scorecard.json"
        path.write_text(json.dumps(valid), encoding="utf-8")
        validate(json.loads(path.read_text(encoding="utf-8")), require_approved=True)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--evidence",
        type=Path,
        default=Path(
            "openspec/changes/v0-5-0-multi-format-viewer/evidence/benchmark-summary.json"
        ),
    )
    parser.add_argument("--require-approved", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        self_test()
        print("multi-format scorecard self-test passed")
        return

    try:
        document = json.loads(args.evidence.read_text(encoding="utf-8"))
        validate(document, require_approved=args.require_approved)
    except (OSError, json.JSONDecodeError, ValueError) as error:
        parser.exit(1, f"multi-format scorecard failed: {error}\n")
    mode = "release" if args.require_approved else "evidence"
    print(f"multi-format scorecard {mode} gate passed: {args.evidence}")


if __name__ == "__main__":
    main()
