#!/usr/bin/env python3
"""Build and install the public C ABI from the exact interop release commit."""
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile

PIN = "9692057c9ade474fb0c4e0b7130539da368eb794"
ROOT = Path(__file__).resolve().parents[3]
LOCAL = ROOT / "interop/.local/c-live"
BUNDLE = LOCAL / f"kmipkit-c-1.0.0-{PIN[:12]}.tar.gz"
INSTALL = LOCAL / "install" / PIN


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def build() -> None:
    with tempfile.TemporaryDirectory(prefix="kmipkit-c-package-") as directory:
        tree = Path(directory)
        archive = subprocess.check_output(["git", "-C", str(ROOT), "archive", PIN])
        with tarfile.open(fileobj=io.BytesIO(archive)) as source:
            source.extractall(tree, filter="data")
        target = tree / "target"
        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = str(target)
        subprocess.run(
            ["cargo", "+1.94", "build", "--locked", "-p", "kmipkit-ffi"],
            cwd=tree,
            env=env,
            check=True,
        )
        libraries = [
            target / "debug/libkmipkit_ffi.so",
            target / "debug/libkmipkit_ffi.dylib",
            target / "debug/kmipkit_ffi.dll",
        ]
        library = next((path for path in libraries if path.is_file()), None)
        if library is None:
            raise SystemExit("Cargo did not produce the KMIPKit C ABI shared library")
        stage = tree / "distribution"
        (stage / "lib").mkdir(parents=True)
        (stage / "include").mkdir(parents=True)
        shutil.copy2(library, stage / "lib" / library.name)
        shutil.copytree(tree / "bindings/c/include", stage / "include", dirs_exist_ok=True)
        (stage / "library-commit").write_text(PIN + "\n")
        hashes = {
            path.relative_to(stage).as_posix(): sha256(path)
            for path in sorted(stage.rglob("*"))
            if path.is_file()
        }
        manifest = {"library_commit": PIN, "files": hashes}
        (stage / "provenance.json").write_text(json.dumps(manifest, indent=2) + "\n")
        with tarfile.open(BUNDLE, "w:gz") as output:
            for member in sorted(stage.iterdir()):
                output.add(member, arcname=member.name)


def install() -> None:
    LOCAL.mkdir(parents=True, exist_ok=True)
    if not BUNDLE.exists():
        build()
    INSTALL.mkdir(parents=True, exist_ok=True)
    with tarfile.open(BUNDLE) as source:
        source.extractall(INSTALL, filter="data")
    installed = INSTALL / "provenance.json"
    data = json.loads(installed.read_text())
    if data.get("library_commit") != PIN:
        raise SystemExit("installed C artifact has the wrong release identity")
    if (INSTALL / "library-commit").read_text().strip() != PIN:
        raise SystemExit("C package release identity mismatch")
    for name, expected in data["files"].items():
        if sha256(INSTALL / name) != expected:
            raise SystemExit(f"C package file digest mismatch: {name}")
    print(json.dumps({
        "library_commit": PIN,
        "artifact_sha256": sha256(BUNDLE),
        "files": data["files"],
    }, sort_keys=True))


if __name__ == "__main__":
    install()
