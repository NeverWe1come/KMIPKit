"""Extract deterministic normative-keyword candidate locators from local HTML."""

from __future__ import annotations

import argparse
import json
import re
import sys
from html.parser import HTMLParser
from pathlib import Path
from typing import Any


MAX_DOCUMENT_BYTES = 8 * 1024 * 1024
MAX_TOTAL_BYTES = 16 * 1024 * 1024
MAX_DEPTH = 64
MAX_EVENTS = 1_000_000
MAX_TEXT_BYTES = 16 * 1024 * 1024
KEYWORDS = (
    "MUST NOT", "SHALL NOT", "SHOULD NOT", "RECOMMENDED", "REQUIRED",
    "OPTIONAL", "MUST", "SHALL", "SHOULD", "MAY",
)
KEYWORD_PATTERN = re.compile(
    r"(?<![A-Za-z0-9_])(" + "|".join(re.escape(item) for item in KEYWORDS) + r")(?![A-Za-z0-9_])",
    re.IGNORECASE,
)
META_TAG_PATTERN = re.compile(rb"<meta\b[^>]*>", re.IGNORECASE)
CHARSET_PATTERN = re.compile(
    rb"\bcharset\s*=\s*(?:\"([^\"]*)\"|'([^']*)'|([^\s;>\"']+))",
    re.IGNORECASE,
)


class SourceAuditError(ValueError):
    """Raised for malformed, unsupported, or over-limit pinned HTML."""


def _decode_document(raw: bytes) -> str:
    if len(raw) > MAX_DOCUMENT_BYTES:
        raise SourceAuditError("HTML document exceeds the 8 MiB limit")
    encoding = "utf-8"
    for tag in META_TAG_PATTERN.findall(raw[:16_384]):
        match = CHARSET_PATTERN.search(tag)
        if match is None:
            continue
        declared_bytes = next((group for group in match.groups() if group is not None), b"")
        declared = declared_bytes.decode("ascii", errors="strict").strip().lower()
        aliases = {"utf-8": "utf-8", "utf8": "utf-8", "windows-1252": "cp1252", "cp1252": "cp1252"}
        encoding = aliases.get(declared, "")
        if not encoding:
            raise SourceAuditError("HTML declares a malformed or unsupported charset")
        break
    try:
        return raw.decode(encoding, errors="strict")
    except (UnicodeDecodeError, UnicodeError) as error:
        raise SourceAuditError("HTML cannot be decoded using its declared charset") from error


class _CandidateParser(HTMLParser):
    def __init__(self, source_id: str) -> None:
        super().__init__(convert_charrefs=True)
        if source_id not in {"KMIPKIT-SRC-spec", "KMIPKIT-SRC-profiles"}:
            raise SourceAuditError("source auditing is limited to the normative Specification and Profiles")
        self.source_id = source_id
        self.prefix = "PROF" if source_id.endswith("profiles") else "SPEC"
        self.section = "0"
        self.section_ordinals: dict[str, int] = {}
        self.stack: list[dict[str, Any]] = []
        self.candidates: list[dict[str, Any]] = []
        self.heading_tag: str | None = None
        self.heading_depth = 0
        self.heading_text: list[str] = []
        self.events = 0
        self.text_bytes = 0

    def _event(self) -> None:
        self.events += 1
        if self.events > MAX_EVENTS:
            raise SourceAuditError("HTML exceeds the 1,000,000 parser-event limit")

    def _active_candidate(self) -> bool:
        return any(context["candidate"] is not None for context in self.stack)

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        self._event()
        if len(self.stack) + 1 > MAX_DEPTH:
            raise SourceAuditError("HTML exceeds the maximum nesting depth of 64")
        attributes = dict(attrs)
        classes = (attributes.get("class") or "").split()
        is_heading = tag in {"h1", "h2", "h3", "h4", "h5", "h6"} or (
            tag == "p" and any(re.fullmatch(r"MsoHeading[1-6]", item, re.IGNORECASE) for item in classes)
        )
        if is_heading:
            self.heading_tag = tag
            self.heading_depth = len(self.stack) + 1
            self.heading_text = []

        block_kind: str | None = None
        if not self._active_candidate():
            if tag == "tr":
                block_kind = "table_row"
            elif tag == "li":
                block_kind = "list_item"
            elif tag in {"dt", "dd"}:
                block_kind = "definition_item"
            elif tag == "p" and not is_heading:
                block_kind = "paragraph"
        candidate = None
        if block_kind is not None:
            candidate = {"block_kind": block_kind, "section": self.section, "text": []}
        if tag in {"td", "th"}:
            for context in reversed(self.stack):
                active = context["candidate"]
                if active is not None:
                    if active["block_kind"] == "table_row":
                        active["text"].append(" ")
                    break
        self.stack.append({"tag": tag, "candidate": candidate})

    def handle_startendtag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        self.handle_starttag(tag, attrs)
        self.handle_endtag(tag)

    def handle_endtag(self, tag: str) -> None:
        self._event()
        matching_index = None
        for index in range(len(self.stack) - 1, -1, -1):
            if self.stack[index]["tag"] == tag:
                matching_index = index
                break
        if matching_index is None:
            return
        closed = self.stack[matching_index:]
        del self.stack[matching_index:]
        for context in reversed(closed):
            candidate = context["candidate"]
            if candidate is not None:
                self._finish_candidate(candidate)
        if self.heading_tag == tag and len(self.stack) < self.heading_depth:
            heading = "".join(self.heading_text).strip()
            match = re.match(r"^([0-9]+(?:\.[0-9]+)*)(?:\s|$)", heading)
            if match:
                self.section = match.group(1)
                self.section_ordinals.setdefault(self.section, 0)
            self.heading_tag = None
            self.heading_text = []

    def handle_data(self, data: str) -> None:
        self._event()
        if not data:
            return
        try:
            self.text_bytes += len(data.encode("utf-8", errors="strict"))
        except UnicodeError as error:
            raise SourceAuditError("HTML contains invalid Unicode text") from error
        if self.text_bytes > MAX_TEXT_BYTES:
            raise SourceAuditError("HTML extracted text exceeds the 16 MiB limit")
        if self.heading_tag is not None:
            self.heading_text.append(data)
        for context in self.stack:
            if context["candidate"] is not None:
                context["candidate"]["text"].append(data)
                break

    def _finish_candidate(self, candidate: dict[str, Any]) -> None:
        text = "".join(candidate["text"])
        found: list[str] = []
        for match in KEYWORD_PATTERN.finditer(text):
            keyword = next(item for item in KEYWORDS if item.casefold() == match.group(1).casefold())
            if keyword not in found:
                found.append(keyword)
        if not found:
            return
        section = candidate["section"]
        ordinal = self.section_ordinals.get(section, 0) + 1
        self.section_ordinals[section] = ordinal
        clause_id = f"KMIPKIT-CLAUSE-{self.prefix}-{section}-{ordinal:03d}"
        self.candidates.append(
            {
                "clause_id": clause_id,
                "source_id": self.source_id,
                "section": section,
                "locator": {"ordinal": ordinal, "block_kind": candidate["block_kind"]},
                "source_keywords": found,
            }
        )


