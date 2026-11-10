#!/usr/bin/env python3
"""Build and install the public C ABI from the exact interop release commit."""
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile

from artifact_support import BUNDLE, INSTALL, LOCAL, PIN, ROOT, sha256, verify_installed_artifact


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
    data = verify_installed_artifact()
    print(json.dumps({
        "library_commit": PIN,
        "artifact_sha256": sha256(BUNDLE),
        "files": data["files"],
    }, sort_keys=True))


if __name__ == "__main__":
    install()
