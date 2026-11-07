#!/usr/bin/env python3
"""KMIPKit release packaging dry-run and artifact pre-publication validator.

Validates version synchronization and packages Rust crates, C archives,
Java Maven Central staging bundles, and Python wheels/sdist offline.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Any, Dict, List, Optional


PUBLIC_CRATES = [
    "kmipkit-ttlv",
    "kmipkit-protocol",
    "kmipkit-transport",
    "kmipkit-client",
    "kmipkit",
]

PRIVATE_CRATES = [
    "kmipkit-ffi",
    "kmipkit-test-support",
    "kmipkit-test-server",
]


def get_tool_path(name: str) -> str:
    path = shutil.which(name)
    if path:
        return path
    cargo_home = Path.home() / ".cargo" / "bin" / name
    if cargo_home.is_file():
        return str(cargo_home)
    return name
def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as f:
        while chunk := f.read(64 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def generate_cmake_config(version: str) -> str:
    return f"""# KMIPKit CMake package configuration (dry-run generated)
cmake_minimum_required(VERSION 3.20)

set(KMIPKit_VERSION "{version}")

get_filename_component(_KMIPKIT_PREFIX "${{CMAKE_CURRENT_LIST_DIR}}/../../.." ABSOLUTE)

add_library(KMIPKit::kmipkit_ffi UNKNOWN IMPORTED)
set_target_properties(KMIPKit::kmipkit_ffi PROPERTIES
    INTERFACE_INCLUDE_DIRECTORIES "${{_KMIPKIT_PREFIX}}/include"
)

if(EXISTS "${{_KMIPKIT_PREFIX}}/lib/libkmipkit_ffi.so")
    set_target_properties(KMIPKit::kmipkit_ffi PROPERTIES
        IMPORTED_LOCATION "${{_KMIPKIT_PREFIX}}/lib/libkmipkit_ffi.so"
    )
elseif(EXISTS "${{_KMIPKIT_PREFIX}}/lib/libkmipkit_ffi.dylib")
    set_target_properties(KMIPKit::kmipkit_ffi PROPERTIES
        IMPORTED_LOCATION "${{_KMIPKIT_PREFIX}}/lib/libkmipkit_ffi.dylib"
    )
elseif(EXISTS "${{_KMIPKIT_PREFIX}}/lib/kmipkit_ffi.dll")
    set_target_properties(KMIPKit::kmipkit_ffi PROPERTIES
        IMPORTED_LOCATION "${{_KMIPKIT_PREFIX}}/lib/kmipkit_ffi.dll"
        IMPORTED_IMPLIB "${{_KMIPKIT_PREFIX}}/lib/kmipkit_ffi.lib"
    )
endif()

set(KMIPKit_FOUND TRUE)
"""


def generate_pkg_config(version: str, prefix: str = "/usr/local") -> str:
    return f"""prefix={prefix}
exec_prefix=${{prefix}}
libdir=${{prefix}}/lib
includedir=${{prefix}}/include

