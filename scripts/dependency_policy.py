"""Fail-closed validation for Cargo dependency policy metadata and exceptions."""

from __future__ import annotations

import argparse
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
SECRET_QUERY_PATTERN = re.compile(
    r"(?:token|secret|pass(?:word|wd)?|credential|authorization|signature|(?:api|access|private|client)[_-]?key)",
    re.I,
)


class PolicyError(ValueError):
    """A dependency policy input is malformed, unsafe, or inconsistent."""


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


def _metadata_path(value: Any, checkout_root: Path, label: str) -> tuple[Path, Path]:
    """Return lexical and canonical metadata paths, rejecting escapes at both layers."""
    if not isinstance(value, str) or not value.strip():
        raise PolicyError(f"{label} path is missing or malformed")
    raw_path = Path(value)
    if not raw_path.is_absolute():
        raise PolicyError(f"{label} path must be absolute")
    lexical_path = Path(os.path.abspath(os.fspath(raw_path)))
    lexical_root = Path(os.path.abspath(os.fspath(checkout_root)))
    if not _is_within(lexical_path, lexical_root):
        raise PolicyError(f"{label} path is outside the checkout")
    try:
        canonical_path = lexical_path.resolve(strict=True)
    except (OSError, RuntimeError):
        raise PolicyError(f"{label} path cannot be canonicalized") from None
    if not _is_within(canonical_path, checkout_root):
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
            metadata.get("workspace_root"), root, f"{workspace_name} workspace root"
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
            _, manifest = _metadata_path(member.get("manifest_path"), root, f"{workspace_name} member manifest")
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
            _, manifest = _metadata_path(package_item.get("manifest_path"), root, f"{label} manifest")
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
    if not isinstance(kind, str) or kind not in {"advisory", "license", "source", "duplicate"}:
        raise PolicyError("exception kind must be advisory, license, source, or duplicate")

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


def validate_exceptions(register: Any, findings: list[dict], *, today: date | None = None) -> None:
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
        if not isinstance(kind, str) or kind not in {"advisory", "license", "source", "duplicate"}:
            raise PolicyError("dependency finding rule is unsupported")
        if not isinstance(package_name, str) or not PACKAGE_PATTERN.fullmatch(package_name):
            raise PolicyError("dependency finding package is malformed")
        if not isinstance(version, str) or not VERSION_PATTERN.fullmatch(version):
            raise PolicyError(f"dependency finding for {package_name} has a malformed version")
        finding_copy = {"kind": kind, "package": package_name, "version": version}
        if "source" in item:
            finding_copy["source"] = _validate_source(item["source"], require_immutable_git=False)
        if "advisory_id" in item:
            finding_copy["advisory_id"] = _text(item["advisory_id"], "finding advisory ID", maximum=64)
        normalized_findings.append(finding_copy)

    matched_entry_indexes: set[int] = set()
    for finding_item in normalized_findings:
        matches = [
            (index, entry)
            for index, entry in enumerate(entries)
            if _finding_matches(entry, finding_item)
        ]
        if len(matches) != 1:
            raise PolicyError(
                f"unexcepted or ambiguously excepted {finding_item['kind']} finding for {finding_item['package']}"
            )
        index, entry = matches[0]
        if index in matched_entry_indexes:
            raise PolicyError(f"exception {entry['id']} matches more than one finding")
        matched_entry_indexes.add(index)
    if len(matched_entry_indexes) != len(entries):
        orphaned = next(entry for index, entry in enumerate(entries) if index not in matched_entry_indexes)
        raise PolicyError(f"exception {orphaned['id']} has no matching current finding")


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


def validate_exception_config(register: Any, config: dict[str, Any], *, today: date | None = None) -> None:
    """Require exact bidirectional correspondence with cargo-deny waiver fields."""
    current_date = today or date.today()
    entries = _validate_register(register, current_date, check_current=False)
    if not isinstance(config, dict):
        raise PolicyError("cargo-deny configuration is malformed")

    expected_advisories = {entry["advisory_id"] for entry in entries if entry["kind"] == "advisory"}
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
    if (
        any(not isinstance(value, str) for value in configured_advisories)
        or len(set(configured_advisories)) != len(configured_advisories)
        or set(configured_advisories) != expected_advisories
    ):
        duplicate_advisories = {
            value
            for value in configured_advisories
            if isinstance(value, str) and configured_advisories.count(value) > 1
        }
        missing_ids = [
            entry["id"]
            for entry in entries
            if entry["kind"] == "advisory"
            and (entry["advisory_id"] not in configured_advisories or entry["advisory_id"] in duplicate_advisories)
        ]
        has_unregistered_waiver = any(
            isinstance(value, str) and value not in expected_advisories for value in configured_advisories
        )
        raise PolicyError(
            "cargo-deny advisory ignores do not match the exception register; "
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
        help="JSON array of advisory, license, and duplicate findings for exact exception matching",
    )
    arguments = parser.parse_args(argv)
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
        findings = validate_workspace_metadata(root, metadata, register)
        validate_exception_config(register, config)
        exceptions = register.get("exceptions") if isinstance(register, dict) else None
        if exceptions:
            if arguments.findings is None and any(
                isinstance(entry, dict) and entry.get("kind") != "source" for entry in exceptions
            ):
                raise PolicyError("non-source exceptions require exact finding evidence via --findings")
            if arguments.findings is not None:
                additional_findings = _read_json(arguments.findings, "dependency findings")
                if not isinstance(additional_findings, list):
                    raise PolicyError("dependency findings must be a JSON array")
                findings.extend(additional_findings)
            validate_exceptions(register, findings)
        elif arguments.findings is not None:
            additional_findings = _read_json(arguments.findings, "dependency findings")
            if not isinstance(additional_findings, list):
                raise PolicyError("dependency findings must be a JSON array")
            if additional_findings:
                raise PolicyError("dependency findings were supplied without registered exceptions")
    except PolicyError as error:
        print(f"dependency policy: {error}", file=sys.stderr)
        return 1
    print("dependency policy metadata and exception register are valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
