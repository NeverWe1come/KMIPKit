"""Fail-closed validation for Cargo dependency policy metadata and exceptions."""

from __future__ import annotations

import argparse
import copy
import json
import os
import re
import sys
import tomllib
from datetime import date, timedelta
from pathlib import Path
from pathlib import PurePosixPath
from typing import Any, Iterable
from urllib.parse import SplitResult, parse_qsl, urlsplit


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
CRATES_IO_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"
ADR_0005_BANNED_PACKAGES = frozenset({"native-tls", "openssl", "openssl-sys"})
EXCEPTION_ID_PATTERN = re.compile(r"KMIPKIT-0011-EX-[0-9]{3,}")
PACKAGE_PATTERN = re.compile(r"[A-Za-z0-9][A-Za-z0-9_-]*")
VERSION_PATTERN = re.compile(
    r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)"
    r"(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?"
)
HEX_REVISION_PATTERN = re.compile(r"[0-9a-fA-F]{40}")
ADVISORY_ID_PATTERN = re.compile(r"(?:RUSTSEC-[0-9]{4}-[0-9]{4}|CVE-[0-9]{4}-[0-9]{4,7}|GHSA-[23456789cfghjmpqrvwx]{4}-[23456789cfghjmpqrvwx]{4}-[23456789cfghjmpqrvwx]{4})")
DIAGNOSTIC_CODE_PATTERN = re.compile(r"[a-z][a-z0-9_-]{0,63}")
SPDX_TOKEN_PATTERN = re.compile(r"\s*(\(|\)|[A-Za-z0-9][A-Za-z0-9.+-]{0,127})")
MAX_DIAGNOSTIC_LICENSE_LENGTH = 512
LICENSE_DIAGNOSTIC_CODES = frozenset(
    {
        "accepted",
        "rejected",
        "unlicensed",
        "skipped-private-workspace-crate",
        "license-not-encountered",
        "license-exception-not-encountered",
        "missing-clarification-file",
        "parse-error",
        "empty-license-field",
        "no-license-field",
        "gather-failure",
    }
)
CARGO_DENY_POLICY_CODES = frozenset(
    {
        "vulnerability",
        "notice",
        "unmaintained",
        "unsound",
        "yanked",
        "banned",
        "not-allowed",
        "duplicate",
        "wildcard",
        "git-source-underspecified",
        "source-not-allowed",
    }
)
BASELINE_FINDING_KINDS = {
    "vulnerability": "advisory",
    "unsound": "advisory",
    "unmaintained": "advisory",
    "yanked": "yanked",
    "rejected": "license",
    "unlicensed": "license",
    "no-license-field": "license",
    "empty-license-field": "license",
    "parse-error": "license",
    "gather-failure": "license",
    "missing-clarification-file": "license",
    "duplicate": "duplicate",
    "source-not-allowed": "source",
    "git-source-underspecified": "source",
    "banned": "ban",
    "wildcard": "wildcard",
}
BASELINE_FINDING_SECTIONS = {
    "advisory": "advisories",
    "yanked": "advisories",
    "ban": "bans",
    "duplicate": "bans",
    "wildcard": "bans",
    "license": "licenses",
    "source": "sources",
}
CARGO_DENY_SUMMARY_SEVERITIES = ("errors", "warnings", "notes", "helps")
# cargo-deny aggregates `helps` without emitting matching diagnostic records.
CARGO_DENY_DIAGNOSTIC_SEVERITIES = ("errors", "warnings", "notes")
CARGO_DENY_CHECK_EXIT_BITS = {
    "advisories": 1,
    "bans": 2,
    "licenses": 4,
    "sources": 8,
}
LOCAL_CARGO_DENY_EXCEPTION_FILES = (
    Path("deny.exceptions.toml"),
    Path(".deny.exceptions.toml"),
    Path(".cargo") / "deny.exceptions.toml",
)
SECRET_QUERY_PATTERN = re.compile(
    r"(?:token|secret|pass(?:word|wd)?|credential|authorization|signature|(?:api|access|private|client)[_-]?key)",
    re.I,
)


class PolicyError(ValueError):
    """A dependency policy input is malformed, unsafe, or inconsistent."""


def _redact_diagnostic_source(source: Any) -> str:
    """Return a safe source reference that never includes URL paths or credentials."""
    if source is None:
        return "path:local"
    if not isinstance(source, str) or len(source) > 4096:
        return "source:unavailable"
    prefix, separator, url = source.partition("+")
    if not separator or prefix not in {"registry", "git"}:
        return "source:unavailable"
    try:
        parsed = urlsplit(url)
        hostname = parsed.hostname
        port = parsed.port
    except ValueError:
        return "source:unavailable"
    if parsed.scheme != "https" or not hostname or not re.fullmatch(r"[A-Za-z0-9.-]{1,253}", hostname):
        return "source:unavailable"
    host = hostname.lower()
    if port is not None:
        if not 1 <= port <= 65535:
            return "source:unavailable"
        host += f":{port}"
    return f"{prefix}+https://{host}/<redacted>"


def _diagnostic_packages(fields: dict[str, Any]) -> list[tuple[str, str]]:
    """Read package coordinates only from cargo-deny's structured inclusion graphs."""
    found: set[tuple[str, str]] = set()
    visited = 0

    def visit(value: Any, depth: int = 0) -> None:
        nonlocal visited
        visited += 1
        if depth > 64 or visited > 100_000:
            return
        if isinstance(value, dict):
            crate = value.get("Krate")
            if isinstance(crate, dict):
                name = crate.get("name")
                version = crate.get("version")
                if (
                    isinstance(name, str)
                    and PACKAGE_PATTERN.fullmatch(name)
                    and isinstance(version, str)
                    and VERSION_PATTERN.fullmatch(version)
                ):
                    found.add((name, version))
            for child in value.values():
                visit(child, depth + 1)
        elif isinstance(value, list):
            for child in value:
                visit(child, depth + 1)

    visit(fields.get("graphs"))
    return sorted(found)


def _top_level_diagnostic_packages(fields: dict[str, Any]) -> list[tuple[str, str]]:
    """Return only affected Krate nodes, excluding dependency graph parents."""
    graphs = fields.get("graphs")
    if not isinstance(graphs, list):
        raise PolicyError("cargo-deny finding graphs are malformed")
    found: set[tuple[str, str]] = set()
    for graph in graphs:
        crate = graph.get("Krate") if isinstance(graph, dict) else None
        if not isinstance(crate, dict):
            raise PolicyError("cargo-deny finding package is malformed")
        name = crate.get("name")
        version = crate.get("version")
        if (
            not isinstance(name, str)
            or not PACKAGE_PATTERN.fullmatch(name)
            or not isinstance(version, str)
            or not VERSION_PATTERN.fullmatch(version)
        ):
            raise PolicyError("cargo-deny finding package coordinates are malformed")
        found.add((name, version))
    if not found:
        raise PolicyError("cargo-deny finding does not identify an affected package")
    return sorted(found)


def _validated_spdx_expression(value: Any) -> str | None:
    """Keep only bounded SPDX expressions with no custom or document references."""
    if (
        not isinstance(value, str)
        or not value
        or len(value) > MAX_DIAGNOSTIC_LICENSE_LENGTH
        or any(ord(character) < 32 for character in value)
    ):
        return None
    tokens: list[str] = []
    position = 0
    while position < len(value):
        match = SPDX_TOKEN_PATTERN.match(value, position)
        if match is None:
            return None
        token = match.group(1)
        tokens.append(token)
        position = match.end()
    if not tokens:
        return None

    index = 0

    def parse_primary() -> bool:
        nonlocal index
        if index >= len(tokens):
            return False
        token = tokens[index]
        parenthesized = token == "("
        if token == "(":
            index += 1
            if not parse_or_expression() or index >= len(tokens) or tokens[index] != ")":
                return False
            index += 1
        elif token not in {"AND", "OR", "WITH", ")"}:
            if token.lower().startswith(("licenseref-", "documentref-")):
                return False
            index += 1
        else:
            return False

        if index < len(tokens) and tokens[index] == "WITH":
            if parenthesized:
                return False
            index += 1
            if index >= len(tokens):
                return False
            exception = tokens[index]
            if exception in {"AND", "OR", "WITH", "(", ")"} or exception.lower().startswith(
                ("licenseref-", "documentref-")
            ):
                return False
            index += 1
        return True

    def parse_and_expression() -> bool:
        nonlocal index
        if not parse_primary():
            return False
        while index < len(tokens) and tokens[index] == "AND":
            index += 1
            if not parse_primary():
                return False
        return True

    def parse_or_expression() -> bool:
        nonlocal index
        if not parse_and_expression():
            return False
        while index < len(tokens) and tokens[index] == "OR":
            index += 1
            if not parse_and_expression():
                return False
        return True

    if not parse_or_expression() or index != len(tokens):
        return None
    return value.strip()