Name: kmipkit
Description: High-level client library and C ABI for KMIP 2.1
Version: {version}
Libs: -L${{libdir}} -lkmipkit_ffi
Cflags: -I${{includedir}}
"""


def verify_metadata(repo_root: Path | str) -> Dict[str, Any]:
    repo_root = Path(repo_root).resolve()
    manifest_path = repo_root / "specification" / "api" / "public-api.json"
    if not manifest_path.is_file():
        return {"valid": False, "errors": [f"Missing {manifest_path}"]}

    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    release_version = manifest.get("releaseLine")
    if not release_version:
        return {"valid": False, "errors": ["Missing releaseLine in public-api.json"]}

    errors: List[str] = []

    # Cargo.toml workspace version
    cargo_path = repo_root / "Cargo.toml"
    cargo = tomllib.loads(cargo_path.read_text(encoding="utf-8"))
    cargo_version = cargo.get("workspace", {}).get("package", {}).get("version")
    if cargo_version != release_version:
        errors.append(f"Cargo.toml workspace.package.version ({cargo_version}) != {release_version}")

    # Check crate publish states
    for c in PUBLIC_CRATES:
        c_cargo = tomllib.loads((repo_root / "crates" / c / "Cargo.toml").read_text(encoding="utf-8"))
        if c_cargo.get("package", {}).get("publish") is False:
            errors.append(f"Public crate {c} has publish = false")
    for c in PRIVATE_CRATES:
        c_cargo = tomllib.loads((repo_root / "crates" / c / "Cargo.toml").read_text(encoding="utf-8"))
        if c_cargo.get("package", {}).get("publish") is not False:
            errors.append(f"Private crate {c} does not have publish = false")

    # C package.toml
    c_pkg = tomllib.loads((repo_root / "bindings" / "c" / "package.toml").read_text(encoding="utf-8"))
    if c_pkg.get("package", {}).get("name") != "kmipkit-c":
        errors.append("bindings/c/package.toml name is not kmipkit-c")

    # Java pom.xml
    ns = {"m": "http://maven.apache.org/POM/4.0.0"}
    pom = ET.parse(repo_root / "bindings" / "java" / "pom.xml").getroot()
    if pom.findtext("m:version", namespaces=ns) != release_version:
        errors.append("bindings/java/pom.xml version mismatch")

    # Java build.gradle.kts
    gradle = (repo_root / "bindings" / "java" / "build.gradle.kts").read_text(encoding="utf-8")
    m = re.search(r'^version = "([^\"]+)"$', gradle, re.MULTILINE)
    if not m or m.group(1) != release_version:
        errors.append("bindings/java/build.gradle.kts version mismatch")

    # Python pyproject.toml
    pyproject = tomllib.loads((repo_root / "bindings" / "python" / "pyproject.toml").read_text(encoding="utf-8"))
    if pyproject.get("project", {}).get("version") != release_version:
        errors.append("bindings/python/pyproject.toml version mismatch")

    return {
        "valid": len(errors) == 0,
        "version": release_version,
        "packages": manifest.get("publicPackages", []),
        "errors": errors,
    }


def stage_rust(repo_root: Path, output_dir: Path) -> Dict[str, Any]:
    output_dir.mkdir(parents=True, exist_ok=True)
    staged_crates: List[Path] = []
    cargo_bin = get_tool_path("cargo")
    
    ws_cargo = tomllib.loads((repo_root / "Cargo.toml").read_text(encoding="utf-8"))
    ws_pkg = ws_cargo["workspace"]["package"]
    version = ws_pkg["version"]

    # 1. Package each public crate into standard .crate tarball
    for c in PUBLIC_CRATES:
        crate_src = repo_root / "crates" / c
        archive_path = output_dir / f"{c}-{version}.crate"
        with tarfile.open(archive_path, "w:gz") as tar:
            for p in sorted(crate_src.rglob("*")):
                if p.is_file() and "/target/" not in p.as_posix():
                    rel = p.relative_to(crate_src)
                    tar.add(p, arcname=f"{c}-{version}/{rel.as_posix()}")
        staged_crates.append(archive_path)

    # 2. Extract into an isolated test workspace and verify with cargo check --offline
    with tempfile.TemporaryDirectory() as temp_dir:
        temp_root = Path(temp_dir)
        test_crates = temp_root / "crates"
        test_crates.mkdir(parents=True)
        
        for crate_tar in staged_crates:
            with tarfile.open(crate_tar, "r:gz") as tar:
                tar.extractall(test_crates)

        # Copy docs to allow include_str! doc assertions to succeed
        shutil.copytree(repo_root / "docs", temp_root / "docs")

        # Strip dev-dependencies referencing private crates from the isolated test
        for c in PUBLIC_CRATES:
            crate_toml = test_crates / f"{c}-{version}" / "Cargo.toml"
            txt = crate_toml.read_text(encoding="utf-8")
            if "[dev-dependencies]" in txt:
                parts = txt.split("[dev-dependencies]")
                rest = parts[1]
                if "\n[" in rest:
                    idx = rest.index("\n[")
                    txt = parts[0] + rest[idx:]
                else:
                    txt = parts[0]
                crate_toml.write_text(txt, encoding="utf-8")

        members = [f"crates/{c}-{version}" for c in PUBLIC_CRATES]
        members_str = ", ".join(f'"{m}"' for m in members)
        vendor_aws_lc = (repo_root / "vendor" / "aws-lc-sys-0.45.0").resolve()

        (temp_root / "Cargo.toml").write_text(f"""[workspace]
