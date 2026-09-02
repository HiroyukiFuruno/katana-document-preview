"""Small TOML reader for release-contract inputs on Python 3.9+.

Only Cargo manifest and lockfile constructs are needed here.  Unsupported
syntax fails closed instead of silently weakening a release verification.
"""

from __future__ import annotations

import ast
import re


TABLE_HEADER = re.compile(r"^\[(?P<name>.+)\]$")
ARRAY_TABLE_HEADER = re.compile(r"^\[\[(?P<name>.+)\]\]$")
KEY_VALUE = re.compile(r"^(?P<key>[A-Za-z0-9_.-]+)\s*=\s*(?P<value>.+)$")


def loads(text: str) -> dict[str, object]:
    """Parse the Cargo TOML subset used by KDV release verifiers."""

    root: dict[str, object] = {}
    current: dict[str, object] = root
    lines = iter(text.splitlines())
    for raw_line in lines:
        line = strip_comment(raw_line).strip()
        if not line:
            continue
        array_header = ARRAY_TABLE_HEADER.fullmatch(line)
        if array_header is not None:
            current = append_array_table(root, array_header.group("name"))
            continue
        table_header = TABLE_HEADER.fullmatch(line)
        if table_header is not None:
            current = table(root, table_header.group("name"))
            continue
        match = KEY_VALUE.fullmatch(line)
        if match is None:
            raise ValueError(f"unsupported TOML syntax: {raw_line}")
        value = match.group("value")
        while not value_is_complete(value):
            try:
                value += "\n" + strip_comment(next(lines)).strip()
            except StopIteration as error:
                raise ValueError(f"unterminated TOML value for {match.group('key')}") from error
        key = match.group("key")
        if key in current:
            raise ValueError(f"duplicate TOML key: {key}")
        current[key] = parse_value(value)
    return root


def table(root: dict[str, object], dotted_name: str) -> dict[str, object]:
    current = root
    for part in dotted_name.split("."):
        child = current.get(part)
        if child is None:
            child = {}
            current[part] = child
        if not isinstance(child, dict):
            raise ValueError(f"TOML table conflicts with value: {dotted_name}")
        current = child
    return current


def append_array_table(root: dict[str, object], dotted_name: str) -> dict[str, object]:
    parts = dotted_name.split(".")
    parent = root if len(parts) == 1 else table(root, ".".join(parts[:-1]))
    name = parts[-1]
    entries = parent.get(name)
    if entries is None:
        entries = []
        parent[name] = entries
    if not isinstance(entries, list):
        raise ValueError(f"TOML array table conflicts with value: {dotted_name}")
    entry: dict[str, object] = {}
    entries.append(entry)
    return entry


def parse_value(value: str) -> object:
    value = value.strip()
    if not value:
        raise ValueError("empty TOML value")
    if value.startswith("[") and value.endswith("]"):
        return [parse_value(item) for item in split_top_level(value[1:-1], ",") if item.strip()]
    if value.startswith("{") and value.endswith("}"):
        result: dict[str, object] = {}
        for item in split_top_level(value[1:-1], ","):
            if not item.strip():
                continue
            key, raw_value = split_key_value(item)
            if key in result:
                raise ValueError(f"duplicate TOML inline-table key: {key}")
            result[key] = parse_value(raw_value)
        return result
    if (value.startswith('"') and value.endswith('"')) or (
        value.startswith("'") and value.endswith("'")
    ):
        try:
            parsed = ast.literal_eval(value)
        except (SyntaxError, ValueError) as error:
            raise ValueError(f"invalid TOML string: {value}") from error
        if not isinstance(parsed, str):
            raise ValueError(f"invalid TOML string: {value}")
        return parsed
    if value == "true":
        return True
    if value == "false":
        return False
    if re.fullmatch(r"[-+]?[0-9]+", value):
        return int(value)
    raise ValueError(f"unsupported TOML value: {value}")


def split_key_value(value: str) -> tuple[str, str]:
    for index, character in enumerate(value):
        if character == "=" and nesting_before(value, index) == 0:
            key = value[:index].strip()
            if not re.fullmatch(r"[A-Za-z0-9_-]+", key):
                raise ValueError(f"unsupported TOML key: {key}")
            return key, value[index + 1 :]
    raise ValueError(f"inline table entry has no key/value separator: {value}")


def split_top_level(value: str, separator: str) -> list[str]:
    parts: list[str] = []
    start = 0
    quote: str | None = None
    escaped = False
    nesting = 0
    for index, character in enumerate(value):
        if quote is not None:
            if escaped:
                escaped = False
            elif character == "\\":
                escaped = True
            elif character == quote:
                quote = None
            continue
        if character in ("'", '"'):
            quote = character
        elif character in "[{":
            nesting += 1
        elif character in "]}":
            nesting -= 1
        elif character == separator and nesting == 0:
            parts.append(value[start:index])
            start = index + 1
    if quote is not None or nesting != 0:
        raise ValueError(f"unterminated TOML value: {value}")
    parts.append(value[start:])
    return parts


def value_is_complete(value: str) -> bool:
    try:
        split_top_level(value, "\0")
    except ValueError:
        return False
    return True


def strip_comment(value: str) -> str:
    quote: str | None = None
    escaped = False
    for index, character in enumerate(value):
        if quote is not None:
            if escaped:
                escaped = False
            elif character == "\\":
                escaped = True
            elif character == quote:
                quote = None
        elif character in ("'", '"'):
            quote = character
        elif character == "#":
            return value[:index]
    return value


def nesting_before(value: str, stop: int) -> int:
    prefix = value[:stop]
    quote: str | None = None
    escaped = False
    nesting = 0
    for character in prefix:
        if quote is not None:
            if escaped:
                escaped = False
            elif character == "\\":
                escaped = True
            elif character == quote:
                quote = None
        elif character in ("'", '"'):
            quote = character
        elif character in "[{":
            nesting += 1
        elif character in "]}":
            nesting -= 1
    return nesting
