#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path

from PIL import Image, ImageChops


def compare_image(candidate_path: Path, reference_path: Path) -> dict[str, object]:
    with Image.open(candidate_path) as candidate_source:
        candidate = candidate_source.convert("RGB")
    with Image.open(reference_path) as reference_source:
        reference = reference_source.convert("RGB")

    if candidate.size != reference.size:
        return {
            "candidate": candidate_path.name,
            "reference": reference_path.name,
            "candidate_size": list(candidate.size),
            "reference_size": list(reference.size),
            "comparable": False,
        }

    difference = ImageChops.difference(candidate, reference)
    histogram = difference.histogram()
    channel_samples = candidate.width * candidate.height * 3
    absolute_sum = sum((index % 256) * count for index, count in enumerate(histogram))
    squared_sum = sum(
        ((index % 256) ** 2) * count for index, count in enumerate(histogram)
    )
    changed_channels = sum(
        count for index, count in enumerate(histogram) if index % 256 != 0
    )
    return {
        "candidate": candidate_path.name,
        "reference": reference_path.name,
        "size": list(candidate.size),
        "comparable": True,
        "normalized_mae": absolute_sum / channel_samples / 255,
        "normalized_rmse": math.sqrt(squared_sum / channel_samples) / 255,
        "changed_channel_ratio": changed_channels / channel_samples,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("candidate", type=Path)
    parser.add_argument("reference", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()

    candidate_paths = sorted(args.candidate.glob("*.png"))
    reference_paths = sorted(args.reference.glob("*.png"))
    if len(candidate_paths) != len(reference_paths):
        raise SystemExit(
            f"page count mismatch: candidate={len(candidate_paths)}, "
            f"reference={len(reference_paths)}"
        )

    pages = [
        compare_image(candidate, reference)
        for candidate, reference in zip(candidate_paths, reference_paths)
    ]
    comparable = [page for page in pages if page["comparable"]]
    summary = {
        "pages": pages,
        "page_count": len(pages),
        "comparable_page_count": len(comparable),
        "mean_normalized_mae": (
            sum(float(page["normalized_mae"]) for page in comparable) / len(comparable)
            if comparable
            else None
        ),
        "mean_normalized_rmse": (
            sum(float(page["normalized_rmse"]) for page in comparable) / len(comparable)
            if comparable
            else None
        ),
        "mean_changed_channel_ratio": (
            sum(float(page["changed_channel_ratio"]) for page in comparable) / len(comparable)
            if comparable
            else None
        ),
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
