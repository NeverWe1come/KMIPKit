#!/usr/bin/env python3
"""Package the KMIPKit C/C++ SDK (headers, libraries, CMake config, pkg-config)."""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import zipfile
from pathlib import Path

SEMVER_RE = re.compile(r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$")

TARGET_FILES = {
    "linux-glibc-x86_64": {
        "shared": ["libkmipkit_ffi.so"],
        "static": ["libkmipkit_ffi.a"],
        "archive": "tar.gz",
        "pkg_libs_private": "-ldl -lpthread -lm",
    },
    "linux-glibc-aarch64": {
        "shared": ["libkmipkit_ffi.so"],
        "static": ["libkmipkit_ffi.a"],
        "archive": "tar.gz",
        "pkg_libs_private": "-ldl -lpthread -lm",
    },
    "macos-aarch64": {
        "shared": ["libkmipkit_ffi.dylib"],
        "static": ["libkmipkit_ffi.a"],
        "archive": "tar.gz",
        "pkg_libs_private": "-framework Security -framework CoreFoundation -framework SystemConfiguration",
    },
    "macos-x86_64": {
        "shared": ["libkmipkit_ffi.dylib"],
        "static": ["libkmipkit_ffi.a"],
        "archive": "tar.gz",
        "pkg_libs_private": "-framework Security -framework CoreFoundation -framework SystemConfiguration",
    },
    "windows-x86_64-msvc": {
        "shared": ["kmipkit_ffi.dll", "kmipkit_ffi.dll.lib"],
        "static": ["kmipkit_ffi.lib"],
        "archive": "zip",
        "pkg_libs_private": "-lws2_32 -lcrypt32 -luserenv -ladvapi32 -lgdi32 -luser32 -lbcrypt",
    },
}


def render_cmake_config(target: str) -> str:
    spec = TARGET_FILES[target]
    shared_lib = spec["shared"][0]
    static_lib = spec["static"][0]
    if target == "windows-x86_64-msvc":
        shared_block = f"""\
if(NOT TARGET KMIPKit::kmipkit_ffi)
    add_library(KMIPKit::kmipkit_ffi SHARED IMPORTED)
    set_target_properties(KMIPKit::kmipkit_ffi PROPERTIES
        IMPORTED_LOCATION "${{_KMIPKIT_PREFIX}}/lib/{shared_lib}"
        IMPORTED_IMPLIB "${{_KMIPKIT_PREFIX}}/lib/kmipkit_ffi.dll.lib"
        INTERFACE_INCLUDE_DIRECTORIES "${{_KMIPKIT_PREFIX}}/include"
    )
endif()"""
    else:
        shared_block = f"""\
if(NOT TARGET KMIPKit::kmipkit_ffi)
    add_library(KMIPKit::kmipkit_ffi SHARED IMPORTED)
    set_target_properties(KMIPKit::kmipkit_ffi PROPERTIES
        IMPORTED_LOCATION "${{_KMIPKIT_PREFIX}}/lib/{shared_lib}"
        INTERFACE_INCLUDE_DIRECTORIES "${{_KMIPKIT_PREFIX}}/include"
    )
endif()"""

    static_block = f"""\
if(NOT TARGET KMIPKit::kmipkit_ffi_static)
    add_library(KMIPKit::kmipkit_ffi_static STATIC IMPORTED)
    set_target_properties(KMIPKit::kmipkit_ffi_static PROPERTIES
        IMPORTED_LOCATION "${{_KMIPKIT_PREFIX}}/lib/{static_lib}"
        INTERFACE_INCLUDE_DIRECTORIES "${{_KMIPKIT_PREFIX}}/include"
    )
endif()"""

    return (
        "# Generated KMIPKit C SDK CMake package configuration.\n"
        "get_filename_component(_KMIPKIT_PREFIX \"${CMAKE_CURRENT_LIST_DIR}/../../..\" ABSOLUTE)\n\n"
        + shared_block
        + "\n\n"
        + static_block
        + "\n"
    )


def render_cmake_version_config(version: str) -> str:
    return (
        f'set(PACKAGE_VERSION "{version}")\n'
        'if(PACKAGE_VERSION VERSION_LESS PACKAGE_FIND_VERSION)\n'
        "    set(PACKAGE_VERSION_COMPATIBLE FALSE)\n"
        "else()\n"
        "    set(PACKAGE_VERSION_COMPATIBLE TRUE)\n"
        '    if(PACKAGE_FIND_VERSION STREQUAL PACKAGE_VERSION)\n'
        "        set(PACKAGE_VERSION_EXACT TRUE)\n"
        "    endif()\n"
        "endif()\n"
    )


def render_pkg_config(version: str, target: str) -> str:
    libs_private = TARGET_FILES[target]["pkg_libs_private"]
    return (
        "prefix=${pcfiledir}/../..\n"
        "exec_prefix=${prefix}\n"
        "libdir=${exec_prefix}/lib\n"
        "includedir=${prefix}/include\n\n"
        "Name: kmipkit\n"
        "Description: KMIPKit C ABI library for KMIP 2.0, 2.1, and 3.0\n"
        f"Version: {version}\n"
        "Libs: -L${libdir} -lkmipkit_ffi\n"
        f"Libs.private: {libs_private}\n"
        "Cflags: -I${includedir}\n"
    )


def _create_archive(staging_root: Path, package_dir_name: str, archive_path: Path, fmt: str) -> None:
    archive_path.parent.mkdir(parents=True, exist_ok=True)
    if fmt == "tar.gz":
        with tarfile.open(archive_path, "w:gz") as tar:
            tar.add(staging_root / package_dir_name, arcname=package_dir_name)
    elif fmt == "zip":
        with zipfile.ZipFile(archive_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
            base = staging_root / package_dir_name
            for path in sorted(base.rglob("*")):
                if path.is_file():
                    zf.write(path, arcname=str(Path(package_dir_name) / path.relative_to(base)))
    else:
        raise ValueError(f"Unsupported archive format: {fmt}")


def package_c_sdk(
    repo_root: Path,
    target_dir: Path,
    output_dir: Path,
    version: str,
    target: str,
    smoke_test: bool = False,
) -> Path:
    if not SEMVER_RE.fullmatch(version):
        raise ValueError(f"Invalid SemVer version: {version!r}")
    if target not in TARGET_FILES:
        raise ValueError(f"Unsupported C SDK target: {target!r}")

    safe_repo = Path(os.path.realpath(repo_root))
    safe_target_dir = Path(os.path.realpath(target_dir))
    safe_output_dir = Path(os.path.realpath(output_dir))

    spec = TARGET_FILES[target]
    package_dir_name = f"kmipkit-c-{version}-{target}"
    archive_path = safe_output_dir / f"{package_dir_name}.{spec['archive']}"

    with tempfile.TemporaryDirectory() as tmp:
        staging_root = Path(tmp)
        pkg_root = staging_root / package_dir_name
        include_dst = pkg_root / "include"
        lib_dst = pkg_root / "lib"
        cmake_dst = lib_dst / "cmake" / "KMIPKit"
        pc_dst = lib_dst / "pkgconfig"

        include_dst.mkdir(parents=True, exist_ok=True)
        cmake_dst.mkdir(parents=True, exist_ok=True)
        pc_dst.mkdir(parents=True, exist_ok=True)

        include_src = safe_repo / "bindings" / "c" / "include"
        for header in sorted(include_src.glob("*.h")):
            shutil.copy2(header, include_dst / header.name)

        for lib_name in [*spec["shared"], *spec["static"]]:
            src_lib = safe_target_dir / lib_name
            if not src_lib.is_file():
                raise FileNotFoundError(f"Expected compiled FFI library not found: {src_lib}")
            shutil.copy2(src_lib, lib_dst / lib_name)

        (cmake_dst / "KMIPKitConfig.cmake").write_text(
            render_cmake_config(target), encoding="utf-8"
        )
        (cmake_dst / "KMIPKitConfigVersion.cmake").write_text(
            render_cmake_version_config(version), encoding="utf-8"
        )
        (pc_dst / "kmipkit.pc").write_text(
            render_pkg_config(version, target), encoding="utf-8"
        )

        for doc_name in ("LICENSE", "NOTICE"):
            doc_src = safe_repo / doc_name
            if doc_src.is_file():
                shutil.copy2(doc_src, pkg_root / doc_name)
        readme_src = safe_repo / "bindings" / "c" / "README.md"
        if readme_src.is_file():
            shutil.copy2(readme_src, pkg_root / "README.md")

        _create_archive(staging_root, package_dir_name, archive_path, spec["archive"])

    if smoke_test:
        run_smoke_test(archive_path, package_dir_name, spec["archive"])

    return archive_path


def run_smoke_test(archive_path: Path, package_dir_name: str, fmt: str) -> None:
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        unpack_dir = tmp_path / "sdk"
        unpack_dir.mkdir()
        if fmt == "tar.gz":
            with tarfile.open(archive_path, "r:gz") as tar:
                tar.extractall(unpack_dir, filter="data")
        else:
            with zipfile.ZipFile(archive_path, "r") as zf:
                zf.extractall(unpack_dir)

        sdk_prefix = unpack_dir / package_dir_name
        consumer_src = tmp_path / "consumer"
        consumer_src.mkdir()
        (consumer_src / "main.c").write_text(
            "#include <kmipkit.h>\n"
            "#include <stdint.h>\n"
            "#include <stdio.h>\n"
            "int main(void) {\n"
            "    uint32_t major = 0, minor = 0, patch = 0, abi = 0;\n"
            "    if (kmipkit_version_get(&major, &minor, &patch, &abi) != 0) {\n"
            "        return 1;\n"
            "    }\n"
            '    printf("kmipkit %u.%u.%u (ABI %u)\\n", major, minor, patch, abi);\n'
            "    return abi == 1 ? 0 : 1;\n"
            "}\n",
            encoding="utf-8",
        )
        (consumer_src / "CMakeLists.txt").write_text(
            "cmake_minimum_required(VERSION 3.20)\n"
            "project(kmipkit_sdk_smoke LANGUAGES C)\n"
            "find_package(KMIPKit REQUIRED)\n"
            "add_executable(smoke main.c)\n"
            "target_compile_features(smoke PRIVATE c_std_11)\n"
            "target_link_libraries(smoke PRIVATE KMIPKit::kmipkit_ffi)\n",
            encoding="utf-8",
        )
        build_dir = tmp_path / "build"
        prefix_arg = f"-DCMAKE_PREFIX_PATH={Path(os.path.realpath(sdk_prefix))}"
        subprocess.run(
            [
                "cmake",
                "-S",
                str(consumer_src),
                "-B",
                str(build_dir),
                prefix_arg,
            ],
            check=True,
        )
        subprocess.run(
            ["cmake", "--build", str(build_dir), "--config", "Release"],
            check=True,
        )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path("."))
    parser.add_argument("--target-dir", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--target", required=True, choices=sorted(TARGET_FILES))
    parser.add_argument("--smoke-test", action="store_true")
    args = parser.parse_args(argv)

    archive = package_c_sdk(
        repo_root=args.repo.resolve(),
        target_dir=args.target_dir.resolve(),
        output_dir=args.output_dir.resolve(),
        version=args.version,
        target=args.target,
        smoke_test=args.smoke_test,
    )
    print(f"Packaged C SDK: {archive}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