def _diagnostic_license_evidence(package: dict[str, Any]) -> str:
    """Read a safe license expression from one Cargo metadata package record."""
    if "license" not in package:
        return "unavailable"
    raw_license = package["license"]
    if raw_license is None:
        return "missing"
    return _validated_spdx_expression(raw_license) or "unavailable"


def _diagnostic_licenses(metadata_by_workspace: Any) -> dict[tuple[str, str], str]:
    licenses: dict[tuple[str, str], set[str]] = {}
    if not isinstance(metadata_by_workspace, dict):
        return {}
    for workspace in ("root", "fuzz"):
        metadata = metadata_by_workspace.get(workspace)
        packages = metadata.get("packages") if isinstance(metadata, dict) else None
        if not isinstance(packages, list):
            continue
        for package in packages:
            if not isinstance(package, dict):
                continue
            name = package.get("name")
            version = package.get("version")
            if (
                isinstance(name, str)
                and PACKAGE_PATTERN.fullmatch(name)
                and isinstance(version, str)
                and VERSION_PATTERN.fullmatch(version)
            ):
                licenses.setdefault((name, version), set()).add(
                    _diagnostic_license_evidence(package)
                )
    return {
        key: next(iter(values)) if len(values) == 1 else "ambiguous"
        for key, values in licenses.items()
    }


def _cargo_deny_error_exit_bitmask(error_counts: dict[str, int]) -> int:
    """Map failed policy sections to cargo-deny's check exit bitset."""
    exit_bitmask = 0
    for check, bit in CARGO_DENY_CHECK_EXIT_BITS.items():
        if error_counts.get(check, 0) > 0:
            exit_bitmask |= bit
    return exit_bitmask


def parse_cargo_deny_findings(
    raw_output: str,
    metadata_by_workspace: Any,
    workspace_name: str,
    exit_code: int,
) -> list[dict[str, str]]:
    """Extract exact policy findings from one complete cargo-deny JSON scan."""
    if not isinstance(raw_output, str) or len(raw_output) > 16 * 1024 * 1024:
        raise PolicyError("cargo-deny baseline output is malformed or exceeds the report limit")
    if workspace_name not in {"root", "fuzz"}:
        raise PolicyError("cargo-deny baseline workspace is unsupported")
    if isinstance(exit_code, bool) or not isinstance(exit_code, int) or not 0 <= exit_code <= 15:
        raise PolicyError("cargo-deny baseline exit status is malformed")
    if not isinstance(metadata_by_workspace, dict):
        raise PolicyError("cargo metadata for cargo-deny findings is malformed")
    workspace_metadata = metadata_by_workspace.get(workspace_name)
    packages = workspace_metadata.get("packages") if isinstance(workspace_metadata, dict) else None
    if not isinstance(packages, list):
        raise PolicyError("cargo metadata omits the scanned workspace packages")

    source_by_coordinate: dict[tuple[str, str], set[str | None]] = {}
    license_by_coordinate = _diagnostic_licenses({workspace_name: workspace_metadata})
    for package_item in packages:
        if not isinstance(package_item, dict):
            raise PolicyError("cargo metadata contains an invalid package")
        name = package_item.get("name")
        version = package_item.get("version")
        source = package_item.get("source")
        if (
            not isinstance(name, str)
            or not PACKAGE_PATTERN.fullmatch(name)
            or not isinstance(version, str)
            or not VERSION_PATTERN.fullmatch(version)
            or (source is not None and not isinstance(source, str))
        ):
            raise PolicyError("cargo metadata package coordinates are malformed")
        source_by_coordinate.setdefault((name, version), set()).add(source)

    diagnostics: list[tuple[str, dict[str, Any]]] = []
    summary: dict[str, dict[str, int]] | None = None
    for line in raw_output.splitlines():
        if not line.strip():
            continue
        try:
            item = json.loads(line)
        except (json.JSONDecodeError, RecursionError, TypeError):
            raise PolicyError("cargo-deny baseline output contains malformed JSON") from None
        if not isinstance(item, dict):
            raise PolicyError("cargo-deny baseline output contains an invalid record")
        record_type = item.get("type")
        fields = item.get("fields")
        if record_type == "diagnostic":
            if not isinstance(fields, dict):
                raise PolicyError("cargo-deny baseline diagnostic is malformed")
            diagnostics.append(("diagnostic", fields))
        elif record_type == "summary":
            if summary is not None or not isinstance(fields, dict):
                raise PolicyError("cargo-deny baseline summary is malformed or duplicated")
            summary = {}
            for check in ("advisories", "bans", "licenses", "sources"):
                check_summary = fields.get(check)
                if not isinstance(check_summary, dict):
                    raise PolicyError("cargo-deny baseline summary omits a policy check")
                counts: dict[str, int] = {}
                for severity in CARGO_DENY_SUMMARY_SEVERITIES:
                    count = check_summary.get(severity)
                    if isinstance(count, bool) or not isinstance(count, int) or count < 0:
                        raise PolicyError("cargo-deny baseline summary counts are malformed")
                    counts[severity] = count
                summary[check] = counts
        else:
            raise PolicyError("cargo-deny baseline output contains an unsupported record")
    if summary is None:
        raise PolicyError("cargo-deny baseline output is missing its completion summary")

    observed_counts = {
        check: dict.fromkeys(CARGO_DENY_DIAGNOSTIC_SEVERITIES, 0)
        for check in ("advisories", "bans", "licenses", "sources")
    }
    findings: set[tuple[str, str, str, str | None, str | None, str | None]] = set()
    for _, fields in diagnostics:
        code = fields.get("code")
        severity = fields.get("severity")
        if not isinstance(code, str) or severity not in {"error", "warning", "note", "help", "bug"}:
            raise PolicyError("cargo-deny baseline diagnostic classification is malformed")
        if code == "license-not-encountered" and severity == "warning":
            observed_counts["licenses"]["warnings"] += 1
            continue
        kind = BASELINE_FINDING_KINDS.get(code)
        section = BASELINE_FINDING_SECTIONS.get(kind)
        if section is None or severity not in {"error", "warning"}:
            raise PolicyError("cargo-deny baseline contains an unsupported policy diagnostic")
        observed_counts[section]["errors" if severity == "error" else "warnings"] += 1
        coordinates = _top_level_diagnostic_packages(fields)
        advisory_id: str | None = None
        if kind == "advisory":
            advisory = fields.get("advisory")
            raw_id = advisory.get("id") if isinstance(advisory, dict) else None
            if not isinstance(raw_id, str) or not ADVISORY_ID_PATTERN.fullmatch(raw_id):
                raise PolicyError("cargo-deny advisory finding has no valid structured identifier")
            advisory_id = raw_id
            advisory_package = advisory.get("package") if isinstance(advisory, dict) else None
            if advisory_package is not None and (
                not isinstance(advisory_package, str)
                or any(package_name != advisory_package for package_name, _ in coordinates)
            ):
                raise PolicyError("cargo-deny advisory package does not match its affected package")
        for package_name, version in coordinates:
            sources = source_by_coordinate.get((package_name, version))
            if not sources or len(sources) != 1:
                raise PolicyError("cargo-deny finding has missing or ambiguous metadata source")
            source = next(iter(sources))
            if source is not None:
                _validate_source(source, require_immutable_git=False)
            license_expression = None
            if kind == "license":
                license_expression = license_by_coordinate.get((package_name, version), "unavailable")
            findings.add((kind, package_name, version, source, advisory_id, license_expression))

    for check, severities in observed_counts.items():
        for severity in CARGO_DENY_DIAGNOSTIC_SEVERITIES:
            count = severities[severity]
            if summary[check][severity] != count:
                raise PolicyError("cargo-deny baseline summary does not match its diagnostics")
    result = [
        {
            "kind": kind,
            "package": package_name,
            "version": version,
            **({"source": source} if source is not None else {}),
            **({"advisory_id": advisory_id} if advisory_id is not None else {}),
            **({"license_expression": license_expression} if license_expression is not None else {}),
        }
        for kind, package_name, version, source, advisory_id, license_expression in sorted(findings)
    ]
    expected_exit_code = _cargo_deny_error_exit_bitmask(
        {check: counts["errors"] for check, counts in observed_counts.items()}
    )
    if exit_code != expected_exit_code:
        raise PolicyError("cargo-deny baseline exit status contradicts error diagnostics")
    return result


