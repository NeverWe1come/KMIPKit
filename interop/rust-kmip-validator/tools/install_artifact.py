#!/usr/bin/env python3
"""Build and install Rust distribution packages from one immutable release tree."""
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile

PIN = "fdafef8a3eb0c7b4b18d6433b78fde4151493ceb"
ROOT = Path(__file__).resolve().parents[3]
LOCAL = ROOT / "interop/.local/rust-live"
CRATES = ["kmipkit-ttlv", "kmipkit-protocol", "kmipkit-transport", "kmipkit-client", "kmipkit"]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def install():
    LOCAL.mkdir(parents=True, exist_ok=True)
    bundle = LOCAL / "kmipkit-rust-1.0.0.tar.gz"
    provenance = LOCAL / "provenance.json"
    if not bundle.exists():
        with tempfile.TemporaryDirectory(prefix="kmipkit-rust-package-") as directory:
            tree = Path(directory)
            archive = subprocess.check_output(["git", "-C", str(ROOT), "archive", PIN])
            with tarfile.open(fileobj=io.BytesIO(archive)) as source:
                source.extractall(tree, filter="data")
            patches = []
            for name in [*CRATES, "kmipkit-test-support"]:
                patches.extend(["--config", f'patch.crates-io.{name}.path="crates/{name}"'])
            for name in CRATES:
                subprocess.run(["cargo", "+1.94", "package", "-p", name,
                                "--no-verify", "--allow-dirty", *patches], cwd=tree, check=True)
            stage = tree / "distribution"
            (stage / "packages").mkdir(parents=True)
            for name in CRATES:
                shutil.copy2(tree / f"target/package/{name}-1.0.0.crate", stage / "packages")
            # Public rustdoc includes these immutable release files outside crate roots.
            shutil.copytree(tree / "docs", stage / "docs")
            shutil.copytree(tree / "vendor/aws-lc-sys-0.45.0", stage / "vendor/aws-lc-sys-0.45.0")
            (stage / "library-commit").write_text(PIN + "\n")
            with tarfile.open(bundle, "w:gz") as target:
                for member in sorted(stage.iterdir()):
                    target.add(member, arcname=member.name)
        provenance.write_text(json.dumps({"library_commit": PIN, "artifact_sha256": digest(bundle)}, indent=2) + "\n")
    data = json.loads(provenance.read_text())
    if data["library_commit"] != PIN or data["artifact_sha256"] != digest(bundle):
        raise SystemExit("installed artifact provenance mismatch")
    target = LOCAL / "install"
    if target.exists():
        shutil.rmtree(target)
    target.mkdir()
    with tarfile.open(bundle) as source:
        source.extractall(target, filter="data")
    if (target / "library-commit").read_text().strip() != PIN:
        raise SystemExit("package release identity mismatch")
    (target / "crates").mkdir()
    for package in (target / "packages").glob("*.crate"):
        with tarfile.open(package) as source:
            source.extractall(target / "crates", filter="data")
    print(json.dumps(data))


if __name__ == "__main__":
    install()
