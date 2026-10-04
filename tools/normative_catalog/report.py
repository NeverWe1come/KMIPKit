"""Render and verify the deterministic Markdown coverage report."""

from __future__ import annotations

import argparse
from collections import defaultdict
import sys
import unicodedata
from pathlib import Path
from typing import Any

if __package__:
    from .safe_io import PathSecurityError, atomic_write_bytes, confined_path, safe_read_bytes
else:
    from safe_io import PathSecurityError, atomic_write_bytes, confined_path, safe_read_bytes


REPORT_PATH = Path("specification/catalog/coverage-report.md")
SECTION_ORDER = (
    "Source documents",
    "Count reconciliation",
    "Source clause dispositions",
    "Source clause review by section",
    "Unassigned requirements",
    "Requirements needing negative verification",
    "Unassigned protocol elements",
    "Profile states",
    "Profiles by applicability and claim state",
    "Test fixture availability",
    "Test evidence by fixture availability",
    "Open discrepancies",
    "Project policies",
)


def _visible_controls(value: Any) -> str:
    text = str(value if value is not None else "")
    output: list[str] = []
    for character in text:
        category = unicodedata.category(character)
        if character == "\n":
            output.append("\\n")
        elif character == "\r":
            output.append("\\r")
        elif character == "\t":
            output.append("\\t")
        elif category in {"Cc", "Cf"}:
            output.append(f"\\u{ord(character):04X}")
        else:
            output.append(character)
    return "".join(output)


def _escape_table_text(value: Any) -> str:
    text = _visible_controls(value)
    replacements = (
        ("\\", "\\\\"),
        ("&", "&amp;"),
        ("<", "&lt;"),
        (">", "&gt;"),
        ("|", "\\|"),
        ("`", "\\`"),
        ("[", "\\["),
        ("]", "\\]"),
        ("(", "\\("),
        (")", "\\)"),
        ("*", "\\*"),
        ("_", "\\_"),
        ("~", "\\~"),
    )
    for old, new in replacements:
        text = text.replace(old, new)
    return text


def _escape_link_label(value: Any) -> str:
    text = _visible_controls(value)
    text = text.replace("\\", "\\\\")
    text = text.replace("[", "\\[").replace("]", "\\]")
    text = text.replace("<", "&lt;").replace(">", "&gt;")
    return text


def _table(headers: tuple[str, ...], rows: list[tuple[Any, ...]]) -> list[str]:
    lines = ["| " + " | ".join(_escape_table_text(item) for item in headers) + " |"]
    lines.append("| " + " | ".join("---" for _ in headers) + " |")
    for row in rows:
        lines.append("| " + " | ".join(_escape_table_text(item) for item in row) + " |")
    if not rows:
        lines.append("| " + " | ".join("—" for _ in headers) + " |")
    return lines


def _count_by(records: list[dict[str, Any]], field: str) -> list[tuple[str, int]]:
    counts: dict[str, int] = {}
    for record in records:
        key = str(record.get(field, "unspecified"))
        counts[key] = counts.get(key, 0) + 1
    return sorted(counts.items())


def _clause_section_rows(clauses: list[dict[str, Any]]) -> list[tuple[str, str, int, str]]:
    grouped: dict[tuple[str, str], dict[str, int]] = defaultdict(dict)
    for clause in clauses:
        key = (str(clause.get("source_id", "unspecified")), str(clause.get("section", "unspecified")))
        disposition = str(clause.get("disposition", "unspecified"))
        counts = grouped[key]
        counts[disposition] = counts.get(disposition, 0) + 1

    def section_key(value: str) -> tuple[int, ...] | tuple[str]:
        parts = value.split(".")
        if all(part.isdigit() for part in parts):
            return tuple(int(part) for part in parts)
        return (value,)

    rows = []
    for (source_id, section), counts in sorted(
        grouped.items(), key=lambda item: (item[0][0], section_key(item[0][1]))
    ):
        disposition_text = ", ".join(f"{name}: {count}" for name, count in sorted(counts.items()))
        rows.append((source_id, section, sum(counts.values()), disposition_text))
    return rows


def _source_reference(record: dict[str, Any]) -> str:
    references = record.get("source_refs") or []
    return ", ".join(f"{item.get('source_id', '?')} §{item.get('section', '?')}" for item in references)


