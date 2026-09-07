#!/usr/bin/env python3
"""Measure the pinned DOCX/XLSX fidelity reference without host-specific fixes.

LibreOffice is deliberately an external comparison oracle only.  KDV continues
to render DOCX through office2pdf-katana and XLSX through its worker/grid
surface; this script captures those public KDV surfaces and records their
observable difference from the pinned source artifact.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import re
import shutil
import subprocess
import tempfile
import zipfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any
from xml.etree import ElementTree


REPO_ROOT = Path(__file__).resolve().parents[2]
CHANGE_ROOT = REPO_ROOT / "openspec/changes/post-v0-5-5-document-fidelity-regressions"
DEFAULT_REFERENCE = CHANGE_ROOT / "evidence/fidelity-reference.json"
SPREADSHEET_NS = "http://schemas.openxmlformats.org/spreadsheetml/2006/main"
NS = {"main": SPREADSHEET_NS}


class FidelityError(RuntimeError):
    """A pinned reference, source artifact, or candidate surface was invalid."""


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def qname(name: str) -> str:
    return f"{{{SPREADSHEET_NS}}}{name}"


def require_tool(command: str) -> str:
    resolved = shutil.which(command)
    if resolved is None:
        raise FidelityError(f"required tool is unavailable: {command}")
    return resolved


def run(
    command: list[str],
    *,
    cwd: Path | None = None,
    env: dict[str, str] | None = None,
    accepted_codes: tuple[int, ...] = (0,),
) -> str:
    completed = subprocess.run(
        command,
        cwd=cwd,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    output = "\n".join(part for part in (completed.stdout, completed.stderr) if part).strip()
    if completed.returncode not in accepted_codes:
        rendered = " ".join(command)
        raise FidelityError(
            f"command failed ({completed.returncode}): {rendered}\n{output}"
        )
    return output


def image_dimensions(path: Path) -> tuple[int, int]:
    output = run([require_tool("identify"), "-format", "%w,%h", str(path)])
    match = re.fullmatch(r"(\d+),(\d+)", output.strip())
    if match is None:
        raise FidelityError(f"could not read raster dimensions for {path}: {output!r}")
    return int(match.group(1)), int(match.group(2))


def geometry_text(path: Path, *, fuzz: str | None = None) -> str:
    command = [require_tool("magick"), str(path)]
    if fuzz is not None:
        command.extend(["-fuzz", fuzz])
    command.extend(["-trim", "-format", "%@", "info:"])
    return run(command).strip()


def difference_bbox(candidate: Path, reference: Path) -> str:
    return run(
        [
            require_tool("magick"),
            str(candidate),
            str(reference),
            "-compose",
            "difference",
            "-composite",
            "-threshold",
            "5%",
            "-trim",
            "-format",
            "%@",
            "info:",
        ]
    ).strip()


def normalized_compare_metric(metric: str, candidate: Path, reference: Path) -> float:
    output = run(
        [require_tool("compare"), "-metric", metric, str(candidate), str(reference), "null:"],
        accepted_codes=(0, 1),
    )
    match = re.search(r"\((-?[0-9]+(?:\.[0-9]+)?)\)", output)
    if match is None:
        raise FidelityError(f"could not parse ImageMagick {metric} metric: {output!r}")
    return float(match.group(1))


def normalize_color(value: Any) -> str | None:
    if not isinstance(value, str):
        return None
    normalized = value.strip().lstrip("#").upper()
    if len(normalized) == 8:
        normalized = normalized[-6:]
    if len(normalized) != 6 or not re.fullmatch(r"[0-9A-F]{6}", normalized):
        return None
    return normalized


def cell_coordinate(reference: str) -> tuple[int, int]:
    match = re.fullmatch(r"([A-Za-z]+)([1-9][0-9]*)", reference)
    if match is None:
        raise FidelityError(f"unsupported XLSX cell reference: {reference!r}")
    letters, row_text = match.groups()
    column = 0
    for letter in letters.upper():
        column = column * 26 + ord(letter) - ord("A") + 1
    return int(row_text) - 1, column - 1


def cell_range(reference: str) -> tuple[int, int, int, int]:
    start, _, end = reference.partition(":")
    start_row, start_column = cell_coordinate(start)
    end_row, end_column = cell_coordinate(end or start)
    return start_row, start_column, end_row, end_column


def xml_text(element: ElementTree.Element | None) -> str:
    if element is None:
        return ""
    return "".join(part for part in element.itertext() if part)


def float_attribute(element: ElementTree.Element | None, attribute: str, default: float) -> float:
    if element is None:
        return default
    try:
        return float(element.attrib.get(attribute, default))
    except ValueError as error:
        raise FidelityError(f"invalid {attribute} in OOXML: {element.attrib.get(attribute)!r}") from error


def integer_attribute(element: ElementTree.Element, attribute: str, default: int = 0) -> int:
    try:
        return int(element.attrib.get(attribute, default))
    except ValueError as error:
        raise FidelityError(f"invalid {attribute} in OOXML: {element.attrib.get(attribute)!r}") from error


def shared_strings(archive: zipfile.ZipFile) -> list[str]:
    try:
        root = ElementTree.fromstring(archive.read("xl/sharedStrings.xml"))
    except KeyError:
        return []
    return [xml_text(item) for item in root.findall("main:si", NS)]


def font_descriptor(font: ElementTree.Element | None) -> dict[str, Any]:
    if font is None:
        return {"name": None, "size_px": None, "bold": False, "italic": False, "color": None}
    color = font.find(qname("color"))
    return {
        "name": font.find(qname("name")).attrib.get("val")
        if font.find(qname("name")) is not None
        else None,
        "size_px": float_attribute(font.find(qname("sz")), "val", math.nan),
        "bold": font.find(qname("b")) is not None,
        "italic": font.find(qname("i")) is not None,
        "color": normalize_color(color.attrib.get("rgb")) if color is not None else None,
    }


def fill_color(fill: ElementTree.Element | None) -> str | None:
    if fill is None:
        return None
    foreground = fill.find(f"{qname('patternFill')}/{qname('fgColor')}")
    return normalize_color(foreground.attrib.get("rgb")) if foreground is not None else None


def border_descriptor(border: ElementTree.Element | None) -> dict[str, dict[str, str | None]]:
    if border is None:
        return {}
    result = {}
    for name in ("left", "right", "top", "bottom"):
        side = border.find(qname(name))
        if side is None or not side.attrib.get("style"):
            continue
        color = side.find(qname("color"))
        result[name] = {
            "style": side.attrib["style"],
            "color": normalize_color(color.attrib.get("rgb")) if color is not None else None,
        }
    return result


def style_descriptors(
    root: ElementTree.Element,
) -> tuple[list[dict[str, Any]], list[str | None], list[dict[str, dict[str, str | None]]], list[dict[str, int]]]:
    fonts_root = root.find(qname("fonts"))
    fills_root = root.find(qname("fills"))
    borders_root = root.find(qname("borders"))
    cell_xfs_root = root.find(qname("cellXfs"))
    fonts = [font_descriptor(font) for font in (list(fonts_root) if fonts_root is not None else [])]
    fills = [fill_color(fill) for fill in (list(fills_root) if fills_root is not None else [])]
    borders = [
        border_descriptor(border) for border in (list(borders_root) if borders_root is not None else [])
    ]
    cell_xfs = []
    for xf in list(cell_xfs_root) if cell_xfs_root is not None else []:
        cell_xfs.append(
            {
                "font_id": integer_attribute(xf, "fontId"),
                "fill_id": integer_attribute(xf, "fillId"),
                "border_id": integer_attribute(xf, "borderId"),
            }
        )
    if not cell_xfs:
        cell_xfs.append({"font_id": 0, "fill_id": 0, "border_id": 0})
    return fonts, fills, borders, cell_xfs


def source_cell_text(cell: ElementTree.Element, strings: list[str]) -> str:
    cell_type = cell.attrib.get("t")
    if cell_type == "inlineStr":
        return xml_text(cell.find(qname("is")))
    value = cell.find(qname("v"))
    if value is None or value.text is None:
        return ""
    if cell_type == "s":
        index = int(value.text)
        try:
            return strings[index]
        except IndexError as error:
            raise FidelityError(f"shared string index is out of range: {index}") from error
    return value.text


def worksheet_paths(archive: zipfile.ZipFile) -> list[str]:
    paths = [
        name
        for name in archive.namelist()
        if re.fullmatch(r"xl/worksheets/sheet[0-9]+\.xml", name)
    ]
    return sorted(paths, key=lambda path: int(re.search(r"sheet([0-9]+)\.xml$", path).group(1)))


def source_xlsx(path: Path) -> dict[str, Any]:
    with zipfile.ZipFile(path) as archive:
        workbook = ElementTree.fromstring(archive.read("xl/workbook.xml"))
        sheet_names = [sheet.attrib.get("name", "") for sheet in workbook.findall("main:sheets/main:sheet", NS)]
        styles = ElementTree.fromstring(archive.read("xl/styles.xml"))
        fonts, fills, borders, cell_xfs = style_descriptors(styles)
        strings = shared_strings(archive)
        paths = worksheet_paths(archive)
        if len(paths) != len(sheet_names):
            raise FidelityError(
                f"workbook sheet mapping is ambiguous: names={len(sheet_names)}, xml={len(paths)}"
            )
        sheets = []
        for index, sheet_path in enumerate(paths):
            root = ElementTree.fromstring(archive.read(sheet_path))
            dimension = root.find(qname("dimension"))
            dimension_ref = dimension.attrib.get("ref", "A1") if dimension is not None else "A1"
            _, _, max_row, max_column = cell_range(dimension_ref)
            format_properties = root.find(qname("sheetFormatPr"))
            default_row_height = float_attribute(format_properties, "defaultRowHeight", 15.0)
            default_column_width = float_attribute(format_properties, "defaultColWidth", 8.0)
            if default_column_width == 8.0 and format_properties is not None:
                default_column_width = float_attribute(format_properties, "baseColWidth", 8.0)
            row_heights = [default_row_height] * (max_row + 1)
            column_widths = [default_column_width] * (max_column + 1)
            for column in root.findall(f"{qname('cols')}/{qname('col')}"):
                start = integer_attribute(column, "min", 1) - 1
                end = integer_attribute(column, "max", start + 1) - 1
                width = float_attribute(column, "width", default_column_width)
                for column_index in range(max(0, start), min(max_column, end) + 1):
                    column_widths[column_index] = width
            cells = []
            for row in root.findall(f"{qname('sheetData')}/{qname('row')}"):
                row_index = integer_attribute(row, "r", 1) - 1
                if 0 <= row_index <= max_row:
                    row_heights[row_index] = float_attribute(row, "ht", default_row_height)
                for cell in row.findall(qname("c")):
                    reference = cell.attrib.get("r")
                    if reference is None:
                        raise FidelityError("XLSX cell has no coordinate")
                    cell_row, cell_column = cell_coordinate(reference)
                    style_index = integer_attribute(cell, "s")
                    if style_index >= len(cell_xfs):
                        raise FidelityError(f"XLSX style index is out of range: {style_index}")
                    style = cell_xfs[style_index]
                    font_id = style["font_id"]
                    fill_id = style["fill_id"]
                    cells.append(
                        {
                            "row": cell_row,
                            "column": cell_column,
                            "text": source_cell_text(cell, strings),
                            "font_required": font_id != 0,
                            "fill_required": fill_id != 0,
                            "borders": borders[style["border_id"]]
                            if style["border_id"] < len(borders)
                            else {},
                            "font": fonts[font_id] if font_id < len(fonts) else font_descriptor(None),
                            "fill_color": fills[fill_id] if fill_id < len(fills) else None,
                        }
                    )
            merges = [
                cell_range(merge.attrib["ref"])
                for merge in root.findall(f"{qname('mergeCells')}/{qname('mergeCell')}")
            ]
            sheets.append(
                {
                    "index": index,
                    "name": sheet_names[index],
                    "row_count": max_row + 1,
                    "column_count": max_column + 1,
                    "row_heights": row_heights,
                    "column_widths": column_widths,
                    "cells": cells,
                    "merges": merges,
                }
            )
    return {"sheet_count": len(sheets), "sheets": sheets}


def source_font_difference(expected: dict[str, Any], candidate: dict[str, Any]) -> bool:
    family = expected.get("name")
    if isinstance(family, str) and family and candidate.get("family") != family:
        return True
    expected_size = expected.get("size_px")
    candidate_size = candidate.get("size_px")
    if isinstance(expected_size, float) and not math.isnan(expected_size):
        if not isinstance(candidate_size, (int, float)) or abs(float(candidate_size) - expected_size) > 0.01:
            return True
    for attribute in ("bold", "italic"):
        if bool(expected.get(attribute)) != bool(candidate.get(attribute)):
            return True
    expected_color = normalize_color(expected.get("color"))
    if expected_color is not None and normalize_color(candidate.get("text_color")) != expected_color:
        return True
    return False


def normalized_track_delta(source: list[float], candidate: list[float | int | None]) -> float | None:
    if len(source) != len(candidate) or not source or any(value is None for value in candidate):
        return None
    source_total = sum(source)
    candidate_total = sum(float(value) for value in candidate if value is not None)
    if source_total <= 0 or candidate_total <= 0:
        return None
    return sum(
        abs(source_value / source_total - float(candidate_value) / candidate_total)
        for source_value, candidate_value in zip(source, candidate)
    ) / len(source)


def candidate_tracks(sheet: dict[str, Any], axis: str) -> list[int | None]:
    count_key = "column_count" if axis == "column" else "row_count"
    span_key = "column_span" if axis == "column" else "row_span"
    size_key = "width" if axis == "column" else "height"
    values: list[int | None] = []
    for index in range(int(sheet[count_key])):
        candidates = [
            int(cell["bounds"][size_key])
            for cell in sheet["cells"]
            if int(cell[axis]) == index and int(cell[span_key]) == 1
        ]
        values.append(min(candidates) if candidates else None)
    return values


def candidate_cells(sheet: dict[str, Any]) -> dict[tuple[int, int], dict[str, Any]]:
    return {
        (int(cell["row"]), int(cell["column"])): cell
        for cell in sheet.get("cells", [])
    }


def candidate_merges(sheet: dict[str, Any]) -> set[tuple[int, int, int, int]]:
    merges = set()
    for cell in sheet.get("cells", []):
        row_span = int(cell["row_span"])
        column_span = int(cell["column_span"])
        if row_span > 1 or column_span > 1:
            row = int(cell["row"])
            column = int(cell["column"])
            merges.add((row, column, row + row_span - 1, column + column_span - 1))
    return merges


def border_metadata_difference(expected: dict[str, dict[str, str | None]], candidate: Any) -> tuple[bool, bool]:
    if not isinstance(candidate, dict):
        return True, False
    difference = False
    for side_name, expected_side in expected.items():
        candidate_side = candidate.get(side_name)
        if not isinstance(candidate_side, dict):
            return True, False
        if candidate_side.get("style") != expected_side["style"]:
            difference = True
        expected_color = normalize_color(expected_side.get("color"))
        if expected_color is not None and normalize_color(candidate_side.get("color")) != expected_color:
            difference = True
    return False, difference


def xlsx_sheet_measure(source: dict[str, Any], candidate: dict[str, Any] | None) -> dict[str, Any]:
    source_cells = source["cells"]
    source_text_cells = [cell for cell in source_cells if cell["text"]]
    source_font_cells = [cell for cell in source_cells if cell["font_required"]]
    source_fill_cells = [cell for cell in source_cells if cell["fill_required"]]
    source_border_cells = [cell for cell in source_cells if cell["borders"]]
    if candidate is None:
        return {
            "index": source["index"],
            "source_name": source["name"],
            "candidate_name": None,
            "row_count_delta": -source["row_count"],
            "column_count_delta": -source["column_count"],
            "text_missing_count": len(source_text_cells),
            "text_mismatch_count": 0,
            "font_missing_count": len(source_font_cells),
            "font_difference_count": 0,
            "fill_missing_count": len(source_fill_cells),
            "fill_difference_count": 0,
            "border_metadata_missing_count": len(source_border_cells),
            "border_difference_count": 0,
            "merged_cell_missing_count": len(source["merges"]),
            "row_track_mean_absolute_delta": None,
            "column_track_mean_absolute_delta": None,
        }

    cells = candidate_cells(candidate)
    text_missing = 0
    text_mismatch = 0
    for source_cell in source_text_cells:
        candidate_cell = cells.get((source_cell["row"], source_cell["column"]))
        candidate_text = candidate_cell.get("text") if candidate_cell is not None else None
        if not isinstance(candidate_text, str) or not candidate_text:
            text_missing += 1
        elif candidate_text != source_cell["text"]:
            text_mismatch += 1

    font_missing = 0
    font_difference = 0
    for source_cell in source_font_cells:
        candidate_cell = cells.get((source_cell["row"], source_cell["column"]))
        font = candidate_cell.get("font") if candidate_cell is not None else None
        if not isinstance(font, dict):
            font_missing += 1
        elif source_font_difference(source_cell["font"], font):
            font_difference += 1

    fill_missing = 0
    fill_difference = 0
    for source_cell in source_fill_cells:
        candidate_cell = cells.get((source_cell["row"], source_cell["column"]))
        font = candidate_cell.get("font") if candidate_cell is not None else None
        color = normalize_color(font.get("fill_color")) if isinstance(font, dict) else None
        if color is None:
            fill_missing += 1
        elif color != source_cell["fill_color"]:
            fill_difference += 1

    border_metadata_missing = 0
    border_difference = 0
    for source_cell in source_border_cells:
        candidate_cell = cells.get((source_cell["row"], source_cell["column"]))
        candidate_borders = candidate_cell.get("borders") if candidate_cell is not None else None
        missing, different = border_metadata_difference(source_cell["borders"], candidate_borders)
        border_metadata_missing += int(missing)
        border_difference += int(different)

    source_merges = set(tuple(value) for value in source["merges"])
    merged_missing = len(source_merges - candidate_merges(candidate))
    return {
        "index": source["index"],
        "source_name": source["name"],
        "candidate_name": candidate.get("label"),
        "row_count_delta": int(candidate["row_count"]) - source["row_count"],
        "column_count_delta": int(candidate["column_count"]) - source["column_count"],
        "text_missing_count": text_missing,
        "text_mismatch_count": text_mismatch,
        "font_missing_count": font_missing,
        "font_difference_count": font_difference,
        "fill_missing_count": fill_missing,
        "fill_difference_count": fill_difference,
        "border_metadata_missing_count": border_metadata_missing,
        "border_difference_count": border_difference,
        "merged_cell_missing_count": merged_missing,
        "row_track_mean_absolute_delta": normalized_track_delta(
            source["row_heights"], candidate_tracks(candidate, "row")
        ),
        "column_track_mean_absolute_delta": normalized_track_delta(
            source["column_widths"], candidate_tracks(candidate, "column")
        ),
    }


def xlsx_measure(
    source: dict[str, Any],
    candidate: dict[str, Any],
    reference_page_count: int,
    candidate_renders_borders: bool,
) -> dict[str, Any]:
    candidate_sheets = {int(sheet["index"]): sheet for sheet in candidate.get("sheets", [])}
    sheets = [
        xlsx_sheet_measure(source_sheet, candidate_sheets.get(source_sheet["index"]))
        for source_sheet in source["sheets"]
    ]
    aggregate_keys = (
        "text_missing_count",
        "text_mismatch_count",
        "font_missing_count",
        "font_difference_count",
        "fill_missing_count",
        "fill_difference_count",
        "border_metadata_missing_count",
        "border_difference_count",
        "merged_cell_missing_count",
    )
    missing_element_count = sum(
        sum(
            int(sheet[key])
            for key in (
                "text_missing_count",
                "font_missing_count",
                "fill_missing_count",
                "border_metadata_missing_count",
                "merged_cell_missing_count",
            )
        )
        for sheet in sheets
    )
    source_border_cell_count = sum(
        sum(bool(cell["borders"]) for cell in source_sheet["cells"])
        for source_sheet in source["sheets"]
    )
    candidate_api_exposes_borders = all(
        int(sheet["border_metadata_missing_count"]) == 0 for sheet in sheets
    )
    return {
        "source_sheet_count": source["sheet_count"],
        "candidate_sheet_count": int(candidate.get("sheet_count", 0)),
        "worksheet_count_delta": int(candidate.get("sheet_count", 0)) - source["sheet_count"],
        "source_reference_pdf_page_count": reference_page_count,
        "candidate_pagination_representation": "continuous_grid",
        "candidate_page_count": None,
        "pagination_missing_page_surface_count": reference_page_count,
        "candidate_api_exposes_borders": candidate_api_exposes_borders,
        "candidate_ui_renders_custom_cell_borders": candidate_renders_borders,
        "border_visual_missing_count": 0 if candidate_renders_borders else source_border_cell_count,
        "missing_element_count": missing_element_count,
        "sheets": sheets,
        "totals": {key: sum(int(sheet[key]) for sheet in sheets) for key in aggregate_keys},
    }


def convert_source_fixture(
    fixture: Path,
    *,
    workdir: Path,
    label: str,
    dpi: int,
) -> dict[str, Any]:
    output_dir = workdir / f"source-{label}"
    raster_dir = workdir / f"source-{label}-raster"
    output_dir.mkdir()
    raster_dir.mkdir()
    run(
        [
            require_tool("soffice"),
            "--headless",
            "--convert-to",
            "pdf",
            "--outdir",
            str(output_dir),
            str(fixture),
        ]
    )
    pdf = output_dir / f"{fixture.stem}.pdf"
    if not pdf.is_file():
        raise FidelityError(f"source renderer did not produce {pdf.name}")
    prefix = raster_dir / "page"
    run([require_tool("pdftoppm"), "-png", "-r", str(dpi), str(pdf), str(prefix)])
    pages = sorted(raster_dir.glob("page-*.png"))
    if not pages:
        raise FidelityError(f"rasterizer produced no pages for {fixture.name}")
    return {
        "pdf": pdf,
        "pages": pages,
        "pdf_sha256": sha256_file(pdf),
        "page_artifacts": [
            {
                "name": page.name,
                "sha256": sha256_file(page),
                "dimensions": list(image_dimensions(page)),
                "foreground_bbox": geometry_text(page, fuzz="5%"),
            }
            for page in pages
        ],
    }


def capture_candidate(workdir: Path) -> dict[str, Any]:
    output = workdir / "candidate"
    environment = os.environ.copy()
    environment["KDV_FIDELITY_OUTPUT_DIR"] = str(output)
    run(
        [
            require_tool("cargo"),
            "test",
            "-p",
            "katana-document-viewer",
            "--test",
            "office_fidelity_capture",
            "--locked",
            "capture_representative_office_fidelity_candidate",
            "--",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ],
        cwd=REPO_ROOT,
        env=environment,
    )
    candidate = output / "candidate.json"
    if not candidate.is_file():
        raise FidelityError("KDV candidate capture did not produce candidate.json")
    try:
        return json.loads(candidate.read_text(encoding="utf-8")) | {"_output": str(output)}
    except json.JSONDecodeError as error:
        raise FidelityError(f"KDV candidate capture is not JSON: {candidate}") from error


def docx_measure(candidate: dict[str, Any], source: dict[str, Any], workdir: Path) -> dict[str, Any]:
    candidate_document = candidate.get("docx")
    if not isinstance(candidate_document, dict):
        raise FidelityError("candidate capture has no DOCX section")
    candidate_pages = candidate_document.get("pages")
    if not isinstance(candidate_pages, list):
        raise FidelityError("candidate DOCX section has no page list")
    candidate_root = Path(str(candidate["_output"])) / "docx"
    comparisons = []
    for index, (candidate_page, reference_page) in enumerate(zip(candidate_pages, source["pages"])):
        if not isinstance(candidate_page, dict):
            raise FidelityError(f"candidate DOCX page {index} is not an object")
        image_name = candidate_page.get("image")
        if not isinstance(image_name, str):
            raise FidelityError(f"candidate DOCX page {index} has no raster name")
        candidate_image = candidate_root / image_name
        if not candidate_image.is_file():
            raise FidelityError(f"candidate DOCX raster is absent: {candidate_image}")
        candidate_size = image_dimensions(candidate_image)
        reference_size = image_dimensions(reference_page)
        normalized = workdir / f"candidate-docx-{index:04}.png"
        run(
            [
                require_tool("magick"),
                str(candidate_image),
                "-resize",
                f"{reference_size[0]}x{reference_size[1]}!",
                str(normalized),
            ]
        )
        comparisons.append(
            {
                "index": index,
                "candidate_sha256": sha256_file(candidate_image),
                "reference_sha256": sha256_file(reference_page),
                "candidate_native_dimensions": list(candidate_size),
                "reference_dimensions": list(reference_size),
                "width_delta": candidate_size[0] - reference_size[0],
                "height_delta": candidate_size[1] - reference_size[1],
                "candidate_foreground_bbox": geometry_text(candidate_image, fuzz="5%"),
                "reference_foreground_bbox": geometry_text(reference_page, fuzz="5%"),
                "difference_bbox_after_normalization": difference_bbox(normalized, reference_page),
                "normalized_mae": normalized_compare_metric("MAE", normalized, reference_page),
                "normalized_rmse": normalized_compare_metric("RMSE", normalized, reference_page),
            }
        )
    if len(candidate_pages) != len(source["pages"]):
        raise FidelityError(
            f"DOCX page count differs: candidate={len(candidate_pages)}, source={len(source['pages'])}"
        )
    return {
        "source_page_count": len(source["pages"]),
        "candidate_page_count": len(candidate_pages),
        "page_count_delta": len(candidate_pages) - len(source["pages"]),
        "baseline_score": {"normalized_mae": 0.0, "normalized_rmse": 0.0},
        "candidate_score": {
            "mean_normalized_mae": sum(page["normalized_mae"] for page in comparisons) / len(comparisons),
            "mean_normalized_rmse": sum(page["normalized_rmse"] for page in comparisons) / len(comparisons),
        },
        "pages": comparisons,
    }


def installed_version(command: str) -> str:
    return run([require_tool(command), "--version"]).strip()


def load_reference(path: Path) -> dict[str, Any]:
    try:
        reference = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as error:
        raise FidelityError(f"reference config is missing: {path}") from error
    except json.JSONDecodeError as error:
        raise FidelityError(f"reference config is not JSON: {path}") from error
    if reference.get("schema_version") != 1:
        raise FidelityError("reference config schema_version must be 1")
    return reference


def fixture_from_reference(reference: dict[str, Any], kind: str) -> tuple[Path, dict[str, Any]]:
    fixtures = reference.get("fixtures")
    if not isinstance(fixtures, dict) or not isinstance(fixtures.get(kind), dict):
        raise FidelityError(f"reference config has no {kind} fixture")
    fixture_config = fixtures[kind]
    relative_path = fixture_config.get("path")
    expected_hash = fixture_config.get("sha256")
    if not isinstance(relative_path, str) or not isinstance(expected_hash, str):
        raise FidelityError(f"reference config {kind} fixture lacks path or sha256")
    path = REPO_ROOT / relative_path
    if not path.is_file():
        raise FidelityError(f"pinned {kind} fixture is absent: {path}")
    actual_hash = sha256_file(path)
    if actual_hash != expected_hash:
        raise FidelityError(
            f"pinned {kind} fixture hash changed: expected={expected_hash}, actual={actual_hash}"
        )
    return path, fixture_config


def validate_source(reference: dict[str, Any]) -> tuple[dict[str, Any], dict[str, Any]]:
    renderer = reference.get("source_renderer")
    rasterizer = reference.get("rasterizer")
    if not isinstance(renderer, dict) or not isinstance(rasterizer, dict):
        raise FidelityError("reference config lacks renderer or rasterizer")
    command = renderer.get("command")
    expected_version = renderer.get("version")
    if not isinstance(command, str) or not isinstance(expected_version, str):
        raise FidelityError("reference renderer command/version is invalid")
    actual_version = installed_version(command)
    if actual_version != expected_version:
        raise FidelityError(
            f"source renderer version changed: expected={expected_version!r}, actual={actual_version!r}"
        )
    dpi = rasterizer.get("dpi")
    if not isinstance(dpi, int) or dpi <= 0:
        raise FidelityError("reference rasterizer dpi must be a positive integer")
    return renderer, rasterizer


def expected_source_pages(fixture_config: dict[str, Any], source: dict[str, Any], kind: str) -> None:
    expected_count = fixture_config.get("expected_page_count")
    expected_dimensions = fixture_config.get("reference_viewport_px")
    if not isinstance(expected_count, int) or not isinstance(expected_dimensions, list) or len(expected_dimensions) != 2:
        raise FidelityError(f"reference config {kind} page count/viewport is invalid")
    if len(source["pages"]) != expected_count:
        raise FidelityError(
            f"source {kind} page count changed: expected={expected_count}, actual={len(source['pages'])}"
        )
    expected = tuple(int(value) for value in expected_dimensions)
    for page in source["pages"]:
        actual = image_dimensions(page)
        if actual != expected:
            raise FidelityError(
                f"source {kind} viewport changed: expected={expected}, actual={actual}"
            )


def tolerance_failures(reference: dict[str, Any], result: dict[str, Any]) -> list[str]:
    tolerances = reference.get("tolerances")
    if not isinstance(tolerances, dict):
        raise FidelityError("reference config has no recorded tolerances")
    docx_tolerance = tolerances.get("docx")
    xlsx_tolerance = tolerances.get("xlsx")
    if not isinstance(docx_tolerance, dict) or not isinstance(xlsx_tolerance, dict):
        raise FidelityError("reference config docx/xlsx tolerances are invalid")
    failures = []
    docx = result["fixtures"]["docx"]["comparison"]
    score = docx["candidate_score"]
    for key, actual in (
        ("max_mean_normalized_mae", score["mean_normalized_mae"]),
        ("max_mean_normalized_rmse", score["mean_normalized_rmse"]),
        ("max_page_count_delta", abs(docx["page_count_delta"])),
        (
            "max_native_width_delta",
            max(abs(int(page["width_delta"])) for page in docx["pages"]),
        ),
        (
            "max_native_height_delta",
            max(abs(int(page["height_delta"])) for page in docx["pages"]),
        ),
    ):
        expected = docx_tolerance.get(key)
        if not isinstance(expected, (int, float)):
            raise FidelityError(f"reference config docx tolerance is missing {key}")
        if actual > expected:
            failures.append(f"docx {key}: actual={actual}, allowed={expected}")
    xlsx = result["fixtures"]["xlsx"]["comparison"]
    for key, actual in (
        ("max_worksheet_count_delta", abs(xlsx["worksheet_count_delta"])),
        ("max_text_missing_count", xlsx["totals"]["text_missing_count"]),
        ("max_text_mismatch_count", xlsx["totals"]["text_mismatch_count"]),
        ("max_font_missing_count", xlsx["totals"]["font_missing_count"]),
        ("max_font_difference_count", xlsx["totals"]["font_difference_count"]),
        ("max_fill_missing_count", xlsx["totals"]["fill_missing_count"]),
        ("max_fill_difference_count", xlsx["totals"]["fill_difference_count"]),
        (
            "max_border_metadata_missing_count",
            xlsx["totals"]["border_metadata_missing_count"],
        ),
        ("max_border_difference_count", xlsx["totals"]["border_difference_count"]),
        ("max_border_visual_missing_count", xlsx["border_visual_missing_count"]),
        ("max_merged_cell_missing_count", xlsx["totals"]["merged_cell_missing_count"]),
        (
            "max_pagination_missing_page_surface_count",
            xlsx["pagination_missing_page_surface_count"],
        ),
    ):
        expected = xlsx_tolerance.get(key)
        if not isinstance(expected, int):
            raise FidelityError(f"reference config xlsx tolerance is missing {key}")
        if actual > expected:
            failures.append(f"xlsx {key}: actual={actual}, allowed={expected}")
    for track_key in ("row_track_mean_absolute_delta", "column_track_mean_absolute_delta"):
        expected = xlsx_tolerance.get(f"max_{track_key}")
        if not isinstance(expected, (int, float)):
            raise FidelityError(f"reference config xlsx tolerance is missing max_{track_key}")
        values = [sheet[track_key] for sheet in xlsx["sheets"]]
        if any(value is None for value in values):
            failures.append(f"xlsx {track_key}: candidate did not expose every source track")
        elif max(float(value) for value in values) > expected:
            failures.append(
                f"xlsx {track_key}: actual={max(float(value) for value in values)}, allowed={expected}"
            )
    return failures


def measure(reference_path: Path) -> dict[str, Any]:
    reference = load_reference(reference_path)
    renderer, rasterizer = validate_source(reference)
    docx_path, docx_config = fixture_from_reference(reference, "docx")
    xlsx_path, xlsx_config = fixture_from_reference(reference, "xlsx")
    with tempfile.TemporaryDirectory(prefix="kdv-office-fidelity-") as temporary:
        workdir = Path(temporary)
        docx_source = convert_source_fixture(
            docx_path, workdir=workdir, label="docx", dpi=int(rasterizer["dpi"])
        )
        xlsx_source_artifact = convert_source_fixture(
            xlsx_path, workdir=workdir, label="xlsx", dpi=int(rasterizer["dpi"])
        )
        expected_source_pages(docx_config, docx_source, "docx")
        expected_source_pages(xlsx_config, xlsx_source_artifact, "xlsx")
        candidate = capture_candidate(workdir)
        docx_comparison = docx_measure(candidate, docx_source, workdir)
        candidate_projection = reference.get("candidate_projection")
        if not isinstance(candidate_projection, dict):
            raise FidelityError("reference config lacks candidate projection capabilities")
        xlsx_comparison = xlsx_measure(
            source_xlsx(xlsx_path),
            candidate.get("xlsx", {}),
            len(xlsx_source_artifact["pages"]),
            bool(candidate_projection.get("custom_cell_border_rendering")),
        )
        result = {
            "schema_version": 1,
            "generated_at": datetime.now(timezone.utc).isoformat(),
            "reference_config": {
                "path": str(reference_path.relative_to(REPO_ROOT)),
                "sha256": sha256_file(reference_path),
            },
            "source_renderer": {
                "command": renderer["command"],
                "version": renderer["version"],
                "comparison_only": bool(renderer.get("comparison_only")),
            },
            "rasterizer": {
                "command": rasterizer["command"],
                "dpi": rasterizer["dpi"],
            },
            "fixtures": {
                "docx": {
                    "path": docx_config["path"],
                    "sha256": sha256_file(docx_path),
                    "source_artifact": {
                        "pdf_sha256": docx_source["pdf_sha256"],
                        "pages": docx_source["page_artifacts"],
                    },
                    "comparison": docx_comparison,
                },
                "xlsx": {
                    "path": xlsx_config["path"],
                    "sha256": sha256_file(xlsx_path),
                    "source_artifact": {
                        "pdf_sha256": xlsx_source_artifact["pdf_sha256"],
                        "pages": xlsx_source_artifact["page_artifacts"],
                    },
                    "comparison": xlsx_comparison,
                },
            },
        }
    return result


def verify_record(reference_path: Path, record_path: Path) -> None:
    reference = load_reference(reference_path)
    try:
        result = json.loads(record_path.read_text(encoding="utf-8"))
    except FileNotFoundError as error:
        raise FidelityError(f"fidelity record is missing: {record_path}") from error
    except json.JSONDecodeError as error:
        raise FidelityError(f"fidelity record is not JSON: {record_path}") from error
    if result.get("schema_version") != 1:
        raise FidelityError("fidelity record schema_version must be 1")
    if result.get("reference_config", {}).get("sha256") != sha256_file(reference_path):
        raise FidelityError("fidelity record does not match the pinned reference config")
    for kind in ("docx", "xlsx"):
        fixture, _ = fixture_from_reference(reference, kind)
        recorded = result.get("fixtures", {}).get(kind, {}).get("sha256")
        if recorded != sha256_file(fixture):
            raise FidelityError(f"fidelity record {kind} hash does not match the pinned fixture")
    failures = tolerance_failures(reference, result)
    if failures:
        raise FidelityError("recorded fidelity baseline exceeds tolerance:\n" + "\n".join(failures))


def self_test() -> None:
    assert cell_coordinate("A1") == (0, 0)
    assert cell_coordinate("AA10") == (9, 26)
    assert cell_range("B2:D4") == (1, 1, 3, 3)
    assert normalize_color("00ffffff") == "FFFFFF"
    assert normalize_color("#183B66") == "183B66"
    assert normalize_color("theme:1") is None
    assert normalized_track_delta([1.0, 3.0], [10, 30]) == 0.0
    assert normalized_track_delta([1.0, 3.0], [10, None]) is None
    assert source_font_difference(
        {"name": None, "size_px": 11.0, "bold": True, "italic": False, "color": "FFFFFF"},
        {"family": "Inter", "size_px": 11, "bold": True, "italic": False, "text_color": "#FFFFFF"},
    ) is False


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--reference", type=Path, default=DEFAULT_REFERENCE)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--verify", action="store_true")
    parser.add_argument("--verify-record", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        self_test()
        return
    if args.verify_record is not None:
        verify_record(args.reference, args.verify_record)
        return
    if args.output is None:
        raise SystemExit("--output is required unless --self-test or --verify-record is used")
    if args.output.exists():
        raise SystemExit(f"refusing to overwrite existing fidelity record: {args.output}")
    result = measure(args.reference)
    failures = tolerance_failures(load_reference(args.reference), result) if args.verify else []
    result["verification"] = {
        "tolerances_checked": args.verify,
        "tolerances_met": not failures,
        "failures": failures,
    }
    if failures:
        raise SystemExit("fidelity measurement exceeds tolerance:\n" + "\n".join(failures))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("x", encoding="utf-8") as output:
        json.dump(result, output, ensure_ascii=False, indent=2)
        output.write("\n")


if __name__ == "__main__":
    main()