def audit_document(raw: bytes, source_id: str) -> list[dict[str, Any]]:
    """Return candidate locator records without retaining normative source text."""
    parser = _CandidateParser(source_id)
    try:
        parser.feed(_decode_document(raw))
        parser.close()
    except SourceAuditError:
        raise
    except (AssertionError, ValueError) as error:
        raise SourceAuditError("malformed HTML source") from error
    return parser.candidates


def audit_git_sources(repo_root: Path, base_sha: str) -> list[dict[str, Any]]:
    """Audit only the allowlisted, checksum-verified normative HTML Git blobs."""
    from tools.normative_catalog.check_immutable_sources import check_immutable_sources
    from tools.normative_catalog.validate import _source_manifest

    check_immutable_sources(repo_root, base_sha)
    manifest = _source_manifest(repo_root)
    total = 0
    records: list[dict[str, Any]] = []
    for source_id in ("KMIPKIT-SRC-spec", "KMIPKIT-SRC-profiles"):
        path = repo_root / manifest[source_id]["local_path"]
        raw = path.read_bytes()
        total += len(raw)
        if total > MAX_TOTAL_BYTES:
            raise SourceAuditError("total pinned HTML input exceeds the 16 MiB limit")
        records.extend(audit_document(raw, source_id))
    return records


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base-sha", required=True, help="exact pull-request base commit SHA")
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--check", action="store_true", help="compare candidates with the checked-in clause ledger")
    arguments = parser.parse_args(argv)
    try:
        records = audit_git_sources(arguments.repo_root, arguments.base_sha)
        if arguments.check:
            catalog_path = arguments.repo_root / "specification" / "catalog" / "kmip-2.1.json"
            catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
            ledger = catalog["source_clauses"]
            candidates = {record["clause_id"]: record for record in records}
            ledger_ids = {record["clause_id"] for record in ledger}
            if set(candidates) != ledger_ids:
                raise SourceAuditError("source candidate and clause-ledger IDs do not reconcile")
            for clause_id, candidate in candidates.items():
                clause = next(item for item in ledger if item["clause_id"] == clause_id)
                for key in ("source_id", "section", "locator", "source_keywords"):
                    if clause.get(key) != candidate[key]:
                        raise SourceAuditError("source candidate metadata differs from the reviewed clause ledger")
    except (OSError, KeyError, TypeError, json.JSONDecodeError, SourceAuditError, ValueError) as error:
        print(f"source audit failed: {error}", file=sys.stderr)
        return 1
    print(f"Audited {len(records)} normative source candidates")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
