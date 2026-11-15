"""Run isolated, offline consumers against an installed KMIPKit wheel."""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path


PROBE = r"""
from pathlib import Path
import socket
import sys

mode, audit_path = sys.argv[1:3]

def deny_network(*_args, **_kwargs):
    Path(audit_path).write_text("network attempted", encoding="utf-8")
    raise RuntimeError("offline package consumer")

socket.socket.connect = deny_network
socket.socket.connect_ex = deny_network
socket.socket.sendto = deny_network
socket.create_connection = deny_network
socket.getaddrinfo = deny_network
for operation in ("bind", "listen", "accept"):
    if hasattr(socket.socket, operation):
        setattr(socket.socket, operation, deny_network)

try:
    from kmipkit.metadata import get_version
    from kmipkit.client import BatchItemResult, Pkcs11Response
    from kmipkit.extensions import client_batch_item_pkcs11
    version = get_version()
    if (not callable(client_batch_item_pkcs11)
            or not hasattr(BatchItemResult, "pkcs11_response")
            or not callable(Pkcs11Response.close)):
        raise RuntimeError("The installed Python package is missing its public PKCS#11 API.")
except BaseException as failure:
    print("PACKAGE_NATIVE_LOAD_ERROR " + str(failure))
    sys.exit(2)

if mode != "success":
    print("PACKAGE_NATIVE_LOAD_UNEXPECTEDLY_SUCCEEDED")
    sys.exit(3)

print(f"PACKAGE_NATIVE_LOAD_OK ABI={version.abi_version}")
"""

def verify_public_pkcs11_api_present(package_root: Path) -> None:
    client_source = package_root / "client.py"
    extension_source = package_root / "extensions.py"
    if not client_source.is_file() or not extension_source.is_file():
        raise RuntimeError("The installed Python wheel is missing its public PKCS#11 API modules.")
    client = client_source.read_text(encoding="utf-8")
    extensions = extension_source.read_text(encoding="utf-8")
    if "class Pkcs11Response" not in client or "pkcs11_response" not in client:
        raise RuntimeError("The installed Python wheel is missing its public PKCS#11 response API.")
    if "def client_batch_item_pkcs11" not in extensions:
        raise RuntimeError("The installed Python wheel is missing its public PKCS#11 request API.")


def run_probe(python: Path, mode: str, audit_path: Path) -> subprocess.CompletedProcess[str]:
    audit_path.unlink(missing_ok=True)
    environment = os.environ.copy()
    environment.pop("KMIPKIT_NATIVE_LIBRARY", None)
    return subprocess.run(
        [str(python), "-I", "-c", PROBE, mode, str(audit_path)],
        check=False,
        capture_output=True,
        errors="replace",
        text=True,
        env=environment,
    )


def require_no_network(result: subprocess.CompletedProcess[str], audit_path: Path) -> None:
    if audit_path.exists() or "offline package consumer" in result.stdout + result.stderr:
        raise RuntimeError("The installed Python package attempted runtime network access.")


def require_output(result: subprocess.CompletedProcess[str], marker: str, *, status: int) -> None:
    output = result.stdout + result.stderr
    if result.returncode != status or marker not in output:
        raise RuntimeError(f"Installed Python package smoke did not produce {marker}.")


def package_native_directory(python: Path) -> Path:
    result = subprocess.run(
        [str(python), "-I", "-c", "import sysconfig; print(sysconfig.get_path('purelib'))"],
        check=True,
        capture_output=True,
        errors="replace",
        text=True,
    )
    return Path(result.stdout.strip()) / "kmipkit" / "_native_ffi"


def run_smokes(consumer_python: Path, wrong_abi_library: Path, smoke_root: Path) -> None:
    native_directory = package_native_directory(consumer_python)
    verify_public_pkcs11_api_present(native_directory.parent)
    libraries = sorted(
        path
        for path in native_directory.iterdir()
        if path.suffix.lower() in {".dll", ".dylib", ".so"}
        and path.is_file()
        and not path.is_symlink()
    )
    if len(libraries) != 1:
        raise RuntimeError("The installed wheel did not contain exactly one native library.")
    bundled_library = libraries[0]
    audit_path = smoke_root / "network-attempted.txt"

    success = run_probe(consumer_python, "success", audit_path)
    require_no_network(success, audit_path)
    require_output(success, "PACKAGE_NATIVE_LOAD_OK ABI=1", status=0)

    backup = bundled_library.with_name(bundled_library.name + ".t028-backup")
    bundled_library.replace(backup)
    try:
        missing = run_probe(consumer_python, "missing", audit_path)
        require_no_network(missing, audit_path)
        require_output(missing, "install a matching package", status=2)
        require_output(missing, "KMIPKIT_NATIVE_LIBRARY", status=2)
        require_redacted(missing, native_directory, wrong_abi_library)
    finally:
        backup.replace(bundled_library)

    original_contents = bundled_library.read_bytes()
    try:
        shutil.copyfile(wrong_abi_library, bundled_library)
        mismatch = run_probe(consumer_python, "wrong-abi", audit_path)
        require_no_network(mismatch, audit_path)
        require_output(mismatch, "incompatible ABI version", status=2)
        require_output(mismatch, "KMIPKIT_NATIVE_LIBRARY", status=2)
        require_redacted(mismatch, native_directory, wrong_abi_library)
    finally:
        bundled_library.write_bytes(original_contents)


def require_redacted(
    result: subprocess.CompletedProcess[str], native_directory: Path, wrong_abi_library: Path
) -> None:
    output = result.stdout + result.stderr
    if str(native_directory) in output or str(wrong_abi_library) in output:
        raise RuntimeError("The installed Python package exposed a native path in its load error.")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--consumer-python", required=True, type=Path)
    parser.add_argument("--wrong-abi-library", required=True, type=Path)
    parser.add_argument("--smoke-root", required=True, type=Path)
    arguments = parser.parse_args()
    arguments.smoke_root.mkdir(parents=True, exist_ok=True)
    run_smokes(arguments.consumer_python.absolute(), arguments.wrong_abi_library.resolve(), arguments.smoke_root)
    print("installed Python wheel load, missing-library, and ABI mismatch probes passed offline")


if __name__ == "__main__":
    main()
