"""Offline negative fixtures for the pinned cargo-deny dependency policy."""

from __future__ import annotations

import hashlib
import io
import json
import os
import re
import subprocess
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
EXPECTED_CARGO_DENY_VERSION = "0.20.2"
DEFAULT_RUSTSEC_DATABASE = "advisory-db-3157b0e258782691"
PROJECT_LOCKFILES = (
    REPOSITORY_ROOT / "Cargo.lock",
    REPOSITORY_ROOT / "fuzz" / "Cargo.lock",
)
CARGO_DENY = os.environ.get("CARGO_DENY")


def lockfile_snapshot() -> dict[Path, tuple[bool, str | None]]:
    """Capture existence and content hashes for project lockfiles."""
    snapshot: dict[Path, tuple[bool, str | None]] = {}
    for path in PROJECT_LOCKFILES:
        if path.is_file():
            snapshot[path] = (True, hashlib.sha256(path.read_bytes()).hexdigest())
        else:
            snapshot[path] = (False, None)
    return snapshot


@unittest.skipUnless(CARGO_DENY, "set CARGO_DENY to the pinned cargo-deny 0.20.2 executable")
class CargoDenyNegativeFixtureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        executable = Path(CARGO_DENY or "")
        if not executable.is_file():
            raise AssertionError("CARGO_DENY must name an existing pinned cargo-deny executable")
        result = subprocess.run(
            [str(executable), "--version"], capture_output=True, text=True, check=False
        )
        match = re.fullmatch(r"cargo-deny\s+(\S+)\s*", result.stdout)
        if result.returncode != 0 or match is None or match.group(1) != EXPECTED_CARGO_DENY_VERSION:
            raise AssertionError("CARGO_DENY must be exactly cargo-deny 0.20.2")
        cls.cargo_deny = str(executable)
        cls.lockfiles_before = lockfile_snapshot()

    @classmethod
    def tearDownClass(cls) -> None:
        if hasattr(cls, "lockfiles_before"):
            if lockfile_snapshot() != cls.lockfiles_before:
                raise AssertionError("cargo-deny fixtures modified a project Cargo.lock")

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="kmipkit-cargo-deny-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.cargo_home = self.root / "cargo-home"
        self.cargo_home.mkdir()
        self.env = os.environ.copy()
        self.env["CARGO_HOME"] = str(self.cargo_home)
        self.env["CARGO_NET_OFFLINE"] = "true"
        self.addCleanup(self.assert_project_lockfiles_unchanged)

    def assert_project_lockfiles_unchanged(self) -> None:
        self.assertEqual(lockfile_snapshot(), self.lockfiles_before)

    def assert_builder_never_requests_online_cargo(self, build_fixture) -> None:
        real_run = subprocess.run
        invocations: list[tuple[tuple[str, ...], str | None]] = []

        class OnlineCargoAttempt(Exception):
            pass

        def guarded_run(command, *args, **kwargs):
            if command and command[0] == "cargo":
                environment = kwargs.get("env") or os.environ
                invocations.append((tuple(command), environment.get("CARGO_NET_OFFLINE")))
                if environment.get("CARGO_NET_OFFLINE") != "true":
                    raise OnlineCargoAttempt(
                        f"Cargo invocation can access the network: {command!r}"
                    )
                if "--offline" not in command:
                    command = [command[0], "--offline", *command[1:]]
            return real_run(command, *args, **kwargs)

        try:
            with patch(
                "scripts.tests.test_cargo_deny_fixtures.subprocess.run",
                side_effect=guarded_run,
            ):
                build_fixture()
        except OnlineCargoAttempt as error:
            self.fail(str(error))

        self.assertTrue(invocations, "fixture builder did not exercise Cargo")
        self.assertTrue(
            all(offline == "true" and "--offline" in command for command, offline in invocations),
            "every Cargo invocation must set CARGO_NET_OFFLINE=true and pass --offline: "
            f"{invocations!r}",
        )

    @staticmethod
    def finding_json_line() -> str:
        return json.dumps(
            {
                "type": "diagnostic",
                "fields": {
                    "code": "rejected",
                    "severity": "error",
                    "graphs": [
                        {
                            "Krate": {"name": "fixture-gpl", "version": "1.2.3"},
                            "parents": [
                                {"Krate": {"name": "fixture-root", "version": "0.1.0"}}
                            ],
                        }
                    ],
                },
            }
        )

    def assert_finding_output(self, output_lines: list[str]) -> None:
        fixture = self.root / "parser-fixture"
        fixture.mkdir()
        (fixture / "Cargo.lock").write_text("version = 4\n", encoding="utf-8")
        result = subprocess.CompletedProcess(
            args=[self.cargo_deny], returncode=1, stdout="\n".join(output_lines), stderr=""
        )
        with patch(
            "scripts.tests.test_cargo_deny_fixtures.subprocess.run", return_value=result
        ):
            self.assert_finding(
                fixture, "licenses", "rejected", {("fixture-gpl", "1.2.3")}
            )

    def create_fixture(
        self,
        dependencies: list[tuple[str, str, str, bool]],
        *,
        config: str | None = None,
    ) -> Path:
        """Create a self-contained path-dependency workspace with a lockfile."""
        fixture = self.root / "fixture"
        fixture.mkdir()
        (fixture / ".cargo").mkdir()
        (fixture / ".cargo" / "deny.toml").write_text(
            config or self.fixture_deny_config(), encoding="utf-8"
        )
        manifest_lines = [
            "[package]",
            'name = "fixture-root"',
            'version = "0.1.0"',
            'edition = "2024"',
            'license = "Apache-2.0"',
            "",
        ]
        for name, version, license, is_dev in dependencies:
            dependency_name = f"{name.replace('-', '_')}_{version.replace('.', '_')}"
            section = "[dev-dependencies]" if is_dev else "[dependencies]"
            if section not in manifest_lines:
                manifest_lines.extend((section,))
            manifest_lines.append(
                f'{dependency_name} = {{ package = "{name}", version = "{version}", '
                f'path = "crates/{dependency_name}" }}'
            )
            package_dir = fixture / "crates" / dependency_name
            package_dir.mkdir(parents=True)
            (package_dir / "Cargo.toml").write_text(
                "\n".join(
                    (
                        "[package]",
                        f'name = "{name}"',
                        f'version = "{version}"',
                        'edition = "2024"',
                        f'license = "{license}"',
                        "",
                    )
                ),
                encoding="utf-8",
            )
            (package_dir / "src").mkdir()
            (package_dir / "src" / "lib.rs").write_text("pub fn fixture() {}\n", encoding="utf-8")

        (fixture / "Cargo.toml").write_text("\n".join(manifest_lines), encoding="utf-8")
        (fixture / "src").mkdir()
        (fixture / "src" / "lib.rs").write_text("pub fn fixture_root() {}\n", encoding="utf-8")
        generated = subprocess.run(
            [
                "cargo",
                "generate-lockfile",
                "--offline",
                "--manifest-path",
                str(fixture / "Cargo.toml"),
            ],
            cwd=fixture,
            env=self.env,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(generated.returncode, 0, generated.stderr)
        return fixture

    def create_unapproved_git_fixture(self) -> tuple[Path, Path]:
        """Create a local pinned Git source and synthetic metadata, with no fetch."""
        source = self.root / "git-source"
        source.mkdir()
        (source / "src").mkdir()
        (source / "Cargo.toml").write_text(
            '[package]\nname = "unapproved-git"\nversion = "1.0.0"\n'
            'edition = "2024"\nlicense = "MIT"\n',
            encoding="utf-8",
        )
        (source / "src" / "lib.rs").write_text("pub fn git_fixture() {}\n", encoding="utf-8")
        for arguments in (
            ("init", "-q"),
            ("config", "user.email", "fixture@example.invalid"),
            ("config", "user.name", "Fixture"),
            ("add", "."),
            ("commit", "-qm", "fixture"),
        ):
            result = subprocess.run(
                ["git", *arguments],
                cwd=source,
                env=self.env,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
        revision = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=source,
            env=self.env,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(revision.returncode, 0, revision.stderr)

        fixture = self.create_fixture([("unapproved-git", "1.0.0", "MIT", False)])
        generated = subprocess.run(
            [
                "cargo",
                "metadata",
                "--offline",
                "--format-version",
                "1",
                "--manifest-path",
                str(fixture / "Cargo.toml"),
            ],
            cwd=fixture,
            env=self.env,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(generated.returncode, 0, generated.stderr)
        metadata = json.loads(generated.stdout)
        self.replace_package_source(
            metadata,
            "unapproved-git",
            "1.0.0",
            f"git+{source.as_uri()}?rev={revision.stdout.strip()}",
        )
        metadata_path = self.root / "git-metadata.json"
        metadata_path.write_text(json.dumps(metadata), encoding="utf-8")
        return fixture, metadata_path

    @staticmethod
    def fixture_deny_config() -> str:
        return """[advisories]
ignore = []

[bans]
multiple-versions = "deny"
multiple-versions-include-dev = true
wildcards = "deny"
allow-wildcard-paths = false
deny = ["native-tls", "openssl", "openssl-sys"]
skip = []

[licenses]
allow = ["Apache-2.0", "MIT"]
include-dev = true
exceptions = []

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
"""

    def assert_finding(
        self,
        fixture: Path,
        check: str,
        code: str,
        packages: set[tuple[str, str]],
        *,
        metadata_path: Path | None = None,
    ) -> None:
        fixture_lockfile = fixture / "Cargo.lock"
        lockfile_before = hashlib.sha256(fixture_lockfile.read_bytes()).hexdigest()
        command = [
            self.cargo_deny,
            "--manifest-path",
            str(fixture / "Cargo.toml"),
            "--config",
            str(fixture / ".cargo" / "deny.toml"),
        ]
        if metadata_path is not None:
            command.extend(("--metadata-path", str(metadata_path)))
        command.extend(
            ("--format", "json", "--color", "never", "--offline", "--locked", "check", check)
        )
        result = subprocess.run(
            command,
            cwd=fixture,
            env=self.env,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(
            hashlib.sha256(fixture_lockfile.read_bytes()).hexdigest(),
            lockfile_before,
            "cargo-deny changed the isolated fixture lockfile",
        )
        diagnostics = []
        unexpected_json_lines = []
        unexpected_log_levels = []
        for line_number, line in enumerate(
            (result.stdout + "\n" + result.stderr).splitlines(), start=1
        ):
            if not line.strip():
                continue
            try:
                item = json.loads(line)
            except json.JSONDecodeError:
                unexpected_json_lines.append(line_number)
                continue
            if not isinstance(item, dict):
                unexpected_json_lines.append(line_number)
            elif item.get("type") == "diagnostic" and isinstance(item.get("fields"), dict):
                diagnostics.append(item["fields"])
            elif item.get("type") == "summary" and isinstance(item.get("fields"), dict):
                continue
            elif item.get("type") == "log" and isinstance(item.get("fields"), dict):
                level = item["fields"].get("level")
                if not isinstance(level, str) or level.upper() != "INFO":
                    unexpected_log_levels.append(level if isinstance(level, str) else "invalid")
            else:
                unexpected_json_lines.append(line_number)
        matched = [item for item in diagnostics if item.get("code") == code]
        error_codes = {
            item.get("code") for item in diagnostics if item.get("severity") == "error"
        }
        observed = set()
        for item in matched:
            for name, version in self.krates(item.get("graphs")):
                observed.add((name, version))
        self.assertEqual(
            result.returncode != 0,
            True,
            f"expected cargo-deny {check} to reject fixture; "
            f"stdout={result.stdout!r} stderr={result.stderr!r}",
        )
        self.assertTrue(
            matched,
            f"expected exact structured finding code {code!r}; diagnostics={diagnostics!r}; "
            f"stdout={result.stdout!r}; stderr={result.stderr!r}",
        )
        self.assertFalse(
            unexpected_json_lines,
            f"non-JSON cargo-deny output on lines {unexpected_json_lines!r}",
        )
        self.assertFalse(
            unexpected_log_levels,
            f"unexpected cargo-deny log levels: {unexpected_log_levels!r}",
        )
        self.assertEqual(error_codes, {code}, f"unexpected error finding codes: {error_codes!r}")
        graph_context = {("fixture-root", "0.1.0")}
        finding_names = {name for name, _ in packages}
        self.assertEqual(
            {item for item in observed if item[0] in finding_names},
            packages,
            f"expected exact package/version findings; saw {observed!r}",
        )
        self.assertLessEqual(observed, packages | graph_context)

    def create_registry_metadata_fixture(
        self,
        package: str,
        version: str,
        *,
        registry_url: str,
        license: str = "MIT",
    ) -> tuple[Path, Path]:
        """Create cargo metadata with a registry source without downloading crates."""
        fixture = self.create_fixture([(package, version, license, False)])
        generated = subprocess.run(
            [
                "cargo",
                "metadata",
                "--offline",
                "--format-version",
                "1",
                "--manifest-path",
                str(fixture / "Cargo.toml"),
            ],
            cwd=fixture,
            env=self.env,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(generated.returncode, 0, generated.stderr)
        metadata = json.loads(generated.stdout)
        self.replace_package_source(metadata, package, version, f"registry+{registry_url}")
        metadata_path = self.root / "cargo-metadata.json"
        metadata_path.write_text(json.dumps(metadata), encoding="utf-8")
        return fixture, metadata_path

    @staticmethod
    def replace_package_source(
        metadata: dict, package: str, version: str, source: str
    ) -> None:
        package_metadata = next(
            item
            for item in metadata["packages"]
            if item["name"] == package and item["version"] == version
        )
        old_id = package_metadata["id"]
        new_id = f"{source}#{package}@{version}"
        package_metadata["id"] = new_id
        package_metadata["source"] = source
        for node in metadata["resolve"]["nodes"]:
            if node["id"] == old_id:
                node["id"] = new_id
            node["dependencies"] = [
                new_id if item == old_id else item for item in node["dependencies"]
            ]
            for dependency in node["deps"]:
                if dependency["pkg"] == old_id:
                    dependency["pkg"] = new_id

    def create_local_registry_fixture(
        self,
        package: str,
        version: str,
        *,
        license: str = "MIT",
        yanked: bool = False,
        advisory: str | None = None,
    ) -> tuple[Path, Path]:
        """Create an offline registry lock entry and metadata-backed package graph."""
        fixture = self.create_fixture([(package, version, license, False)])
        generated = subprocess.run(
            [
                "cargo",
                "metadata",
                "--offline",
                "--format-version",
                "1",
                "--manifest-path",
                str(fixture / "Cargo.toml"),
            ],
            cwd=fixture,
            env=self.env,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(generated.returncode, 0, generated.stderr)
        metadata = json.loads(generated.stdout)

        crate_contents = {
            "Cargo.toml": (
                f'[package]\nname = "{package}"\nversion = "{version}"\n'
                f'edition = "2024"\nlicense = "{license}"\n'
            ).encode("utf-8"),
            "src/lib.rs": b"pub fn fixture() {}\n",
        }
        archive_buffer = io.BytesIO()
        with tarfile.open(fileobj=archive_buffer, mode="w:gz") as archive:
            for relative_path, contents in crate_contents.items():
                item = tarfile.TarInfo(f"{package}-{version}/{relative_path}")
                item.size = len(contents)
                item.mtime = 0
                archive.addfile(item, io.BytesIO(contents))
        crate_archive = archive_buffer.getvalue()
        checksum = hashlib.sha256(crate_archive).hexdigest()

        index_relative = self.registry_index_path(package)
        crates_io_hash = "index.crates.io-1949cf8c6b5b557f"
        crates_io_index = self.cargo_home / "registry" / "index" / crates_io_hash
        crates_io_index.mkdir(parents=True)
        (crates_io_index / "config.json").write_text(
            json.dumps(
                {"dl": "https://static.crates.io/crates/{crate}/{crate}-{version}.crate"}
            ),
            encoding="utf-8",
        )
        index_entry = {
            "name": package,
            "vers": version,
            "deps": [],
            "cksum": checksum,
            "features": {},
            "yanked": yanked,
        }
        sparse_cache_entry = crates_io_index / ".cache" / index_relative
        sparse_cache_entry.parent.mkdir(parents=True, exist_ok=True)
        sparse_cache_entry.write_bytes(
            b'\x03\x02\x00\x00\x00etag: "kmipkit-fixture"\x00'
            + version.encode("utf-8")
            + b"\x00"
            + json.dumps(index_entry, separators=(",", ":")).encode("utf-8")
            + b"\x00"
        )
        crates_io_cache = self.cargo_home / "registry" / "cache" / crates_io_hash
        crates_io_cache.mkdir(parents=True, exist_ok=True)
        (crates_io_cache / f"{package}-{version}.crate").write_bytes(crate_archive)

        source = "registry+https://github.com/rust-lang/crates.io-index"
        self.replace_package_source(metadata, package, version, source)
        metadata_path = self.root / "cargo-metadata.json"
        metadata_path.write_text(json.dumps(metadata), encoding="utf-8")

        advisory_database = self.cargo_home / "advisory-dbs" / DEFAULT_RUSTSEC_DATABASE
        (advisory_database / "crates").mkdir(parents=True)
        sentinel = advisory_database / "crates" / "fixture-sentinel" / "RUSTSEC-2026-9999.md"
        sentinel.parent.mkdir(parents=True)
        sentinel.write_text(
            "```toml\n[advisory]\nid = \"RUSTSEC-2026-9999\"\n"
            'package = "fixture-sentinel"\ndate = "2026-10-05"\n'
            'url = "https://example.invalid/fixture-sentinel"\n\n'
            "[affected]\n\n[versions]\npatched = [\">= 1.0.0\"]\n```\n",
            encoding="utf-8",
        )
        if advisory is not None:
            advisory_path = advisory_database / "crates" / package / f"RUSTSEC-2026-0001.md"
            advisory_path.parent.mkdir(parents=True, exist_ok=True)
            advisory_path.write_text(advisory, encoding="utf-8")
        self.git_command(advisory_database, ("init", "-q"))
        self.git_command(advisory_database, ("config", "user.email", "fixture@example.invalid"))
        self.git_command(advisory_database, ("config", "user.name", "Fixture"))
        self.git_command(advisory_database, ("add", "."))
        self.git_command(advisory_database, ("commit", "-qm", "local advisory database"))

        deny_config = fixture / ".cargo" / "deny.toml"
        config_text = deny_config.read_text(encoding="utf-8").replace(
            "ignore = []", f'ignore = []\nyanked = "{"deny" if yanked else "warn"}"'
        )
        if advisory is not None:
            config_text = config_text.replace(
                'yanked = "warn"',
                'yanked = "warn"\ndisable-yank-checking = true',
            )
        deny_config.write_text(config_text, encoding="utf-8")
        return fixture, metadata_path

    def git_command(self, cwd: Path, arguments: tuple[str, ...]) -> None:
        result = subprocess.run(
            ["git", *arguments], cwd=cwd, env=self.env, capture_output=True, text=True, check=False
        )
        self.assertEqual(result.returncode, 0, result.stderr)

    @staticmethod
    def registry_index_path(package: str) -> Path:
        normalized = package.lower()
        if len(normalized) == 1:
            return Path("1") / normalized
        if len(normalized) == 2:
            return Path("2") / normalized
        if len(normalized) == 3:
            return Path("3") / normalized[0] / normalized
        return Path(normalized[:2]) / normalized[2:4] / normalized

    @classmethod
    def krates(cls, value: object) -> set[tuple[str, str]]:
        found: set[tuple[str, str]] = set()
        if isinstance(value, dict):
            krate = value.get("Krate")
            if isinstance(krate, dict) and isinstance(krate.get("name"), str) and isinstance(
                krate.get("version"), str
            ):
                found.add((krate["name"], krate["version"]))
            for child in value.values():
                found.update(cls.krates(child))
        elif isinstance(value, list):
            for child in value:
                found.update(cls.krates(child))
        return found

    def test_disallowed_license_reports_rejected_package_and_version(self) -> None:
        fixture = self.create_fixture([("fixture-gpl", "1.2.3", "GPL-3.0-only", False)])
        self.assert_finding(fixture, "licenses", "rejected", {("fixture-gpl", "1.2.3")})

    def test_each_architecture_ban_reports_banned_package_and_version(self) -> None:
        banned_packages = {"native-tls", "openssl", "openssl-sys"}
        fixture = self.create_fixture(
            [(package, "1.0.0", "MIT", False) for package in sorted(banned_packages)]
        )
        self.assert_finding(
            fixture,
            "bans",
            "banned",
            {(package, "1.0.0") for package in banned_packages},
        )

    def test_normal_and_dev_duplicate_versions_report_both_versions(self) -> None:
        fixture = self.create_fixture(
            [
                ("duplicate-fixture", "1.0.0", "MIT", False),
                ("duplicate-fixture", "2.0.0", "MIT", True),
            ]
        )
        self.assert_finding(
            fixture,
            "bans",
            "duplicate",
            {("duplicate-fixture", "1.0.0"), ("duplicate-fixture", "2.0.0")},
        )

    def test_unapproved_local_git_source_reports_package_and_version(self) -> None:
        fixture, metadata = self.create_unapproved_git_fixture()
        self.assert_finding(
            fixture,
            "sources",
            "source-not-allowed",
            {("unapproved-git", "1.0.0")},
            metadata_path=metadata,
        )

    def test_unapproved_git_fixture_builder_keeps_cargo_offline(self) -> None:
        self.assert_builder_never_requests_online_cargo(
            lambda: self.create_unapproved_git_fixture()
        )

    def test_local_registry_fixture_builder_keeps_cargo_offline(self) -> None:
        self.assert_builder_never_requests_online_cargo(
            lambda: self.create_local_registry_fixture("fixture-offline", "1.0.0")
        )

    def test_finding_parser_rejects_unexpected_plain_text(self) -> None:
        with self.assertRaisesRegex(AssertionError, "non-JSON cargo-deny output"):
            self.assert_finding_output(
                [self.finding_json_line(), "cargo-deny emitted unrelated warning text"]
            )

    def test_finding_parser_rejects_warning_logs(self) -> None:
        warning = json.dumps(
            {"type": "log", "fields": {"level": "WARN", "message": "unexpected warning"}}
        )
        with self.assertRaisesRegex(AssertionError, "unexpected cargo-deny log"):
            self.assert_finding_output([self.finding_json_line(), warning])

    def test_finding_parser_preserves_informational_logs(self) -> None:
        information = json.dumps(
            {"type": "log", "fields": {"level": "INFO", "message": "local cache used"}}
        )
        self.assert_finding_output([self.finding_json_line(), information])

    def test_unknown_registry_reports_package_and_version(self) -> None:
        fixture, metadata = self.create_registry_metadata_fixture(
            "fixture-registry",
            "1.0.0",
            registry_url="https://registry.example.invalid/index",
        )
        self.assert_finding(
            fixture,
            "sources",
            "source-not-allowed",
            {("fixture-registry", "1.0.0")},
            metadata_path=metadata,
        )

    def test_yanked_registry_version_reports_exact_yanked_code(self) -> None:
        fixture, metadata = self.create_local_registry_fixture(
            "fixture-yanked", "1.0.0", yanked=True
        )
        self.assert_finding(
            fixture,
            "advisories",
            "yanked",
            {("fixture-yanked", "1.0.0")},
            metadata_path=metadata,
        )

    def test_rustsec_vulnerability_reports_exact_vulnerability_code(self) -> None:
        fixture, metadata = self.create_local_registry_fixture(
            "fixture-vulnerable",
            "1.0.0",
            advisory=self.rustsec_advisory("fixture-vulnerable"),
        )
        self.assert_finding(
            fixture,
            "advisories",
            "vulnerability",
            {("fixture-vulnerable", "1.0.0")},
            metadata_path=metadata,
        )

    def test_rustsec_unsoundness_reports_exact_unsound_code(self) -> None:
        fixture, metadata = self.create_local_registry_fixture(
            "fixture-unsound",
            "1.0.0",
            advisory=self.rustsec_advisory("fixture-unsound", informational="unsound"),
        )
        self.assert_finding(
            fixture,
            "advisories",
            "unsound",
            {("fixture-unsound", "1.0.0")},
            metadata_path=metadata,
        )

    def test_rustsec_unmaintained_reports_exact_unmaintained_code(self) -> None:
        fixture, metadata = self.create_local_registry_fixture(
            "fixture-unmaintained",
            "1.0.0",
            advisory=self.rustsec_advisory("fixture-unmaintained", informational="unmaintained"),
        )
        self.assert_finding(
            fixture,
            "advisories",
            "unmaintained",
            {("fixture-unmaintained", "1.0.0")},
            metadata_path=metadata,
        )

    @staticmethod
    def rustsec_advisory(package: str, *, informational: str | None = None) -> str:
        information = f'informational = "{informational}"\n' if informational else ""
        return (
            "```toml\n[advisory]\nid = \"RUSTSEC-2026-0001\"\n"
            f'package = "{package}"\ndate = "2026-10-05"\n'
            f'url = "https://example.invalid/{package}"\n{information}'
            "categories = [\"code-execution\"]\n\n"
            "[affected]\n\n[versions]\npatched = [\">= 1.0.1\"]\n```\n"
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