def _diagnostic_sources(metadata_by_workspace: Any) -> dict[tuple[str, str], str]:
    sources: dict[tuple[str, str], set[str]] = {}
    if not isinstance(metadata_by_workspace, dict):
        return {}
    for workspace in ("root", "fuzz"):
        metadata = metadata_by_workspace.get(workspace)
        packages = metadata.get("packages") if isinstance(metadata, dict) else None
        if not isinstance(packages, list):
            continue
        for package in packages:
            if not isinstance(package, dict):
                continue
            name = package.get("name")
            version = package.get("version")
            if (
                isinstance(name, str)
                and PACKAGE_PATTERN.fullmatch(name)
                and isinstance(version, str)
                and VERSION_PATTERN.fullmatch(version)
            ):
                sources.setdefault((name, version), set()).add(
                    _redact_diagnostic_source(package.get("source"))
                )
    return {
        key: next(iter(values)) if len(values) == 1 else "source:ambiguous"
        for key, values in sources.items()
    }


def format_cargo_deny_diagnostics(raw_output: str, metadata_by_workspace: Any) -> str:
    """Format cargo-deny JSON diagnostics using only validated, redacted fields."""
    if not isinstance(raw_output, str) or len(raw_output) > 16 * 1024 * 1024:
        return "cargo-deny diagnostics unavailable (output was malformed or exceeded the report limit)."
    sources = _diagnostic_sources(metadata_by_workspace)
    licenses = _diagnostic_licenses(metadata_by_workspace)
    reports: list[str] = []
    recognized = False
    for line in raw_output.splitlines():
        if not line.strip():
            continue
        try:
            item = json.loads(line)
        except (json.JSONDecodeError, RecursionError, TypeError):
            continue
        if not isinstance(item, dict) or item.get("type") != "diagnostic":
            continue
        fields = item.get("fields")
        if not isinstance(fields, dict):
            continue
        recognized = True
        raw_code = fields.get("code")
        if (
            isinstance(raw_code, str)
            and DIAGNOSTIC_CODE_PATTERN.fullmatch(raw_code)
            and raw_code in LICENSE_DIAGNOSTIC_CODES | CARGO_DENY_POLICY_CODES
        ):
            rule = (
                f"license-{raw_code}"
                if raw_code in LICENSE_DIAGNOSTIC_CODES and not raw_code.startswith("license-")
                else raw_code
            )
        else:
            rule = "policy-check"
        severity_value = fields.get("severity")
        severity = (
            severity_value
            if isinstance(severity_value, str)
            and severity_value in {"error", "warning", "note", "help", "bug"}
            else "finding"
        )
        packages = _diagnostic_packages(fields)
        if not packages:
            packages = [("package-unavailable", "version-unavailable")]
        evidence = ""
        if rule in {"vulnerability", "unmaintained", "unsound"}:
            advisory = fields.get("advisory")
            candidates = []
            if isinstance(advisory, dict):
                candidates.extend((advisory.get("id"), advisory.get("aliases")))
            for candidate in candidates:
                values = candidate if isinstance(candidate, list) else [candidate]
                valid_ids = [
                    value for value in values
                    if isinstance(value, str) and ADVISORY_ID_PATTERN.fullmatch(value)
                ]
                if valid_ids:
                    evidence = f" advisory={','.join(sorted(set(valid_ids)))}"
                    break
        for name, version in packages:
            source = sources.get((name, version), "source:unavailable")
            license_detail = (
                f" license={licenses.get((name, version), 'unavailable')}"
                if rule.startswith("license-")
                else ""
            )
            reports.append(
                f"{severity} {name}@{version} source={source} rule={rule}{license_detail}{evidence}"
            )
    if not recognized:
        return "cargo-deny diagnostics unavailable (no recognized structured findings)."
    return "\n".join(reports)


def parse_rustc_host(verbose_output: str) -> str:
    """Return the unique target triple reported by ``rustc -vV``."""
    if not isinstance(verbose_output, str):
        raise PolicyError("rustc -vV output is unavailable or malformed")
    hosts = [line[5:].strip() for line in verbose_output.splitlines() if line.startswith("host:")]
    if len(hosts) != 1 or not hosts[0] or not re.fullmatch(r"[A-Za-z0-9_]+(?:-[A-Za-z0-9_]+)+", hosts[0]):
        raise PolicyError("rustc -vV must report exactly one valid host triple")
    return hosts[0]


def validate_host_triple(observed: str, reviewed_hosts: Iterable[str]) -> None:
    """Fail unless the observed runner host is in the reviewed host inventory."""
    try:
        allowed = set(reviewed_hosts)
    except TypeError:
        raise PolicyError("reviewed CI host inventory is malformed") from None
    if any(not isinstance(host, str) or not host for host in allowed):
        raise PolicyError("reviewed CI host inventory is malformed")
    if not isinstance(observed, str) or not observed or observed not in allowed:
        raise PolicyError("executing runner host is absent from the reviewed CI host set")


