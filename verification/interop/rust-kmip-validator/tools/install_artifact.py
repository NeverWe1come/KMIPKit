#!/usr/bin/env python3
"""Build and install Rust distribution packages from the checked-out Git commit."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile

from verify_artifact import LOCAL, ROOT, source_commit

CRATES = ["kmipkit-ttlv", "kmipkit-protocol", "kmipkit-transport", "kmipkit-client", "kmipkit"]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def installed_file(member, destination):
    checked = tarfile.data_filter(member, destination)
    # Cargo builds these packages as path dependencies and uses source mtimes
    # to decide whether to recompile. Crate archives carry normalized mtimes.
    return None if checked is None else checked.replace(mtime=None)


def install():
    LOCAL.mkdir(parents=True, exist_ok=True)
    bundle = LOCAL / "kmipkit-rust-1.0.0.tar.gz"
    provenance = LOCAL / "provenance.json"
    commit = source_commit()
    data = json.loads(provenance.read_text()) if provenance.is_file() else {}
    if not bundle.is_file() or data.get("library_commit") != commit:
        with tempfile.TemporaryDirectory(prefix="kmipkit-rust-package-") as directory:
            tree = Path(directory)
            with tempfile.TemporaryFile() as archive:
                subprocess.run(["git", "-C", str(ROOT), "archive", commit], stdout=archive, check=True)
                archive.seek(0)
                with tarfile.open(fileobj=archive) as source:
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
            # Public rustdoc includes these documentation files outside crate roots.
            shutil.copytree(tree / "docs", stage / "docs")
            (stage / "library-commit").write_text(commit + "\n")
            pending = LOCAL / "kmipkit-rust-1.0.0.tar.gz.tmp"
            with tarfile.open(pending, "w:gz") as target:
                for member in sorted(stage.iterdir()):
                    target.add(member, arcname=member.name)
            pending.replace(bundle)
        provenance.write_text(json.dumps({"library_commit": commit, "artifact_sha256": digest(bundle)}, indent=2) + "\n")
    data = json.loads(provenance.read_text())
    if data["library_commit"] != commit or data["artifact_sha256"] != digest(bundle):
        raise SystemExit("installed artifact provenance mismatch")
    target = LOCAL / "install"
    if target.exists():
        shutil.rmtree(target)
    target.mkdir()
    with tarfile.open(bundle) as source:
        source.extractall(target, filter=installed_file)
    if (target / "library-commit").read_text().strip() != commit:
        raise SystemExit("package source identity mismatch")
    (target / "crates").mkdir()
    for package in (target / "packages").glob("*.crate"):
        with tarfile.open(package) as source:
            source.extractall(target / "crates", filter=installed_file)
    print(json.dumps(data))


if __name__ == "__main__":
    install()
