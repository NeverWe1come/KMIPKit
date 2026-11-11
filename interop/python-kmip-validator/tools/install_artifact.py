#!/usr/bin/env python3
"""Build and install the Python wheel from one exact KMIPKit release commit."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile


PROJECT = Path(__file__).resolve().parents[1]
ROOT = PROJECT.parents[1]
DEFAULT_PIN = PROJECT / "library-pin.json"
DEFAULT_WHEELHOUSE = PROJECT / "build" / "wheelhouse"
DEFAULT_MANIFEST = PROJECT / "build" / "artifact-manifest.json"


def resolve_project_output(path: Path) -> Path:
    """Keep generated wheel artifacts and manifests within this validator."""
    candidate = Path(path)
    resolved = (candidate if candidate.is_absolute() else PROJECT / candidate).resolve()
    if not resolved.is_relative_to(PROJECT.resolve()):
        raise ValueError("artifact output must remain under the validator project")
    return resolved


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def build_wheel(repository: Path, commit: str, wheelhouse: Path) -> Path:
    wheelhouse.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="kmipkit-python-release-") as temporary:
        source = Path(temporary) / "source"
        source.mkdir()
        archive = subprocess.run(
            ["git", "-C", str(repository), "archive", "--format=tar", commit],
            check=True,
            capture_output=True,
        )
        with tarfile.open(fileobj=io.BytesIO(archive.stdout), mode="r:") as bundle:
            for member in bundle.getmembers():
                parts = Path(member.name).parts
                if Path(member.name).is_absolute() or ".." in parts:
                    raise ValueError("release archive contains an unsafe path")
            bundle.extractall(source, filter="data")
        package_root = source / "bindings" / "python"
        environment = os.environ.copy()
        environment["PATH"] = os.pathsep.join(
            (str(Path(sys.executable).parent), environment.get("PATH", ""))
        )
        subprocess.run(
            [
                sys.executable,
                "-m",
                "pip",
                "wheel",
                "--no-deps",
                "--no-build-isolation",
                "--wheel-dir",
                str(wheelhouse.resolve()),
                str(package_root),
            ],
            check=True,
            env=environment,
        )
    wheels = sorted(wheelhouse.glob("kmipkit-1.0.0-*.whl"), key=lambda item: item.stat().st_mtime_ns)
    if not wheels:
        raise RuntimeError("the pinned KMIPKit Python wheel was not produced")
    return wheels[-1]


def install(wheel: Path) -> None:
    subprocess.run(
        [sys.executable, "-m", "pip", "install", "--no-deps", "--force-reinstall", str(wheel.resolve())],
        check=True,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, default=ROOT)
    parser.add_argument("--pin-file", type=Path, default=DEFAULT_PIN)
    parser.add_argument("--wheelhouse", type=Path, default=DEFAULT_WHEELHOUSE)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    args = parser.parse_args()
    try:
        pin = json.loads(args.pin_file.resolve(strict=True).read_text(encoding="utf-8"))
        commit = pin["commit"]
        if pin.get("release_ref") != "release/1.0.0" or not isinstance(commit, str) or len(commit) != 40:
            raise ValueError("pin must select a full commit from release/1.0.0")
        resolved = subprocess.run(
            ["git", "-C", str(args.repository.resolve(strict=True)), "rev-parse", "--verify", f"{commit}^{{commit}}"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        if resolved != commit:
            raise ValueError("resolved library commit differs from the requested pin")
        wheelhouse = resolve_project_output(args.wheelhouse)
        manifest_path = resolve_project_output(args.manifest)
        wheel = build_wheel(args.repository.resolve(), commit, wheelhouse)
        install(wheel)
        manifest = {
            "schema_version": "1.0",
            "release_ref": "release/1.0.0",
            "library_commit": commit,
            "package_version": "1.0.0",
            "artifact_file": str(wheel.resolve()),
            "artifact_sha256": digest(wheel),
        }
        manifest_path.parent.mkdir(parents=True, exist_ok=True)
        manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    except (KeyError, OSError, ValueError, RuntimeError, subprocess.CalledProcessError, json.JSONDecodeError) as error:
        print(f"Python artifact installation failed: {type(error).__name__}", file=sys.stderr)
        return 1
    print(json.dumps(manifest, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
