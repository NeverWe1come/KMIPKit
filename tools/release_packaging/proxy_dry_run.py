"""Create a deterministic inventory for dry-run proxy release artifacts."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

PROVIDERS = (
    "alibaba",
    "aws",
    "azure",
    "gcp",
    "ibm",
    "infisical",
    "oci",
    "otc",
    "scaleway",
    "vault",
)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as artifact:
        for chunk in iter(lambda: artifact.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def assemble_proxy_manifest(
    input_dir: Path | str, output_dir: Path | str, version: str
) -> dict[str, Any]:
    """Write sorted artifact metadata and SHA256SUMS for proxy packages."""
    source = Path(input_dir).resolve()
    destination = Path(output_dir).resolve()
    artifacts = sorted(path for path in source.rglob("*") if path.is_file())
    if not artifacts:
        raise ValueError(f"no proxy release artifacts found in {source}")
    actual_paths = {path.relative_to(source).as_posix() for path in artifacts}
    expected_paths = {
        "kmipkit-proxy-universal.tar",
        *(f"kmipkit-proxy-{provider}.tar" for provider in PROVIDERS),
        f"kmipkit-proxy-{version}.tgz",
    }
    if actual_paths != expected_paths:
        missing = sorted(expected_paths - actual_paths)
        unexpected = sorted(actual_paths - expected_paths)
        raise ValueError(f"proxy artifact set mismatch; missing={missing}, unexpected={unexpected}")

    records = [
        {
            "path": path.relative_to(source).as_posix(),
            "size": path.stat().st_size,
            "sha256": sha256_file(path),
        }
        for path in artifacts
    ]
    destination.mkdir(parents=True, exist_ok=True)
    (destination / "SHA256SUMS").write_text(
        "".join(f"{entry['sha256']}  {entry['path']}\n" for entry in records),
        encoding="utf-8",
    )
    manifest = {
        "version": version,
        "artifacts_count": len(records),
        "artifacts": records,
    }
    (destination / "proxy-artifacts-manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    return manifest


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input-dir", required=True)
    parser.add_argument("--output-dir", required=True)
    parser.add_argument("--version", required=True)
    args = parser.parse_args()
    try:
        report = assemble_proxy_manifest(args.input_dir, args.output_dir, args.version)
    except (OSError, ValueError) as error:
        parser.error(str(error))
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