def render_report(catalog: dict[str, Any]) -> str:
    """Render a stable report; all catalog strings are inert escaped text."""
    lines = [
        "# KMIP 2.1 inventory coverage",
        "",
        "This report records inventory coverage and evidence state. It does not claim profile conformance or certification.",
        "",
    ]

    sources = sorted(catalog.get("sources", []), key=lambda row: row.get("source_id", ""))
    lines.extend(["## Source documents", ""])
    lines.extend(_table(
        ("Source ID", "Title", "Authority", "SHA-256"),
        [(row.get("source_id"), row.get("title"), row.get("authority_class"), row.get("sha256")) for row in sources],
    ))
    lines.append("")

    elements = catalog.get("elements", [])
    requirements = catalog.get("requirements", [])
    operations = [row for row in elements if row.get("kind") == "operation"]
    count_rows: list[tuple[Any, ...]] = [
        ("Sources", len(sources)),
        ("Source clauses", len(catalog.get("source_clauses", []))),
        ("Protocol elements", len(elements)),
        ("Client-to-server operations", sum(row.get("direction") == "client_to_server" for row in operations)),
        ("Server-to-client operations", sum(row.get("direction") == "server_to_client" for row in operations)),
        ("Tag ranges", len(catalog.get("tag_ranges", []))),
        ("Normative requirements", len(requirements)),
        ("Profiles", len(catalog.get("profiles", []))),
        ("Test cases", len(catalog.get("test_cases", []))),
        ("Open discrepancies", sum(row.get("state") == "open" for row in catalog.get("discrepancies", []))),
        ("Project policies", len(catalog.get("policies", []))),
    ]
    lines.extend(["## Count reconciliation", ""])
    lines.extend(_table(("Record class", "Count"), count_rows))
    lines.extend(["", "### Elements by kind", ""])
    lines.extend(_table(("Kind", "Count"), _count_by(elements, "kind")))
    lines.extend(["", "### Requirements by strength and scope", ""])
    lines.extend(_table(("Dimension", "Value", "Count"), [
        *(("Strength", key, value) for key, value in _count_by(requirements, "normative_strength")),
        *(("Scope", key, value) for key, value in _count_by(requirements, "scope_state")),
        *(("Direction", key, value) for key, value in _count_by(requirements, "direction")),
    ]))
    lines.append("")

    lines.extend(["## Source clause dispositions", ""])
    lines.extend(_table(("Disposition", "Count"), _count_by(catalog.get("source_clauses", []), "disposition")))
    lines.append("")
    lines.extend([
        "## Source clause review by section",
        "",
        "Every row summarizes audited candidate locators by their pinned source section. The immutable-source audit separately requires exact candidate and clause-ledger locator equality.",
        "",
    ])
    lines.extend(_table(
        ("Source", "Section", "Candidates", "Dispositions"),
        _clause_section_rows(catalog.get("source_clauses", [])),
    ))
    lines.append("")

    unassigned = [
        row for row in requirements
        if not row.get("feature_spec") or not row.get("implementation_refs") or not row.get("verification_refs")
    ]
    unassigned.sort(key=lambda row: row.get("requirement_id", ""))
    lines.extend(["## Unassigned requirements", ""])
    lines.extend(_table(
        ("Requirement", "Strength", "Scope", "Source"),
        [(row.get("requirement_id"), row.get("normative_strength"), row.get("scope_state"), _source_reference(row)) for row in unassigned],
    ))
    lines.append("")

    requirements_without_tests = sorted(
        (row for row in requirements if not row.get("test_case_ids")),
        key=lambda row: row.get("requirement_id", ""),
    )
    lines.extend(["## Requirements without official test-case links", ""])
    lines.extend(_table(
        ("Requirement", "Evidence gap", "Source"),
        [(
            row.get("requirement_id"),
            row.get("review_note") or "No requirement-level official test evidence is linked.",
            _source_reference(row),
        ) for row in requirements_without_tests],
    ))
    lines.append("")

    negative_requirements = sorted(
        (row for row in requirements if row.get("negative_verification_required") is True),
        key=lambda row: row.get("requirement_id", ""),
    )
    lines.extend(["## Requirements needing negative verification", ""])
    lines.extend(_table(
        ("Requirement", "Strength", "Negative verification", "Status", "Source"),
        [(
            row.get("requirement_id"), row.get("normative_strength"), "required",
            row.get("status"), _source_reference(row),
        ) for row in negative_requirements],
    ))
    lines.append("")

    unassigned_elements = [
        row for row in elements
        if not row.get("feature_spec") or not row.get("implementation_refs") or not row.get("verification_refs")
    ]
    unassigned_elements.sort(key=lambda row: row.get("element_id", ""))
    lines.extend(["## Unassigned protocol elements", ""])
    lines.extend(_table(
        ("Element", "Kind", "Direction", "Scope", "Source"),
        [(
            row.get("element_id"), row.get("kind"), row.get("direction"), row.get("scope_state"),
            _source_reference(row),
        ) for row in unassigned_elements],
    ))
    lines.append("")

    profiles = sorted(catalog.get("profiles", []), key=lambda row: row.get("profile_id", ""))
    lines.extend(["## Profile states", ""])
    lines.extend(_table(
        ("Profile", "Name", "Applicability", "Claim state", "Source"),
        [(row.get("profile_id"), row.get("name"), row.get("applicability"), row.get("claim_state"), _source_reference(row)) for row in profiles],
    ))
    lines.append("")
    lines.extend(["### Profiles by applicability and claim state", ""])
    lines.extend(_table(("Dimension", "Value", "Count"), [
        *(("Applicability", key, value) for key, value in _count_by(profiles, "applicability")),
        *(("Claim state", key, value) for key, value in _count_by(profiles, "claim_state")),
    ]))
    lines.append("")

    test_cases = sorted(catalog.get("test_cases", []), key=lambda row: (row.get("source_id", ""), row.get("official_case_id", ""), row.get("test_id", "")))
    lines.extend(["## Test fixture availability", ""])
    lines.extend(_table(
        ("Test", "Official ID", "Fixture status", "Local fixture"),
        [(row.get("test_id"), row.get("official_case_id"), row.get("fixture_availability"), row.get("fixture_path") or "—") for row in test_cases],
    ))
    lines.append("")
    lines.extend(["### Test evidence by fixture availability", ""])
    lines.extend(_table(("Fixture state", "Count"), _count_by(test_cases, "fixture_availability")))
    lines.append("")

    discrepancies = sorted(catalog.get("discrepancies", []), key=lambda row: row.get("discrepancy_id", ""))
    lines.extend(["## Open discrepancies", ""])
    lines.extend(_table(
        ("Discrepancy", "State", "Implementation gate", "Affected records", "Summary", "Source"),
        [(
            row.get("discrepancy_id"), row.get("state"),
            "blocked for affected records"
            if row.get("state") == "open" and any(
                row.get(field) for field in (
                    "affected_requirement_ids", "affected_element_ids", "affected_profile_ids", "affected_policy_ids",
                )
            )
            else "review before dependent implementation",
            ", ".join(
                f"{len(row.get(field) or [])} {label}"
                for field, label in (
                    ("affected_requirement_ids", "requirements"),
                    ("affected_element_ids", "elements"),
                    ("affected_profile_ids", "profiles"),
                    ("affected_policy_ids", "policies"),
                )
                if row.get(field)
            ) or "none linked",
            row.get("summary"), _source_reference(row),
        ) for row in discrepancies],
    ))
    lines.append("")

    policies = sorted(catalog.get("policies", []), key=lambda row: row.get("policy_id", ""))
    lines.extend(["## Project policies", ""])
    lines.extend(_table(
        ("Policy", "Provenance", "Summary"),
        [(row.get("policy_id"), row.get("provenance"), row.get("summary")) for row in policies],
    ))
    lines.append("")

    return "\n".join(lines)


