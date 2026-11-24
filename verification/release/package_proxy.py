#!/usr/bin/env python3
"""Build and package kmipkit-proxy variants for release distribution."""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import tarfile
import tempfile
import zipfile
from pathlib import Path

ALL_VARIANTS = (
    "all-clouds",
    "aws",
    "azure",
    "gcp",
    "oci",
    "vault",
    "infisical",
    "ibm",
    "alibaba",
    "scaleway",
    "otc",
)

ARCHIVE_FORMATS = {
    "linux-glibc-x86_64": "tar.gz",
    "linux-glibc-aarch64": "tar.gz",
    "macos-aarch64": "tar.gz",
    "macos-x86_64": "tar.gz",
    "windows-x86_64-msvc": "zip",
}

DOCKER_ARCH_BY_TARGET = {
    "linux-glibc-x86_64": "amd64",
    "linux-glibc-aarch64": "arm64",
}


def resolve_variants(raw: str) -> list[str]:
    cleaned = raw.strip()
    if not cleaned or cleaned == "all":
        return list(ALL_VARIANTS)
    selected = [item.strip() for item in cleaned.split(",") if item.strip()]
    unknown = [item for item in selected if item not in ALL_VARIANTS]
    if unknown:
        raise ValueError(f"Unknown proxy variant(s): {', '.join(unknown)}")
    return selected


def archive_stem(variant: str, version: str, target: str) -> str:
    if variant == "all-clouds":
        return f"kmipkit-proxy-{version}-{target}"
    return f"kmipkit-proxy-{variant}-{version}-{target}"


def package_variant(
    repo_root: Path,
    output_dir: Path,
    docker_staging_dir: Path | None,
    version: str,
    target: str,
    variant: str,
) -> Path:
    manifest_path = repo_root / "kmip-proxy" / "Cargo.toml"
    cargo_target_dir = repo_root / "kmip-proxy" / "target"

    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(cargo_target_dir)

    subprocess.run(
        [
            "cargo",
            "build",
            "--locked",
            "--release",
            "--manifest-path",
            str(manifest_path),
            "--features",
            variant,
        ],
        cwd=repo_root,
        env=env,
        check=True,
    )

    binary_name = "kmipkit-proxy.exe" if target.startswith("windows-") else "kmipkit-proxy"
    built_binary = cargo_target_dir / "release" / binary_name
    if not built_binary.is_file():
        raise FileNotFoundError(f"Built proxy binary missing: {built_binary}")

    # Smoke test: verify CLI responds cleanly to --help
    subprocess.run([str(built_binary), "--help"], check=True, stdout=subprocess.DEVNULL)

    stem = archive_stem(variant, version, target)
    fmt = ARCHIVE_FORMATS[target]
    archive_path = output_dir / f"{stem}.{fmt}"
    archive_path.parent.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory() as tmp:
        stage = Path(tmp) / stem
        stage.mkdir(parents=True, exist_ok=True)
        shutil.copy2(built_binary, stage / binary_name)
        for doc_name in ("LICENSE", "NOTICE"):
            doc_src = repo_root / doc_name
            if doc_src.is_file():
                shutil.copy2(doc_src, stage / doc_name)
        readme_src = repo_root / "kmip-proxy" / "README.md"
        if readme_src.is_file():
            shutil.copy2(readme_src, stage / "README.md")

        if fmt == "tar.gz":
            with tarfile.open(archive_path, "w:gz") as tar:
                tar.add(stage, arcname=stem)
        else:
            with zipfile.ZipFile(archive_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
                for file_path in sorted(stage.rglob("*")):
                    if file_path.is_file():
                        zf.write(file_path, arcname=str(Path(stem) / file_path.relative_to(stage)))

    if docker_staging_dir is not None and target in DOCKER_ARCH_BY_TARGET:
        arch = DOCKER_ARCH_BY_TARGET[target]
        staged_bin = docker_staging_dir / variant / arch / "kmipkit-proxy"
        staged_bin.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(built_binary, staged_bin)

    return archive_path


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path("."))
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--docker-staging-dir", type=Path, default=None)
    parser.add_argument("--version", required=True)
    parser.add_argument("--target", required=True, choices=sorted(ARCHIVE_FORMATS))
    parser.add_argument("--variants", default="all")
    args = parser.parse_args(argv)

    repo_root = args.repo.resolve()
    output_dir = args.output_dir.resolve()
    docker_staging = args.docker_staging_dir.resolve() if args.docker_staging_dir else None
    variants = resolve_variants(args.variants)

    for variant in variants:
        archive = package_variant(
            repo_root=repo_root,
            output_dir=output_dir,
            docker_staging_dir=docker_staging,
            version=args.version,
            target=args.target,
            variant=variant,
        )
        print(f"Packaged proxy ({variant}): {archive}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
