"""Executable contract tests for the repository Cargo dependency policy."""

from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import tomllib
import unittest
from datetime import date
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
POLICY_PATH = REPOSITORY_ROOT / "scripts" / "dependency_policy.py"
POLICY = None
POLICY_LOAD_ERROR = None
if POLICY_PATH.is_file():
    try:
        SPEC = importlib.util.spec_from_file_location("dependency_policy", POLICY_PATH)
        if SPEC is None or SPEC.loader is None:
            raise ImportError("dependency policy has no importable module loader")
        POLICY = importlib.util.module_from_spec(SPEC)
        sys.modules[SPEC.name] = POLICY
        SPEC.loader.exec_module(POLICY)
    except Exception as error:  # Keep RED failures as assertions, not import errors.
        POLICY_LOAD_ERROR = error


def package(name: str, version: str, manifest: Path, *, source: str | None = None) -> dict:
    return {
        "id": f"{name}@{version}",
        "name": name,
        "version": version,
        "source": source,
        "manifest_path": str(manifest),
    }


def workspace_metadata(workspace_root: Path, packages: list[dict], members: list[dict]) -> dict:
    return {
        "workspace_root": str(workspace_root),
        "workspace_members": [item["id"] for item in members],
        "packages": packages,
    }


def exact_exception(
    kind: str,
    package_name: str = "example-crate",
    version: str = "1.2.3",
    **specific: str,
) -> dict:
    entry = {
        "id": "KMIPKIT-0011-EX-001",
        "kind": kind,
        "package": package_name,
        "version": version,
        "rationale": "The transitive package cannot be replaced before the next release.",
        "mitigation": "Keep the package isolated and remove it in the next dependency update.",
        "owner": "KMIPKit maintainers",
        "reviewed_by": "Security reviewer",
        "reviewed_on": "2025-12-01",
        "expires_on": "2026-02-28",
        "approval_ref": "https://github.com/NeverWe1come/KMIPKit/pull/35",
    }
    entry.update(specific)
    return entry


def finding(kind: str, package_name: str = "example-crate", version: str = "1.2.3", **specific: str) -> dict:
    item = {"kind": kind, "package": package_name, "version": version}
    item.update(specific)
    return item