def _is_within(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
    except ValueError:
        return False
    return True


def _metadata_path(
    value: Any,
    lexical_checkout_root: Path,
    canonical_checkout_root: Path,
    label: str,
) -> tuple[Path, Path]:
    """Return lexical and canonical metadata paths, rejecting escapes at both layers."""
    if not isinstance(value, str) or not value.strip():
        raise PolicyError(f"{label} path is missing or malformed")
    raw_path = Path(value)
    if not raw_path.is_absolute():
        raise PolicyError(f"{label} path must be absolute")
    lexical_path = Path(os.path.abspath(os.fspath(raw_path)))
    lexical_root = Path(os.path.abspath(os.fspath(lexical_checkout_root)))
    if not _is_within(lexical_path, lexical_root):
        raise PolicyError(f"{label} path is outside the checkout")
    try:
        canonical_path = lexical_path.resolve(strict=True)
    except (OSError, RuntimeError):
        raise PolicyError(f"{label} path cannot be canonicalized") from None
    if not _is_within(canonical_path, canonical_checkout_root):
        raise PolicyError(f"{label} path resolves outside the checkout")
    return lexical_path, canonical_path


def validate_workspace_metadata(
    checkout_root: Path | str,
    metadata_by_workspace: dict[str, dict],
    exception_register: dict[str, Any] | None = None,
    *,
    today: date | None = None,
) -> list[dict[str, str]]:
    """Validate source-less Cargo packages against both workspaces' member manifests.

    Both metadata graphs are considered together because the root workspace owns
    ``kmipkit-ttlv`` while the isolated fuzz workspace consumes that same path.
    Paths must be lexically inside the checkout and remain inside after resolving
    symlinks; path packages must match a canonical root or fuzz member manifest.
    """
    root_input = Path(checkout_root)
    try:
        lexical_root = Path(os.path.abspath(os.fspath(root_input)))
        root = root_input.resolve(strict=True)
    except (OSError, RuntimeError, ValueError):
        raise PolicyError("checkout root cannot be canonicalized") from None
    if not root.is_dir():
        raise PolicyError("checkout root is not a directory")
    if not isinstance(metadata_by_workspace, dict) or set(metadata_by_workspace) != {"root", "fuzz"}:
        raise PolicyError("metadata must contain exactly the root and fuzz workspaces")

    expected_workspace_roots = {"root": root, "fuzz": root / "fuzz"}
    member_manifests: set[Path] = set()
    parsed_workspaces: dict[str, tuple[dict[str, dict], set[str]]] = {}

    for workspace_name in ("root", "fuzz"):
        metadata = metadata_by_workspace[workspace_name]
        if not isinstance(metadata, dict):
            raise PolicyError(f"{workspace_name} Cargo metadata is malformed")
        _, actual_workspace_root = _metadata_path(
            metadata.get("workspace_root"),
            lexical_root,
            root,
            f"{workspace_name} workspace root",
        )
        if actual_workspace_root != expected_workspace_roots[workspace_name]:
            raise PolicyError(f"{workspace_name} metadata identifies the wrong workspace root")

        packages = metadata.get("packages")
        workspace_members = metadata.get("workspace_members")
        if not isinstance(packages, list) or not isinstance(workspace_members, list):
            raise PolicyError(f"{workspace_name} Cargo metadata omits packages or workspace members")
        package_by_id: dict[str, dict] = {}
        for package_item in packages:
            if not isinstance(package_item, dict):
                raise PolicyError(f"{workspace_name} Cargo metadata contains an invalid package")
            package_id = package_item.get("id")
            if not isinstance(package_id, str) or not package_id or package_id in package_by_id:
                raise PolicyError(f"{workspace_name} Cargo metadata contains duplicate or invalid package IDs")
            if "source" not in package_item:
                raise PolicyError(f"{workspace_name} package source is missing")
            package_by_id[package_id] = package_item

        if any(not isinstance(member, str) or not member for member in workspace_members):
            raise PolicyError(f"{workspace_name} workspace member IDs are malformed")
        member_ids = set(workspace_members)
        if len(member_ids) != len(workspace_members) or not member_ids.issubset(package_by_id):
            raise PolicyError(f"{workspace_name} workspace member list is inconsistent")
        for member_id in member_ids:
            member = package_by_id[member_id]
            if member.get("source") is not None:
                raise PolicyError(f"{workspace_name} workspace member is not a local package")
            _, manifest = _metadata_path(
                member.get("manifest_path"),
                lexical_root,
                root,
                f"{workspace_name} member manifest",
            )
            if not manifest.is_file() or manifest.name != "Cargo.toml":
                raise PolicyError(f"{workspace_name} workspace member manifest is invalid")
            member_manifests.add(manifest)
        parsed_workspaces[workspace_name] = (package_by_id, member_ids)

    for workspace_name in ("root", "fuzz"):
        package_by_id, _ = parsed_workspaces[workspace_name]
        for package_item in package_by_id.values():
            if package_item.get("source") is not None:
                continue
            name = package_item.get("name")
            version = package_item.get("version")
            if not isinstance(name, str) or not PACKAGE_PATTERN.fullmatch(name):
                raise PolicyError(f"{workspace_name} local package name is malformed")
            if not isinstance(version, str) or not VERSION_PATTERN.fullmatch(version):
                raise PolicyError(f"{workspace_name} package {name} version is malformed")
            label = f"{workspace_name} package {name}"
            _, manifest = _metadata_path(
                package_item.get("manifest_path"), lexical_root, root, f"{label} manifest"
            )
            if not manifest.is_file() or manifest.name != "Cargo.toml":
                raise PolicyError(f"{label} manifest is invalid")
            if manifest not in member_manifests:
                raise PolicyError(f"{label} is not a canonical member of either workspace")
    return validate_metadata_sources(metadata_by_workspace, exception_register, today=today)


def _text(value: Any, field: str, *, maximum: int = 4096) -> str:
    if not isinstance(value, str) or not value.strip() or len(value) > maximum:
        raise PolicyError(f"exception {field} is missing or malformed")
    if any(ord(character) < 32 and character not in "\r\n\t" for character in value):
        raise PolicyError(f"exception {field} contains invalid control characters")
    return value.strip()


def _safe_urlsplit(value: str, label: str) -> SplitResult:
    """Parse an untrusted URL without exposing parser input in diagnostics."""
    try:
        return urlsplit(value)
    except ValueError:
        raise PolicyError(f"{label} is malformed") from None


def _url_contains_credentials(value: str) -> bool:
    candidate = value
    for prefix in ("git+", "registry+"):
        if candidate.startswith(prefix):
            candidate = candidate[len(prefix) :]
            break
    parsed = _safe_urlsplit(candidate, "dependency URL")
    if parsed.username is not None or parsed.password is not None:
        return True
    if any(SECRET_QUERY_PATTERN.search(key) for key, _ in parse_qsl(parsed.query, keep_blank_values=True)):
        return True
    return False


def _validate_source(value: Any, *, require_immutable_git: bool = False) -> str:
    source = _text(value, "source", maximum=2048)
    if _url_contains_credentials(source):
        raise PolicyError("exception source contains credentials or secret query values")
    if source.startswith("registry+"):
        parsed = _safe_urlsplit(source.removeprefix("registry+"), "exception source")
        if parsed.scheme != "https" or not parsed.hostname or parsed.query or parsed.fragment:
            raise PolicyError("exception registry source is not a canonical HTTPS source")
        if require_immutable_git:
            raise PolicyError("source exceptions must identify an immutable Git revision")
        return source
    if source.startswith("git+"):
        parsed = _safe_urlsplit(source.removeprefix("git+"), "exception source")
        query = parse_qsl(parsed.query, keep_blank_values=True)
        if (
            parsed.scheme != "https"
            or not parsed.hostname
            or parsed.username is not None
            or parsed.password is not None
            or len(query) != 1
            or query[0][0] != "rev"
            or not HEX_REVISION_PATTERN.fullmatch(query[0][1])
            or not HEX_REVISION_PATTERN.fullmatch(parsed.fragment)
            or query[0][1].lower() != parsed.fragment.lower()
        ):
            raise PolicyError("exception Git source must pin one immutable HTTPS revision")
        return source
    raise PolicyError("exception source must use a registry+ or immutable git+ source")


def _approval_reference(value: Any, field: str = "approval_ref") -> str:
    reference = _text(value, field, maximum=2048)
    if _url_contains_credentials(reference):
        raise PolicyError(f"exception {field} contains credentials or secret query values")
    if "://" in reference:
        parsed = _safe_urlsplit(reference, f"exception {field}")
        if parsed.scheme != "https" or parsed.hostname != "github.com" or parsed.username or parsed.password:
            raise PolicyError(f"exception {field} must reference durable project evidence")
        path = parsed.path.rstrip("/")
        project_prefix = "/NeverWe1come/KMIPKit/"
        if not path.startswith(project_prefix):
            raise PolicyError(f"exception {field} must reference this repository")
        durable_suffix = path[len(project_prefix) :]
        if not re.fullmatch(r"(?:pull|issues)/[1-9][0-9]*|commit/[0-9a-fA-F]{40}", durable_suffix):
            raise PolicyError(f"exception {field} must identify a review, issue, or commit")
    else:
        local = Path(reference)
        if local.is_absolute() or ".." in local.parts:
            raise PolicyError(f"exception {field} must be repository-relative evidence")
        try:
            resolved = (REPOSITORY_ROOT / local).resolve(strict=True)
        except (OSError, RuntimeError):
            raise PolicyError(f"exception {field} does not resolve to project evidence") from None
        if not _is_within(resolved, REPOSITORY_ROOT) or not resolved.is_file():
            raise PolicyError(f"exception {field} does not resolve to project evidence")
    return reference


def _validate_entry(entry: Any, today: date, *, check_current: bool = True) -> dict[str, Any]:
    if not isinstance(entry, dict):
        raise PolicyError("exception entry must be an object")
    kind = entry.get("kind")
    if not isinstance(kind, str) or kind not in {"advisory", "yanked", "license", "source", "duplicate"}:
        raise PolicyError("exception kind must be advisory, yanked, license, source, or duplicate")

    required = {
        "id",
        "kind",
        "package",
        "version",
        "rationale",
        "mitigation",
        "owner",
        "reviewed_by",
        "reviewed_on",
        "expires_on",
        "approval_ref",
    }
    optional = {"source", "advisory_id", "license_evidence"}
    if kind == "advisory":
        required.add("advisory_id")
    elif kind == "source":
        required.add("source")
    elif kind == "license":
        required.add("license_evidence")
    allowed_fields = required | optional
    if not required.issubset(entry) or set(entry) - allowed_fields:
        raise PolicyError("exception entry has missing or unsupported fields")

    exception_id = _text(entry["id"], "id", maximum=64)
    if not EXCEPTION_ID_PATTERN.fullmatch(exception_id):
        raise PolicyError("exception id does not match the stable KMIPKIT-0011 format")
    package_name = _text(entry["package"], "package", maximum=255)
    if not PACKAGE_PATTERN.fullmatch(package_name):
        raise PolicyError(f"exception {exception_id} package scope must be one exact crate name")
    version = _text(entry["version"], "version", maximum=128)
    if not VERSION_PATTERN.fullmatch(version):
        raise PolicyError(f"exception {exception_id} version scope must be one exact semantic version")
    for field in ("rationale", "mitigation"):
        _text(entry[field], field)

    owner = _text(entry["owner"], "owner", maximum=256)
    reviewer = _text(entry["reviewed_by"], "reviewed_by", maximum=256)
    if owner.casefold() == reviewer.casefold():
        raise PolicyError(f"exception {exception_id} owner and reviewer must be different people or teams")
    if re.search(r"(?:\bbot\b|github[ -]?actions|dependabot|renovate\[bot\])", reviewer, re.I):
        raise PolicyError(f"exception {exception_id} reviewer must be a human")
    approval_ref = _approval_reference(entry["approval_ref"])

    try:
        reviewed_on = date.fromisoformat(_text(entry["reviewed_on"], "reviewed_on", maximum=10))
        expires_on = date.fromisoformat(_text(entry["expires_on"], "expires_on", maximum=10))
    except ValueError:
        raise PolicyError(f"exception {exception_id} dates must use valid ISO calendar dates") from None
    if reviewed_on.isoformat() != entry["reviewed_on"] or expires_on.isoformat() != entry["expires_on"]:
        raise PolicyError(f"exception {exception_id} dates must use YYYY-MM-DD format")
    if check_current and reviewed_on > today:
        raise PolicyError(f"exception {exception_id} review date is in the future")
    if check_current and expires_on <= today:
        raise PolicyError(f"exception {exception_id} is expired")
    if expires_on <= reviewed_on or expires_on > reviewed_on + timedelta(days=90):
        raise PolicyError(f"exception {exception_id} expiry must be within 90 days after review")

    normalized: dict[str, Any] = {
        "id": exception_id,
        "kind": kind,
        "package": package_name,
        "version": version,
        "rationale": entry["rationale"].strip(),
        "mitigation": entry["mitigation"].strip(),
        "owner": owner,
        "reviewed_by": reviewer,
        "reviewed_on": reviewed_on,
        "expires_on": expires_on,
        "approval_ref": approval_ref,
    }
    if "source" in entry:
        normalized["source"] = _validate_source(entry["source"], require_immutable_git=(kind == "source"))
    if kind == "advisory":
        advisory_id = _text(entry["advisory_id"], "advisory_id", maximum=64)
        if not re.fullmatch(r"RUSTSEC-[0-9]{4}-[0-9]{4,}", advisory_id):
            raise PolicyError(f"exception {exception_id} advisory identifier is malformed")
        normalized["advisory_id"] = advisory_id
    elif "advisory_id" in entry:
        raise PolicyError(f"exception {exception_id} has an advisory ID for a non-advisory rule")
    if kind == "source" and not entry["source"].startswith("git+"):
        raise PolicyError(f"exception {exception_id} source exception must be an immutable Git source")

    if kind == "license":
        evidence = entry["license_evidence"]
        evidence_fields = {
            "reviewed_by",
            "reference",
            "disposition",
            "expression",
            "license_files",
        }
        if not isinstance(evidence, dict) or set(evidence) != evidence_fields:
            raise PolicyError(f"exception {exception_id} requires complete human-reviewed license evidence")
        evidence_reviewer = _text(evidence.get("reviewed_by"), "license evidence reviewer", maximum=256)
        if evidence_reviewer.casefold() != reviewer.casefold() or re.search(
            r"(?:\bbot\b|github[ -]?actions|dependabot|renovate\[bot\])", evidence_reviewer, re.I
        ):
            raise PolicyError(f"exception {exception_id} license evidence must name its human reviewer")
        evidence_reference = _text(evidence.get("reference"), "license evidence reference", maximum=2048)
        if _url_contains_credentials(evidence_reference):
            raise PolicyError(f"exception {exception_id} license evidence contains credentials")
        if "://" in evidence_reference:
            _approval_reference(evidence_reference, "license evidence reference")
        elif not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._:-]{1,127}", evidence_reference):
            raise PolicyError(f"exception {exception_id} license evidence reference is malformed")
        disposition = _text(evidence.get("disposition"), "license disposition", maximum=2048)
        if disposition != "clarify":
            raise PolicyError(f"exception {exception_id} must record a cargo-deny license clarification")
        expression = _text(evidence.get("expression"), "license expression", maximum=2048)
        license_files = _license_files(evidence.get("license_files"), f"exception {exception_id} license evidence")
        normalized["license_evidence"] = {
            "reviewed_by": evidence_reviewer,
            "reference": evidence_reference,
            "disposition": disposition,
            "expression": expression,
            "license_files": license_files,
        }
    elif "license_evidence" in entry:
        raise PolicyError(f"exception {exception_id} includes license evidence for a non-license rule")
    return normalized


