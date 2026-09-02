#!/usr/bin/env python3
"""Generate and verify the exact KatanA-reproduced DOCX data-descriptor fixture."""

from __future__ import annotations

import argparse
import hashlib
import io
import struct
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
FIXTURE_DIR = ROOT / "assets/fixtures/multi-format"
SOURCE = FIXTURE_DIR / "representative.docx"
FIXTURE = FIXTURE_DIR / "data-descriptor.docx"
SOURCE_SHA256 = "ce8ec76f77935e63824331f783bbb73f086333c6b331dda5a6d01c1e5f69d0dc"
FIXTURE_SHA256 = "a1b7e22021218d314bc2d90c526d6d682981828b67cef6e61d8cb2a71ef5742a"
ENTRY_COUNT = 20
DATA_DESCRIPTOR_FLAG = 0x0008
LOCAL_FILE_HEADER = struct.Struct("<IHHHHHIIIHH")
LOCAL_FILE_SIGNATURE = 0x04034B50
DOCUMENT_ENTRY = "word/document.xml"
DOCUMENT_SIZES = (1383, 4907)


class NonSeekableBuffer(io.RawIOBase):
    """Write-only stream that forces zipfile to emit data descriptors."""

    def __init__(self) -> None:
        self._data = bytearray()

    def writable(self) -> bool:
        return True

    def seekable(self) -> bool:
        return False

    def write(self, data: bytes | bytearray) -> int:
        self._data.extend(data)
        return len(data)

    def tell(self) -> int:
        return len(self._data)

    def value(self) -> bytes:
        return bytes(self._data)


def digest(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def generate(source: Path) -> bytes:
    sink = NonSeekableBuffer()
    with zipfile.ZipFile(source) as source_archive:
        with zipfile.ZipFile(
            sink,
            mode="w",
            compression=zipfile.ZIP_DEFLATED,
            compresslevel=9,
            strict_timestamps=True,
        ) as output_archive:
            for source_info in sorted(source_archive.infolist(), key=lambda info: info.filename):
                info = zipfile.ZipInfo(source_info.filename, (2026, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                info.create_system = 3
                info.external_attr = source_info.external_attr
                output_archive.writestr(info, source_archive.read(source_info))
    return sink.value()


def descriptor_entries(payload: bytes) -> list[zipfile.ZipInfo]:
    with zipfile.ZipFile(io.BytesIO(payload)) as archive:
        entries = archive.infolist()
        if archive.testzip() is not None:
            raise ValueError("fixture ZIP payload CRC validation failed")
        for info in entries:
            header = LOCAL_FILE_HEADER.unpack_from(payload, info.header_offset)
            signature, flags = header[0], header[2]
            crc32, compressed_size, uncompressed_size = header[6:9]
            if signature != LOCAL_FILE_SIGNATURE:
                raise ValueError(f"invalid local header for {info.filename}")
            if flags & DATA_DESCRIPTOR_FLAG == 0 or info.flag_bits & DATA_DESCRIPTOR_FLAG == 0:
                raise ValueError(f"data descriptor flag is absent for {info.filename}")
            if (crc32, compressed_size, uncompressed_size) != (0, 0, 0):
                raise ValueError(f"local header contains sizes for {info.filename}")
        return entries


def validate_payload(payload: bytes) -> None:
    if digest(payload) != FIXTURE_SHA256:
        raise ValueError("data-descriptor DOCX SHA-256 does not match the KatanA reproduction")
    entries = descriptor_entries(payload)
    if len(entries) != ENTRY_COUNT:
        raise ValueError(f"expected {ENTRY_COUNT} data-descriptor entries, got {len(entries)}")
    document = next((entry for entry in entries if entry.filename == DOCUMENT_ENTRY), None)
    if document is None:
        raise ValueError(f"fixture has no {DOCUMENT_ENTRY}")
    if (document.compress_size, document.file_size) != DOCUMENT_SIZES:
        raise ValueError("word/document.xml central-directory sizes differ from the reproduction")


def write_fixture(source: Path, output: Path) -> None:
    source_bytes = source.read_bytes()
    if digest(source_bytes) != SOURCE_SHA256:
        raise ValueError("representative DOCX SHA-256 changed; refusing to regenerate fixture")
    payload = generate(source)
    validate_payload(payload)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(payload)
    print(f"generated {output} with {ENTRY_COUNT} data-descriptor entries")


def verify_fixture(source: Path, fixture: Path) -> None:
    if digest(source.read_bytes()) != SOURCE_SHA256:
        raise ValueError("representative DOCX SHA-256 changed")
    payload = fixture.read_bytes()
    validate_payload(payload)
    if payload != generate(source):
        raise ValueError("data-descriptor DOCX is not the deterministic source rewrite")
    print(f"data-descriptor DOCX fixture verified: sha256={FIXTURE_SHA256}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, default=SOURCE)
    parser.add_argument("--fixture", type=Path, default=FIXTURE)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    if args.write:
        write_fixture(args.source, args.fixture)
    else:
        verify_fixture(args.source, args.fixture)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
