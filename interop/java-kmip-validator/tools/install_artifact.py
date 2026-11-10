#!/usr/bin/env python3
"""Build and install the Java JNI package from one exact KMIPKit git commit."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path


PACKAGE_VERSION = "1.0.0"


def host_classifier() -> str:
    system = platform.system().lower()
    machine = platform.machine().lower()
    arch = "x86_64" if machine in {"x86_64", "amd64"} else "aarch64" if machine in {"aarch64", "arm64"} else ""
    if system == "linux" and arch:
        return f"linux-glibc-{arch}"
    if system == "darwin" and arch:
        return f"macos-{arch}"
    if system == "windows" and arch == "x86_64":
        return "windows-x86_64-msvc"
    raise ValueError("This host is outside the KMIPKit Java native target matrix")


def run_checked(command: list[str], *, cwd: Path, env: dict[str, str]) -> None:
    result = subprocess.run(command, cwd=cwd, env=env, text=True, capture_output=True, check=False)
    if result.returncode:
        if result.stdout:
            sys.stderr.write(result.stdout)
        if result.stderr:
            sys.stderr.write(result.stderr)
        raise RuntimeError(f"Build command failed with exit code {result.returncode}")
    if result.stdout:
        sys.stdout.write(result.stdout)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as artifact:
        for block in iter(lambda: artifact.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def resolve_pin_file(pin_file: Path, project_root: Path) -> Path:
    """Resolve the consumer's pin file and reject traversal and symlink escapes."""
    root = project_root.resolve(strict=True)
    resolved = pin_file.resolve(strict=True)
    try:
        resolved.relative_to(root)
    except ValueError as error:
        raise ValueError("--pin-file must resolve inside the Java validator project") from error
    if not resolved.is_file():
        raise ValueError("--pin-file must identify a regular file")
    return resolved


def extract_git_archive(repository: Path, commit: str, destination: Path) -> None:
    archive = subprocess.run(
        ["git", "-C", str(repository), "archive", "--format=tar", commit],
        check=True,
        capture_output=True,
    )
    with tarfile.open(fileobj=__import__("io").BytesIO(archive.stdout), mode="r:") as source:
        for entry in source.getmembers():
            parts = Path(entry.name).parts
            if Path(entry.name).is_absolute() or ".." in parts:
                raise ValueError("Git archive contains an unsafe path")
        source.extractall(destination, filter="data")


def install(repository: Path, commit: str, release_ref: str, maven_repository: Path,
            manifest_path: Path) -> dict[str, object]:
    if release_ref != "release/1.0.0":
        raise ValueError("The Java consumer artifact must come from release/1.0.0")
    if not re.fullmatch(r"[0-9a-f]{40}", commit):
        raise ValueError("--commit must be a full lowercase git commit id")
    resolved = subprocess.run(
        ["git", "-C", str(repository), "rev-parse", "--verify", f"{commit}^{{commit}}"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if resolved != commit:
        raise ValueError("Resolved commit differs from the requested exact commit")

    classifier = host_classifier()
    artifact_name = f"kmipkit-java-{PACKAGE_VERSION}-{classifier}.jar"
    coordinate_dir = maven_repository / "org" / "kmipkit" / "kmipkit-java" / PACKAGE_VERSION
    coordinate_dir.mkdir(parents=True, exist_ok=True)
    artifact_path = coordinate_dir / artifact_name

    with tempfile.TemporaryDirectory(prefix="kmipkit-java-source-") as temp_name:
        temp_root = Path(temp_name)
        source_root = temp_root / "source"
        source_root.mkdir()
        extract_git_archive(repository, commit, source_root)
        tool_home = temp_root / "tool-home"
        env = os.environ.copy()
        env["GRADLE_USER_HOME"] = str(tool_home / "gradle")
        env["CARGO_HOME"] = str(tool_home / "cargo")
        env["CARGO_TARGET_DIR"] = str(tool_home / "cargo-target")
        wrapper = source_root / "bindings" / "java" / "gradlew"
        if os.name == "nt":
            command = [str(wrapper.with_suffix(".bat")), "--no-daemon", "-p", str(wrapper.parent), "nativeJar"]
        else:
            command = ["bash", str(wrapper), "--no-daemon", "-p", str(wrapper.parent), "nativeJar"]
        run_checked(command, cwd=source_root, env=env)
        built_jar = source_root / "bindings" / "java" / "build" / "libs" / artifact_name
        if not built_jar.is_file():
            raise RuntimeError("The requested platform JNI JAR was not produced")
        shutil.copyfile(built_jar, artifact_path)

    pom = coordinate_dir / f"kmipkit-java-{PACKAGE_VERSION}.pom"
    pom.write_text(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n"
        "<project xmlns=\"http://maven.apache.org/POM/4.0.0\">"
        "<modelVersion>4.0.0</modelVersion><groupId>org.kmipkit</groupId>"
        "<artifactId>kmipkit-java</artifactId><version>1.0.0</version>"
        "<packaging>jar</packaging></project>\n",
        encoding="utf-8",
    )
    manifest: dict[str, object] = {
        "schema_version": "1.0",
        "release_ref": release_ref,
        "library_commit": commit,
        "maven_coordinate": f"org.kmipkit:kmipkit-java:{PACKAGE_VERSION}:{classifier}",
        "native_classifier": classifier,
        "artifact_file": str(artifact_path.resolve()),
        "artifact_sha256": sha256(artifact_path),
    }
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return manifest


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--pin-file", type=Path, required=True)
    parser.add_argument("--maven-repository", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    args = parser.parse_args()
    try:
        project_root = Path(__file__).resolve().parents[1]
        pin_path = resolve_pin_file(args.pin_file, project_root)
        pin = json.loads(pin_path.read_text(encoding="utf-8"))
        result = install(args.repository.resolve(), pin["commit"], pin["release_ref"],
                         args.maven_repository.resolve(), args.manifest.resolve())
    except (OSError, ValueError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"artifact installation failed: {error}", file=sys.stderr)
        return 1
    print(f"Installed {result['maven_coordinate']} from {result['library_commit']}")
    print(f"SHA-256: {result['artifact_sha256']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