def _validate_register(register: Any, today: date, *, check_current: bool = True) -> list[dict[str, Any]]:
    if not isinstance(register, dict) or set(register) != {"schema_version", "exceptions"}:
        raise PolicyError("exception register must contain schema_version and exceptions")
    version = register.get("schema_version")
    if isinstance(version, bool) or version != 1:
        raise PolicyError("exception register schema_version must be 1")
    entries = register.get("exceptions")
    if not isinstance(entries, list):
        raise PolicyError("exception register exceptions must be an array")
    normalized = [_validate_entry(entry, today, check_current=check_current) for entry in entries]
    ids = [entry["id"] for entry in normalized]
    if len(set(ids)) != len(ids):
        raise PolicyError("exception register contains duplicate IDs")
    scopes = [
        (entry["kind"], entry["package"], entry["version"], entry.get("source"), entry.get("advisory_id"))
        for entry in normalized
    ]
    if len(set(scopes)) != len(scopes):
        raise PolicyError("exception register contains duplicate exact scopes")
    return normalized


def validate_metadata_sources(
    metadata_by_workspace: dict[str, dict],
    exception_register: dict[str, Any] | None = None,
    *,
    today: date | None = None,
) -> list[dict[str, str]]:
    """Return exact Git source findings and reject all other unreviewed sources.

    cargo-deny permits Git sources by repository URL, which is broader than a
    single immutable commit. The register and metadata are therefore matched on
    package, resolved version, and the complete revision-qualified Cargo source
    string. The crates.io registry is the only source allowed without an entry.
    """
    current_date = today or date.today()
    if not isinstance(current_date, date):
        raise PolicyError("current date for source validation is malformed")
    if not isinstance(metadata_by_workspace, dict) or set(metadata_by_workspace) != {"root", "fuzz"}:
        raise PolicyError("metadata must contain exactly the root and fuzz workspaces")
    register = (
        {"schema_version": 1, "exceptions": []}
        if exception_register is None
        else exception_register
    )
    entries = _validate_register(register, current_date)
    source_entries = [entry for entry in entries if entry["kind"] == "source"]
    source_scopes = {
        (entry["package"], entry["version"], entry["source"]): entry
        for entry in source_entries
    }
    observed_git_scopes: set[tuple[str, str, str]] = set()
    for workspace_name in ("root", "fuzz"):
        metadata = metadata_by_workspace.get(workspace_name)
        if not isinstance(metadata, dict) or not isinstance(metadata.get("packages"), list):
            raise PolicyError(f"{workspace_name} Cargo metadata is malformed for source validation")
        for package_item in metadata["packages"]:
            if not isinstance(package_item, dict):
                raise PolicyError(f"{workspace_name} Cargo metadata contains an invalid package")
            source = package_item.get("source")
            if source is None:
                continue
            name = package_item.get("name")
            version = package_item.get("version")
            if not isinstance(name, str) or not PACKAGE_PATTERN.fullmatch(name):
                raise PolicyError(f"{workspace_name} package name is malformed")
            if not isinstance(version, str) or not VERSION_PATTERN.fullmatch(version):
                raise PolicyError(f"{workspace_name} package {name} version is malformed")
            normalized_source = _validate_source(source)
            if normalized_source == CRATES_IO_SOURCE:
                continue
            if normalized_source.startswith("registry+"):
                raise PolicyError(f"{workspace_name} package {name} uses an unapproved registry source")
            if not normalized_source.startswith("git+"):
                raise PolicyError(f"{workspace_name} package {name} uses an unsupported dependency source")
            scope = (name, version, normalized_source)
            if scope not in source_scopes:
                raise PolicyError(f"{workspace_name} package {name}@{version} has no exact Git source exception")
            observed_git_scopes.add(scope)

    orphaned = set(source_scopes) - observed_git_scopes
    if orphaned:
        orphaned_entry = source_scopes[next(iter(orphaned))]
        raise PolicyError(f"source exception {orphaned_entry['id']} has no matching metadata package")
    return [
        {"kind": "source", "package": name, "version": version, "source": source}
        for name, version, source in sorted(observed_git_scopes)
    ]


