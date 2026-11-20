"""Build the official OpenSSL release in GitHub Actions' temporary directory."""

import hashlib
import os
import platform
import shutil
import subprocess
import tarfile
import tempfile
import urllib.request
from pathlib import Path

VERSION = "3.5.9"
SHA256 = "603f5602e2eef00d77fbd429d34dcd5822bb301757a1bc9cdb24c670f1eb859a"


def run_windows(source: Path, options: list[str]) -> None:
    vswhere = Path(os.environ["ProgramFiles(x86)"]) / "Microsoft Visual Studio/Installer/vswhere.exe"
    vs = subprocess.check_output(
        [str(vswhere), "-latest", "-products", "*", "-requires",
         "Microsoft.VisualStudio.Component.VC.Tools.x86.x64", "-property", "installationPath"],
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
    build = source / "build-kmipkit.cmd"
    configure = subprocess.list2cmdline(["perl", "Configure", "VC-WIN64A", *options])
    build.write_text(
        f'@echo off\ncall "{vcvars}" || exit /b 1\n'
        f'{configure} || exit /b 1\n'
        'nmake build_sw || exit /b 1\nnmake install_sw || exit /b 1\n',
        encoding="utf-8",
    )
    subprocess.run(["cmd.exe", "/d", "/c", build.name], cwd=source, check=True)


def build() -> None:
    runner_temp = Path(os.environ["RUNNER_TEMP"])
    if os.name == "nt":
        # Keep paths short for the Windows build tools.
        runner_temp = Path(os.environ["SystemDrive"] + "/")
    work = Path(tempfile.mkdtemp(prefix="kmip-ossl-", dir=runner_temp))
    install = work / "install"
    archive = work / "openssl.tar.gz"
    release = f"openssl-{VERSION}"
    url = f"https://github.com/openssl/openssl/releases/download/{release}/{release}.tar.gz"
    url_request = urllib.request.Request(url, headers={"User-Agent": "KMIPKit CI"})
    with urllib.request.urlopen(url_request, timeout=60) as response, archive.open("wb") as output:
        shutil.copyfileobj(response, output)
    with archive.open("rb") as data:
        if hashlib.file_digest(data, "sha256").hexdigest() != SHA256:
            raise RuntimeError("OpenSSL archive SHA-256 mismatch")
    with tarfile.open(archive, "r:gz") as package:
        package.extractall(work, filter="data")

    source = work / release
    options = ["no-shared", "no-tests", "--libdir=lib", f"--prefix={install}"]
    if platform.system() == "Windows":
        if platform.machine().lower() not in {"amd64", "x86_64"}:
            raise RuntimeError("The Windows build requires an x64 runner")
        run_windows(source, options)
    else:
        subprocess.run(["perl", "Configure", *options], cwd=source, check=True)
        subprocess.run(["make", "-j", str(os.cpu_count() or 2), "build_sw"], cwd=source, check=True)
        subprocess.run(["make", "install_sw"], cwd=source, check=True)

    executable = "openssl.exe" if os.name == "nt" else "openssl"
    version = subprocess.check_output([str(install / "bin" / executable), "version"], text=True)
    if not version.startswith(f"OpenSSL {VERSION} "):
        raise RuntimeError(f"unexpected linked OpenSSL version: {version}")
    print(version.strip())
    with Path(os.environ["GITHUB_ENV"]).open("a", encoding="utf-8") as env:
        env.write(f"OPENSSL_DIR={install}\nOPENSSL_STATIC=1\n")


if __name__ == "__main__":
    build()