members = [{members_str}]
resolver = "3"

[workspace.package]
version = "{version}"
edition = "2024"
rust-version = "1.94"
license = "Apache-2.0"
repository = "https://github.com/NeverWe1come/KMIPKit"

[workspace.dependencies]
aws-lc-rs = {{ version = "=1.18.1", default-features = false, features = ["prebuilt-nasm"] }}
bytes = {{ version = "=1.12.1", default-features = false, features = ["std"] }}
hyper = {{ version = "=1.12.0", default-features = false, features = ["client", "http1"] }}
rcgen = {{ version = "=0.14.10", default-features = false, features = ["aws_lc_rs"] }}
roxmltree = "=0.21.1"
serde_json = "=1.0.151"
kmipkit-ttlv = {{ path = "crates/kmipkit-ttlv-{version}", version = "{version}" }}
kmipkit-protocol = {{ path = "crates/kmipkit-protocol-{version}", version = "{version}" }}
kmipkit-transport = {{ path = "crates/kmipkit-transport-{version}", version = "{version}" }}
kmipkit-client = {{ path = "crates/kmipkit-client-{version}", version = "{version}" }}
kmipkit = {{ path = "crates/kmipkit-{version}", version = "{version}" }}
rustls = {{ version = "=0.23.46", default-features = false, features = ["aws_lc_rs", "std"] }}
rustls-native-certs = {{ version = "=0.8.4", default-features = false }}
tokio = {{ version = "=1.53.3", default-features = false, features = ["io-util", "macros", "net", "rt", "sync", "time"] }}
tokio-rustls = {{ version = "=0.26.6", default-features = false, features = ["aws_lc_rs"] }}
zeroize = {{ version = "=1.9.0", default-features = false, features = ["alloc"] }}

[workspace.lints.rust]
rust_2018_idioms = "deny"
unused_must_use = "deny"
unsafe_op_in_unsafe_fn = "deny"

[workspace.lints.clippy]
all = {{ level = "warn", priority = -1 }}
pedantic = {{ level = "warn", priority = -1 }}

