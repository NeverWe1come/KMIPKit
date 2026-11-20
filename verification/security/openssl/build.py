"""Build the repository's checksum-pinned OpenSSL source for local or CI use."""

from __future__ import annotations

import argparse
import atexit
import hashlib
import json
import os
import platform
import shlex
import shutil
import subprocess
import tarfile
import tempfile
import urllib.request
from pathlib import Path

PIN_FILE = Path(__file__).with_name("native.json")


def load_pin() -> dict[str, str]:
    pin = json.loads(PIN_FILE.read_text(encoding="utf-8"))
    version = pin.get("version", "")
    series = pin.get("series", "")
    digest = pin.get("archive_sha256", "")
    if not version.startswith(f"{series}."):
        raise ValueError("pinned OpenSSL version is outside the declared release series")
    if len(digest) != 64 or any(char not in "0123456789abcdef" for char in digest):
        raise ValueError("native.json must contain a lowercase SHA-256 digest")
    return pin


def run_windows(source: Path, options: list[str]) -> None:
    vswhere = Path(os.environ["ProgramFiles(x86)"]) / "Microsoft Visual Studio/Installer/vswhere.exe"
    vs = subprocess.check_output(
        [
            str(vswhere), "-latest", "-products", "*", "-requires",
            "Microsoft.VisualStudio.Component.VC.Tools.x86.x64", "-property", "installationPath",
        ],
        text=True,
    ).strip()
    if not vs:
        raise RuntimeError("MSVC build tools are required")
    vcvars = Path(vs) / "VC/Auxiliary/Build/vcvars64.bat"
    if not shutil.which("nasm"):
        nasm = Path(os.environ["ProgramFiles"]) / "NASM"
        if (nasm / "nasm.exe").is_file():
            os.environ["PATH"] += os.pathsep + str(nasm)
        else:
            options.append("no-asm")
    build_script = source / "build-kmipkit.cmd"
    configure = subprocess.list2cmdline(["perl", "Configure", "VC-WIN64A", *options])
    build_script.write_text(
        f'@echo off\ncall "{vcvars}" || exit /b 1\n'
        f"{configure} || exit /b 1\n"
        "nmake build_sw || exit /b 1\nnmake install_sw || exit /b 1\n",
        encoding="utf-8",
    )
    subprocess.run(["cmd.exe", "/d", "/c", build_script.name], cwd=source, check=True)


def default_install_dir(version: str) -> Path:
    if os.name == "nt":
        local_app_data = Path(os.environ.get("LOCALAPPDATA", Path.home() / "AppData/Local"))
        return local_app_data / "KMIPKit" / "OpenSSL" / version
    return Path.home() / ".cache" / "kmipkit" / "openssl" / version


def write_environment(install: Path) -> None:
    environment_file = os.environ.get("GITHUB_ENV")
    if environment_file:
        with Path(environment_file).open("a", encoding="utf-8") as output:
            output.write(f"OPENSSL_DIR={install}\nOPENSSL_STATIC=1\n")
        return

    if os.name == "nt":
        escaped = str(install).replace("'", "''")
        print(f"PowerShell: $env:OPENSSL_DIR = '{escaped}'")
        print('PowerShell: $env:OPENSSL_STATIC = "1"')
    else:
        print(f"export OPENSSL_DIR={shlex.quote(str(install))}")
        print("export OPENSSL_STATIC=1")


def build(install_dir: Path | None) -> None:
    pin = load_pin()
    version = pin["version"]
    runner_temp = Path(os.environ.get("RUNNER_TEMP", tempfile.gettempdir()))
    if os.name == "nt" and os.environ.get("RUNNER_TEMP"):
        # Keep the source and intermediate paths short for Windows build tools.
        runner_temp = Path(os.environ["SystemDrive"] + "/")
    work = Path(tempfile.mkdtemp(prefix="kmip-ossl-", dir=runner_temp))
    install = install_dir or (
        work / "install" if os.environ.get("GITHUB_ENV") else default_install_dir(version)
    )
    if not install.resolve().is_relative_to(work.resolve()):
        atexit.register(shutil.rmtree, work, ignore_errors=True)
    if install.exists() and any(install.iterdir()):
        raise FileExistsError(f"installation directory must be empty: {install}")

    install.parent.mkdir(parents=True, exist_ok=True)
    staging = work / "stage"
    archive = work / "openssl.tar.gz"
    release = f"openssl-{version}"
    url = f"https://github.com/openssl/openssl/releases/download/{release}/{release}.tar.gz"
    request = urllib.request.Request(url, headers={"User-Agent": "KMIPKit OpenSSL build"})
    with urllib.request.urlopen(request, timeout=60) as response, archive.open("wb") as output:
        shutil.copyfileobj(response, output)
    with archive.open("rb") as data:
        if hashlib.file_digest(data, "sha256").hexdigest() != pin["archive_sha256"]:
            raise RuntimeError("OpenSSL archive SHA-256 mismatch")
    with tarfile.open(archive, "r:gz") as package:
        package.extractall(work, filter="data")

    source = work / release
    options = ["no-shared", "no-tests", "--libdir=lib", f"--prefix={staging}"]
    if platform.system() == "Windows":
        if platform.machine().lower() not in {"amd64", "x86_64"}:
            raise RuntimeError("The Windows build requires an x64 runner")
        run_windows(source, options)
    else:
        subprocess.run(["perl", "Configure", *options], cwd=source, check=True)
        subprocess.run(["make", "-j", str(os.cpu_count() or 2), "build_sw"], cwd=source, check=True)
        subprocess.run(["make", "install_sw"], cwd=source, check=True)

    executable = "openssl.exe" if os.name == "nt" else "openssl"
    openssl_version = subprocess.check_output(
        [str(staging / "bin" / executable), "version"], text=True
    )
    if not openssl_version.startswith(f"OpenSSL {version} "):
        raise RuntimeError(f"unexpected linked OpenSSL version: {openssl_version}")
    if install.exists():
        install.rmdir()
    shutil.move(str(staging), str(install))
    if not install.resolve().is_relative_to(work.resolve()):
        shutil.rmtree(work)
    print(openssl_version.strip())
    print(f"Installed checksum-verified OpenSSL {version} at {install}")
    write_environment(install)


def main() -> None:
    pin = load_pin()
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--install-dir",
        type=Path,
        help=(
            "installation prefix (defaults to the KMIPKit user cache locally "
            "or the GitHub Actions temporary directory in CI)"
        ),
    )
    parser.add_argument("--version", action="version", version=f"OpenSSL {pin['version']}")
    args = parser.parse_args()
    build(args.install_dir)


if __name__ == "__main__":
    main()
