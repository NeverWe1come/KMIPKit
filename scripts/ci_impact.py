#!/usr/bin/env python3
"""Classify a pull-request tree diff into conservative CI job and coverage scopes."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path, PurePosixPath
from typing import Any, Iterable, Mapping


SCHEMA_VERSION = 1
SHA_PATTERN = re.compile(r"^[0-9a-fA-F]{40}(?:[0-9a-fA-F]{24})?$")
MAX_PLAN_OUTPUT_BYTES = 60_000

PULL_REQUEST_JOB_IDS = (
    "docs-contracts",
    "core",
    "script-contracts",
    "language-c",
    "language-java",
    "language-python",
    "ffi-sanitizer-c",
    "ffi-sanitizer-jni",
    "fuzz-smoke",
    "normative-inventory",
    "coverage",
    "coverage-java",
    "coverage-python",
    "coverage-jni",
    "coverage-gate",
    "dependency-policy",
)
ALL_COVERAGE_SCOPES = ("rust", "ffi-c", "java", "python", "jni")
CLASS_ORDER = ("documentation", "java", "python", "c-consumer", "jni", "full")
KNOWN_CLASSES = set(CLASS_ORDER)
KNOWN_STATUSES = {"A", "M", "D", "T", "U", "X", "B", "R", "C"}

FULL_PREFIXES = (
    ".github/",
    ".specify/",
    ".cargo/",
    "crates/",
    "fuzz/",
    "scripts/",
    "specification/",
    "tests/",
    "tools/",
)
FULL_FILES = {
    "AGENTS.md",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain",
    "rust-toolchain.toml",
    "rust-toolchain.yaml",
    "Makefile",
    "justfile",
}
DOC_ROOT_FILES = {"README.md", "CHANGELOG.md", "CONTRIBUTING.md", "SECURITY.md"}


class ImpactPlanError(ValueError):
    """Raised when a diff or impact plan cannot be trusted."""


def _valid_sha(value: Any) -> bool:
    return isinstance(value, str) and SHA_PATTERN.fullmatch(value) is not None


def _normalized_path(path: str) -> str | None:
    if not isinstance(path, str) or not path or "\\" in path or "\x00" in path:
        return None
    candidate = PurePosixPath(path)
    if candidate.is_absolute() or any(part in {"", ".", ".."} for part in candidate.parts):
        return None
    return candidate.as_posix()


def _path_class(path: str) -> tuple[str, str]:
    normalized = _normalized_path(path)
    if normalized is None:
        return "full", "path is absolute, malformed, or escapes the repository"
    path = normalized

    if path in FULL_FILES or path.startswith(FULL_PREFIXES):
        return "full", "shared source, workflow, tooling, normative, or repository configuration changed"

    name = PurePosixPath(path).name
    if name in DOC_ROOT_FILES and "/" not in path:
        return "documentation", "reviewed documentation file changed"
    if path.startswith("docs/") and path.endswith(".md"):
        return "documentation", "documentation changed"
    if path.startswith("specs/") and path.endswith(".md"):
        return "documentation", "feature specification or traceability documentation changed"
    if path.startswith("changelog.d/") and path.endswith(".md"):
        return "documentation", "release-note documentation changed"

    if path.startswith("bindings/java/native/"):
        if name in {"build.sh", "build.ps1", "CMakeLists.txt"}:
            return "full", "native build configuration changed"
        return "jni", "JNI native source, header, test, or example changed"

    if path.startswith("bindings/java/src/") or path.startswith("bindings/java/examples/"):
        return "java", "Java source, test, or example changed"
    if path.startswith("bindings/python/src/") or path.startswith("bindings/python/tests/") or path.startswith("bindings/python/examples/"):
        return "python", "Python source, test, or example changed"
    if (
        path.startswith("bindings/c/include/")
        or path.startswith("bindings/c/tests/")
        or path.startswith("bindings/c/examples/")
    ):
        return "c-consumer", "C consumer interface, test, or example changed"

    if path.startswith("bindings/"):
        return "full", "unreviewed binding path or binding build configuration changed"
    return "full", "unrecognized path conservatively selects full CI"


def parse_name_status_z(output: bytes) -> list[dict[str, Any]]:
    """Parse `git diff --name-status -z`, preserving path bytes and rename sides."""
    if not isinstance(output, bytes):
        raise ImpactPlanError("Git name-status output must be bytes.")
    if not output:
        return []
    if not output.endswith(b"\0"):
        raise ImpactPlanError("Git name-status output is truncated (missing NUL terminator).")

    fields = output[:-1].split(b"\0")
    if any(not field for field in fields):
        raise ImpactPlanError("Git name-status output contains an empty record.")
    changes: list[dict[str, Any]] = []
    index = 0
    while index < len(fields):
        try:
            status = fields[index].decode("ascii")
        except UnicodeDecodeError as error:
            raise ImpactPlanError("Git returned a non-ASCII file status.") from error
        index += 1
        kind = status[:1]
        if kind not in KNOWN_STATUSES or (kind in {"R", "C"} and not status[1:].isdigit()):
            raise ImpactPlanError(f"Git returned an unsupported file status: {status!r}.")
        path_count = 2 if kind in {"R", "C"} else 1
        if index + path_count > len(fields):
            raise ImpactPlanError(f"Git status {status!r} has an incomplete path record.")
        paths = [os.fsdecode(value) for value in fields[index : index + path_count]]
        index += path_count
        if any(not path for path in paths):
            raise ImpactPlanError(f"Git status {status!r} contains an empty path.")
        changes.append({"status": status, "paths": paths})
    return changes


def _empty_plan(base_sha: str, merge_sha: str, reason: str, *, fallback: bool) -> dict[str, Any]:
    return {
        "schema_version": SCHEMA_VERSION,
        "base_sha": base_sha if _valid_sha(base_sha) else None,
        "merge_sha": merge_sha if _valid_sha(merge_sha) else None,
        "classes": ["full"],
        "paths": [],
        "selected_jobs": list(PULL_REQUEST_JOB_IDS),
        "coverage_scopes": list(ALL_COVERAGE_SCOPES),
        "full": True,
        "reason": "Full CI selected: " + " ".join(str(reason).split())[:500],
        "fallback": fallback,
    }


def classify_changes(
    changes: Iterable[Mapping[str, Any]], *, base_sha: str, merge_sha: str
) -> dict[str, Any]:
    """Build a deterministic plan from parsed status/path records."""
    if not _valid_sha(base_sha) or not _valid_sha(merge_sha):
        return _empty_plan(base_sha, merge_sha, "base or merge SHA is invalid", fallback=True)

    class_reasons: dict[str, list[str]] = {}
    path_records: list[dict[str, str]] = []
    try:
        for change in changes:
            status = change.get("status")
            paths = change.get("paths")
            if not isinstance(status, str) or not isinstance(paths, list) or not paths:
                raise ImpactPlanError("A changed-file record has an invalid status or path list.")
            kind = status[:1]
            if kind not in KNOWN_STATUSES and kind not in {"R", "C"}:
                raise ImpactPlanError(f"A changed-file record has an unsupported status: {status!r}.")
            expected_path_count = 2 if kind in {"R", "C"} else 1
            if len(paths) != expected_path_count:
                raise ImpactPlanError(f"Changed-file status {status!r} has the wrong number of paths.")
            for raw_path in paths:
                if not isinstance(raw_path, str):
                    raise ImpactPlanError("A changed-file path is not text.")
                path_class, reason = _path_class(raw_path)
                normalized = _normalized_path(raw_path)
                safe_path = normalized if normalized is not None else "<invalid path>"
                path_records.append({"path": safe_path, "class": path_class})
                class_reasons.setdefault(path_class, []).append(reason)
    except (ImpactPlanError, AttributeError, TypeError) as error:
        return _empty_plan(base_sha, merge_sha, f"change list could not be classified: {error}", fallback=True)

    if not class_reasons:
        class_reasons["documentation"] = ["no changed paths; run documentation contracts"]

    classes = [name for name in CLASS_ORDER if name in class_reasons]
    if "full" in class_reasons:
        reasons = sorted(set(class_reasons["full"]))
        return _empty_plan(
            base_sha,
            merge_sha,
            "; ".join(reasons),
            fallback=any(record["path"] == "<invalid path>" for record in path_records)
            or any("unrecognized" in reason for reason in reasons),
        ) | {"classes": classes or ["full"], "paths": path_records}

    selected: set[str] = set()
    scopes: set[str] = set()
    if "documentation" in class_reasons:
        selected.add("docs-contracts")
    if "java" in class_reasons:
        selected.update(("language-java", "coverage-java", "coverage-gate"))
        scopes.add("java")
    if "python" in class_reasons:
        selected.update(("language-python", "coverage-python", "coverage-gate"))
        scopes.add("python")
    if "c-consumer" in class_reasons:
        selected.update(("language-c", "ffi-sanitizer-c", "coverage", "coverage-gate"))
        scopes.update(("rust", "ffi-c"))
    if "jni" in class_reasons:
        selected.update(("language-java", "ffi-sanitizer-jni", "coverage-jni", "coverage-gate"))
        scopes.add("jni")

    reasons = sorted({reason for entries in class_reasons.values() for reason in entries})
    return {
        "schema_version": SCHEMA_VERSION,
        "base_sha": base_sha.lower(),
        "merge_sha": merge_sha.lower(),
        "classes": classes,
        "paths": path_records,
        "selected_jobs": [job for job in PULL_REQUEST_JOB_IDS if job in selected],
        "coverage_scopes": [scope for scope in ALL_COVERAGE_SCOPES if scope in scopes],
        "full": False,
        "reason": "; ".join(reasons) if reasons else "No changed paths; run documentation contracts.",
        "fallback": False,
    }


def classify_diff(repository: str | Path, base_sha: str, merge_sha: str) -> dict[str, Any]:
    """Classify exact Git trees; any diff failure returns a full-CI fallback plan."""
    if not _valid_sha(base_sha) or not _valid_sha(merge_sha):
        return _empty_plan(base_sha, merge_sha, "base or merge SHA is invalid", fallback=True)
    command = [
        "git",
        "-C",
        str(Path(repository)),
        "diff",
        "--name-status",
        "-z",
        "--find-renames",
        "--find-copies",
        "--find-copies-harder",
        base_sha,
        merge_sha,
        "--",
    ]
    try:
        result = subprocess.run(command, check=True, capture_output=True)
        changes = parse_name_status_z(result.stdout)
    except (OSError, subprocess.CalledProcessError, ImpactPlanError) as error:
        detail = error.stderr if isinstance(error, subprocess.CalledProcessError) else str(error)
        if isinstance(detail, bytes):
            detail = os.fsdecode(detail)
        return _empty_plan(base_sha, merge_sha, f"exact base-to-merge diff failed: {detail}", fallback=True)
    plan = classify_changes(changes, base_sha=base_sha, merge_sha=merge_sha)
    if len(json.dumps(plan, ensure_ascii=True, separators=(",", ":")).encode("utf-8")) > MAX_PLAN_OUTPUT_BYTES:
        return _empty_plan(base_sha, merge_sha, "impact plan exceeded the safe GitHub output size", fallback=True)
    return plan


def validate_plan(
    plan: Any, *, expected_base_sha: str | None = None, expected_merge_sha: str | None = None
) -> tuple[bool, str]:
    """Validate a plan at the Summary boundary and verify job/scope consistency."""
    if not isinstance(plan, dict):
        return False, "impact plan is not a JSON object"
    if plan.get("schema_version") != SCHEMA_VERSION:
        return False, "impact plan has an unsupported schema version"
    base_sha, merge_sha = plan.get("base_sha"), plan.get("merge_sha")
    if not _valid_sha(base_sha) or not _valid_sha(merge_sha):
        return False, "impact plan is not bound to valid base and merge SHAs"
    if expected_base_sha and base_sha.lower() != expected_base_sha.lower():
        return False, "impact plan base SHA does not match the pull request"
    if expected_merge_sha and merge_sha.lower() != expected_merge_sha.lower():
        return False, "impact plan merge SHA does not match this run"
    classes, paths = plan.get("classes"), plan.get("paths")
    jobs, scopes = plan.get("selected_jobs"), plan.get("coverage_scopes")
    if not isinstance(classes, list) or not classes or any(value not in KNOWN_CLASSES for value in classes):
        return False, "impact plan has unknown or missing path classes"
    if not isinstance(paths, list) or any(
        not isinstance(item, dict)
        or not isinstance(item.get("path"), str)
        or item.get("class") not in KNOWN_CLASSES
        for item in paths
    ):
        return False, "impact plan has malformed path records"
    if not isinstance(jobs, list) or any(job not in PULL_REQUEST_JOB_IDS for job in jobs) or len(set(jobs)) != len(jobs):
        return False, "impact plan has unknown or duplicate job identifiers"
    if not isinstance(scopes, list) or any(scope not in ALL_COVERAGE_SCOPES for scope in scopes) or len(set(scopes)) != len(scopes):
        return False, "impact plan has unknown or duplicate coverage scopes"
    if type(plan.get("full")) is not bool or type(plan.get("fallback")) is not bool:
        return False, "impact plan full/fallback fields are invalid"
    if not isinstance(plan.get("reason"), str) or not plan["reason"].strip():
        return False, "impact plan has no routing reason"
    if plan["full"]:
        if set(jobs) != set(PULL_REQUEST_JOB_IDS) or set(scopes) != set(ALL_COVERAGE_SCOPES):
            return False, "full impact plan does not select every pull-request job and coverage scope"
        return True, ""

    expected = classify_changes(
        [
            {"status": "M", "paths": [record["path"]]}
            for record in paths
        ],
        base_sha=base_sha,
        merge_sha=merge_sha,
    )
    if expected["full"]:
        return False, "narrow impact plan contains a path that requires full CI"
    if set(expected["classes"]) != set(classes):
        return False, "impact plan classes do not match its paths"
    if set(expected["selected_jobs"]) != set(jobs) or set(expected["coverage_scopes"]) != set(scopes):
        return False, "impact plan jobs or coverage scopes do not match its paths"
    return True, ""


def _write_plan(plan: Mapping[str, Any], output_path: str | None, github_output: str | None) -> None:
    serialized = json.dumps(plan, ensure_ascii=True, separators=(",", ":"))
    if output_path:
        Path(output_path).write_text(serialized + "\n", encoding="utf-8", newline="\n")
    if github_output:
        with Path(github_output).open("a", encoding="utf-8", newline="\n") as output:
            output.write(f"plan_json={serialized}\n")
            output.write(f"full={'true' if plan['full'] else 'false'}\n")
            output.write("selected_jobs=" + ",".join(plan["selected_jobs"]) + "\n")
            output.write("coverage_scopes=" + json.dumps(plan["coverage_scopes"], separators=(",", ":")) + "\n")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    classify = subparsers.add_parser("classify", help="classify exact pull-request base and merge trees")
    classify.add_argument("--repo", default=".")
    classify.add_argument("--base", required=True)
    classify.add_argument("--merge", required=True)
    classify.add_argument("--output")
    classify.add_argument("--github-output")
    args = parser.parse_args(argv)

    if not _valid_sha(args.base) or not _valid_sha(args.merge):
        parser.error("--base and --merge must be full 40- or 64-character hexadecimal SHAs")
    try:
        plan = classify_diff(args.repo, args.base, args.merge)
        _write_plan(plan, args.output, args.github_output)
    except OSError as error:
        print(f"Could not write CI impact plan: {error}", file=sys.stderr)
        return 2
    print(json.dumps(plan, indent=2, ensure_ascii=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