def _finding_matches(entry: dict[str, Any], item: dict[str, Any]) -> bool:
    if item.get("kind") != entry["kind"]:
        return False
    if item.get("package") != entry["package"] or item.get("version") != entry["version"]:
        return False
    for field in ("source", "advisory_id"):
        if field in entry and item.get(field) != entry[field]:
            return False
    return True


def _format_exception_diagnostics(diagnostics: list[str]) -> str:
    """Keep every exact-exception finding visible in the policy report."""
    return "; ".join(diagnostics)


def _describe_exception_finding(finding_item: dict[str, Any]) -> str:
    """Describe one finding with only normalized, safely displayable evidence."""
    source = _redact_diagnostic_source(finding_item.get("source"))
    advisory = finding_item.get("advisory_id")
    advisory_detail = f" advisory={advisory}" if advisory is not None else ""
    license_expression = finding_item.get("license_expression")
    license_detail = (
        f" license={license_expression}"
        if finding_item["kind"] == "license" and license_expression is not None
        else ""
    )
    return (
        f"finding {finding_item['package']}@{finding_item['version']} "
        f"source={source} rule={finding_item['kind']}{license_detail}{advisory_detail}"
    )


def validate_exceptions(register: Any, findings: list[dict], *, today: date | None = None) -> list[str]:
    """Require a one-to-one exact match between current exceptions and findings."""
    current_date = today or date.today()
    if not isinstance(current_date, date):
        raise PolicyError("current date for exception validation is malformed")
    entries = _validate_register(register, current_date)
    if not isinstance(findings, list):
        raise PolicyError("dependency findings must be an array")
    normalized_findings: list[dict[str, Any]] = []
    for item in findings:
        if not isinstance(item, dict):
            raise PolicyError("dependency finding is malformed")
        kind = item.get("kind")
        package_name = item.get("package")
        version = item.get("version")
        if isinstance(kind, str) and kind in {"ban", "wildcard"}:
            if package_name in ADR_0005_BANNED_PACKAGES:
                raise PolicyError(f"ADR-0005 architecture ban for {package_name} cannot be excepted")
            raise PolicyError(f"{kind} dependency findings cannot be excepted")
        if not isinstance(kind, str) or kind not in {"advisory", "yanked", "license", "source", "duplicate"}:
            raise PolicyError("dependency finding rule is unsupported")
        if not isinstance(package_name, str) or not PACKAGE_PATTERN.fullmatch(package_name):
            raise PolicyError("dependency finding package is malformed")
        if not isinstance(version, str) or not VERSION_PATTERN.fullmatch(version):
            raise PolicyError(f"dependency finding for {package_name} has a malformed version")
        finding_copy = {"kind": kind, "package": package_name, "version": version}
        if "source" in item:
            finding_copy["source"] = _validate_source(item["source"], require_immutable_git=False)
        if "advisory_id" in item:
            advisory_id = _text(item["advisory_id"], "finding advisory ID", maximum=64)
            if not ADVISORY_ID_PATTERN.fullmatch(advisory_id):
                raise PolicyError("dependency finding advisory ID is malformed")
            finding_copy["advisory_id"] = advisory_id
        if "license_expression" in item:
            if kind != "license":
                raise PolicyError("license expression is only valid for a license finding")
            raw_license = item["license_expression"]
            if isinstance(raw_license, str) and raw_license in {"missing", "unavailable", "ambiguous"}:
                finding_copy["license_expression"] = raw_license
            else:
                license_expression = _validated_spdx_expression(raw_license)
                if license_expression is None:
                    raise PolicyError("dependency finding license expression is malformed")
                finding_copy["license_expression"] = license_expression
        normalized_findings.append(finding_copy)

    matched_entry_indexes: set[int] = set()
    exception_diagnostics: list[str] = []
    for finding_item in normalized_findings:
        matches = [
            (index, entry)
            for index, entry in enumerate(entries)
            if _finding_matches(entry, finding_item)
        ]
        if len(matches) != 1:
            match_status = (
                "has no exact registered exception"
                if not matches
                else "matches multiple registered exceptions"
            )
            exception_diagnostics.append(f"{_describe_exception_finding(finding_item)} {match_status}")
            continue
        index, entry = matches[0]
        if index in matched_entry_indexes:
            exception_diagnostics.append(
                f"exception {entry['id']} matches more than one "
                f"{_describe_exception_finding(finding_item)}"
            )
            continue
        matched_entry_indexes.add(index)
    for index, entry in enumerate(entries):
        if index not in matched_entry_indexes:
            exception_diagnostics.append(f"exception {entry['id']} has no matching current finding")
    if exception_diagnostics:
        details = _format_exception_diagnostics(exception_diagnostics)
        raise PolicyError(f"dependency exception validation failed: {details}")
    return sorted(entry["id"] for entry in entries)


def _git_config_source(source: str) -> str:
    parsed = _safe_urlsplit(source.removeprefix("git+"), "exception source")
    return f"{parsed.scheme}://{parsed.netloc}{parsed.path}"


def _license_files(value: Any, label: str) -> list[dict[str, Any]]:
    if not isinstance(value, list) or not value:
        raise PolicyError(f"{label} must include at least one reviewed license file")
    normalized: list[dict[str, Any]] = []
    seen_paths: set[str] = set()
    for item in value:
        if not isinstance(item, dict) or set(item) != {"path", "hash"}:
            raise PolicyError(f"{label} contains malformed license-file evidence")
        path_value = _text(item.get("path"), "license file path", maximum=512)
        path = PurePosixPath(path_value)
        if (
            path.is_absolute()
            or ".." in path.parts
            or path_value in {".", ""}
            or "\\" in path_value
            or ":" in path_value
            or path.as_posix() != path_value
        ):
            raise PolicyError(f"{label} license-file paths must be crate-relative")
        if path_value in seen_paths:
            raise PolicyError(f"{label} contains duplicate license-file paths")
        seen_paths.add(path_value)

        raw_hash = item.get("hash")
        if isinstance(raw_hash, bool):
            raise PolicyError(f"{label} license-file hash is malformed")
        if isinstance(raw_hash, int):
            hash_value = raw_hash
        elif isinstance(raw_hash, str) and re.fullmatch(r"0x[0-9a-fA-F]{1,16}", raw_hash):
            hash_value = int(raw_hash, 16)
        else:
            raise PolicyError(f"{label} license-file hash must be an exact hexadecimal cargo-deny hash")
        if hash_value < 0 or hash_value > 0xFFFFFFFFFFFFFFFF:
            raise PolicyError(f"{label} license-file hash is outside the cargo-deny hash range")
        normalized.append({"path": path_value, "hash": hash_value})
    return normalized


def validate_no_local_exception_files(manifests: Iterable[Path | str]) -> None:
    """Reject cargo-deny exception files auto-discovered beside either manifest."""
    try:
        manifest_paths = tuple(Path(manifest) for manifest in manifests)
    except (TypeError, ValueError):
        raise PolicyError("Cargo manifest inventory is malformed") from None
    if not manifest_paths:
        raise PolicyError("Cargo manifest inventory is empty")
    for manifest in manifest_paths:
        lexical_manifest = Path(os.path.abspath(os.fspath(manifest)))
        try:
            canonical_manifest = manifest.resolve(strict=True)
        except (OSError, RuntimeError, ValueError):
            raise PolicyError("Cargo manifest cannot be canonicalized") from None
        if not canonical_manifest.is_file():
            raise PolicyError("Cargo manifest is not a file")
        for start in {lexical_manifest.parent, canonical_manifest.parent}:
            for directory in (start, *start.parents):
                for relative_path in LOCAL_CARGO_DENY_EXCEPTION_FILES:
                    candidate = directory / relative_path
                    if candidate.is_file() or candidate.is_symlink():
                        raise PolicyError("local cargo-deny exception file is not permitted")


