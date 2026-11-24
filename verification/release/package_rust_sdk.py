#!/usr/bin/env python3
"""Package the publishable KMIPKit Rust SDK crates into a self-contained tarball."""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import tarfile
import tempfile
from pathlib import Path

SEMVER_RE = re.compile(r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$")

PUBLISHABLE_CRATES = (
    "kmipkit-ttlv",
    "kmipkit-protocol",
    "kmipkit-transport",
    "kmipkit-client",
    "kmipkit",
)
WORKSPACE_SUPPORT_CRATES = ("kmipkit-test-support",)


def rewrite_workspace_toml(cargo_toml_text: str) -> str:
    members_block = (
        "members = [\n"
        + "".join(f'    "crates/{c}",\n' for c in PUBLISHABLE_CRATES)
        + "]"
    )
    updated = re.sub(
        r"members\s*=\s*\[[^\]]*\]",
        members_block,
        cargo_toml_text,
        count=1,
        flags=re.DOTALL,
    )
    # Remove non-packaged workspace dependency entries so Cargo workspace loads cleanly.
    for excluded in ("kmipkit-ffi", "kmipkit-test-server"):
        updated = re.sub(rf"(?m)^{re.escape(excluded)}\s*=.*\n", "", updated)
    return updated


def package_rust_sdk(
    repo_root: Path,
    output_dir: Path,
    version: str,
    smoke_test: bool = False,
) -> Path:
    if not SEMVER_RE.fullmatch(version):
        raise ValueError(f"Invalid SemVer version: {version!r}")
    safe_repo = Path(os.path.realpath(repo_root))
    safe_output = Path(os.path.realpath(output_dir))
    stem = f"kmipkit-rust-sdk-{version}"
    archive_path = safe_output / f"{stem}.tar.gz"
    safe_output.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory() as tmp:
        stage = Path(tmp) / stem
        stage.mkdir(parents=True)

        root_toml = (safe_repo / "Cargo.toml").read_text(encoding="utf-8")
        (stage / "Cargo.toml").write_text(
            rewrite_workspace_toml(root_toml), encoding="utf-8"
        )
        shutil.copy2(safe_repo / "Cargo.lock", stage / "Cargo.lock")

        for doc in ("LICENSE", "NOTICE", "README.md"):
            src = safe_repo / doc
            if src.is_file():
                shutil.copy2(src, stage / doc)

        crates_dst = stage / "crates"
        crates_dst.mkdir()
        for crate in (*PUBLISHABLE_CRATES, *WORKSPACE_SUPPORT_CRATES):
            shutil.copytree(
                safe_repo / "crates" / crate,
                crates_dst / crate,
                ignore=shutil.ignore_patterns("target", "*.swp"),
            )

        if smoke_test:
            subprocess.run(
                ["cargo", "check", "--offline", "-p", "kmipkit"],
                cwd=stage,
                check=True,
            )
            target_dir = stage / "target"
            if target_dir.exists():
                shutil.rmtree(target_dir)

        with tarfile.open(archive_path, "w:gz") as tar:
            tar.add(stage, arcname=stem)

    return archive_path


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path("."))
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--smoke-test", action="store_true")
    args = parser.parse_args(argv)

    archive = package_rust_sdk(
        repo_root=args.repo.resolve(),
        output_dir=args.output_dir.resolve(),
        version=args.version,
        smoke_test=args.smoke_test,
    )
    print(f"Packaged Rust SDK: {archive}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
