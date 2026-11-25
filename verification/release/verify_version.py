#!/usr/bin/env python3
"""Verify SemVer X.Y.Z consistency across all KMIPKit release manifests."""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import tomllib
import xml.etree.ElementTree as ET
from pathlib import Path

SEMVER_RE = re.compile(r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$")

INTERNAL_CRATES = (
    "kmipkit-ttlv",
    "kmipkit-protocol",
    "kmipkit-transport",
    "kmipkit-client",
    "kmipkit",
    "kmipkit-ffi",
    "kmipkit-test-support",
    "kmipkit-test-server",
)


def parse_semver(version: str) -> tuple[int, int, int]:
    match = SEMVER_RE.fullmatch(version.strip())
    if not match:
        raise ValueError(
            f"Version {version!r} does not match strict SemVer X.Y.Z (no pre-release or build metadata)"
        )
    return int(match.group(1)), int(match.group(2)), int(match.group(3))


def extract_versions(repo_root: Path) -> dict[str, str]:
    versions: dict[str, str] = {}

    cargo_toml = tomllib.loads((repo_root / "Cargo.toml").read_text(encoding="utf-8"))
    workspace_version = cargo_toml["workspace"]["package"]["version"]
    versions["Cargo.toml [workspace.package]"] = workspace_version

    ws_deps = cargo_toml["workspace"].get("dependencies", {})
    for crate_name in INTERNAL_CRATES:
        if crate_name in ws_deps and isinstance(ws_deps[crate_name], dict):
            dep_ver = ws_deps[crate_name].get("version")
            if dep_ver is not None:
                versions[f"Cargo.toml [workspace.dependencies.{crate_name}]"] = dep_ver

    proxy_toml = tomllib.loads(
        (repo_root / "kmip-proxy" / "Cargo.toml").read_text(encoding="utf-8")
    )
    versions["kmip-proxy/Cargo.toml"] = proxy_toml["package"]["version"]

    gradle_text = (repo_root / "bindings" / "java" / "build.gradle.kts").read_text(
        encoding="utf-8"
    )
    gradle_match = re.search(
        r'(?m)^version\s*=\s*"([^"]+)"', gradle_text
    )
    if not gradle_match:
        raise ValueError("Could not find version in bindings/java/build.gradle.kts")
    versions["bindings/java/build.gradle.kts"] = gradle_match.group(1)

    pom_root = ET.parse(repo_root / "bindings" / "java" / "pom.xml").getroot()
    ns = {"m": "http://maven.apache.org/POM/4.0.0"}
    pom_ver_el = pom_root.find("m:version", ns)
    if pom_ver_el is None or not pom_ver_el.text:
        raise ValueError("Could not find <version> in bindings/java/pom.xml")
    versions["bindings/java/pom.xml"] = pom_ver_el.text.strip()

    pyproject = tomllib.loads(
        (repo_root / "bindings" / "python" / "pyproject.toml").read_text(
            encoding="utf-8"
        )
    )
    versions["bindings/python/pyproject.toml"] = pyproject["project"]["version"]

    api_manifest = json.loads(
        (repo_root / "bindings" / "shared" / "public-api.json").read_text(
            encoding="utf-8"
        )
    )
    release_line = api_manifest.get("releaseLine")
    if not isinstance(release_line, str):
        raise ValueError("Could not find releaseLine in bindings/shared/public-api.json")
    versions["bindings/shared/public-api.json (releaseLine)"] = release_line

    return versions


def verify_repo_version(repo_root: Path, expected_version: str | None = None) -> str:
    versions = extract_versions(repo_root)
    distinct = set(versions.values())
    if len(distinct) != 1:
        details = "\n".join(f"  - {k}: {v}" for k, v in versions.items())
        raise ValueError(f"Version mismatch across manifests:\n{details}")

    version = next(iter(distinct))
    parse_semver(version)

    if expected_version is not None and version != expected_version:
        raise ValueError(
            f"Repository version {version!r} does not match expected {expected_version!r}"
        )
    return version


def write_github_outputs(version: str) -> None:
    github_output = os.environ.get("GITHUB_OUTPUT")
    if not github_output:
        raise ValueError("GITHUB_OUTPUT environment variable is not set")
    major, minor, patch = parse_semver(version)
    lines = [
        f"version={version}",
        f"tag=v{version}",
        f"major={major}",
        f"minor={minor}",
        f"patch={patch}",
    ]
    with open(github_output, "a", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path("."))
    parser.add_argument("--expected-version", default=None)
    parser.add_argument("--write-github-output", action="store_true")
    args = parser.parse_args(argv)

    try:
        version = verify_repo_version(args.repo.resolve(), args.expected_version)
    except ValueError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    if args.write_github_output:
        write_github_outputs(version)

    print(f"Verified unified KMIPKit SemVer: {version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
