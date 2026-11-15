"""Audit catalogued official case IDs and links against pinned local OASIS HTML."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from dataclasses import dataclass, field
from html.parser import HTMLParser
from pathlib import Path, PurePosixPath
from typing import Any

if __package__:
    from .safe_io import PathSecurityError, safe_read_bytes
    from .validate import _source_manifest
else:
    from safe_io import PathSecurityError, safe_read_bytes
    from validate import _source_manifest

MAX_CATALOG_BYTES = 16 * 1024 * 1024
MAX_SOURCE_BYTES = 16 * 1024 * 1024
MAX_HTML_DEPTH = 64
MAX_HTML_EVENTS = 1_000_000
MAX_EXTRACTED_TEXT_BYTES = 8 * 1024 * 1024
CASE_ID_TOKEN_PATTERN = re.compile(r"[A-Za-z0-9-]+")

SOURCE_BY_CATEGORY = {
    "profile_test_case": "KMIPKIT-SRC-profiles",
    "official_conformance_test": "KMIPKIT-SRC-testcases",
}


class CaseAuditError(ValueError):
    """Raised when the catalog and checksum-pinned case evidence disagree."""


def _raw_href_value(raw_tag: str) -> str | None:
    """Return the unmodified value of an actual href attribute in a start tag."""
    html_whitespace = " \t\n\f\r"
    if not raw_tag.startswith("<"):
        return None

    index = 1
    while index < len(raw_tag) and raw_tag[index] not in html_whitespace + "/>":
        index += 1

    while index < len(raw_tag):
        while index < len(raw_tag) and raw_tag[index] in html_whitespace:
            index += 1
        if index >= len(raw_tag) or raw_tag[index] in "/>":
            return None

        name_start = index
        while index < len(raw_tag) and raw_tag[index] not in html_whitespace + "/>=":
            index += 1
        if index == name_start:
            return None
        attribute_name = raw_tag[name_start:index].casefold()

        while index < len(raw_tag) and raw_tag[index] in html_whitespace:
            index += 1
        value: str | None = None
        if index < len(raw_tag) and raw_tag[index] == "=":
            index += 1
            while index < len(raw_tag) and raw_tag[index] in html_whitespace:
                index += 1
            if index < len(raw_tag) and raw_tag[index] in "\"'":
                quote = raw_tag[index]
                value_start = index + 1
                value_end = raw_tag.find(quote, value_start)
                if value_end < 0:
                    return None
                value = raw_tag[value_start:value_end]
                index = value_end + 1
            else:
                value_start = index
                while index < len(raw_tag) and raw_tag[index] not in html_whitespace + ">":
                    index += 1
                value = raw_tag[value_start:index]

        if attribute_name == "href":
            return value

    return None


@dataclass
class _CaseEvidence:
    section: str
    official_case_id: str
    hrefs: set[str] = field(default_factory=set)


class _CaseHeadingParser(HTMLParser):
    """Collect numeric case headings and links up to the next HTML heading."""

    def __init__(self, official_ids: set[str]) -> None:
        super().__init__(convert_charrefs=True)
        self.official_ids = official_ids
        self.stack: list[str] = []
        self.heading_tag: str | None = None
        self.heading_text: list[str] = []
        self.active_case: _CaseEvidence | None = None
        self.cases: list[_CaseEvidence] = []
        self.events = 0
        self.text_bytes = 0

    def _event(self) -> None:
        self.events += 1
        if self.events > MAX_HTML_EVENTS:
            raise CaseAuditError("HTML exceeds the 1,000,000 parser-event limit")

    def _finish_heading(self) -> None:
        if self.heading_tag is None:
            return
        text = "".join(self.heading_text)
        self.heading_tag = None
        self.heading_text = []
        section_match = re.match(r"^\s*([0-9]+(?:\.[0-9]+)*)\b", text)
        if section_match is None:
            self.active_case = None
            return
        section = section_match.group(1)
        matches = {
            token.group(0)
            for token in CASE_ID_TOKEN_PATTERN.finditer(text)
            if token.group(0) in self.official_ids
        }
        if len(matches) == 1:
            self.active_case = _CaseEvidence(section, next(iter(matches)))
            self.cases.append(self.active_case)
        else:
            self.active_case = None

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        self._event()
        if len(self.stack) + 1 > MAX_HTML_DEPTH:
            raise CaseAuditError("HTML exceeds the maximum nesting depth of 64")
        if re.fullmatch(r"h[1-6]", tag):
            self._finish_heading()
            self.active_case = None
            self.heading_tag = tag
            self.heading_text = []
        elif tag == "a" and self.active_case is not None:
            raw_tag = self.get_starttag_text() or ""
            href = _raw_href_value(raw_tag) if any(name.casefold() == "href" for name, _ in attrs) else None
            if href is not None:
                self.active_case.hrefs.add(href)
        self.stack.append(tag)

    def handle_startendtag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        self.handle_starttag(tag, attrs)
        self.handle_endtag(tag)

    def handle_endtag(self, tag: str) -> None:
        self._event()
        for index in range(len(self.stack) - 1, -1, -1):
            if self.stack[index] == tag:
                del self.stack[index:]
                break
        if tag == self.heading_tag:
            self._finish_heading()

    def handle_data(self, data: str) -> None:
        self._event()
        if self.heading_tag is None:
            return
        self.text_bytes += len(data.encode("utf-8", errors="strict"))
        if self.text_bytes > MAX_EXTRACTED_TEXT_BYTES:
            raise CaseAuditError("HTML heading text exceeds the 8 MiB limit")
        self.heading_text.append(data)

    def close(self) -> None:
        super().close()
        self._finish_heading()


def _decode_html(raw: bytes) -> str:
    """Decode the pinned HTML using its declared Windows-1252 or UTF-8 encoding."""
    if len(raw) > MAX_SOURCE_BYTES:
        raise CaseAuditError("pinned HTML source exceeds the 16 MiB input limit")
    charset_pattern = re.compile(
        rb"\bcharset\s*=\s*(?:\"([^\"]*)\"|'([^']*)'|([^\s;>\"']+))",
        re.IGNORECASE,
    )
    encoding = "utf-8-sig"
    for meta in re.findall(rb"<meta\b[^>]*>", raw[:16_384], re.IGNORECASE):
        match = charset_pattern.search(meta)
        if match is None:
            continue
        declared = next((item for item in match.groups() if item is not None), b"")
        try:
            name = declared.decode("ascii", errors="strict").strip().lower()
        except UnicodeDecodeError as error:
            raise CaseAuditError("pinned HTML declares an invalid charset") from error
        encodings = {"utf-8": "utf-8-sig", "utf8": "utf-8-sig", "windows-1252": "cp1252", "cp1252": "cp1252"}
        if name not in encodings:
            raise CaseAuditError("pinned HTML declares an unsupported charset")
        encoding = encodings[name]
        break
    try:
        return raw.decode(encoding, errors="strict")
    except UnicodeDecodeError as error:
        raise CaseAuditError("pinned HTML cannot be decoded using its declared charset") from error


def _parse_case_evidence(raw: bytes, official_ids: set[str]) -> list[_CaseEvidence]:
    parser = _CaseHeadingParser(official_ids)
    try:
        parser.feed(_decode_html(raw))
        parser.close()
    except CaseAuditError:
        raise
    except (AssertionError, ValueError, UnicodeError) as error:
        raise CaseAuditError("pinned HTML case headings could not be parsed") from error
    return parser.cases


def audit_catalog_cases(repository: Path, catalog: dict[str, Any]) -> tuple[dict[str, int], list[dict[str, str]]]:
    """Verify every catalogued case against its declared pinned source record."""
    sources = catalog.get("sources")
    cases = catalog.get("test_cases")
    if not isinstance(sources, list) or not isinstance(cases, list):
        raise CaseAuditError("catalog must contain source and test_cases arrays")

    try:
        pinned_sources = _source_manifest(repository)
    except (OSError, ValueError) as error:
        raise CaseAuditError("pinned OASIS source manifest is unavailable") from error

    source_records: dict[str, dict[str, Any]] = {}
    for source in sources:
        if isinstance(source, dict) and isinstance(source.get("source_id"), str):
            if source["source_id"] in source_records:
                raise CaseAuditError("catalog contains duplicate source IDs")
            source_records[source["source_id"]] = source

    categories: dict[str, list[dict[str, Any]]] = {category: [] for category in SOURCE_BY_CATEGORY}
    for case in cases:
        if not isinstance(case, dict):
            raise CaseAuditError("catalog test case record must be an object")
        category = case.get("evidence_category")
        if not isinstance(category, str) or category not in SOURCE_BY_CATEGORY:
            raise CaseAuditError("catalog contains an unsupported official case evidence category")
        expected_source_id = SOURCE_BY_CATEGORY[category]
        if case.get("source_id") != expected_source_id:
            raise CaseAuditError("evidence category is bound to a different source ID")
        for field_name in ("test_id", "official_case_id", "source_section", "raw_href"):
            if not isinstance(case.get(field_name), str) or not case[field_name]:
                raise CaseAuditError("catalog test case is missing required source evidence")
        categories[category].append(case)

    parsed_by_source: dict[str, list[_CaseEvidence]] = {}
    for source_id in sorted(set(SOURCE_BY_CATEGORY.values())):
        source = source_records.get(source_id)
        if source is None:
            raise CaseAuditError(f"catalog source record is missing: {source_id}")
        pinned_source = pinned_sources.get(source_id)
        catalog_metadata = {key: value for key, value in source.items() if key != "source_id"}
        if not isinstance(pinned_source, dict) or catalog_metadata != pinned_source:
            raise CaseAuditError("catalog source metadata does not match the pinned OASIS source manifest")
        relative_path = source.get("local_path")
        expected_digest = source.get("sha256")
        if not isinstance(relative_path, str) or not isinstance(expected_digest, str):
            raise CaseAuditError(f"catalog source record is incomplete: {source_id}")
        if re.fullmatch(r"[0-9a-f]{64}", expected_digest) is None:
            raise CaseAuditError(f"catalog source record has an invalid SHA-256: {source_id}")
        try:
            raw = safe_read_bytes(repository, relative_path, max_bytes=MAX_SOURCE_BYTES)
        except (OSError, PathSecurityError) as error:
            raise CaseAuditError(f"pinned source cannot be read safely: {source_id}") from error
        if hashlib.sha256(raw).hexdigest() != expected_digest:
            raise CaseAuditError(f"SHA-256 does not match the catalog source record: {source_id}")
        official_ids = {case["official_case_id"] for case in categories[next(
            category for category, bound_source_id in SOURCE_BY_CATEGORY.items()
            if bound_source_id == source_id
        )]}
        parsed_by_source[source_id] = _parse_case_evidence(raw, official_ids)

    evidence_by_source: dict[str, dict[tuple[str, str], list[_CaseEvidence]]] = {}
    for source_id, evidence in parsed_by_source.items():
        evidence_index: dict[tuple[str, str], list[_CaseEvidence]] = {}
        for item in evidence:
            evidence_index.setdefault((item.section, item.official_case_id), []).append(item)
        evidence_by_source[source_id] = evidence_index

    counts: dict[str, int] = {}
    divergences: list[dict[str, str]] = []
    for category, expected_source_id in SOURCE_BY_CATEGORY.items():
        rows = categories[category]
        counts[category] = len(rows)
        evidence_index = evidence_by_source[expected_source_id]
        for row in rows:
            matches = evidence_index.get((row["source_section"], row["official_case_id"]), [])
            if len(matches) != 1 or row["raw_href"] not in matches[0].hrefs:
                raise CaseAuditError(
                    "catalog section, official ID, and exact href do not match the pinned source"
                )
            href_name = PurePosixPath(row["raw_href"]).name
            if href_name != f"{row['official_case_id']}.xml":
                divergences.append(
                    {
                        "source_id": expected_source_id,
                        "section": row["source_section"],
                        "official_case_id": row["official_case_id"],
                        "href_name": href_name,
                    }
                )
    divergences.sort(
        key=lambda item: (
            item["source_id"],
            tuple(int(part) for part in item["section"].split(".")),
            item["official_case_id"],
        )
    )
    return counts, divergences


def _load_catalog(repository: Path, relative_path: str) -> dict[str, Any]:
    try:
        raw = safe_read_bytes(repository, relative_path, max_bytes=MAX_CATALOG_BYTES)
        catalog = json.loads(raw.decode("utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError, PathSecurityError, RecursionError) as error:
        raise CaseAuditError("catalog is unavailable or invalid JSON") from error
    if not isinstance(catalog, dict):
        raise CaseAuditError("catalog root must be an object")
    return catalog


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--catalog", default="specification/catalog/kmip-2.1.json")
    arguments = parser.parse_args(argv)
    try:
        repository = arguments.repo_root.resolve(strict=True)
        catalog = _load_catalog(repository, arguments.catalog)
        counts, divergences = audit_catalog_cases(repository, catalog)
    except (OSError, CaseAuditError, ValueError) as error:
        print(f"official case source audit failed: {error}", file=sys.stderr)
        return 1

    print("Official case source audit (offline)")
    for category, source_id in SOURCE_BY_CATEGORY.items():
        print(f"{category}: {counts[category]} rows (source {source_id})")
    print("Catalog-to-source mismatches: 0")
    print(f"Source-internal official-ID/href-name divergences: {len(divergences)}")
    for item in divergences:
        print(
            f"  {item['source_id']} section {item['section']} "
            f"{item['official_case_id']} -> {item['href_name']}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
