"""Tests for catalog-derived, fail-closed client profile metadata."""

from __future__ import annotations

import io
import json
import re
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import call, patch

from tools.normative_catalog.generate_profiles import (
    CATALOG_PATH,
    MAX_CATALOG_BYTES,
    ProfileGenerationError,
    load_profile_targets,
    compute_profile_closure,
    main as generate_profiles_main,
    parse_profile_targets,
    ProfileTargets,
    render_profiles_source,
    rust_string_literal,
)
from tools.normative_catalog.validate import load_validated_catalog


ROOT = Path(__file__).resolve().parents[3]


def closure_catalog() -> dict[str, object]:
    return {
        "source_clauses": [
            {"clause_id": "KMIPKIT-CLAUSE-PROF-6.1-001", "source_id": "profiles", "section": "6.1"},
            {"clause_id": "KMIPKIT-CLAUSE-PROF-6.1.1-001", "source_id": "profiles", "section": "6.1.1"},
            {"clause_id": "KMIPKIT-CLAUSE-SPEC-1-001", "source_id": "spec", "section": "1"},
            {"clause_id": "KMIPKIT-CLAUSE-SPEC-1.1-001", "source_id": "spec", "section": "1.1"},
            {"clause_id": "KMIPKIT-CLAUSE-SPEC-2-001", "source_id": "spec", "section": "2"},
            {"clause_id": "KMIPKIT-CLAUSE-PROF-3.1-001", "source_id": "profiles", "section": "3.1"},
            {"clause_id": "KMIPKIT-CLAUSE-PROF-3.1.1-001", "source_id": "profiles", "section": "3.1.1"},
        ],
        "normative_inclusions": [
            {
                "inclusion_id": "KMIPKIT-INCLUSION-001",
                "source_clause_id": "KMIPKIT-CLAUSE-PROF-6.1-001",
                "included_source_id": "spec",
                "included_section": None,
            },
            {
                "inclusion_id": "KMIPKIT-INCLUSION-002",
                "source_clause_id": "KMIPKIT-CLAUSE-SPEC-1-001",
                "included_source_id": "profiles",
                "included_section": "3.1",
            },
        ],
        "requirements": [
            {
                "requirement_id": "KMIPKIT-REQ-SPEC-1-001",
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-1-001"],
                "role": "client",
                "scope_state": "client_1_0",
            },
            {
                "requirement_id": "KMIPKIT-REQ-SPEC-1.1-001",
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-1.1-001"],
                "role": "client",
                "scope_state": "client_1_0",
            },
            {
                "requirement_id": "KMIPKIT-REQ-SPEC-2-001",
                "source_clause_ids": ["KMIPKIT-CLAUSE-SPEC-2-001"],
                "role": "server",
                "scope_state": "server_only",
            },
            {
                "requirement_id": "KMIPKIT-REQ-PROF-3.1.1-001",
                "source_clause_ids": ["KMIPKIT-CLAUSE-PROF-3.1.1-001"],
                "role": "client",
                "scope_state": "profile_conditional",
                "profile_ids": ["KMIPKIT-PROFILE-BASELINE-CLIENT"],
            },
        ],
    }