def write_report(catalog: dict[str, Any], path: Path, *, check: bool, repo_root: Path | None = None) -> bool:
    """Check byte equality or write the deterministic UTF-8/LF report."""
    expected = render_report(catalog).encode("utf-8")
    root = (repo_root or path.parent).resolve(strict=True)
    try:
        relative = path.relative_to(root).as_posix()
    except ValueError as error:
        raise PathSecurityError("coverage report path is outside the repository") from error
    target = confined_path(root, relative, allow_missing_leaf=True)
    try:
        current = safe_read_bytes(root, relative, max_bytes=16 * 1024 * 1024) if target.exists() else None
    except PathSecurityError:
        raise
    except OSError as error:
        raise PathSecurityError("coverage report could not be read safely") from error
    if check:
        return current == expected
    atomic_write_bytes(root, relative, expected)
    return True


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="write the generated report")
    mode.add_argument("--check", action="store_true", help="fail if the checked-in report is stale")
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument(
        "--structural-only",
        action="store_true",
        help="render an incomplete catalog for authoring and fixture checks",
    )
    arguments = parser.parse_args(argv)
    try:
        repository = arguments.repo_root.resolve(strict=True)
        code_root = Path(__file__).resolve().parents[2]
        if str(code_root) not in sys.path:
            sys.path.insert(0, str(code_root))
        from tools.normative_catalog.validate import load_validated_catalog

        catalog_path = repository / "specification" / "catalog" / "kmip-2.1.json"
        report_path = repository / REPORT_PATH
        raw = safe_read_bytes(repository, "specification/catalog/kmip-2.1.json", max_bytes=16 * 1024 * 1024)
        catalog = load_validated_catalog(raw, repository, require_complete=not arguments.structural_only)
        valid = write_report(catalog, report_path, check=arguments.check, repo_root=repository)
    except (OSError, ValueError, PathSecurityError) as error:
        print(f"coverage report failed: {error}", file=sys.stderr)
        return 1
    if not valid:
        print("coverage report is stale; run report.py --write", file=sys.stderr)
        return 1
    print(f"Coverage report {'verified' if arguments.check else 'written'}: {report_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