def _exception_free_config(config: dict[str, Any]) -> dict[str, Any]:
    """Copy all reviewed policy settings while removing every waiver surface."""
    if not isinstance(config, dict):
        raise PolicyError("cargo-deny configuration is malformed")
    baseline = copy.deepcopy(config)
    for section, field in (
        ("advisories", "ignore"),
        ("bans", "skip"),
        ("licenses", "clarify"),
        ("licenses", "exceptions"),
        ("sources", "allow-git"),
    ):
        section_value = baseline.setdefault(section, {})
        if not isinstance(section_value, dict):
            raise PolicyError("cargo-deny configuration is malformed")
        section_value[field] = []
    bans = baseline.get("bans")
    if isinstance(bans, dict) and bans.get("skip-tree") == []:
        bans.pop("skip-tree")
    return baseline


def validate_exception_config(
    register: Any,
    config: dict[str, Any],
    *,
    baseline_config: dict[str, Any] | None = None,
    today: date | None = None,
) -> None:
    """Require exact bidirectional correspondence with cargo-deny waiver fields."""
    current_date = today or date.today()
    entries = _validate_register(register, current_date, check_current=False)
    if not isinstance(config, dict):
        raise PolicyError("cargo-deny configuration is malformed")

    expected_advisories = {entry["advisory_id"] for entry in entries if entry["kind"] == "advisory"}
    expected_yanked = {
        (f"{entry['package']}@{entry['version']}", entry["id"])
        for entry in entries
        if entry["kind"] == "yanked"
    }
    expected_clarifications = {
        f"{entry['package']}@{entry['version']}": {
            "expression": entry["license_evidence"]["expression"],
            "license_files": entry["license_evidence"]["license_files"],
        }
        for entry in entries
        if entry["kind"] == "license"
    }
    expected_git_sources = {
        _git_config_source(entry["source"]) for entry in entries if entry["kind"] == "source"
    }
    expected_duplicates = {
        (f"{entry['package']}@{entry['version']}", entry["id"])
        for entry in entries
        if entry["kind"] == "duplicate"
    }

    def mismatch_details(exception_ids: list[str], has_unregistered_waiver: bool) -> str:
        details = []
        if exception_ids:
            details.append(f"registered exception ID(s): {', '.join(exception_ids)}")
        if has_unregistered_waiver:
            details.append("no registered exception ID exists for a configured waiver")
        if not details:
            details.append("no registered exception ID exists")
        return "; ".join(details)

    advisories = config.get("advisories", {})
    if not isinstance(advisories, dict) or not isinstance(advisories.get("ignore", []), list):
        raise PolicyError("cargo-deny advisory exception configuration is malformed")
    configured_advisories = advisories.get("ignore", [])
    configured_advisory_ids: list[str] = []
    configured_yanked: set[tuple[str, str]] = set()
    for item in configured_advisories:
        if isinstance(item, str):
            configured_advisory_ids.append(item)
            continue
        if (
            not isinstance(item, dict)
            or set(item) != {"crate", "reason"}
            or not isinstance(item.get("crate"), str)
            or not isinstance(item.get("reason"), str)
        ):
            raise PolicyError("cargo-deny advisory ignore entry is malformed")
        crate = item["crate"]
        reason = item["reason"]
        ids = [
            entry["id"]
            for entry in entries
            if entry["kind"] == "yanked" and f"{entry['package']}@{entry['version']}" == crate
        ]
        if len(ids) != 1 or ids[0] not in reason:
            raise PolicyError("cargo-deny yanked ignore does not cite one exact registered exception ID")
        configured_yanked.add((crate, ids[0]))
    if (
        len(set(configured_advisory_ids)) != len(configured_advisory_ids)
        or set(configured_advisory_ids) != expected_advisories
        or len(configured_yanked) != sum(
            1 for item in configured_advisories if isinstance(item, dict)
        )
        or configured_yanked != expected_yanked
    ):
        duplicate_advisories = {
            value
            for value in configured_advisory_ids
            if configured_advisory_ids.count(value) > 1
        }
        missing_ids = [
            entry["id"]
            for entry in entries
            if entry["kind"] == "advisory"
            and (
                entry["advisory_id"] not in configured_advisory_ids
                or entry["advisory_id"] in duplicate_advisories
            )
        ]
        has_unregistered_waiver = any(
            value not in expected_advisories for value in configured_advisory_ids
        )
        missing_ids.extend(
            exception_id for crate, exception_id in expected_yanked - configured_yanked
        )
        has_unregistered_waiver = has_unregistered_waiver or any(
            value not in expected_yanked for value in configured_yanked
        )
        raise PolicyError(
            "cargo-deny advisory and yanked ignores do not match the exception register; "
            f"{mismatch_details(missing_ids, has_unregistered_waiver)}"
        )

    licenses = config.get("licenses", {})
    if not isinstance(licenses, dict):
        raise PolicyError("cargo-deny license configuration is malformed")
    license_exceptions = licenses.get("exceptions", [])
    if not isinstance(license_exceptions, list) or license_exceptions:
        raise PolicyError("cargo-deny licenses.exceptions cannot represent a reviewed clarification")
    clarification_items = licenses.get("clarify", [])
    if not isinstance(clarification_items, list):
        raise PolicyError("cargo-deny license clarification configuration is malformed")
    configured_clarifications: dict[str, dict[str, Any]] = {}
    for item in clarification_items:
        if (
            not isinstance(item, dict)
            or set(item) != {"crate", "expression", "license-files"}
            or not isinstance(item.get("crate"), str)
            or not isinstance(item.get("expression"), str)
        ):
            raise PolicyError("cargo-deny license clarification entry is malformed")
        crate = item["crate"]
        if crate in configured_clarifications:
            duplicate_ids = [
                entry["id"]
                for entry in entries
                if entry["kind"] == "license" and crate == f"{entry['package']}@{entry['version']}"
            ]
            has_unregistered_waiver = not duplicate_ids
            raise PolicyError(
                "cargo-deny license clarification entry is duplicated; "
                f"{mismatch_details(duplicate_ids, has_unregistered_waiver)}"
            )
        configured_clarifications[crate] = {
            "expression": item["expression"],
            "license_files": _license_files(item["license-files"], "cargo-deny license clarification"),
        }
    if configured_clarifications != expected_clarifications:
        missing_ids = [
            entry["id"]
            for entry in entries
            if entry["kind"] == "license"
            and configured_clarifications.get(f"{entry['package']}@{entry['version']}")
            != expected_clarifications[f"{entry['package']}@{entry['version']}"]
        ]
        has_unregistered_waiver = any(crate not in expected_clarifications for crate in configured_clarifications)
        raise PolicyError(
            "cargo-deny license clarifications do not match reviewed evidence in the register; "
            f"{mismatch_details(missing_ids, has_unregistered_waiver)}"
        )

    sources = config.get("sources", {})
    if not isinstance(sources, dict) or not isinstance(sources.get("allow-git", []), list):
        raise PolicyError("cargo-deny Git source exception configuration is malformed")
    configured_git_sources = sources.get("allow-git", [])
    if any(not isinstance(value, str) for value in configured_git_sources):
        raise PolicyError("cargo-deny Git source exception configuration is malformed")
    if len(set(configured_git_sources)) != len(configured_git_sources):
        duplicate_sources = {
            value for value in configured_git_sources if configured_git_sources.count(value) > 1
        }
        duplicate_ids = [
            entry["id"]
            for entry in entries
            if entry["kind"] == "source" and _git_config_source(entry["source"]) in duplicate_sources
        ]
        has_unregistered_waiver = any(value not in expected_git_sources for value in duplicate_sources)
        raise PolicyError(
            "cargo-deny Git source exception configuration contains duplicate entries; "
            f"{mismatch_details(duplicate_ids, has_unregistered_waiver)}"
        )
    if set(configured_git_sources) != expected_git_sources:
        missing_ids = [
            entry["id"]
            for entry in entries
            if entry["kind"] == "source" and _git_config_source(entry["source"]) not in configured_git_sources
        ]
        has_unregistered_waiver = any(value not in expected_git_sources for value in configured_git_sources)
        raise PolicyError(
            "cargo-deny Git sources do not match the exception register; "
            f"{mismatch_details(missing_ids, has_unregistered_waiver)}"
        )

    bans = config.get("bans", {})
    if not isinstance(bans, dict) or not isinstance(bans.get("skip", []), list):
        raise PolicyError("cargo-deny duplicate exception configuration is malformed")
    skip_trees = bans.get("skip-tree", [])
    if not isinstance(skip_trees, list) or skip_trees:
        raise PolicyError("cargo-deny duplicate exception configuration cannot contain skip-tree entries")
    configured_duplicates: set[tuple[str, str]] = set()
    for item in bans.get("skip", []):
        if (
            not isinstance(item, dict)
            or set(item) != {"crate", "reason"}
            or not isinstance(item.get("crate"), str)
            or not isinstance(item.get("reason"), str)
        ):
            raise PolicyError("cargo-deny duplicate exception entry is malformed")
        crate = item["crate"]
        reason = item["reason"]
        if crate.rsplit("@", 1)[0] in ADR_0005_BANNED_PACKAGES:
            raise PolicyError("ADR-0005 architecture bans cannot appear in cargo-deny duplicate exceptions")
        ids = [
            entry["id"]
            for entry in entries
            if entry["kind"] == "duplicate" and f"{entry['package']}@{entry['version']}" == crate
        ]
        if not ids:
            raise PolicyError(
                "cargo-deny duplicate exception has no matching register record; "
                "no registered exception ID exists for the configured waiver"
            )
        if len(ids) != 1:
            raise PolicyError(
                "cargo-deny duplicate exception has ambiguous registered exception IDs: "
                + ", ".join(ids)
            )
        if ids[0] not in reason:
            raise PolicyError(f"cargo-deny duplicate exception must cite registered exception ID {ids[0]}")
        configured_duplicates.add((crate, ids[0]))
    if len(configured_duplicates) != len(bans.get("skip", [])) or configured_duplicates != expected_duplicates:
        missing_ids = [
            exception_id
            for _, exception_id in sorted(expected_duplicates - configured_duplicates)
        ]
        related_ids = sorted(set(missing_ids) | {exception_id for _, exception_id in configured_duplicates})
        raise PolicyError(
            "cargo-deny duplicate exceptions do not match the exception register; "
            f"{mismatch_details(related_ids, False)}"
        )

    if baseline_config is not None and baseline_config != _exception_free_config(config):
        raise PolicyError("waiver-free cargo-deny baseline policy does not match the reviewed policy")