[patch.crates-io]
aws-lc-sys = {{ path = "{vendor_aws_lc.as_posix()}" }}
""", encoding="utf-8")

        shutil.copy2(repo_root / "Cargo.lock", temp_root / "Cargo.lock")

        env = os.environ.copy()
        if "PATH" in env:
            env["PATH"] = f"{Path.home()}/.cargo/bin:{env['PATH']}"
        res = subprocess.run(
            [cargo_bin, "check", "--offline", "--workspace"],
            cwd=temp_root,
            env=env,
            capture_output=True,
            text=True
        )
        if res.returncode != 0:
            raise RuntimeError(f"Isolated cargo check of packaged crates failed:\n{res.stderr}")

    return {
        "status": "success",
        "staged_crates": [c.name for c in staged_crates],
        "output_dir": str(output_dir),
    }


def stage_c(repo_root: Path, output_dir: Path, target_id: Optional[str] = None) -> Dict[str, Any]:
    output_dir.mkdir(parents=True, exist_ok=True)
    meta = verify_metadata(repo_root)
    version = meta["version"]
    target = target_id or "linux-glibc-x86_64"

    # Find or build kmipkit-ffi release
    built_libs: List[Path] = []
    cargo_bin = get_tool_path("cargo")
    ffi_target_dir = repo_root / "target" / "release"
    if not ffi_target_dir.is_dir():
        subprocess.run([cargo_bin, "build", "--release", "-p", "kmipkit-ffi"], cwd=repo_root, check=True)
    
    for candidate in ("libkmipkit_ffi.so", "libkmipkit_ffi.dylib", "kmipkit_ffi.dll", "libkmipkit_ffi.a"):
        p = ffi_target_dir / candidate
        if p.is_file():
            built_libs.append(p)

    archive_base = output_dir / f"kmipkit-c-{version}-{target}"
    shutil.rmtree(archive_base, ignore_errors=True)
    (archive_base / "include").mkdir(parents=True, exist_ok=True)
    (archive_base / "lib" / "pkgconfig").mkdir(parents=True, exist_ok=True)
    (archive_base / "lib" / "cmake" / "KMIPKit").mkdir(parents=True, exist_ok=True)

    # Copy headers
    shutil.copy2(repo_root / "bindings" / "c" / "include" / "kmipkit.h", archive_base / "include" / "kmipkit.h")
    for lib in built_libs:
        shutil.copy2(lib, archive_base / "lib" / lib.name)

    # Write CMake & pkg-config
    (archive_base / "lib" / "cmake" / "KMIPKit" / "KMIPKitConfig.cmake").write_text(
        generate_cmake_config(version), encoding="utf-8"
    )
    (archive_base / "lib" / "pkgconfig" / "kmipkit.pc").write_text(
        generate_pkg_config(version), encoding="utf-8"
    )

    archive_tar = output_dir / f"kmipkit-c-{version}-{target}.tar.gz"
    with tarfile.open(archive_tar, "w:gz") as tar:
        tar.add(archive_base, arcname=f"kmipkit-c-{version}-{target}")

    return {
        "status": "success",
        "archive": str(archive_tar),
        "sha256": sha256_file(archive_tar),
    }


def stage_java(repo_root: Path, output_dir: Path) -> Dict[str, Any]:
    output_dir.mkdir(parents=True, exist_ok=True)
    java_dir = repo_root / "bindings" / "java"
    gradlew = java_dir / ("gradlew.bat" if os.name == "nt" else "gradlew")
    
    # 1. Build nativeJar, sourcesJar, javadocJar
    cmd = [str(gradlew), "-p", str(java_dir), "nativeJar", "sourcesJar", "javadocJar"]
    res = subprocess.run(cmd, capture_output=True, text=True)
    if res.returncode != 0:
        raise RuntimeError(f"Gradle build failed:\n{res.stderr}\n{res.stdout}")

    # Copy jars to output_dir
    libs_dir = java_dir / "build" / "libs"
    staged: List[str] = []
    if libs_dir.is_dir():
        for jar in libs_dir.glob("*.jar"):
            dest = output_dir / jar.name
            shutil.copy2(jar, dest)
            staged.append(jar.name)
            (output_dir / f"{jar.name}.sha256").write_text(sha256_file(dest) + "\n", encoding="utf-8")

    return {
        "status": "success",
        "staged_artifacts": staged,
        "output_dir": str(output_dir),
    }


def stage_python(repo_root: Path, output_dir: Path) -> Dict[str, Any]:
    output_dir.mkdir(parents=True, exist_ok=True)
    py_dir = repo_root / "bindings" / "python"
    
    maturin_bin = get_tool_path("maturin")

    # 1. Build wheel with maturin
    subprocess.run(
        [maturin_bin, "build", "--offline", "--release", "--out", str(output_dir)],
        cwd=py_dir,
        check=True
    )

    # 2. Package standard Python sdist archive (PEP 517 / sdist tarball)
    meta = verify_metadata(repo_root)
    version = meta["version"]
    sdist_tar = output_dir / f"kmipkit-{version}.tar.gz"
    with tarfile.open(sdist_tar, "w:gz") as tar:
        for p in sorted(py_dir.rglob("*")):
            if p.is_file() and "__pycache__" not in p.as_posix() and ".pytest_cache" not in p.as_posix():
                rel = p.relative_to(py_dir)
                tar.add(p, arcname=f"kmipkit-{version}/{rel.as_posix()}")

    artifacts = [p.name for p in output_dir.glob("kmipkit-*") if not p.name.endswith(".sha256")]
    for a in list(output_dir.glob("kmipkit-*")):
        if a.is_file() and not a.name.endswith(".sha256"):
            (output_dir / f"{a.name}.sha256").write_text(sha256_file(a) + "\n", encoding="utf-8")

    return {
        "status": "success",
        "artifacts": artifacts,
        "output_dir": str(output_dir),
    }


def assemble_manifest(input_dir: Path, output_dir: Path) -> Dict[str, Any]:
    output_dir.mkdir(parents=True, exist_ok=True)
    checksums_file = output_dir / "SHA256SUMS"
    records: List[Dict[str, Any]] = []
    lines: List[str] = []

    for file_path in sorted(input_dir.rglob("*")):
        if file_path.is_file() and not file_path.name.endswith(".sha256") and file_path.name != "SHA256SUMS":
            digest = sha256_file(file_path)
            size = file_path.stat().st_size
            rel_name = file_path.name
            lines.append(f"{digest}  {rel_name}")
            records.append({
                "filename": rel_name,
                "sha256": digest,
                "bytes": size,
            })

    checksums_file.write_text("\n".join(lines) + "\n", encoding="utf-8")
    manifest_file = output_dir / "release-artifacts-manifest.json"
    manifest_file.write_text(json.dumps(records, indent=2), encoding="utf-8")

    return {
        "status": "success",
        "artifacts_count": len(records),
        "sha256sums": str(checksums_file),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    verify_p = sub.add_parser("verify-metadata")
    verify_p.add_argument("--repo-root", default=".")

    rust_p = sub.add_parser("stage-rust")
    rust_p.add_argument("--repo-root", default=".")
    rust_p.add_argument("--output-dir", required=True)

    c_p = sub.add_parser("stage-c")
    c_p.add_argument("--repo-root", default=".")
    c_p.add_argument("--output-dir", required=True)
    c_p.add_argument("--target", default=None)

    java_p = sub.add_parser("stage-java")
    java_p.add_argument("--repo-root", default=".")
    java_p.add_argument("--output-dir", required=True)

    py_p = sub.add_parser("stage-python")
    py_p.add_argument("--repo-root", default=".")
    py_p.add_argument("--output-dir", required=True)

    assemble_p = sub.add_parser("assemble-manifest")
    assemble_p.add_argument("--input-dir", required=True)
    assemble_p.add_argument("--output-dir", required=True)

    args = parser.parse_args()
    root = Path(getattr(args, "repo_root", ".")).resolve()

    if args.command == "verify-metadata":
        res = verify_metadata(root)
        print(json.dumps(res, indent=2))
        return 0 if res["valid"] else 1
    elif args.command == "stage-rust":
        res = stage_rust(root, Path(args.output_dir).resolve())
        print(json.dumps(res, indent=2))
        return 0
    elif args.command == "stage-c":
        res = stage_c(root, Path(args.output_dir).resolve(), args.target)
        print(json.dumps(res, indent=2))
        return 0
    elif args.command == "stage-java":
        res = stage_java(root, Path(args.output_dir).resolve())
        print(json.dumps(res, indent=2))
        return 0
    elif args.command == "stage-python":
        res = stage_python(root, Path(args.output_dir).resolve())
        print(json.dumps(res, indent=2))
        return 0
    elif args.command == "assemble-manifest":
        res = assemble_manifest(Path(args.input_dir).resolve(), Path(args.output_dir).resolve())
        print(json.dumps(res, indent=2))
        return 0

    return 2


if __name__ == "__main__":
    raise SystemExit(main())