class DependencyPolicyApiTests(unittest.TestCase):
    def require_policy(self):
        self.assertIsNotNone(
            POLICY,
            "scripts/dependency_policy.py must implement the dependency-policy contract; "
            f"load error: {POLICY_LOAD_ERROR}",
        )
        return POLICY

    def policy_error(self):
        policy = self.require_policy()
        error_type = getattr(policy, "PolicyError", None)
        self.assertIsNotNone(error_type, "dependency_policy.py must expose its documented PolicyError.")
        self.assertTrue(issubclass(error_type, ValueError), "PolicyError must be a ValueError subclass.")
        return error_type

    def make_repository(self, root: Path) -> tuple[Path, dict[str, dict]]:
        (root / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
        ttlv_manifest = root / "crates" / "kmipkit-ttlv" / "Cargo.toml"
        ttlv_manifest.parent.mkdir(parents=True)
        ttlv_manifest.write_text("[package]\nname = 'kmipkit-ttlv'\n", encoding="utf-8")
        fuzz_manifest = root / "fuzz" / "Cargo.toml"
        fuzz_manifest.parent.mkdir(parents=True)
        fuzz_manifest.write_text("[package]\nname = 'kmipkit-ttlv-fuzz'\n", encoding="utf-8")

        root_package = package("kmipkit", "1.0.0", root / "Cargo.toml")
        ttlv_package = package("kmipkit-ttlv", "1.0.0", ttlv_manifest)
        fuzz_package = package("kmipkit-ttlv-fuzz", "0.0.0", fuzz_manifest)
        metadata_by_workspace = {
            "root": workspace_metadata(root, [root_package, ttlv_package], [root_package, ttlv_package]),
            "fuzz": workspace_metadata(root / "fuzz", [fuzz_package, ttlv_package], [fuzz_package]),
        }
        return ttlv_manifest, metadata_by_workspace

    def test_rustc_verbose_output_yields_the_observed_host_triple(self) -> None:
        policy = self.require_policy()
        observed = policy.parse_rustc_host(
            "rustc 1.94.0 (example)\nbinary: rustc\ncommit-hash: abc\n"
            "host: aarch64-unknown-linux-gnu\nrelease: 1.94.0\n"
        )
        self.assertEqual("aarch64-unknown-linux-gnu", observed)

    def test_policy_runner_rejects_a_host_absent_from_the_reviewed_ci_set(self) -> None:
        policy = self.require_policy()
        policy.validate_host_triple("aarch64-unknown-linux-gnu", {"aarch64-unknown-linux-gnu"})
        with self.assertRaises(self.policy_error()):
            policy.validate_host_triple("x86_64-pc-windows-msvc", {"aarch64-unknown-linux-gnu"})

    def test_cli_normalizes_missing_checkout_root_without_echoing_path(self) -> None:
        self.require_policy()
        with tempfile.TemporaryDirectory() as directory:
            missing_root = Path(directory) / "sentinel-private-checkout"
            completed = subprocess.run(
                [
                    sys.executable,
                    str(POLICY_PATH),
                    "--checkout-root",
                    str(missing_root),
                    "--root-metadata",
                    str(Path(directory) / "root.json"),
                    "--fuzz-metadata",
                    str(Path(directory) / "fuzz.json"),
                ],
                cwd=REPOSITORY_ROOT,
                capture_output=True,
                text=True,
                check=False,
            )
        self.assertNotEqual(0, completed.returncode)
        self.assertIn("dependency policy:", completed.stderr)
        self.assertNotIn("Traceback", completed.stderr)
        self.assertNotIn("sentinel-private-checkout", completed.stderr)

    def test_path_dependencies_may_resolve_to_the_canonical_union_of_workspace_members(self) -> None:
        policy = self.require_policy()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _, metadata = self.make_repository(root)
            policy.validate_workspace_metadata(root, metadata)

    def test_checkout_root_symlink_alias_matches_metadata_paths(self) -> None:
        policy = self.require_policy()
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "checkout"
            root.mkdir()
            alias = parent / "checkout-alias"
            try:
                alias.symlink_to(root, target_is_directory=True)
            except OSError as error:
                self.skipTest(f"directory symlinks are unavailable: {error}")
            _, metadata = self.make_repository(alias)
            policy.validate_workspace_metadata(alias, metadata)

    def test_optional_feature_only_external_path_dependency_is_rejected(self) -> None:
        policy = self.require_policy()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _, metadata = self.make_repository(root)
            outside = Path(directory).parent / f"{Path(directory).name}-optional-path" / "Cargo.toml"
            optional_package = package("optional-path", "0.4.0", outside)
            optional_package["dependencies"] = []
            optional_package["features"] = {"optional-path": []}
            metadata["root"]["packages"].append(optional_package)
            metadata["root"]["packages"][0]["dependencies"] = [
                {
                    "name": "optional-path",
                    "source": None,
                    "req": "*",
                    "kind": None,
                    "optional": True,
                    "target": None,
                    "uses_default_features": True,
                    "features": [],
                    "rename": None,
                    "registry": None,
                    "path": str(outside.parent),
                }
            ]
            with self.assertRaises(self.policy_error()):
                policy.validate_workspace_metadata(root, metadata)

    def test_target_specific_external_path_dependency_is_rejected_without_platform_filtering(self) -> None:
        policy = self.require_policy()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _, metadata = self.make_repository(root)
            target_manifest = root / "vendor" / "uefi-helper" / "Cargo.toml"
            target_package = package("uefi-helper", "0.2.0", target_manifest)
            metadata["fuzz"]["packages"].append(target_package)
            metadata["fuzz"]["packages"][0]["dependencies"] = [
                {
                    "name": "uefi-helper",
                    "source": None,
                    "req": "*",
                    "kind": None,
                    "optional": False,
                    "target": "cfg(target_os = \"uefi\")",
                    "uses_default_features": True,
                    "features": [],
                    "rename": None,
                    "registry": None,
                    "path": str(target_manifest.parent),
                }
            ]
            with self.assertRaises(self.policy_error()):
                policy.validate_workspace_metadata(root, metadata)

    def test_symlink_that_escapes_the_checkout_is_rejected_after_canonicalization(self) -> None:
        policy = self.require_policy()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "checkout"
            root.mkdir()
            _, metadata = self.make_repository(root)
            external = Path(directory) / "external-member"
            external.mkdir()
            (external / "Cargo.toml").write_text("[package]\nname = 'linked'\n", encoding="utf-8")
            link = root / "crates" / "linked"
            try:
                link.symlink_to(external, target_is_directory=True)
            except (OSError, NotImplementedError) as error:
                self.skipTest(f"directory symlinks are unavailable: {error}")
            linked_package = package("linked", "1.0.0", link / "Cargo.toml")
            metadata["root"]["packages"].append(linked_package)
            metadata["root"]["workspace_members"].append(linked_package["id"])
            with self.assertRaises(self.policy_error()):
                policy.validate_workspace_metadata(root, metadata)

    def test_git_source_exception_covers_only_the_exact_revision_in_both_workspace_graphs(self) -> None:
        policy = self.require_policy()
        source_validator = getattr(policy, "validate_metadata_sources", None)
        self.assertTrue(callable(source_validator), "dependency_policy.py must validate exact source metadata.")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _, metadata = self.make_repository(root)
            allowed_hash = "a" * 40
            other_hash = "b" * 40
            allowed_source = (
                "git+https://example.invalid/dependency?rev=" + allowed_hash + "#" + allowed_hash
            )
            other_revision = (
                "git+https://example.invalid/dependency?rev=" + other_hash + "#" + other_hash
            )
            metadata["root"]["packages"].append(
                package("git-crate", "1.0.0", root / "vendor" / "git-crate" / "Cargo.toml", source=other_revision)
            )
            register = {
                "schema_version": 1,
                "exceptions": [
                    exact_exception("source", package_name="git-crate", version="1.0.0", source=allowed_source)
                ],
            }
            with self.assertRaises(self.policy_error()):
                source_validator(metadata, register, today=date(2026, 1, 15))

            metadata["root"]["packages"][-1]["source"] = allowed_source
            source_validator(metadata, register, today=date(2026, 1, 15))

    def test_non_crates_io_registry_is_rejected_without_an_exact_source_policy(self) -> None:
        policy = self.require_policy()
        source_validator = getattr(policy, "validate_metadata_sources", None)
        self.assertTrue(callable(source_validator), "dependency_policy.py must validate exact source metadata.")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _, metadata = self.make_repository(root)
            metadata["root"]["packages"].append(
                package(
                    "mirror-crate",
                    "1.0.0",
                    root / "mirror" / "Cargo.toml",
                    source="registry+https://mirror.example.invalid/index",
                )
            )
            with self.assertRaises(self.policy_error()):
                source_validator(metadata, {"schema_version": 1, "exceptions": []})

    def test_fuzz_package_declares_apache_license_and_matching_local_crate_version(self) -> None:
        fuzz_manifest = tomllib.loads((REPOSITORY_ROOT / "fuzz" / "Cargo.toml").read_text(encoding="utf-8"))
        workspace_manifest = tomllib.loads((REPOSITORY_ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        ttlv_manifest = tomllib.loads(
            (REPOSITORY_ROOT / "crates" / "kmipkit-ttlv" / "Cargo.toml").read_text(encoding="utf-8")
        )
        expected_version = ttlv_manifest["package"].get("version")
        if isinstance(expected_version, dict) and expected_version.get("workspace") is True:
            expected_version = workspace_manifest["workspace"]["package"]["version"]
        if expected_version is None:
            expected_version = workspace_manifest["workspace"]["package"]["version"]
        fuzz_package = fuzz_manifest["package"]
        edge = fuzz_manifest["dependencies"]["kmipkit-ttlv"]

        self.assertEqual("Apache-2.0", fuzz_package.get("license"))
        self.assertEqual("../crates/kmipkit-ttlv", edge["path"])
        self.assertEqual(expected_version, edge.get("version"))

    def test_private_packages_are_not_exempt_from_license_policy(self) -> None:
        policy_config = tomllib.loads(
            (REPOSITORY_ROOT / ".cargo" / "deny.toml").read_text(encoding="utf-8")
        )
        private_policy = policy_config["licenses"].get("private")
        self.assertIsInstance(private_policy, dict, "license private-package policy must be explicit")
        self.assertIs(private_policy.get("ignore"), False)

    def test_fuzz_candidate_scan_no_longer_reports_metadata_policy_findings(self) -> None:
        deny_config = REPOSITORY_ROOT / ".cargo" / "deny.toml"
        deny = os.environ.get("CARGO_DENY")
        if not deny_config.is_file() or not deny:
            self.skipTest("the candidate scan runs only with the pinned cargo-deny from the policy runner")

        lockfile = REPOSITORY_ROOT / "fuzz" / "Cargo.lock"
        before = lockfile.read_bytes()
        completed = subprocess.run(
            [
                deny,
                "--manifest-path",
                str(REPOSITORY_ROOT / "fuzz" / "Cargo.toml"),
                "--config",
                str(deny_config),
                "--workspace",
                "--all-features",
                "--locked",
                "check",
            ],
            cwd=REPOSITORY_ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            check=False,
            timeout=300,
        )
        output = completed.stdout + completed.stderr

        self.assertEqual(0, completed.returncode, f"fuzz candidate policy scan failed:\n{output}")
        self.assertNotRegex(output, r"(?i)(?:unlicensed|wildcard).{0,100}kmipkit-ttlv-fuzz")
        self.assertNotRegex(output, r"(?i)wildcard.{0,100}kmipkit-ttlv")
        self.assertEqual(before, lockfile.read_bytes(), "the candidate scan must preserve fuzz/Cargo.lock")

    def test_policy_scans_preserve_both_lockfiles_and_resolved_package_versions(self) -> None:
        executable = os.environ.get("CARGO_DENY")
        if not executable:
            self.skipTest("the pinned policy runner sets CARGO_DENY after refreshing the advisory database")

        deny = Path(executable)
        self.assertTrue(deny.is_file(), "CARGO_DENY must identify the pinned cargo-deny executable")
        version = subprocess.run(
            [str(deny), "--version"], capture_output=True, text=True, check=False, timeout=30
        )
        self.assertEqual(0, version.returncode, version.stderr)
        self.assertRegex(version.stdout, r"^cargo-deny\s+0\.20\.2(?:\s|$)")

        workspaces = (
            ("root", REPOSITORY_ROOT / "Cargo.toml", REPOSITORY_ROOT / "Cargo.lock"),
            ("fuzz", REPOSITORY_ROOT / "fuzz" / "Cargo.toml", REPOSITORY_ROOT / "fuzz" / "Cargo.lock"),
        )

        def snapshot(lockfile: Path) -> tuple[bytes, frozenset[str]]:
            contents = lockfile.read_bytes()
            parsed = tomllib.loads(contents.decode("utf-8"))
            package_versions = frozenset(
                f"{package['name']}@{package['version']}" for package in parsed.get("package", [])
            )
            self.assertTrue(package_versions, f"{lockfile} must contain resolved packages")
            return contents, package_versions

        before = {name: snapshot(lockfile) for name, _, lockfile in workspaces}
        environment = os.environ.copy()
        environment["CARGO_NET_OFFLINE"] = "true"

        for name, manifest, _ in workspaces:
            with self.subTest(workspace=name):
                completed = subprocess.run(
                    [
                        str(deny),
                        "--manifest-path",
                        str(manifest),
                        "--config",
                        str(REPOSITORY_ROOT / ".cargo" / "deny.toml"),
                        "--workspace",
                        "--all-features",
                        "--format",
                        "json",
                        "--color",
                        "never",
                        "--offline",
                        "--locked",
                        "check",
                    ],
                    cwd=REPOSITORY_ROOT,
                    env=environment,
                    capture_output=True,
                    text=True,
                    encoding="utf-8",
                    errors="replace",
                    check=False,
                    timeout=300,
                )
                self.assertEqual(
                    0,
                    completed.returncode,
                    f"{name} cargo-deny scan failed:\n{completed.stdout}\n{completed.stderr}",
                )

        after = {name: snapshot(lockfile) for name, _, lockfile in workspaces}
        self.assertEqual(before, after, "policy scans must preserve both lockfile bytes and resolved package versions")


class DependencyExceptionTests(unittest.TestCase):
    def require_policy(self):
        self.assertIsNotNone(
            POLICY,
            "scripts/dependency_policy.py must implement the dependency-policy contract; "
            f"load error: {POLICY_LOAD_ERROR}",
        )
        return POLICY

    def policy_error(self):
        policy = self.require_policy()
        error_type = getattr(policy, "PolicyError", None)
        self.assertIsNotNone(error_type, "dependency_policy.py must expose its documented PolicyError.")
        self.assertTrue(issubclass(error_type, ValueError), "PolicyError must be a ValueError subclass.")
        return error_type

    def validate(self, entries: list[dict], findings: list[dict]) -> None:
        policy = self.require_policy()
        policy.validate_exceptions({"schema_version": 1, "exceptions": entries}, findings, today=date(2026, 1, 15))

    def test_registered_exception_must_have_a_matching_policy_config_waiver(self) -> None:
        policy = self.require_policy()
        entry = exact_exception("duplicate", source="registry+https://github.com/rust-lang/crates.io-index")
        register = {"schema_version": 1, "exceptions": [entry]}
        with self.assertRaises(self.policy_error()):
            policy.validate_exception_config(register, {"bans": {"skip": []}})

    def test_policy_config_cannot_contain_an_unregistered_exception_waiver(self) -> None:
        policy = self.require_policy()
        with self.assertRaises(self.policy_error()):
            policy.validate_exception_config(
                {"schema_version": 1, "exceptions": []},
                {"bans": {"skip": [{"crate": "example-crate@1.2.3", "reason": "unregistered"}]}},
            )

    def test_policy_config_rejects_unregistered_duplicate_skip_trees(self) -> None:
        policy = self.require_policy()
        with self.assertRaises(self.policy_error()) as context:
            policy.validate_exception_config(
                {"schema_version": 1, "exceptions": []},
                {
                    "bans": {
                        "skip": [],
                        "skip-tree": [{"crate": "example-crate@1.2.3", "reason": "unregistered"}],
                    }
                },
            )
        self.assertIn("skip-tree", str(context.exception))

    def test_each_registered_exception_has_a_matching_cargo_deny_surface(self) -> None:
        policy = self.require_policy()
        source = "git+https://github.com/example/dependency?rev=" + "a" * 40 + "#" + "a" * 40
        entries = [
            exact_exception("advisory", package_name="advisory-crate", advisory_id="RUSTSEC-2025-0001"),
            exact_exception(
                "license",
                package_name="license-crate",
                license_evidence={
                    "reviewed_by": "Security reviewer",
                    "reference": "review-42",
                    "disposition": "clarify",
                    "expression": "MIT",
                    "license_files": [{"path": "LICENSE", "hash": "0xbd0eed23"}],
                },
            ),
            exact_exception("source", package_name="source-crate", source=source),
            exact_exception("duplicate", package_name="duplicate-crate"),
        ]
        for index, entry in enumerate(entries, start=1):
            entry["id"] = f"KMIPKIT-0011-EX-{index:03d}"
        register = {"schema_version": 1, "exceptions": entries}
        config = {
            "advisories": {"ignore": ["RUSTSEC-2025-0001"]},
            "licenses": {
                "clarify": [
                    {
                        "crate": "license-crate@1.2.3",
                        "expression": "MIT",
                        "license-files": [{"path": "LICENSE", "hash": 3171872035}],
                    }
                ]
            },
            "sources": {
                "allow-registry": [],
                "allow-git": [source.removeprefix("git+").split("?", 1)[0]],
            },
            "bans": {"skip": [{"crate": "duplicate-crate@1.2.3", "reason": "KMIPKIT-0011-EX-004"}]},
        }
        policy.validate_exception_config(register, config)

    def test_exception_config_mismatch_reports_rule_and_exception_id_without_secrets(self) -> None:
        policy = self.require_policy()
        source = "git+https://github.com/example/dependency?rev=" + "b" * 40 + "#" + "b" * 40
        entries_and_configs = (
            (
                "advisory",
                exact_exception("advisory", advisory_id="RUSTSEC-2025-0001"),
                {"advisories": {"ignore": []}},
            ),
            (
                "license",
                exact_exception(
                    "license",
                    license_evidence={
                        "reviewed_by": "Security reviewer",
                        "reference": "review-42",
                        "disposition": "clarify",
                        "expression": "MIT",
                        "license_files": [{"path": "LICENSE", "hash": "0xbd0eed23"}],
                    },
                ),
                {"licenses": {"clarify": []}},
            ),
            (
                "source",
                exact_exception("source", package_name="source-crate", source=source),
                {"sources": {"allow-git": ["https://token:secret-value@example.invalid/repo"]}},
            ),
            (
                "duplicate",
                exact_exception("duplicate", package_name="duplicate-crate"),
                {"bans": {"skip": []}},
            ),
        )

        for index, (rule, entry, config) in enumerate(entries_and_configs, start=1):
            entry["id"] = f"KMIPKIT-0011-EX-{index:03d}"
            with self.subTest(rule=rule):
                with self.assertRaises(self.policy_error()) as raised:
                    policy.validate_exception_config(
                        {"schema_version": 1, "exceptions": [entry]},
                        config,
                        today=date(2026, 1, 15),
                    )
                diagnostic = str(raised.exception)
                self.assertIn(rule, diagnostic.lower())
                self.assertIn(entry["id"], diagnostic)
                self.assertNotIn("secret-value", diagnostic)

    def test_duplicate_configured_waivers_report_registered_exception_ids_without_values(self) -> None:
        policy = self.require_policy()
        source = "git+https://github.com/example/dependency?rev=" + "c" * 40 + "#" + "c" * 40
        source_value = source.removeprefix("git+").split("?", 1)[0]
        clarification = {
            "crate": "license-crate@1.2.3",
            "expression": "MIT",
            "license-files": [{"path": "LICENSE", "hash": 0xBD0EED23}],
        }
        cases = (
            (
                "advisory",
                exact_exception("advisory", advisory_id="RUSTSEC-2025-0001"),
                {"advisories": {"ignore": ["RUSTSEC-2025-0001", "RUSTSEC-2025-0001"]}},
                "RUSTSEC-2025-0001",
            ),
            (
                "license",
                exact_exception(
                    "license",
                    package_name="license-crate",
                    license_evidence={
                        "reviewed_by": "Security reviewer",
                        "reference": "review-42",
                        "disposition": "clarify",
                        "expression": "MIT",
                        "license_files": [{"path": "LICENSE", "hash": "0xbd0eed23"}],
                    },
                ),
                {"licenses": {"clarify": [clarification, clarification.copy()]}},
                "license-crate@1.2.3",
            ),
            (
                "source",
                exact_exception("source", package_name="source-crate", source=source),
                {"sources": {"allow-git": [source_value, source_value]}},
                source_value,
            ),
            (
                "duplicate",
                exact_exception("duplicate", package_name="duplicate-crate"),
                {
                    "bans": {
                        "skip": [
                            {"crate": "duplicate-crate@1.2.3", "reason": "KMIPKIT-0011-EX-004"},
                            {"crate": "duplicate-crate@1.2.3", "reason": "KMIPKIT-0011-EX-004"},
                        ]
                    }
                },
                "duplicate-crate@1.2.3",
            ),
        )

        for index, (rule, entry, config, configured_value) in enumerate(cases, start=1):
            entry["id"] = f"KMIPKIT-0011-EX-{index:03d}"
            with self.subTest(rule=rule):
                with self.assertRaises(self.policy_error()) as raised:
                    policy.validate_exception_config(
                        {"schema_version": 1, "exceptions": [entry]},
                        config,
                        today=date(2026, 1, 15),
                    )
                diagnostic = str(raised.exception)
                self.assertIn(rule, diagnostic.lower())
                self.assertIn(entry["id"], diagnostic)
                self.assertNotIn(configured_value, diagnostic)

    def test_unregistered_secret_bearing_git_waiver_names_rule_without_echoing_value(self) -> None:
        policy = self.require_policy()
        configured_source = "https://token:secret-value@example.invalid/repo"
        with self.assertRaises(self.policy_error()) as raised:
            policy.validate_exception_config(
                {"schema_version": 1, "exceptions": []},
                {"sources": {"allow-git": [configured_source]}},
                today=date(2026, 1, 15),
            )

        diagnostic = str(raised.exception).lower()
        self.assertIn("git source", diagnostic)
        self.assertIn("no registered exception id exists", diagnostic)
        self.assertNotIn("secret-value", diagnostic)
        self.assertNotIn(configured_source, diagnostic)

    def test_exact_advisory_license_source_and_duplicate_exceptions_cover_only_their_findings(self) -> None:
        cases = (
            (
                "advisory",
                {"advisory_id": "RUSTSEC-2025-0001", "source": "registry+https://github.com/rust-lang/crates.io-index"},
            ),
            (
                "license",
                {
                    "source": "registry+https://github.com/rust-lang/crates.io-index",
                    "license_evidence": {
                        "reviewed_by": "Security reviewer",
                        "reference": "https://github.com/NeverWe1come/KMIPKit/pull/35",
                        "disposition": "clarify",
                        "expression": "Apache-2.0",
                        "license_files": [{"path": "LICENSE", "hash": "0xbd0eed23"}],
                    },
                },
            ),
            ("source", {"source": "git+https://github.com/example/crate?rev=" + "a" * 40 + "#" + "a" * 40}),
            ("duplicate", {"source": "registry+https://github.com/rust-lang/crates.io-index"}),
        )
        for kind, specific in cases:
            with self.subTest(kind=kind):
                entry = exact_exception(kind, **specific)
                issue = finding(kind, **{key: value for key, value in specific.items() if key != "license_evidence"})
                self.validate([entry], [issue])

    def test_exception_for_one_crate_version_does_not_accept_another_version(self) -> None:
        entry = exact_exception(
            "advisory",
            advisory_id="RUSTSEC-2025-0001",
            source="registry+https://github.com/rust-lang/crates.io-index",
        )
        original = finding(
            "advisory",
            version="1.2.3",
            advisory_id="RUSTSEC-2025-0001",
            source="registry+https://github.com/rust-lang/crates.io-index",
        )
        later = finding(
            "advisory",
            version="1.2.4",
            advisory_id="RUSTSEC-2025-0001",
            source="registry+https://github.com/rust-lang/crates.io-index",
        )
        with self.assertRaises(self.policy_error()):
            self.validate([entry], [original, later])

    def test_exception_does_not_accept_an_unrelated_package(self) -> None:
        entry = exact_exception(
            "duplicate",
            source="registry+https://github.com/rust-lang/crates.io-index",
        )
        issue = finding(
            "duplicate",
            package_name="other-crate",
            source="registry+https://github.com/rust-lang/crates.io-index",
        )
        with self.assertRaises(self.policy_error()):
            self.validate([entry], [issue])

    def test_architecture_ban_exceptions_cannot_waive_the_three_adr_0005_crates(self) -> None:
        for banned in ("native-tls", "openssl", "openssl-sys"):
            with self.subTest(package=banned):
                entry = exact_exception(
                    "duplicate",
                    package_name=banned,
                    source="registry+https://github.com/rust-lang/crates.io-index",
                )
                issue = finding(
                    "ban",
                    package_name=banned,
                    rule="ADR-0005 rustls TLS backend ban",
                    source="registry+https://github.com/rust-lang/crates.io-index",
                )
                with self.assertRaises(self.policy_error()) as context:
                    self.validate([entry], [issue])
                self.assertIn("ADR-0005", str(context.exception))

    def test_wildcard_requirement_cannot_be_waived_by_an_exception(self) -> None:
        entry = exact_exception("duplicate", version="*", source="registry+https://github.com/rust-lang/crates.io-index")
        issue = finding("wildcard", version="*", source="registry+https://github.com/rust-lang/crates.io-index")
        with self.assertRaises(self.policy_error()):
            self.validate([entry], [issue])

    def test_missing_required_exception_fields_fail_validation(self) -> None:
        required_fields = ("rationale", "mitigation", "owner", "reviewed_by", "approval_ref")
        issue = finding("duplicate", source="registry+https://github.com/rust-lang/crates.io-index")
        for field in required_fields:
            with self.subTest(field=field):
                entry = exact_exception("duplicate", source=issue["source"])
                del entry[field]
                with self.assertRaises(self.policy_error()):
                    self.validate([entry], [issue])

    def test_license_waiver_without_human_reviewed_evidence_is_rejected(self) -> None:
        entry = exact_exception("license", source="registry+https://github.com/rust-lang/crates.io-index")
        issue = finding("license", source="registry+https://github.com/rust-lang/crates.io-index")
        with self.assertRaises(self.policy_error()):
            self.validate([entry], [issue])

    def test_license_clarification_requires_safe_file_paths_and_valid_hashes(self) -> None:
        source = "registry+https://github.com/rust-lang/crates.io-index"
        valid_evidence = {
            "reviewed_by": "Security reviewer",
            "reference": "review-42",
            "disposition": "clarify",
            "expression": "MIT",
            "license_files": [{"path": "LICENSE", "hash": "0xbd0eed23"}],
        }
        invalid_evidence = (
            {**valid_evidence, "license_files": []},
            {**valid_evidence, "license_files": [{"path": "LICENSE"}]},
            {**valid_evidence, "license_files": [{"path": "../LICENSE", "hash": "0xbd0eed23"}]},
            {**valid_evidence, "license_files": [{"path": "LICENSE", "hash": "0xnot-a-hash"}]},
            {**valid_evidence, "license_files": [{"path": "LICENSE", "hash": "0x10000000000000000"}]},
        )
        for evidence in invalid_evidence:
            with self.subTest(evidence=evidence):
                entry = exact_exception("license", source=source, license_evidence=evidence)
                with self.assertRaises(self.policy_error()):
                    self.validate([entry], [finding("license", source=source)])

    def test_review_date_cannot_be_future_dated(self) -> None:
        entry = exact_exception(
            "duplicate",
            source="registry+https://github.com/rust-lang/crates.io-index",
            reviewed_on="2026-01-16",
        )
        with self.assertRaises(self.policy_error()):
            self.validate([entry], [finding("duplicate", source=entry["source"])])

    def test_expired_exception_is_rejected(self) -> None:
        entry = exact_exception(
            "duplicate",
            source="registry+https://github.com/rust-lang/crates.io-index",
            reviewed_on="2025-08-01",
            expires_on="2025-10-01",
        )
        with self.assertRaises(self.policy_error()):
            self.validate([entry], [finding("duplicate", source=entry["source"])])

    def test_exception_expiry_cannot_exceed_ninety_days_after_review(self) -> None:
        entry = exact_exception(
            "duplicate",
            source="registry+https://github.com/rust-lang/crates.io-index",
            expires_on="2026-03-02",
        )
        with self.assertRaises(self.policy_error()):
            self.validate([entry], [finding("duplicate", source=entry["source"])])

    def test_duplicate_exception_ids_are_rejected(self) -> None:
        source = "registry+https://github.com/rust-lang/crates.io-index"
        first = exact_exception("duplicate", package_name="first-crate", source=source)
        second = exact_exception("duplicate", package_name="second-crate", source=source)
        issues = [
            finding("duplicate", package_name="first-crate", source=source),
            finding("duplicate", package_name="second-crate", source=source),
        ]
        with self.assertRaises(self.policy_error()):
            self.validate([first, second], issues)

    def test_wildcards_and_version_ranges_are_not_exact_exception_scope(self) -> None:
        source = "registry+https://github.com/rust-lang/crates.io-index"
        for field, scope in (("package", "example-*"), ("version", "*"), ("version", ">=1.2")):
            with self.subTest(field=field, scope=scope):
                entry = exact_exception("duplicate", source=source)
                entry[field] = scope
                with self.assertRaises(self.policy_error()):
                    self.validate([entry], [finding("duplicate", source=source)])

    def test_valid_but_orphaned_exception_entry_is_rejected(self) -> None:
        entry = exact_exception("duplicate", source="registry+https://github.com/rust-lang/crates.io-index")
        with self.assertRaises(self.policy_error()):
            self.validate([entry], [])

    def test_secret_bearing_source_is_rejected_without_echoing_credentials(self) -> None:
        secret_source = "https://kmip-user:sentinel-password@example.invalid/crate?token=sentinel-token"
        entry = exact_exception("source", source=secret_source)
        issue = finding("source", source=secret_source)
        with self.assertRaises(self.policy_error()) as context:
            self.validate([entry], [issue])
        message = str(context.exception)
        self.assertNotIn("sentinel-password", message)
        self.assertNotIn("sentinel-token", message)
        self.assertNotIn("kmip-user", message)

    def test_malformed_source_url_fails_with_a_safe_policy_diagnostic(self) -> None:
        malformed_source = "git+https://[invalid"
        entry = exact_exception("source", source=malformed_source)
        issue = finding("source", source=malformed_source)
        try:
            self.validate([entry], [issue])
        except Exception as error:
            self.assertIsInstance(error, self.policy_error(), "malformed URLs must be normalized as PolicyError")
            self.assertNotIn(malformed_source, str(error))
        else:
            self.fail("malformed source URLs must fail closed")


class DependencyPolicyRunnerContractTests(unittest.TestCase):
    RUNNER = REPOSITORY_ROOT / "scripts" / "Test-DependencyPolicy.ps1"
    LOCKFILE = REPOSITORY_ROOT / "Cargo.lock"

    @classmethod
    def setUpClass(cls) -> None:
        cls.runner_contents = cls.RUNNER.read_text(encoding="utf-8") if cls.RUNNER.is_file() else None

    def require_runner(self) -> str:
        self.assertIsNotNone(
            self.runner_contents,
            "scripts/Test-DependencyPolicy.ps1 must enforce the dependency-policy runner contract.",
        )
        return self.runner_contents

    def test_runner_selects_both_locked_workspaces_and_lockfiles(self) -> None:
        contents = self.require_runner()
        for path in ("Cargo.toml", "fuzz/Cargo.toml", "Cargo.lock", "fuzz/Cargo.lock"):
            with self.subTest(path=path):
                self.assertIn(path, contents)
        self.assertIn("--locked", contents)
        self.assertIn("--workspace", contents)
        self.assertIn("--all-features", contents)

    def test_runner_fails_for_missing_lockfiles_and_stale_locks_remain_locked(self) -> None:
        contents = self.require_runner()
        self.assertRegex(contents, r"(?i)Test-Path")
        self.assertIn("--locked", contents)

    def test_metadata_enables_all_features_without_filtering_platforms(self) -> None:
        contents = self.require_runner()
        for argument in ("metadata", "--format-version", "--all-features", "--locked"):
            with self.subTest(argument=argument):
                self.assertIn(argument, contents)
        self.assertNotIn("--filter-platform", contents)

    def test_runner_preserves_the_unfiltered_graph_including_the_uefi_lockfile_edge(self) -> None:
        contents = self.require_runner()
        self.assertNotIn("--target", contents)
        self.assertNotIn("--filter-platform", contents)
        self.assertRegex(self.LOCKFILE.read_text(encoding="utf-8"), r'(?m)^name = "r-efi"$')

    def test_runner_captures_and_validates_the_observed_rustc_host(self) -> None:
        contents = self.require_runner()
        self.assertRegex(contents, r"(?i)rustc(?:\.exe)?\s+-vV")
        self.assertIn("parse_rustc_host", contents)
        self.assertIn("validate_host_triple", contents)

    def test_runner_refreshes_rustsec_per_workspace_and_reports_sha_and_timestamp(self) -> None:
        contents = self.require_runner()
        for required in ("RustSec", "CARGO_HOME", "root", "fuzz", "timestamp", "commit"):
            with self.subTest(required=required):
                self.assertIn(required.lower(), contents.lower())
        self.assertNotIn("--offline", contents)
        self.assertNotIn("--frozen", contents)

    def test_runner_validates_database_remote_and_fails_on_missing_evidence(self) -> None:
        contents = self.require_runner()
        self.assertRegex(contents, r"(?i)remote")
        self.assertRegex(contents, r"(?i)RustSec.{0,100}(?:Advisory|advisory).{0,100}(?:DB|db)")
        self.assertRegex(contents, r"(?i)(?:commit|revision).{0,100}(?:timestamp|date)")
        self.assertRegex(contents, r"(?i)(?:throw|Write-Error|exit\s+1)")

    def test_runner_checks_the_python_metadata_validator_and_preserves_lockfiles(self) -> None:
        contents = self.require_runner()
        self.assertIn("dependency_policy.py", contents)
        self.assertRegex(contents, r"(?i)Get-FileHash|SHA256")
        self.assertRegex(contents, r"(?i)(?:Cargo\.lock|fuzz/Cargo\.lock).{0,200}(?:compare|unchanged|hash)")

    def test_runner_requests_structured_diagnostics_and_formats_failures_safely(self) -> None:
        contents = self.require_runner()
        for required in ("'--format', 'json'", "'--color', 'never'", "-CargoDenyDiagnostics", "Format-CargoDenyFailure"):
            with self.subTest(required=required):
                self.assertIn(required, contents)

    def test_runner_executes_negative_fixtures_with_the_verified_pinned_binary(self) -> None:
        contents = self.require_runner()
        self.assertIn("$env:CARGO_DENY = $denyExecutable", contents)
        self.assertIn(
            "scripts.tests.test_dependency_policy.DependencyPolicyApiTests.test_policy_scans_preserve_both_lockfiles_and_resolved_package_versions",
            contents,
        )
        self.assertIn("root and fuzz lockfile/resolved-version invariance", contents)
        self.assertIn("scripts.tests.test_cargo_deny_fixtures", contents)
        self.assertIn("cargo-deny negative fixtures", contents)
        self.assertLess(
            contents.index("Installed cargo-deny version did not match"),
            contents.index("scripts.tests.test_cargo_deny_fixtures"),
        )
        self.assertLess(
            contents.index("$env:CARGO_DENY = $denyExecutable"),
            contents.index("test_policy_scans_preserve_both_lockfiles_and_resolved_package_versions"),
        )


class CargoDenyDiagnosticTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        if POLICY is None:
            raise AssertionError(f"dependency policy module is unavailable: {POLICY_LOAD_ERROR}")

    def test_failure_report_retains_allowlisted_finding_fields_and_redacts_secrets(self) -> None:
        formatter = getattr(POLICY, "format_cargo_deny_diagnostics", None)
        self.assertTrue(callable(formatter), "cargo-deny diagnostic formatter must be implemented")
        raw_output = "\n".join(
            (
                '{"type":"diagnostic","fields":{"severity":"error","code":"rejected",'
                '"message":"license sentinel-message-secret","labels":[{"span":"GPL-3.0-only"}],'
                '"graphs":[{"Krate":{"name":"bad-license","version":"2.3.4","kind":null},'
                '"repeat":false,"parents":[]}]}}',
                '{"type":"diagnostic","fields":{"severity":"error","code":"vulnerability",'
                '"message":"advisory sentinel-message-token","advisory":{"id":"RUSTSEC-2026-0001"},'
                '"graphs":[{"Krate":{"name":"unsafe-crate","version":"1.2.3","kind":null},'
                '"repeat":false,"parents":[]}]}}',
            )
        )
        secret_source = (
            "registry+https://sentinel-user:sentinel-password@packages.example.invalid/index"
            "?token=sentinel-token&safe=sentinel-query-value"
        )
        metadata = {
            "root": {"packages": [{"name": "bad-license", "version": "2.3.4", "source": secret_source}]},
            "fuzz": {"packages": [{"name": "unsafe-crate", "version": "1.2.3", "source": None}]},
        }

        report = formatter(raw_output, metadata)

        for expected in (
            "bad-license@2.3.4",
            "rule=license-rejected",
            "unsafe-crate@1.2.3",
            "rule=vulnerability",
            "advisory=RUSTSEC-2026-0001",
            "source=registry+https://packages.example.invalid/<redacted>",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, report)
        for secret in (
            "sentinel-user",
            "sentinel-password",
            "sentinel-token",
            "sentinel-query-value",
            "sentinel-message-secret",
            "sentinel-message-token",
        ):
            with self.subTest(secret=secret):
                self.assertNotIn(secret, report)
        self.assertNotIn("GPL-3.0-only", report)

    def test_malformed_or_unrecognized_diagnostics_never_echo_raw_output(self) -> None:
        formatter = getattr(POLICY, "format_cargo_deny_diagnostics", None)
        self.assertTrue(callable(formatter), "cargo-deny diagnostic formatter must be implemented")
        raw_output = '{"unexpected":"sentinel-secret-material"}'

        report = formatter(raw_output, {"root": {"packages": []}, "fuzz": {"packages": []}})

        self.assertIn("diagnostics unavailable", report)
        self.assertNotIn("sentinel-secret-material", report)

    def test_failure_report_omits_untrusted_license_ref_labels(self) -> None:
        formatter = getattr(POLICY, "format_cargo_deny_diagnostics", None)
        self.assertTrue(callable(formatter), "cargo-deny diagnostic formatter must be implemented")
        raw_output = (
            '{"type":"diagnostic","fields":{"severity":"error","code":"rejected",'
            '"labels":[{"span":"LicenseRef-SENTINELSECRET00000000"}],'
            '"graphs":[{"Krate":{"name":"bad-license","version":"2.3.4"}}]}}'
        )

        report = formatter(raw_output, {"root": {"packages": []}, "fuzz": {"packages": []}})

        self.assertNotIn("LicenseRef-SENTINELSECRET00000000", report)

    def test_untrusted_json_field_shapes_fail_safely(self) -> None:
        formatter = getattr(POLICY, "format_cargo_deny_diagnostics", None)
        self.assertTrue(callable(formatter), "cargo-deny diagnostic formatter must be implemented")
        raw_output = (
            '{"type":"diagnostic","fields":{"severity":[],"code":"sentinel-secret-code",'
            '"message":"sentinel-secret",'
            '"graphs":[{"Krate":{"name":"safe-crate","version":"1.0.0"}}]}}'
        )

        report = formatter(raw_output, {"root": {"packages": []}, "fuzz": {"packages": []}})

        self.assertIn("safe-crate@1.0.0", report)
        self.assertIn("rule=policy-check", report)
        self.assertNotIn("sentinel-secret", report)

    def test_cli_formats_piped_json_without_persisting_or_echoing_raw_diagnostics(self) -> None:
        raw_output = (
            '{"type":"diagnostic","fields":{"severity":"error","code":"rejected",'
            '"message":"sentinel-raw-message","labels":[{"span":"GPL-3.0-only"}],'
            '"graphs":[{"Krate":{"name":"cli-crate","version":"3.2.1"}}]}}'
        )
        with tempfile.TemporaryDirectory() as temporary:
            temp_root = Path(temporary)
            root_metadata = temp_root / "root.json"
            fuzz_metadata = temp_root / "fuzz.json"
            root_metadata.write_text(
                json.dumps({"packages": [{"name": "cli-crate", "version": "3.2.1", "source": None}]}),
                encoding="utf-8",
            )
            fuzz_metadata.write_text(json.dumps({"packages": []}), encoding="utf-8")
            result = subprocess.run(
                [
                    sys.executable,
                    str(POLICY_PATH),
                    "--format-cargo-deny-diagnostics",
                    "--root-metadata",
                    str(root_metadata),
                    "--fuzz-metadata",
                    str(fuzz_metadata),
                ],
                input=raw_output,
                capture_output=True,
                text=True,
                check=False,
            )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("cli-crate@3.2.1", result.stdout)
        self.assertIn("rule=license-rejected", result.stdout)
        self.assertNotIn("sentinel-raw-message", result.stdout)


if __name__ == "__main__":
    unittest.main(verbosity=2)