def _read_json(path: Path, label: str) -> Any:
    try:
        with path.open("r", encoding="utf-8") as stream:
            return json.load(stream)
    except (OSError, UnicodeError, json.JSONDecodeError):
        raise PolicyError(f"{label} cannot be read as valid JSON") from None


def main(argv: list[str] | None = None) -> int:
    """Validate Cargo metadata and exception evidence.

    Non-source exception records require a JSON findings array via ``--findings``.
    Exact Git source findings are derived from both Cargo metadata graphs.
    """
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--checkout-root", type=Path, default=REPOSITORY_ROOT)
    parser.add_argument("--root-metadata", type=Path, required=True)
    parser.add_argument("--fuzz-metadata", type=Path, required=True)
    parser.add_argument("--deny-config", type=Path, default=REPOSITORY_ROOT / ".cargo" / "deny.toml")
    parser.add_argument(
        "--exceptions",
        type=Path,
        default=REPOSITORY_ROOT / "specification" / "compliance" / "dependency-policy-exceptions.json",
    )
    parser.add_argument(
        "--findings",
        type=Path,
        help="JSON array of structured cargo-deny findings for exact exception matching",
    )
    parser.add_argument(
        "--baseline-deny-config",
        type=Path,
        help="waiver-free cargo-deny config that must differ only by registered waiver fields",
    )
    parser.add_argument(
        "--format-cargo-deny-diagnostics",
        action="store_true",
        help="read cargo-deny JSON from stdin and print a safely redacted failure report",
    )
    parser.add_argument(
        "--extract-cargo-deny-findings",
        action="store_true",
        help="parse one waiver-free cargo-deny JSON scan from stdin into safe exact findings",
    )
    parser.add_argument("--scan-workspace", choices=("root", "fuzz"))
    parser.add_argument("--scan-exit-code", type=int)
    parser.add_argument("--preflight-only", action="store_true")
    arguments = parser.parse_args(argv)
    if arguments.extract_cargo_deny_findings:
        try:
            metadata = {
                "root": _read_json(arguments.root_metadata, "root Cargo metadata"),
                "fuzz": _read_json(arguments.fuzz_metadata, "fuzz Cargo metadata"),
            }
            if arguments.scan_workspace is None or arguments.scan_exit_code is None:
                raise PolicyError("baseline scan context is incomplete")
            raw_output = sys.stdin.read(16 * 1024 * 1024 + 1)
            findings = parse_cargo_deny_findings(
                raw_output,
                metadata,
                arguments.scan_workspace,
                arguments.scan_exit_code,
            )
            print(json.dumps(findings, separators=(",", ":")))
        except (OSError, UnicodeError, PolicyError):
            print("dependency policy: waiver-free cargo-deny findings are incomplete or invalid", file=sys.stderr)
            return 1
        return 0
    if arguments.format_cargo_deny_diagnostics:
        try:
            metadata = {
                "root": _read_json(arguments.root_metadata, "root Cargo metadata"),
                "fuzz": _read_json(arguments.fuzz_metadata, "fuzz Cargo metadata"),
            }
            raw_output = sys.stdin.read(16 * 1024 * 1024 + 1)
            print(format_cargo_deny_diagnostics(raw_output, metadata))
        except (OSError, UnicodeError, PolicyError):
            print("cargo-deny diagnostics unavailable (metadata could not be safely read).")
        return 0
    try:
        try:
            root = arguments.checkout_root.resolve(strict=True)
        except (OSError, RuntimeError, ValueError):
            raise PolicyError("checkout root cannot be canonicalized") from None
        metadata = {
            "root": _read_json(arguments.root_metadata, "root Cargo metadata"),
            "fuzz": _read_json(arguments.fuzz_metadata, "fuzz Cargo metadata"),
        }
        register = _read_json(arguments.exceptions, "dependency exception register")
        try:
            with arguments.deny_config.open("rb") as stream:
                config = tomllib.load(stream)
        except (OSError, tomllib.TOMLDecodeError):
            raise PolicyError("cargo-deny configuration cannot be read as valid TOML") from None
        baseline_config: dict[str, Any] | None = None
        if arguments.baseline_deny_config is not None:
            try:
                with arguments.baseline_deny_config.open("rb") as stream:
                    baseline_config = tomllib.load(stream)
            except (OSError, tomllib.TOMLDecodeError):
                raise PolicyError("waiver-free cargo-deny baseline cannot be read as valid TOML") from None
        validate_no_local_exception_files((root / "Cargo.toml", root / "fuzz" / "Cargo.toml"))
        findings = validate_workspace_metadata(root, metadata, register)
        validate_exception_config(register, config, baseline_config=baseline_config)
        exceptions = register.get("exceptions") if isinstance(register, dict) else None
        matched_exception_ids: list[str] = []
        if arguments.preflight_only:
            print("Dependency policy inputs, local exception discovery, and waiver-free config are valid.")
            return 0
        if arguments.findings is None:
            if exceptions and any(
                isinstance(entry, dict) and entry.get("kind") != "source" for entry in exceptions
            ):
                raise PolicyError("non-source exceptions require exact finding evidence via --findings")
        else:
            additional_findings = _read_json(arguments.findings, "dependency findings")
            if not isinstance(additional_findings, list):
                raise PolicyError("dependency findings must be a JSON array")
            unique_findings = {
                json.dumps(item, sort_keys=True, separators=(",", ":")): item
                for item in additional_findings
            }
            findings.extend(unique_findings.values())
        if exceptions or arguments.findings is not None:
            matched_exception_ids = validate_exceptions(register, findings)
    except PolicyError as error:
        print(f"dependency policy: {error}", file=sys.stderr)
        return 1
    if matched_exception_ids:
        print("Validated exception IDs: " + ", ".join(matched_exception_ids))
    else:
        print("No dependency exceptions were registered.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