class ProfileMetadataGeneratorTests(unittest.TestCase):
    def test_source_oracle_covers_all_profiles_and_expected_scope_counts(self) -> None:
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        profiles = oracle["profiles"]

        self.assertEqual(oracle["counts"]["profiles"], 18)
        self.assertEqual(len(profiles), 18)
        self.assertEqual(
            sum(profile["applicability"] in {"client_1_0", "conditional"} for profile in profiles),
            15,
        )
        self.assertEqual(
            sum(profile["applicability"] == "out_of_scope" for profile in profiles),
            3,
        )
        baseline = next(
            profile for profile in profiles
            if profile["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )
        mandatory_unavailable = [
            case for case in baseline["test_cases"]
            if case["fixture_status"] == "unavailable"
        ]
        self.assertEqual(len(baseline["test_cases"]), 16)
        self.assertEqual(len(mandatory_unavailable), 16)
        self.assertTrue(all("-M-" in case["official_case_id"] for case in baseline["test_cases"]))

    def test_checked_in_client_profile_closures_match_current_scope(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))

        violations = []
        for profile in catalog["profiles"]:
            if profile["role"] not in {"client", "both"}:
                continue
            closure = compute_profile_closure(
                catalog,
                profile["direct_source_clause_ids"],
                profile_id=profile["profile_id"],
            )
            computed_ids = set(closure.requirement_ids)
            stored_ids = set(profile["requirement_ids"])
            stale_ids = sorted(stored_ids - computed_ids)
            missing_ids = sorted(computed_ids - stored_ids)
            if (
                profile["source_clause_ids"] != list(closure.source_clause_ids)
                or stale_ids
                or missing_ids
            ):
                violations.append(
                    f"{profile['profile_id']}: stale={stale_ids[:3]}, missing={missing_ids[:3]}"
                )

        self.assertEqual(violations, [], "client profile closures differ from current catalog scope")

    def test_client_profile_closure_keeps_client_obligations_with_mixed_scope(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        baseline = next(
            profile for profile in catalog["profiles"]
            if profile["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )

        closure = compute_profile_closure(
            catalog,
            baseline["direct_source_clause_ids"],
            profile_id=baseline["profile_id"],
        )

        # Their reviewed source atoms separate a client obligation from a
        # server obligation; the catalog aggregates those atoms as `mixed`.
        for requirement_id in (
            "KMIPKIT-REQ-SPEC-6.1.5-001",
            "KMIPKIT-REQ-SPEC-6.1.7-005",
        ):
            with self.subTest(requirement_id=requirement_id):
                requirement = next(
                    row for row in catalog["requirements"]
                    if row["requirement_id"] == requirement_id
                )
                self.assertEqual(requirement["role"], "client")
                self.assertEqual(requirement["scope_state"], "mixed")
                self.assertTrue(
                    requirement_id in closure.requirement_ids,
                    f"missing mixed-scope client requirement from closure: {requirement_id}",
                )

    def test_generated_client_requirement_metadata_covers_profile_closures(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = load_validated_catalog(catalog_path.read_bytes(), ROOT, require_complete=True)
        targets = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-BASELINE-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )

        generated = render_profiles_source(catalog, targets)
        requirement_block = generated.split("pub const CLIENT_REQUIREMENTS: &[GeneratedClientRequirement] = &[", 1)[1]
        requirement_block = requirement_block.split("\n];", 1)[0]
        metadata_ids = set(re.findall(r'requirement_id: "([^"]+)"', requirement_block))
        closed_requirement_ids = {
            requirement_id
            for profile in catalog["profiles"]
            if profile["role"] in {"client", "both"}
            for requirement_id in profile["requirement_ids"]
        }

        missing_metadata = sorted(closed_requirement_ids - metadata_ids)
        self.assertEqual(
            missing_metadata,
            [],
            "every requirement in a client profile closure needs generated runtime metadata",
        )

    def test_profile_closure_generator_matches_the_pinned_source_oracle(self) -> None:
        """Compare closure expansion with static source-derived arrays for all 18 clients."""
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
        oracle_path = ROOT / "tools" / "normative_catalog" / "tests" / "fixtures" / "profile-closure-v2.1.json"
        oracle = json.loads(oracle_path.read_text(encoding="utf-8"))
        oracle_profiles = {profile["profile_id"]: profile for profile in oracle["profiles"]}
        catalog_profiles = {
            profile["profile_id"]: profile
            for profile in catalog["profiles"]
            if profile["role"] == "client"
        }

        self.assertEqual(
            oracle["oracle_status"],
            "complete",
            "profile closure comparison requires the completed pinned-source oracle",
        )
        self.assertIs(
            oracle["counts"]["requirement_crosswalk_complete"],
            True,
            "a partial requirement crosswalk cannot stand in for exact profile closure",
        )
        self.assertEqual(oracle["counts"]["profiles"], 18)
        self.assertEqual(len(oracle_profiles), 18)
        profiles_without_full_requirements = sorted(
            profile_id
            for profile_id, expected in oracle_profiles.items()
            if "requirement_ids" not in expected
        )
        self.assertEqual(
            profiles_without_full_requirements,
            [],
            "every profile must carry a complete source-derived requirement set",
        )

        self.assertEqual(set(catalog_profiles), set(oracle_profiles))
        for profile_id, expected in oracle_profiles.items():
            with self.subTest(profile_id=profile_id):
                profile = catalog_profiles[profile_id]
                self.assertEqual(
                    profile["direct_source_clause_ids"],
                    expected["direct_source_clause_ids"],
                    f"{profile_id} direct source roots differ from pinned Profiles section headings",
                )
                closure = compute_profile_closure(
                    catalog,
                    profile["direct_source_clause_ids"],
                    profile_id=profile_id,
                )
                self.assertEqual(
                    closure.source_clause_ids,
                    tuple(expected["source_clause_ids"]),
                    f"{profile_id} clause closure differs from the pinned-source oracle",
                )
                self.assertEqual(
                    set(closure.requirement_ids),
                    set(expected["requirement_ids"]),
                    f"{profile_id} generated requirement closure differs from the complete pinned-source oracle",
                )

    def test_closure_expands_whole_sources_sections_and_applicable_requirements(self) -> None:
        catalog = closure_catalog()

        closure = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-PROF-6.1-001"],
            profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
        )

        self.assertEqual(
            closure.source_clause_ids,
            (
                "KMIPKIT-CLAUSE-PROF-3.1-001",
                "KMIPKIT-CLAUSE-PROF-3.1.1-001",
                "KMIPKIT-CLAUSE-PROF-6.1-001",
                "KMIPKIT-CLAUSE-PROF-6.1.1-001",
                "KMIPKIT-CLAUSE-SPEC-1-001",
                "KMIPKIT-CLAUSE-SPEC-1.1-001",
                "KMIPKIT-CLAUSE-SPEC-2-001",
            ),
        )
        self.assertEqual(
            closure.requirement_ids,
            (
                "KMIPKIT-REQ-PROF-3.1.1-001",
                "KMIPKIT-REQ-SPEC-1-001",
                "KMIPKIT-REQ-SPEC-1.1-001",
            ),
        )

    def test_closure_does_not_expand_informative_references(self) -> None:
        catalog = closure_catalog()
        catalog["normative_inclusions"] = catalog["normative_inclusions"][:1]
        catalog["informative_references"] = [
            {
                "source_clause_id": "KMIPKIT-CLAUSE-SPEC-1-001",
                "included_source_id": "profiles",
                "included_section": "3.1",
            }
        ]

        closure = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-PROF-6.1-001"],
            profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
        )

        self.assertIn("KMIPKIT-CLAUSE-SPEC-1-001", closure.source_clause_ids)
        self.assertNotIn("KMIPKIT-CLAUSE-PROF-3.1-001", closure.source_clause_ids)

    def test_conditional_requirement_applies_only_to_profiles_linked_by_the_catalog(self) -> None:
        catalog = closure_catalog()

        baseline = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-PROF-6.1-001"],
            profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
        )
        unrelated = compute_profile_closure(
            catalog,
            ["KMIPKIT-CLAUSE-PROF-6.1-001"],
            profile_id="KMIPKIT-PROFILE-UNRELATED-CLIENT",
        )

        self.assertIn("KMIPKIT-REQ-PROF-3.1.1-001", baseline.requirement_ids)
        self.assertNotIn("KMIPKIT-REQ-PROF-3.1.1-001", unrelated.requirement_ids)

    def test_closure_rejects_recursive_normative_inclusion_cycles(self) -> None:
        catalog = closure_catalog()
        inclusions = catalog["normative_inclusions"]
        assert isinstance(inclusions, list)
        inclusions.append(
            {
                "inclusion_id": "KMIPKIT-INCLUSION-CYCLE",
                "source_clause_id": "KMIPKIT-CLAUSE-PROF-3.1-001",
                "included_source_id": "profiles",
                "included_section": "6.1",
            }
        )

        with self.assertRaisesRegex(ProfileGenerationError, "cyclic normative inclusion"):
            compute_profile_closure(
                catalog,
                ["KMIPKIT-CLAUSE-PROF-6.1-001"],
                profile_id="KMIPKIT-PROFILE-BASELINE-CLIENT",
            )

    def test_profile_generator_rejects_invalid_and_over_limit_catalog_inputs(self) -> None:
        for label, raw_catalog in (
            ("invalid JSON", b"{"),
            ("over-limit input", b"x" * (MAX_CATALOG_BYTES + 1)),
        ):
            with self.subTest(label=label):
                reads: list[tuple[Path, str, int]] = []

                def read_catalog(repository: Path, path: str, *, max_bytes: int) -> bytes:
                    reads.append((repository, path, max_bytes))
                    if path != CATALOG_PATH:
                        raise AssertionError(f"generator read {path} after catalog validation should have failed")
                    return raw_catalog

                stderr = io.StringIO()
                with (
                    patch("tools.normative_catalog.generate_profiles.safe_read_bytes", side_effect=read_catalog),
                    patch("sys.stderr", new=stderr),
                ):
                    result = generate_profiles_main(["--check", "--repo-root", str(ROOT)])

                self.assertEqual(result, 1)
                self.assertIn("profile metadata generation failed:", stderr.getvalue())
                self.assertEqual(reads, [(ROOT.resolve(), CATALOG_PATH, MAX_CATALOG_BYTES)])

    def test_rust_string_literal_escapes_syntax_and_control_characters(self) -> None:
        value = 'quote" slash\\ line\n tab\t control\x01 snowman ☃'

        self.assertEqual(
            rust_string_literal(value),
            '"quote\\" slash\\\\ line\\n tab\\t control\\u{1} snowman ☃"',
        )


    @unittest.skipUnless(shutil.which("rustc") and shutil.which("rustfmt"), "Rust toolchain is required")
    def test_generated_source_compiles_and_adversarial_metadata_round_trips(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = load_validated_catalog(catalog_path.read_bytes(), ROOT, require_complete=True)
        baseline = next(
            profile
            for profile in catalog["profiles"]
            if profile["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )
        value = 'quote" slash\\ line\n carriage\r tab\t nul\x00 control\x01 DEL\x7f C1\x85 snowman ☃ emoji 🦄'
        baseline["name"] = value
        targets = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-BASELINE-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )
        generated = render_profiles_source(catalog, targets)
        expected_bytes = ", ".join(str(byte) for byte in value.encode("utf-8"))
        generated += "\n".join([
            "#[cfg(test)]",
            "mod generated_metadata_literal_test {",
            "    use super::CLIENT_PROFILES;",
            "",
            "    #[test]",
            "    fn source_metadata_is_data_after_compilation() {",
            "        let baseline = CLIENT_PROFILES",
            "            .iter()",
            "            .find(|profile| profile.profile_id == \"KMIPKIT-PROFILE-BASELINE-CLIENT\")",
            "            .expect(\"the fixture catalog includes Baseline Client\");",
            f"        assert_eq!(baseline.name.as_bytes(), &[{expected_bytes}]);",
            "    }",
            "}",
            "",
        ])
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "profiles_generated.rs"
            binary = Path(directory) / "profiles_generated_tests.exe"
            source.write_text(generated, encoding="utf-8")
            compile_result = subprocess.run(
                ["rustc", "--edition=2024", "--test", str(source), "-o", str(binary)],
                cwd=directory,
                capture_output=True,
                check=False,
                text=True,
                encoding="utf-8",
            )
            self.assertEqual(compile_result.returncode, 0, compile_result.stderr)
            run_result = subprocess.run(
                [str(binary)],
                cwd=directory,
                capture_output=True,
                check=False,
                text=True,
                encoding="utf-8",
            )
            self.assertEqual(run_result.returncode, 0, run_result.stdout + run_result.stderr)


class ProfileTargetManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.catalog = {
            "profiles": [
                {
                    "profile_id": "KMIPKIT-PROFILE-BASELINE-CLIENT",
                    "role": "client",
                    "applicability": "client_1_0",
                },
                {
                    "profile_id": "KMIPKIT-PROFILE-SERVER-ONLY",
                    "role": "server",
                    "applicability": "server_only",
                },
            ]
        }
        self.manifest = {
            "release_line": "1.0",
            "profile_ids": ["KMIPKIT-PROFILE-BASELINE-CLIENT"],
            "decision_ref": "KMIPKIT-0010/FR-003",
        }

    def parse(self, manifest: dict[str, object] | bytes | str):
        if isinstance(manifest, dict):
            raw = json.dumps(manifest).encode("utf-8")
        elif isinstance(manifest, str):
            raw = manifest.encode("utf-8")
        else:
            raw = manifest
        return parse_profile_targets(raw, self.catalog, ROOT)

    def test_loads_fixed_baseline_target_and_keeps_selection_separate(self) -> None:
        targets = self.parse(self.manifest)

        self.assertEqual(targets.release_line, "1.0")
        self.assertEqual(targets.profile_ids, ("KMIPKIT-PROFILE-BASELINE-CLIENT",))
        self.assertEqual(targets.decision_ref, "KMIPKIT-0010/FR-003")

    def test_rejects_duplicate_keys_and_invalid_utf8(self) -> None:
        duplicate_keys = (
            b'{"release_line":"1.0","release_line":"1.0",'
            b'"profile_ids":["KMIPKIT-PROFILE-BASELINE-CLIENT"],'
            b'"decision_ref":"KMIPKIT-0010/FR-003"}'
        )
        with self.assertRaises(ProfileGenerationError):
            self.parse(duplicate_keys)
        with self.assertRaises(ProfileGenerationError):
            self.parse(b"\xff")

    def test_rejects_wrong_release_unknown_profile_and_unapproved_role(self) -> None:
        for overrides in (
            {"release_line": "2.0"},
            {"profile_ids": ["KMIPKIT-PROFILE-UNKNOWN"]},
            {"profile_ids": ["KMIPKIT-PROFILE-SERVER-ONLY"]},
        ):
            with self.subTest(overrides=overrides):
                with self.assertRaises(ProfileGenerationError):
                    self.parse(self.manifest | overrides)

    def test_rejects_empty_duplicate_and_unresolved_decision_targets(self) -> None:
        for overrides in (
            {"profile_ids": []},
            {"profile_ids": ["KMIPKIT-PROFILE-BASELINE-CLIENT"] * 2},
            {"decision_ref": "KMIPKIT-0010/FR-999"},
            {"extra": "unknown"},
        ):
            with self.subTest(overrides=overrides):
                with self.assertRaises(ProfileGenerationError):
                    self.parse(self.manifest | overrides)

    def test_rejects_missing_fields_wrong_types_empty_ids_and_invalid_json(self) -> None:
        invalid_documents = (
            {},
            {"release_line": "1.0", "profile_ids": ["KMIPKIT-PROFILE-BASELINE-CLIENT"]},
            self.manifest | {"release_line": 1},
            self.manifest | {"profile_ids": "KMIPKIT-PROFILE-BASELINE-CLIENT"},
            self.manifest | {"profile_ids": [None]},
            self.manifest | {"profile_ids": [""]},
            self.manifest | {"decision_ref": []},
            self.manifest | {"decision_ref": ""},
        )
        for document in invalid_documents:
            with self.subTest(document=document), self.assertRaises(ProfileGenerationError):
                self.parse(document)

        with self.assertRaises(ProfileGenerationError):
            self.parse(b'{"release_line":')
        oversized = json.dumps(self.manifest).encode("utf-8")
        oversized += b" " * (65_537 - len(oversized))
        with self.assertRaisesRegex(ProfileGenerationError, "size limit"):
            self.parse(oversized)

    def test_fixed_loader_rejects_a_symlinked_target_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            target = repository / "specification" / "catalog" / "profile-targets.json"
            target.parent.mkdir(parents=True)
            external = repository / "external-targets.json"
            external.write_text(json.dumps(self.manifest), encoding="utf-8")
            try:
                target.symlink_to(external)
            except OSError as error:
                self.skipTest(f"symlink creation is unavailable: {error}")

            with self.assertRaises(ProfileGenerationError):
                load_profile_targets(repository, self.catalog)

    def test_fixed_loader_rejects_a_junctioned_target_manifest_directory(self) -> None:
        if not shutil.which("cmd.exe"):
            self.skipTest("Windows cmd.exe is required to create a directory junction")

        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory) / "repository"
            specification = repository / "specification"
            specification.mkdir(parents=True)
            external = Path(directory) / "external-catalog"
            external.mkdir()
            (external / "profile-targets.json").write_text(
                json.dumps(self.manifest), encoding="utf-8"
            )
            junction = specification / "catalog"
            mklink_command = (
                "mklink /J "
                + subprocess.list2cmdline([str(junction), str(external)])
            )
            result = subprocess.run(
                ["cmd.exe", "/d", "/c", mklink_command],
                capture_output=True,
                check=False,
                text=True,
            )
            self.assertEqual(
                result.returncode,
                0,
                msg=f"could not create test junction: {result.stdout}{result.stderr}",
            )

            with self.assertRaises(ProfileGenerationError):
                load_profile_targets(repository, self.catalog)

    def test_fixed_loader_uses_a_64_kib_read_bound(self) -> None:
        with patch(
            "tools.normative_catalog.generate_profiles.safe_read_bytes",
            side_effect=lambda _root, path, *, max_bytes: (
                json.dumps(self.manifest).encode("utf-8")
                if path == "specification/catalog/profile-targets.json"
                else b"**FR-003**"
            ),
        ) as safe_read:
            load_profile_targets(ROOT, self.catalog)

        self.assertIn(
            call(ROOT, "specification/catalog/profile-targets.json", max_bytes=65_536),
            safe_read.call_args_list,
        )

    def test_generated_metadata_preserves_closure_evidence_and_separate_target_state(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = load_validated_catalog(catalog_path.read_bytes(), ROOT, require_complete=True)
        targets = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-BASELINE-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )

        generated = render_profiles_source(catalog, targets)

        self.assertEqual(generated, render_profiles_source(catalog, targets))
        self.assertEqual(generated.count("\n    GeneratedClientProfile {"), 18)
        self.assertIn('claim_state: "not_claimed"', generated)
        self.assertIn("selected_target: true", generated)
        self.assertIn('fixture_availability: "unavailable"', generated)
        self.assertIn('"KMIPKIT-REQ-SPEC-2.1-001"', generated)
        self.assertNotIn("ProfileRuleMapping", generated)

    def test_target_and_profile_prose_do_not_change_requirement_evidence_metadata(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = load_validated_catalog(catalog_path.read_bytes(), ROOT, require_complete=True)
        baseline_target = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-BASELINE-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )
        alternate_target = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-AES-XTS-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )

        baseline_output = render_profiles_source(catalog, baseline_target)
        alternate_target_output = render_profiles_source(catalog, alternate_target)
        changed_prose = json.loads(json.dumps(catalog))
        baseline = next(
            profile
            for profile in changed_prose["profiles"]
            if profile["profile_id"] == "KMIPKIT-PROFILE-BASELINE-CLIENT"
        )
        baseline["name"] = "This prose does not define executable profile rules"
        changed_prose_output = render_profiles_source(changed_prose, baseline_target)

        def requirement_metadata(source: str) -> str:
            return source.split("pub const CLIENT_REQUIREMENTS: &[GeneratedClientRequirement] = &[", 1)[1].split(
                "pub const CLIENT_PROFILES:", 1
            )[0]

        self.assertEqual(requirement_metadata(baseline_output), requirement_metadata(alternate_target_output))
        self.assertEqual(requirement_metadata(baseline_output), requirement_metadata(changed_prose_output))
        self.assertIn("selected_target: true", baseline_output)
        self.assertIn("selected_target: true", alternate_target_output)
        self.assertNotIn("ProfileRuleMapping", baseline_output)
        self.assertNotIn("ProfileRuleMapping", alternate_target_output)

    def test_generated_requirement_metadata_preserves_exact_normative_keywords(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = load_validated_catalog(catalog_path.read_bytes(), ROOT, require_complete=True)
        targets = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-BASELINE-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )

        generated = render_profiles_source(catalog, targets)

        self.assertTrue("pub struct GeneratedClientRequirement" in generated)
        self.assertTrue("pub const CLIENT_REQUIREMENTS" in generated)
        expected_keywords = {
            requirement["source_keyword"]
            for requirement in catalog["requirements"]
            if requirement["role"] in {"client", "both"}
            and requirement["scope_state"] in {"client_1_0", "mixed", "profile_conditional"}
        }
        for keyword in expected_keywords:
            with self.subTest(keyword=keyword):
                self.assertTrue(f'source_keyword: "{keyword}"' in generated)
        self.assertNotIn("ProfileRuleMapping", generated)

    def test_generated_requirements_classify_keyword_specific_verification_categories(self) -> None:
        catalog_path = ROOT / "specification" / "catalog" / "kmip-2.1.json"
        catalog = load_validated_catalog(catalog_path.read_bytes(), ROOT, require_complete=True)
        catalog = json.loads(json.dumps(catalog))
        targets = ProfileTargets(
            "1.0",
            ("KMIPKIT-PROFILE-BASELINE-CLIENT",),
            "KMIPKIT-0010/FR-003",
        )
        expected_categories = {
            "MUST": "RequiredBehavior",
            "SHALL": "RequiredBehavior",
            "REQUIRED": "RequiredBehavior",
            "MUST NOT": "ProhibitedBehavior",
            "SHALL NOT": "ProhibitedBehavior",
            "SHOULD": "RecommendedBehaviorOrDeviation",
            "RECOMMENDED": "RecommendedBehaviorOrDeviation",
            "SHOULD NOT": "DiscouragedAbsenceOrDeviation",
            "MAY": "OptionalCapability",
            "OPTIONAL": "OptionalCapability",
        }
        observed_keywords = {
            requirement["source_keyword"]
            for requirement in catalog["requirements"]
            if requirement["role"] in {"client", "both"}
            and requirement["scope_state"] in {"client_1_0", "mixed", "profile_conditional"}
        }
        template = next(
            requirement
            for requirement in catalog["requirements"]
            if requirement["role"] == "client" and requirement["scope_state"] == "client_1_0"
        )
        for index, keyword in enumerate(sorted(set(expected_categories) - observed_keywords), start=1):
            synthetic_requirement = template.copy()
            synthetic_requirement["requirement_id"] = f"KMIPKIT-REQ-SPEC-TEST-KEYWORD-{index:02}"
            synthetic_requirement["source_keyword"] = keyword
            catalog["requirements"].append(synthetic_requirement)

        generated = render_profiles_source(catalog, targets)
        blocks = re.findall(
            r"    GeneratedClientRequirement \{\n(.*?)\n    \},",
            generated,
            flags=re.DOTALL,
        )
        actual_categories: dict[str, str] = {}
        for block in blocks:
            requirement_id = re.search(r'requirement_id: "([^"]+)"', block)
            category = re.search(
                r"verification_category: GeneratedVerificationCategory::([A-Za-z]+)",
                block,
            )
            if requirement_id is not None and category is not None:
                actual_categories[requirement_id.group(1)] = category.group(1)

        scoped_requirements = {
            requirement["requirement_id"]: expected_categories[requirement["source_keyword"]]
            for requirement in catalog["requirements"]
            if requirement["role"] in {"client", "both"}
            and requirement["scope_state"] in {"client_1_0", "mixed", "profile_conditional"}
        }
        observed_scoped_keywords = {
            requirement["source_keyword"]
            for requirement in catalog["requirements"]
            if requirement["role"] in {"client", "both"}
            and requirement["scope_state"] in {"client_1_0", "mixed", "profile_conditional"}
        }

        self.assertEqual(observed_scoped_keywords, set(expected_categories))
        self.assertEqual(actual_categories, scoped_requirements)
        self.assertEqual(
            set(actual_categories.values()),
            set(expected_categories.values()),
        )


if __name__ == "__main__":
    unittest.main()
