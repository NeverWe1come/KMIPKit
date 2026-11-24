#!/usr/bin/env python3
"""Generate SHA256SUMS.txt, CycloneDX SBOM, and RELEASE_NOTES.md for KMIPKit."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import tomllib
from datetime import datetime, timezone
from pathlib import Path

SEMVER_RE = re.compile(r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$")
REPO_ROOT = Path(__file__).resolve().parents[2]


def compute_sha256(file_path: Path) -> str:
    digest = hashlib.sha256()
    with file_path.open("rb") as handle:
        while chunk := handle.read(65536):
            digest.update(chunk)
    return digest.hexdigest()


def write_sha256_sums(artifacts_dir: Path, output_file: Path) -> list[tuple[str, str]]:
    entries: list[tuple[str, str]] = []
    for path in sorted(artifacts_dir.rglob("*")):
        if not path.is_file():
            continue
        if path.name in {"SHA256SUMS.txt", "RELEASE_NOTES.md"}:
            continue
        rel = path.relative_to(artifacts_dir).as_posix()
        entries.append((compute_sha256(path), rel))

    output_file.parent.mkdir(parents=True, exist_ok=True)
    output_file.write_text(
        "".join(f"{digest}  {rel}\n" for digest, rel in entries),
        encoding="utf-8",
    )
    return entries


def parse_lockfile_packages(repo_root: Path, lock_rel: str) -> list[dict[str, str]]:
    if lock_rel not in {"Cargo.lock", "kmip-proxy/Cargo.lock"}:
        raise ValueError(f"Unsupported lockfile path: {lock_rel!r}")
    lockfile_path = repo_root / lock_rel
    if not lockfile_path.is_file():
        return []
    data = tomllib.loads(lockfile_path.read_text(encoding="utf-8"))
    packages = []
    for pkg in data.get("package", []):
        name = pkg.get("name")
        version = pkg.get("version")
        if name and version:
            packages.append({"name": str(name), "version": str(version)})
    return packages


def generate_cyclonedx_sbom(
    repo_root: Path,
    version: str,
    output_file: Path,
) -> dict[str, object]:
    if not SEMVER_RE.fullmatch(version):
        raise ValueError(f"Invalid SemVer version: {version!r}")
    seen: set[tuple[str, str]] = set()
    components: list[dict[str, object]] = []

    for lock_rel in ("Cargo.lock", "kmip-proxy/Cargo.lock"):
        for pkg in parse_lockfile_packages(repo_root, lock_rel):
            key = (pkg["name"], pkg["version"])
            if key in seen:
                continue
            seen.add(key)
            purl = f"pkg:cargo/{pkg['name']}@{pkg['version']}"
            components.append(
                {
                    "type": "library",
                    "name": pkg["name"],
                    "version": pkg["version"],
                    "purl": purl,
                    "bom-ref": purl,
                }
            )

    components.sort(key=lambda item: (str(item["name"]), str(item["version"])))

    sbom: dict[str, object] = {
        "bomFormat": "CycloneDX",
        "specVersion": "1.5",
        "version": 1,
        "metadata": {
            "timestamp": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "component": {
                "type": "application",
                "name": "KMIPKit",
                "version": version,
                "licenses": [{"license": {"id": "Apache-2.0"}}],
            },
        },
        "components": components,
    }

    output_file.parent.mkdir(parents=True, exist_ok=True)
    output_file.write_text(json.dumps(sbom, indent=2) + "\n", encoding="utf-8")
    return sbom


def generate_release_notes(
    version: str,
    entries: list[tuple[str, str]],
    output_file: Path,
) -> None:
    if not SEMVER_RE.fullmatch(version):
        raise ValueError(f"Invalid SemVer version: {version!r}")
    artifact_lines = "\n".join(f"- `{rel}` (`{digest[:12]}...`)" for digest, rel in entries)
    content = f"""# KMIPKit v{version}

## Included Developer Packages
- **Rust crates**: `kmipkit-ttlv`, `kmipkit-protocol`, `kmipkit-transport`, `kmipkit-client`, `kmipkit` (`.crate` + workspace SDK)
- **C / C++ SDK**: Headers, shared & static `kmipkit_ffi` (statically linked OpenSSL 3.5.9), `KMIPKitConfig.cmake`, and `kmipkit.pc`
- **Java 17**: `kmipkit-java-{version}.jar` + native classifier JARs (`linux-glibc-x86_64`, `linux-glibc-aarch64`, `windows-x86_64-msvc`, `macos-aarch64`)
- **Python 3.12**: Source distribution (`sdist`) + platform CFFI wheels (`.whl`)

## Included KMIPKit Proxy Distributions (`kmipkit-proxy`)
- **11 Variants**: `all-clouds`, `aws`, `azure`, `gcp`, `oci`, `vault`, `infisical`, `ibm`, `alibaba`, `scaleway`, `otc`
- **Platforms**: Linux x86_64, Linux ARM64, Windows x86_64, macOS ARM64
- **Multi-arch OCI images**: `linux/amd64`, `linux/arm64`

## Integrity Verification
Verify downloaded artifacts against `SHA256SUMS.txt`:
```sh
sha256sum -c SHA256SUMS.txt
```

## Packaged Files ({len(entries)} total)
{artifact_lines}
"""
    output_file.parent.mkdir(parents=True, exist_ok=True)
    output_file.write_text(content, encoding="utf-8")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True)
    args = parser.parse_args(argv)

    if not SEMVER_RE.fullmatch(args.version):
        raise ValueError(f"Invalid SemVer version: {args.version!r}")

    bundle_dir = Path(
        os.environ.get("KMIPKIT_RELEASE_BUNDLE_DIR", str(REPO_ROOT / "release-bundle" / "files"))
    ).resolve()

    sbom_path = bundle_dir / f"kmipkit-{args.version}-sbom.cdx.json"
    generate_cyclonedx_sbom(REPO_ROOT, args.version, sbom_path)

    sums_path = bundle_dir / "SHA256SUMS.txt"
    entries = write_sha256_sums(bundle_dir, sums_path)

    notes_path = bundle_dir / "RELEASE_NOTES.md"
    generate_release_notes(args.version, entries, notes_path)

    print(
        f"Generated metadata for v{args.version}: "
        f"{len(entries)} artifacts, SBOM={sbom_path.name}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
