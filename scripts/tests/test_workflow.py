"""Static security and trigger contract checks for the pull-request workflow."""

from __future__ import annotations

import re
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = REPOSITORY_ROOT / ".github" / "workflows" / "ci.yml"
TESTING_GUIDE = REPOSITORY_ROOT / "docs" / "development" / "testing.md"
POLICY_RUNNER = REPOSITORY_ROOT / "scripts" / "Test-DependencyPolicy.ps1"


class WorkflowContractTests(unittest.TestCase):
    COVERAGE_COLLECTION_GUARD = "needs.impact-plan.outputs.coverage_scopes"
    RELEASE_EVIDENCE_JOB_IDS = (
        "core",
        "script-contracts",
        "normative-inventory",
        "coverage",
        "coverage-gate",
        "dependency-policy",
    )
    UV_PYTHON_SETUP = (
        "astral-sh/setup-uv@c771a70e6277c0a99b617c7a806ffedaca235ff9 # astral-sh/setup-uv v9.0.0",
        "version: '0.12.23'",
        "python-version: '3.12'",
        "activate-environment: true",
        "no-project: true",
    )

    @classmethod
    def setUpClass(cls) -> None:
        cls.contents = WORKFLOW.read_text(encoding="utf-8") if WORKFLOW.is_file() else None

    def require_workflow(self) -> str:
        self.assertIsNotNone(self.contents, "The CI workflow must exist before its contract can pass.")
        return self.contents

    def require_job(self, contents: str, job: str) -> str:
        match = re.search(rf"(?ms)^  {re.escape(job)}:\n(.*?)(?=^  [a-z][\w-]*:|\Z)", contents)
        self.assertIsNotNone(match, f"The {job} job must exist.")
        return match.group(1)

    def assert_generator_check_step(
        self,
        job: str,
        *,
        name: str,
        step_id: str,
        command: str,
    ) -> None:
        step = re.search(
            rf"(?ms)^      - name: {re.escape(name)}\n(.*?)(?=^      - name: |\Z)",
            job,
        )
        self.assertIsNotNone(step, f"The normative inventory job must check {name.lower()}.")
        self.assertRegex(step.group(1), rf"(?m)^        id: {re.escape(step_id)}$")
        self.assertRegex(step.group(1), rf"(?m)^        run: {re.escape(command)}$")

    def require_named_step(self, job: str, name: str) -> re.Match[str]:
        match = re.search(
            rf"(?ms)^      - name: {re.escape(name)}\n(.*?)(?=^      - name: |\\Z)",
            job,
        )
        self.assertIsNotNone(match, f"The {name} workflow step must exist.")
        assert match is not None
        return match

    def assert_github_hosted_runner(self, body: str, runner: str = "ubuntu-latest") -> None:
        self.assertRegex(body, rf"(?m)^    runs-on: {re.escape(runner)}$")

    def assert_uv_managed_python_312(self, job: str) -> None:
        for required in self.UV_PYTHON_SETUP:
            with self.subTest(required=required):
                self.assertIn(required, job)
        self.assertNotIn("actions/setup-python@", job)

    def require_policy_job(self, contents: str, job: str) -> str:
        return self.require_job(contents, job)

    def require_policy_runner(self) -> str:
        self.assertTrue(
            POLICY_RUNNER.is_file(),
            "The pinned dependency-policy runner must exist before its workflow contract can pass.",
        )
        return POLICY_RUNNER.read_text(encoding="utf-8")

    def scheduled_policy_job(self, contents: str) -> str:
        jobs_section = contents.split("\njobs:\n", maxsplit=1)[1]
        jobs = re.findall(r"(?ms)^  ([a-z][\w-]*):\n(.*?)(?=^  [a-z][\w-]*:|\Z)", jobs_section)
        for _, body in jobs:
            if "Test-DependencyPolicy.ps1" in body and re.search(r"(?i)github\.event_name.*schedule", body):
                return body
        self.fail("A scheduled job must invoke the dependency-policy runner.")

    @staticmethod
    def inline_run_scalars(contents: str) -> list[tuple[int, str]]:
        scalars: list[tuple[int, str]] = []
        for line_number, line in enumerate(contents.splitlines(), start=1):
            match = re.match(r"^\s*run:\s*(.*?)\s*$", line)
            if match is not None:
                scalars.append((line_number, match.group(1)))
        return scalars

    def test_pull_request_targets_only_supported_integration_branches(self) -> None:
        contents = self.require_workflow()
        self.assertRegex(contents, r"(?m)^\s*pull_request\s*:")
        self.assertIn("master", contents)
        self.assertRegex(contents, r"release/\*\*")
        self.assertNotRegex(contents, r"(?m)^\s*pull_request_target\s*:")
        self.assertNotRegex(contents, r"(?m)^\s*workflow_run\s*:")

    def test_workflow_permissions_are_read_only(self) -> None:
        contents = self.require_workflow()
        self.assertRegex(contents, r"(?ms)^permissions:\s*\n\s*contents:\s*read\b")
        self.assertNotRegex(contents, r"(?m)^\s*(?:write-all|packages:\s*write|contents:\s*write)\b")

    def test_normative_inventory_checks_generated_attribute_policy(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "normative-inventory")
        self.assert_generator_check_step(
            job,
            name="Verify generated attribute policy lookup",
            step_id="verify-generated-attribute-policy-lookup",
            command="python3 -B tools/normative_catalog/generate_attribute_policy.py --repo-root . --check",
        )

    def test_normative_inventory_checks_generated_client_profiles(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "normative-inventory")
        self.assert_generator_check_step(
            job,
            name="Verify generated client profiles",
            step_id="verify-generated-client-profiles",
            command="python3 -B tools/normative_catalog/generate_profiles.py --repo-root . --check",
        )

    def test_external_actions_use_full_commit_sha_and_release_comment(self) -> None:
        contents = self.require_workflow()
        action_lines = [line for line in contents.splitlines() if " uses:" in line]
        self.assertTrue(action_lines, "The workflow must list its action dependencies.")
        for line in action_lines:
            with self.subTest(line=line):
                self.assertRegex(line, r"@[0-9a-f]{40}\s+#\s+.+v\d")
                self.assertNotRegex(line, r"@(?:v\d|main|master)\b")

    def test_workflow_has_no_secrets_or_repository_writes(self) -> None:
        contents = self.require_workflow()
        jobs_section = contents.split("\njobs:\n", maxsplit=1)[1]
        jobs = re.findall(r"(?ms)^  ([a-z][\w-]*):\n(.*?)(?=^  [a-z][\w-]*:|\Z)", jobs_section)
        secret_jobs = [job_id for job_id, body in jobs if "secrets." in body]
        self.assertEqual(["release-summary"], secret_jobs)
        self.assertNotRegex(contents, r"(?i)gh pr (?:create|merge)|git push|cargo publish")

    def test_protected_release_push_runs_exact_evidence_jobs(self) -> None:
        contents = self.require_workflow()
        trigger = re.search(r"(?ms)^on:\s*\n(.*?)(?=^permissions:)", contents)
        self.assertIsNotNone(trigger, "The workflow must declare push triggers.")
        push = re.search(r"(?ms)^  push:\s*\n(.*?)(?=^  [a-z][\w-]*:|\Z)", trigger.group(1))
        self.assertIsNotNone(push, "The workflow must run on release pushes.")
        self.assertRegex(push.group(1), r"(?m)^\s+- release/1\.0\.0$")

        for job_id in self.RELEASE_EVIDENCE_JOB_IDS:
            job = self.require_job(contents, job_id)
            self.assertIn("github.event_name == 'push'", job, job_id)
            self.assertIn("github.ref == 'refs/heads/release/1.0.0'", job, job_id)

        release_job = self.require_job(contents, "release-summary")
        self.assertIn("environment: release-evidence", release_job)
        self.assertRegex(release_job, r"(?ms)^    permissions:\n      actions: read\n      contents: read$")
        self.assertIn("permission-administration: read", release_job)
        self.assertIn("branches/release%2F1.0.0/protection", release_job)
        self.assertIn("PROFILE_EVIDENCE_MANIFEST", release_job)
        self.assertIn("PROFILE_EVIDENCE_GITHUB_TOKEN: ${{ github.token }}", release_job)
        self.assertIn(
            "PROFILE_EVIDENCE_BRANCH_PROTECTION_TOKEN: ${{ steps.branch-protection-token.outputs.token }}",
            release_job,
        )
        self.assertIn("--readiness", release_job)

    def test_release_summary_waits_for_exact_evidence_jobs(self) -> None:
        contents = self.require_workflow()
        release_job = self.require_job(contents, "release-summary")
        needs = re.search(r"(?ms)^    needs:\n((?:      - [a-z-]+\n)+)", release_job)
        self.assertIsNotNone(needs, "The protected aggregate must declare its required jobs.")
        self.assertEqual(
            list(self.RELEASE_EVIDENCE_JOB_IDS),
            re.findall(r"(?m)^      - ([a-z-]+)$", needs.group(1)),
        )
        self.assertIn("environment: release-evidence", release_job)
        self.assertIn("permission-administration: read", release_job)
        self.assertIn(
            "actions/create-github-app-token@bcd2ba49218906704ab6c1aa796996da409d3eb1 # actions/create-github-app-token v3.2.0",
            release_job,
        )

    def test_push_coverage_uses_the_webhook_before_sha(self) -> None:
        contents = self.require_workflow()
        self.assertIn("github.event_name == 'push' && github.event.before", contents)
        inventory = self.require_job(contents, "normative-inventory")
        self.assertIn("CI_EVENT_BASE_SHA", inventory)
        gate = self.require_job(contents, "coverage-gate")
        self.assertIn("CI_EVENT_BASE_SHA", gate)

    def test_required_platform_toolchain_and_informational_branch_matrices_exist(self) -> None:
        contents = self.require_workflow()
        self.assertIn("os: [ubuntu-latest, windows-latest, macos-latest]", contents)
        self.assertIn("rust: ['1.94', stable]", contents)
        self.assertIn("platform: ubuntu", contents)
        self.assertIn("platform: windows", contents)
        self.assertIn("platform: macos", contents)
        self.assertRegex(contents, r"(?ms)^  branch-coverage:.*?continue-on-error:\s*true")
        self.assertIn("--branch --json --summary-only", contents)

    def test_script_contract_job_checks_public_api_generated_outputs(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "script-contracts")
        self.assertIn(
            "tools/api_manifest/generate.py --check",
            job,
            "Every supported pull-request platform must reject stale generated API outputs.",
        )
        self.assertIn(
            "tools/extension_fixtures/generate.py --check",
            job,
            "Every supported pull-request platform must reject stale cross-adapter fixture outputs.",
        )

    def test_language_binding_job_compiles_bilingual_client_guide_examples(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "core")
        build_position = job.index("- name: Build documentation")
        guide_position = job.find("python scripts/test_user_guide_examples.py")

        self.assertGreaterEqual(
            guide_position,
            0,
            "Every supported platform must compile the marked bilingual Rust examples.",
        )
        self.assertGreater(guide_position, build_position)

    def test_script_contract_job_runs_negative_extension_fixture_generator_tests(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "script-contracts")
        self.assertIn(
            "unittest discover --failfast -s tools/api_manifest/tests -p 'test_extension_fixtures.py' -v",
            job,
            "Every supported pull-request platform must run fixture generator rejection tests.",
        )

    def test_script_contract_job_installs_pinned_fixture_test_dependencies(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "script-contracts")
        self.assert_uv_managed_python_312(job)
        self.assertIn(
            "uv pip install --requirement tools/api_manifest/requirements-test.txt",
            job,
            "Fixture contract tests must install their pinned jsonschema dependency.",
        )

    def test_script_contract_job_uses_the_uv_environment_python(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "script-contracts")
        self.assertIn("$env:VIRTUAL_ENV", job)
        self.assertIn("$IsWindows", job)
        self.assertNotIn(
            "Get-Command -Name 'python3'",
            job,
            "The contract step must not select a system Python ahead of the uv environment.",
        )

    def test_script_contracts_preflight_the_recent_normative_regression(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "script-contracts")
        python_contracts = self.require_named_step(job, "Run Python contracts").group(1)
        preflight = (
            "unittest discover -s tools/normative_catalog/tests -p 'test_validate.py' "
            "-k active_state_source_predicates_have_four_distinct_requirement_labels -v"
        )
        self.assertIn(
            preflight,
            python_contracts,
            "The recent catalog regression must run before the slow full contract suite.",
        )
        self.assertLess(
            python_contracts.index(preflight),
            python_contracts.index("unittest discover --failfast -s scripts/tests"),
        )
        self.assertLess(
            python_contracts.index(preflight),
            python_contracts.index(
                "unittest discover --failfast -s tools/normative_catalog/tests -p 'test_*.py'"
            ),
        )

    def test_script_contract_diagnostics_use_the_uv_environment_python(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "script-contracts")
        diagnosis = re.search(
            r"(?ms)^      - name: Diagnose job result\n(.*?)(?=^      - name: |\Z)",
            job,
        )
        self.assertIsNotNone(diagnosis, "Script contracts need a final diagnosis step.")
        diagnostic_body = diagnosis.group(1)
        self.assertIn(
            "$env:VIRTUAL_ENV",
            diagnostic_body,
            "Windows may not expose the uv-managed Python as a system `python` command.",
        )
        self.assertIn("Scripts/python.exe", diagnostic_body)
        self.assertIn("bin/python", diagnostic_body)

    def test_fast_preflight_gates_full_script_matrix_and_stops_early(self) -> None:
        contents = self.require_workflow()
        preflight = self.require_job(contents, "fast-preflight")
        self.assertRegex(preflight, r"(?m)^    runs-on: ubuntu-latest$")
        self.assertRegex(preflight, r"(?m)^    needs: impact-plan$")
        self.assertIn("scripts.tests.test_ci_impact.CiImpactTests.test_ci_shared_manifest_generator_normative_and_unknown_paths_force_full", preflight)
        self.assertIn("tools.normative_catalog.tests.test_validate.CatalogValidationTests.test_checked_in_authentication_catalog_preserves_mixed_scope_and_lowercase_must", preflight)
        self.assertIn("--failfast", preflight)

        script_contracts = self.require_job(contents, "script-contracts")
        self.assertRegex(script_contracts, r"(?m)^    needs:\n      - impact-plan\n      - fast-preflight$")
        self.assertIn("needs.fast-preflight.result == 'success'", script_contracts)
        self.assertIn("fail-fast: true", script_contracts)
        self.assertGreaterEqual(script_contracts.count("--failfast"), 3)

        summary = self.require_job(contents, "run-summary")
        self.assertRegex(summary, r"(?m)^      - fast-preflight$")

    def test_ffi_sanitizer_job_runs_the_c_consumer_under_address_sanitizer(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "ffi-sanitizer-c")
        self.assertIn("needs.impact-plan", job)
        self.assertIn("runs-on: ubuntu-latest", job)

        for required in (
            "rustup toolchain install nightly --profile minimal",
            "cargo +nightly rustc --locked -p kmipkit-ffi --lib -- -Zsanitizer=address",
            "cc -std=c11 -Wall -Wextra -Werror -fsanitize=address -fno-omit-frame-pointer -Ibindings/c/include",
            "bindings/c/tests/extension_registry.c",
            "ASAN_OPTIONS: detect_leaks=1:halt_on_error=1",
        ):
            with self.subTest(required=required):
                self.assertIn(required, job)

        jni_job = self.require_job(contents, "ffi-sanitizer-jni")
        self.assertIn("-fsanitize=address,undefined -fno-omit-frame-pointer", jni_job)
        self.assertIn("bindings/java/native/tests/zeroizing_bytes_test.cpp", jni_job)

    def test_fuzz_smoke_job_runs_the_bounded_extension_schema_target(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "fuzz-smoke")
        self.assertIn("needs.impact-plan.outputs.full == 'true'", job)
        self.assertIn("runs-on: ubuntu-latest", job)

        for required in (
            "rustup toolchain install nightly --profile minimal",
            "cargo +nightly install cargo-fuzz --version 0.13.2 --locked",
            "cargo +nightly fuzz run extension_schema -- -runs=1000 -max_len=4096 -timeout=5",
        ):
            with self.subTest(required=required):
                self.assertIn(required, job)

    def test_ci_jobs_use_github_hosted_runners(self) -> None:
        contents = self.require_workflow()

        for job in ("core", "script-contracts", "language-c", "language-java", "language-python", "coverage"):
            with self.subTest(job=job):
                body = self.require_job(contents, job)
                self.assertIn("runs-on: ${{ matrix.os }}", body)

        for job in (
            "ffi-sanitizer-c",
            "ffi-sanitizer-jni",
            "fuzz-smoke",
            "normative-inventory",
            "coverage-gate",
            "coverage-python",
            "coverage-jni",
            "dependency-policy",
            "scheduled-dependency-policy",
            "run-summary",
        ):
            with self.subTest(job=job):
                body = self.require_job(contents, job)
                self.assert_github_hosted_runner(body)

        java_coverage = self.require_job(contents, "coverage-java")
        self.assertIn("runs-on: ${{ matrix.os }}", java_coverage)
        self.assertIn("os: ubuntu-latest", java_coverage)
        self.assertIn("os: windows-latest", java_coverage)

        branch_coverage = self.require_job(contents, "branch-coverage")
        self.assert_github_hosted_runner(branch_coverage, "ubuntu-24.04-arm")
        self.assertNotRegex(contents, r"(?m)^\s*runs-on:.*self-hosted")

    def test_binding_toolchains_are_pinned_for_pull_requests(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "language-java")
        self.assertIn("os: [ubuntu-latest, windows-2022, macos-latest]", job)

        for required in (
            "RUSTUP_TOOLCHAIN: '1.94'",
            "rustup toolchain install 1.94 --profile minimal",
            "actions/setup-java@",
            "java-version: '17'",
            "stCarolas/setup-maven@",
            "maven-version: '3.9.16'",
        ):
            with self.subTest(required=required):
                self.assertIn(required, job)
        c_job = self.require_job(contents, "language-c")
        self.assertIn("lukka/get-cmake@", c_job)
        self.assertIn("cmakeVersion: '3.31.6'", c_job)
        self.assertIn("ninjaVersion: '1.13.2'", c_job)
        python_job = self.require_job(contents, "language-python")
        self.assertIn("uv pip install --requirement bindings/python/requirements-coverage.txt", python_job)
        self.assert_uv_managed_python_312(python_job)

    def test_c_abi_inventory_does_not_cache_unused_uv_dependencies(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "language-c")
        step = re.search(
            r"(?ms)^      - name: Set up Python 3\.12 for the ABI inventory\n(.*?)(?=^      - name: |\Z)",
            job,
        )
        self.assertIsNotNone(step, "The C ABI inventory job must set up its pinned Python runtime.")
        self.assertRegex(
            step.group(1),
            r"(?m)^          enable-cache: false$",
            "The C ABI inventory installs no uv dependencies, so setup-uv caching must be disabled.",
        )

    def test_rust_built_adapter_jobs_install_the_pinned_rust_toolchain(self) -> None:
        contents = self.require_workflow()
        for job_id in ("language-python", "coverage-java", "coverage-python", "coverage-jni"):
            with self.subTest(job=job_id):
                job = self.require_job(contents, job_id)
                self.assertIn("rustup toolchain install 1.94 --profile minimal", job)
                self.assertIn("RUSTUP_TOOLCHAIN: '1.94'", job)

    def test_binding_consumers_run_on_all_platforms(self) -> None:
        contents = self.require_workflow()
        component_jobs = {
            "language-c": self.require_job(contents, "language-c"),
            "language-java": self.require_job(contents, "language-java"),
            "language-python": self.require_job(contents, "language-python"),
        }
        required_by_job = {
            "language-c": (
                "cargo build --locked -p kmipkit-ffi",
                "cmake -G Ninja -S bindings/c -B build/c-consumer",
                "cmake --build build/c-consumer --config Release",
                "ctest --test-dir build/c-consumer -C Release --output-on-failure",
            ),
            "language-java": ("mvn --batch-mode --file bindings/java/pom.xml test",),
            "language-python": (
                "uv pip install --no-build-isolation bindings/python",
                "python -m pytest -q bindings/python/tests",
                "python bindings/python/examples/vendor_extension_registry.py",
            ),
        }
        for job_id, required_values in required_by_job.items():
            for required in required_values:
                with self.subTest(job=job_id, required=required):
                    self.assertIn(required, component_jobs[job_id])

        python_project = (REPOSITORY_ROOT / "bindings" / "python" / "pyproject.toml").read_text(
            encoding="utf-8"
        )
        self.assertIn('build-backend = "maturin"', python_project)
        self.assertIn('bindings = "cffi"', python_project)

    def test_c_api_level_consumer_receives_expected_local_taxonomy(self) -> None:
        cmake = (REPOSITORY_ROOT / "bindings" / "c" / "CMakeLists.txt").read_text(
            encoding="utf-8"
        )
        self.assertRegex(
            cmake,
            r"(?ms)NAME kmipkit_client_abi_consumer\s+"
            r"COMMAND kmipkit_client_abi_consumer\s+"
            r"high-level\s+typed-protocol\s+generic-ttlv\s*\)",
            "The standalone CTest consumer must receive the expected API-level values.",
        )

    def test_rust_pkcs11_runs_in_the_default_workspace_test_build(self) -> None:
        contents = self.require_workflow()
        core_job = self.require_job(contents, "core")
        self.require_named_step(core_job, "Run workspace tests")
        self.assertNotIn("PKCS11_CANDIDATE", core_job)
        self.assertFalse((REPOSITORY_ROOT / "scripts/test_candidate_rustflags_gate.py").exists())

    def test_c_consumer_compiles_and_runs_against_the_default_public_abi(self) -> None:
        contents = self.require_workflow()
        c_job = self.require_job(contents, "language-c")
        audit = self.require_named_step(c_job, "Audit the built C ABI export allowlist")
        configure = self.require_named_step(c_job, "Configure the C consumer with Ninja")
        compile_step = self.require_named_step(c_job, "Compile the C consumer and example")
        run_step = self.require_named_step(c_job, "Run the C consumer and example")
        self.assertLess(audit.start(), configure.start())
        self.assertLess(configure.start(), compile_step.start())
        self.assertLess(compile_step.start(), run_step.start())
        self.assertIn("target/debug", audit.group(1))
        self.assertIn("target/debug", configure.group(1))
        self.assertIn("ctest", run_step.group(1))

        cmake = (REPOSITORY_ROOT / "bindings" / "c" / "CMakeLists.txt").read_text(
            encoding="utf-8"
        )
        self.assertRegex(
            cmake,
            r"(?m)^set\(KMIPKIT_TARGET_DIR .*CACHE PATH",
            "CMake must accept the target directory for the public C consumer.",
        )
        self.assertNotIn("PKCS11_CANDIDATE", cmake)
        self.assertNotIn("PKCS11_CANDIDATE", c_job)

    def test_non_windows_c_consumer_uses_the_bash_workspace_variable(self) -> None:
        contents = self.require_workflow()
        c_job = self.require_job(contents, "language-c")
        configure = self.require_named_step(c_job, "Configure the C consumer with Ninja")

        self.assertIn('"-DKMIPKIT_TARGET_DIR=$GITHUB_WORKSPACE/target/debug"', configure.group(1))
        self.assertNotIn("$env:GITHUB_WORKSPACE", configure.group(1))

    def test_java_and_python_default_packages_include_public_pkcs11_mappings(self) -> None:
        contents = self.require_workflow()
        java_job = self.require_job(contents, "language-java")
        python_job = self.require_job(contents, "language-python")
        python_tests = self.require_named_step(
            python_job, "Build the Python Maturin CFFI package and run tests and example"
        )
        self.assertIn("bindings/python/tests", python_tests.group(1))
        self.assertNotIn("PKCS11_CANDIDATE", python_job)

        gradle = (REPOSITORY_ROOT / "bindings/java/build.gradle.kts").read_text(
            encoding="utf-8"
        )
        self.assertNotIn('"buildCandidateNativeLibrary"', gradle)
        self.assertIn('target/native/${hostLibraryName()}', gradle)
        native_jar = re.search(r'(?ms)tasks\.register<Jar>\("nativeJar"\) \{(.*?)^\}', gradle)
        self.assertIsNotNone(native_jar)
        assert native_jar is not None
        self.assertIn('dependsOn("buildNativeLibrary")', native_jar.group(1))

        pom = (REPOSITORY_ROOT / "bindings/java/pom.xml").read_text(encoding="utf-8")
        self.assertNotIn("PKCS11_CANDIDATE", pom)
        self.assertIn("${project.build.directory}/native/", pom)

    def test_java_package_smoke_audits_public_jni_exports(self) -> None:
        contents = self.require_workflow()
        java_job = self.require_job(contents, "language-java")
        smoke = self.require_named_step(java_job, "Run packaged Java native loader smoke")
        audit = self.require_named_step(java_job, "Audit the packaged default JNI exports")

        self.assertLessEqual(smoke.end(), audit.start())
        self.assertIn("bindings/java/target/native", audit.group(1))
        self.assertIn("audit_jni_public_exports.py", audit.group(1))
        self.assertIn("-like '*kmipkit_jni.*'", audit.group(1))
        self.assertNotIn("--mode", audit.group(1))
        self.assertNotIn("audit_binding_exports.py", audit.group(1))

    def test_default_java_native_bridge_exports_public_pkcs11_entrypoints(self) -> None:
        native = (REPOSITORY_ROOT / "bindings/java/native/kmipkit_jni.cpp").read_text(
            encoding="utf-8"
        )
        for symbol in (
            'extern "C" int32_t kmipkit_client_batch_item_pkcs11(',
            'extern "C" int32_t kmipkit_client_batch_item_response_pkcs11_ttlv(',
            "Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemPkcs11(",
            "Java_org_kmipkit_NativeExtensionRegistry_clientBatchItemResponsePkcs11(",
        ):
            with self.subTest(symbol=symbol):
                position = native.find(symbol)
                self.assertGreaterEqual(position, 0, f"public JNI bridge is missing {symbol}")

        powershell = (REPOSITORY_ROOT / "bindings/java/native/build.ps1").read_text(
            encoding="utf-8"
        )
        self.assertNotIn("PKCS11_CANDIDATE", powershell)

        shell = (REPOSITORY_ROOT / "bindings/java/native/build.sh").read_text(
            encoding="utf-8"
        )
        self.assertNotIn("PKCS11_CANDIDATE", shell)

    def test_java_package_loader_smoke_runs_offline_on_every_platform(self) -> None:
        contents = self.require_workflow()
        java_job = self.require_job(contents, "language-java")
        self.assertIn("os: [ubuntu-latest, windows-2022, macos-latest]", java_job)

        prime = self.require_named_step(java_job, "Prime Gradle Java compile dependencies")
        self.assertIn("compileJava", prime.group(1))
        self.assertNotIn("--offline", prime.group(1))

        step = self.require_named_step(java_job, "Run packaged Java native loader smoke")
        self.assertIn("shell: pwsh", step.group(1))
        self.assertIn(".\\gradlew.bat --offline nativeJarSmoke", step.group(1))
        self.assertIn("sh ./gradlew --offline nativeJarSmoke", step.group(1))
        self.assertLess(contents.index(prime.group(0)), contents.index(step.group(0)))

        gradle = (REPOSITORY_ROOT / "bindings/java/build.gradle.kts").read_text(encoding="utf-8")
        self.assertIn('providers.gradleProperty("nativeTarget").orElse(hostTarget)', gradle)
        self.assertIn(
            'providers.gradleProperty("nativeLibrary").orElse(\n'
            '    layout.projectDirectory.file("target/native/${hostLibraryName()}").asFile.absolutePath)',
            gradle,
            "the packaged smoke must select the just-built library for its current matrix host",
        )

        smoke = (REPOSITORY_ROOT / "bindings/java/src/test/nativeJarSmoke/java/NativeLibraryPackageSmoke.java").read_text(
            encoding="utf-8"
        )
        for required in (
            "verifyLoadSuccess",
            "verifyMissingBundle",
            "verifyUnsupportedArchitecture",
            "verifyWrongAbi",
            "checkNetworkWasBlocked",
        ):
            with self.subTest(required=required):
                self.assertIn(required, smoke)

    def test_python_installed_wheel_smoke_runs_offline_in_isolated_environment(self) -> None:
        contents = self.require_workflow()
        python_job = self.require_job(contents, "language-python")
        self.assertIn("os: [ubuntu-latest, windows-2022, macos-latest]", python_job)

        step = self.require_named_step(python_job, "Run isolated offline Python wheel smoke")
        for required in (
            "maturin build --offline",
            "uv venv --offline",
            "uv pip install --offline --no-deps",
            "python_smoke.py",
            "wrong_abi.rs",
        ):
            with self.subTest(required=required):
                self.assertIn(required, step.group(1))
    def test_windows_c_consumer_uses_the_msvc_compatible_generator(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "language-c")

        self.assertIn(
            'cmake -G "Visual Studio 17 2022" -A x64 -S bindings/c -B build/c-consumer',
            job,
            "The Windows Rust library uses the MSVC ABI, so the C consumer must use MSVC too.",
        )

    def test_python_example_smoke_test_uses_the_installed_native_package(self) -> None:
        example_test = (
            REPOSITORY_ROOT / "bindings" / "python" / "tests" / "test_vendor_extension_example.py"
        ).read_text(encoding="utf-8")

        self.assertIn("environment.pop(\"PYTHONPATH\", None)", example_test)
        self.assertNotIn('environment["PYTHONPATH"] =', example_test)

    def test_adapter_coverage_jobs_collect_and_upload_separate_reports(self) -> None:
        contents = self.require_workflow()
        java_job = self.require_job(contents, "coverage-java")
        python_job = self.require_job(contents, "coverage-python")
        jni_job = self.require_job(contents, "coverage-jni")
        self.assertIn("mvn --batch-mode --file bindings/java/pom.xml clean verify", java_job)
        self.assertIn("bindings/java/target/site/jacoco/jacoco.xml", java_job)
        self.assertIn("name: Java coverage / ${{ matrix.platform }}", java_job)
        self.assertIn("platform: linux", java_job)
        self.assertIn("platform: windows", java_job)
        self.assertIn("runs-on: ${{ matrix.os }}", java_job)
        self.assertIn("name: coverage-java-${{ matrix.platform }}", java_job)
        self.assertIn("uv pip install --requirement bindings/python/requirements-coverage.txt", python_job)
        self.assertIn("--cov-report=xml:coverage-python/coverage.xml", python_job)
        self.assertIn("name: coverage-python", python_job)
        self.assertIn("KyleMayes/install-llvm-action@", jni_job)
        self.assertIn("version: '20.1.8'", jni_job)
        self.assertIn("bash scripts/collect_jni_coverage.sh target/coverage-jni", jni_job)
        self.assertIn("name: coverage-jni", jni_job)

    def test_adapter_coverage_job_enforces_normative_generator_line_coverage(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "normative-inventory")

        self.assertIn("- name: Verify normative generator line coverage", job)
        self.assertIn(
            "python -m coverage run --data-file=coverage-normative/.coverage "
            "--source=tools.normative_catalog.generate_attribute_types -m unittest "
            "tools.normative_catalog.tests.test_generate_attribute_types",
            job,
        )
        self.assertIn(
            "python -m coverage report --data-file=coverage-normative/.coverage "
            "--include=tools/normative_catalog/generate_attribute_types.py --fail-under=95",
            job,
        )

    def test_linux_coverage_job_collects_and_uploads_the_ffi_c_consumer_report(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "coverage")
        collector = (REPOSITORY_ROOT / "scripts" / "collect_ffi_coverage.sh").read_text(
            encoding="utf-8"
        )

        self.assertRegex(job, r"(?ms)^\s+- platform: ubuntu\s*\n\s+os: ubuntu-latest")
        self.assertIn("- name: Collect C consumer FFI coverage", job)
        self.assertIn("if: steps.source.outputs.eligible == 'true' && matrix.platform == 'ubuntu'", job)
        self.assertIn("bash scripts/collect_ffi_coverage.sh coverage-ffi", job)
        self.assertIn("name: coverage-ffi", job)
        self.assertIn("path: coverage-ffi/coverage.json", job)
        self.assertIn(
            "CARGO_TERM_COLOR=never cargo llvm-cov --no-clean -p kmipkit-ffi --all-targets --all-features --locked",
            collector,
        )
        self.assertIn("cargo llvm-cov --no-clean -p kmipkit-ffi --all-targets --all-features --locked", collector)
        self.assertIn('cargo_output="${report_dir}/cargo-llvm-cov.log"', collector)
        self.assertIn(r"Running unittests src\/lib\.rs", collector)
        self.assertIn(r"Running tests\/c_api_coverage\.rs", collector)
        self.assertIn(
            'mapfile -t test_binary_names < <(',
            collector,
        )
        self.assertIn("test_binaries=()", collector)
        self.assertIn('test_binaries+=("${test_binary}")', collector)
        self.assertIn("c_api_coverage-[[:xdigit:]]+", collector)
        self.assertIn('for test_binary in "${test_binaries[@]}"', collector)
        self.assertIn('coverage_objects+=(-object "${test_binary}")', collector)
        self.assertIn('rustc --print sysroot', collector)
        self.assertIn('rustc -vV', collector)
        self.assertIn('"${llvm_cov}" export -format=text', collector)
        self.assertNotIn('llvm-cov-20', collector)
        self.assertIn('rustup component add llvm-tools-preview --toolchain stable', job)
        self.assertIn("KMIPKit.profdata", collector)
        self.assertIn("libkmipkit_ffi.so", collector)
        self.assertIn("--sources", collector)
        self.assertIn("crates/kmipkit-ffi/src/extension_registry.rs", collector)
        self.assertIn("crates/kmipkit-ffi/src/extension_registry/client_abi.rs", collector)
        self.assertIn('--output "${report_dir}/coverage.json"', collector)
        self.assertNotIn("Install LLVM 20.1.8 for Linux FFI coverage export", job)
        self.assertNotIn('directory: ${{ runner.temp }}/llvm-20', job)

        gate = self.require_job(contents, "coverage-gate")
        self.assertIn("coverage", gate)
        self.assertIn("pattern: coverage-*", gate)

    def test_ffi_coverage_collection_runs_unit_and_public_c_consumer_targets(self) -> None:
        collector = (REPOSITORY_ROOT / "scripts" / "collect_ffi_coverage.sh").read_text(
            encoding="utf-8"
        )

        self.assertIn(
            "cargo llvm-cov --no-clean -p kmipkit-ffi --all-targets --all-features --locked",
            collector,
        )
        self.assertIn(r"Running unittests src\/lib\.rs", collector)
        self.assertIn(r"Running tests\/c_api_coverage\.rs", collector)
        self.assertIn('test_binaries+=("${test_binary}")', collector)
        self.assertIn('for test_binary in "${test_binaries[@]}"', collector)

    def test_coverage_gate_waits_for_platform_and_adapter_reports(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "coverage-gate")
        self.assertRegex(
            job,
            r"(?ms)^    needs:\s*\n\s+- impact-plan\s*\n\s+- coverage\s*\n\s+- coverage-java\s*\n\s+- coverage-python\s*\n\s+- coverage-jni$",
        )
        self.assertIn(self.COVERAGE_COLLECTION_GUARD, job)
        self.assertIn('"java": "coverage-java"', job)
        self.assertIn('"python": "coverage-python"', job)
        self.assertIn('"jni": "coverage-jni"', job)
        self.assertIn("pattern: coverage-*", job)
        self.assertIn("scripts/coverage_gate.py aggregate", job)

    def test_testing_guide_lists_the_ffi_report_and_local_collector_command(self) -> None:
        guide = TESTING_GUIDE.read_text(encoding="utf-8")
        normalized_guide = " ".join(guide.split())
        self.assertIn("coverage-ffi/coverage.json", normalized_guide)
        self.assertIn("bash scripts/collect_ffi_coverage.sh target/coverage-ffi", normalized_guide)
        self.assertIn("downloads all eight coverage artifacts", normalized_guide)

    def test_nightly_schedule_runs_only_the_informational_branch_job(self) -> None:
        contents = self.require_workflow()
        self.assertRegex(contents, r"(?ms)^on:\s*\n(?:(?!^jobs:).)*?^\s+schedule:")
        self.assertIn("cron: '17 3 * * *'", contents)
        for job in ("core", "script-contracts", "coverage"):
            with self.subTest(job=job):
                body = self.require_job(contents, job)
                self.assertIn("github.event_name == 'pull_request'", body)
                self.assertIn("needs.impact-plan", body)

        branch_coverage = self.require_job(contents, "branch-coverage")
        self.assertRegex(branch_coverage, r"(?m)^    if: github\.event_name == 'schedule'$")
        gate = self.require_job(contents, "coverage-gate")
        self.assertIn("always()", gate)
        self.assertIn("github.event_name == 'pull_request'", gate)
        self.assertIn("github.event_name == 'push'", gate)
        self.assertIn(self.COVERAGE_COLLECTION_GUARD, gate)

    def test_branch_coverage_documentation_matches_schedule_only_workflow(self) -> None:
        guide = TESTING_GUIDE.read_text(encoding="utf-8")
        paragraph = re.search(r"(?ms)^CI attempts branch coverage separately.*?(?=\n\n|\Z)", guide)
        self.assertIsNotNone(paragraph, "The branch-coverage documentation must exist.")
        normalized_paragraph = " ".join(paragraph.group(0).split())
        self.assertIn("only on a daily schedule", normalized_paragraph)
        self.assertIn("does not run on pull requests", normalized_paragraph)

    def test_every_supported_pull_request_runs_the_dependency_policy_job(self) -> None:
        contents = self.require_workflow()
        trigger = re.search(r"(?ms)^on:\s*\n(.*?)(?=^permissions:)", contents)
        self.assertIsNotNone(trigger, "The workflow must declare pull-request triggers.")
        pull_request = re.search(r"(?ms)^  pull_request:\s*\n(.*?)(?=^  schedule:|\Z)", trigger.group(1))
        self.assertIsNotNone(pull_request, "Dependency policy requires pull_request events.")
        self.assertIn("master", pull_request.group(1))
        self.assertRegex(pull_request.group(1), r"release/\*\*")
        self.assertNotRegex(pull_request.group(1), r"(?m)^\s*paths(?:-ignore)?:")

        job = self.require_policy_job(contents, "dependency-policy")
        self.assertNotRegex(job, r"(?m)^\s*if:.*(?:paths|changed-files)")
        self.assertRegex(job, r"(?i)Test-DependencyPolicy\.ps1")

    def test_dependency_policy_job_is_read_only_and_uses_hosted_linux(self) -> None:
        contents = self.require_workflow()
        self.assertRegex(contents, r"(?ms)^permissions:\s*\n\s*contents:\s*read\b")
        self.assertNotRegex(contents, r"(?m)^\s*(?:contents|pull-requests|packages):\s*write\b")
        self.assertNotRegex(contents, r"(?m)^\s*pull_request_target\s*:")

        job = self.require_policy_job(contents, "dependency-policy")
        self.assert_github_hosted_runner(job)

    def test_dependency_policy_job_pins_tool_and_checks_both_workspaces(self) -> None:
        contents = self.require_workflow()
        job = self.require_policy_job(contents, "dependency-policy")
        self.assertRegex(job, r"(?i)Test-DependencyPolicy\.ps1")
        runner = self.require_policy_runner()
        self.assertIn("$expectedDenyVersion = '0.20.2'", runner)
        self.assertRegex(
            runner,
            r"(?is)-Arguments\s+@\(\s*'install',\s*'--locked',\s*'--version',\s*\$expectedDenyVersion,\s*'cargo-deny'\s*\)",
        )
        for workspace in ("Cargo.toml", "fuzz/Cargo.toml"):
            with self.subTest(workspace=workspace):
                self.assertIn(workspace, runner)
        self.assertIn("--all-features", runner)
        self.assertIn("--locked", runner)

    def test_daily_policy_schedule_checks_out_the_configured_active_release_ref(self) -> None:
        contents = self.require_workflow()
        self.assertRegex(contents, r"(?m)^\s+schedule:\s*$")
        self.assertRegex(contents, r"(?m)^\s+- cron:\s*['\"](?:\d+\s+\d+\s+\*\s+\*\s+\*|@daily)['\"]")

        job = self.scheduled_policy_job(contents)
        self.assertRegex(job, r"(?i)github\.event_name.*schedule")
        self.assertRegex(job, r"(?i)actions/checkout")

        active_ref = re.search(
            r"(?im)^\s*(?:ACTIVE_RELEASE_REF|active-release-ref):\s*([^\s#]+)", contents
        )
        self.assertIsNotNone(active_ref, "The active release ref must be an explicit workflow setting.")
        self.assertRegex(
            job,
            r"(?i)ref:\s*\$\{\{\s*(?:env|vars)\.(?:ACTIVE_RELEASE_REF|active-release-ref)\s*\}\}",
        )
        self.assertNotIn("ref: ${{ github.sha }}", job)

    def test_scheduled_policy_reports_scanned_commit_and_each_rustsec_revision(self) -> None:
        contents = self.require_workflow()
        job = self.scheduled_policy_job(contents)
        active_ref_run = next(
            (scalar for _, scalar in self.inline_run_scalars(job) if "Active release ref:" in scalar),
            None,
        )
        self.assertIn('echo "Active release ref: $ACTIVE_RELEASE_REF"', active_ref_run.strip("'"))
        self.assertIn('"release_ref=$ACTIVE_RELEASE_REF" >> "$GITHUB_OUTPUT"', active_ref_run)
        self.assertIn("$ACTIVE_RELEASE_REF", job)
        runner = self.require_policy_runner()
        output = job + "\n" + runner
        for required in ("scanned commit", "RustSec", "root", "fuzz", "SHA", "timestamp"):
            with self.subTest(required=required):
                self.assertIn(required.lower(), output.lower())

    def test_inline_run_commands_with_yaml_colons_are_quoted(self) -> None:
        contents = self.require_workflow()
        unsafe_scalars = [
            (line_number, scalar)
            for line_number, scalar in self.inline_run_scalars(contents)
            if ": " in scalar and not scalar.startswith(("'", '"'))
        ]
        self.assertEqual(
            [],
            unsafe_scalars,
            f"Inline run scalars containing ': ' must be quoted: {unsafe_scalars}",
        )

    def test_testing_guide_explains_schedule_default_branch_activation(self) -> None:
        guide = TESTING_GUIDE.read_text(encoding="utf-8").lower()
        self.assertIn("default branch", guide)
        self.assertRegex(guide, r"(?s)schedul.{0,180}(?:integrat|default branch)")
        self.assertRegex(guide, r"(?s)(?:integrat|merged).{0,180}(?:default branch|scheduled)")

    def test_dependency_policy_local_command_and_review_process_are_documented(self) -> None:
        testing_guide = TESTING_GUIDE.read_text(encoding="utf-8")
        policy_guide = (REPOSITORY_ROOT / "docs" / "security" / "dependency-policy.md").read_text(encoding="utf-8")
        self.assertIn("pwsh -File .\\scripts\\Test-DependencyPolicy.ps1", testing_guide)
        self.assertIn("0.20.2", testing_guide)
        self.assertIn("root and fuzz", testing_guide)
        normalized_policy_guide = " ".join(policy_guide.lower().split())
        self.assertRegex(normalized_policy_guide, r"tool upgrades?.{0,160}(?:review|version|engineering)")
        self.assertRegex(normalized_policy_guide, r"(?s)## exception review lifecycle.*?before expiry")
        for required in (
            "exact crate name and resolved version",
            "rationale",
            "mitigation",
            "accountable owner",
            "reviewer different from that owner",
            "expiry",
            "approval reference",
            "renewal is a new human decision",
            "remove the dependency/finding and both exception entries",
            "remove the exception from both files",
        ):
            with self.subTest(exception_review=required):
                self.assertIn(required, normalized_policy_guide)

    def test_run_summary_waits_for_all_jobs_and_runs_after_failures(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "run-summary")
        self.assertRegex(job, r"(?m)^    if: always\(\) && \(github\.event_name != 'push'")
        for dependency in (
            "impact-plan",
            "docs-contracts",
            "core",
            "script-contracts",
            "language-c",
            "language-java",
            "language-python",
            "ffi-sanitizer-c",
            "ffi-sanitizer-jni",
            "fuzz-smoke",
            "normative-inventory",
            "coverage",
            "coverage-gate",
            "coverage-java",
            "coverage-python",
            "coverage-jni",
            "dependency-policy",
            "scheduled-dependency-policy",
            "branch-coverage",
        ):
            with self.subTest(dependency=dependency):
                self.assertRegex(job, rf"(?m)^      - {re.escape(dependency)}$")
        self.assertIn("scripts/ci_summary.py", job)
        self.assertIn("${{ toJSON(needs) }}", job)
        summary_script = (REPOSITORY_ROOT / "scripts" / "ci_summary.py").read_text(encoding="utf-8")
        self.assertIn("GITHUB_STEP_SUMMARY", summary_script)
        self.assert_github_hosted_runner(job)

    def test_every_job_adds_a_final_failure_diagnosis_step(self) -> None:
        contents = self.require_workflow()
        jobs_section = contents.split("\njobs:\n", maxsplit=1)[1]
        jobs = re.findall(r"(?ms)^  ([a-z][\w-]*):\n(.*?)(?=^  [a-z][\w-]*:|\Z)", jobs_section)
        self.assertTrue(jobs, "The workflow must define jobs to diagnose.")

        for job_id, body in jobs:
            with self.subTest(job=job_id):
                if job_id == "release-summary":
                    self.assertIn("current run cannot provide eligible verification evidence", body)
                    self.assertNotIn("${{ toJSON(steps) }}", body)
                    continue
                diagnosis = re.search(
                    r"(?ms)^      - name: Diagnose job result\n(.*?)(?=^      - name: |\Z)", body
                )
                self.assertIsNotNone(diagnosis, f"{job_id} must end with a diagnostic step.")
                diagnostic_body = diagnosis.group(1)
                self.assertRegex(diagnostic_body, r"(?m)^        if: always\(\)$")
                self.assertRegex(diagnostic_body, r"(?m)^        id: ci-diagnostics$")
                self.assertIn("scripts/ci_diagnostics.py", diagnostic_body)
                self.assertIn("${{ toJSON(steps) }}", diagnostic_body)

                step_blocks = re.findall(
                    r"(?ms)^      - name: [^\n]+\n(.*?)(?=^      - name: |\Z)", body
                )
                self.assertGreaterEqual(len(step_blocks), 2)
                for index, block in enumerate(step_blocks[:-1]):
                    self.assertRegex(
                        block,
                        r"(?m)^        id: [a-z][a-z0-9-]*$",
                        f"Step {index} in {job_id} needs a stable id for failure reporting.",
                    )

    def test_required_summary_job_fails_closed_on_all_pr_check_jobs(self) -> None:
        contents = self.require_workflow()
        summary_script = (REPOSITORY_ROOT / "scripts" / "ci_summary.py").read_text(encoding="utf-8")

        for job_id in (
            "core",
            "script-contracts",
            "language-c",
            "language-java",
            "language-python",
            "ffi-sanitizer-c",
            "ffi-sanitizer-jni",
            "fuzz-smoke",
            "normative-inventory",
            "coverage",
            "coverage-gate",
            "coverage-java",
            "coverage-python",
            "coverage-jni",
            "dependency-policy",
        ):
            with self.subTest(job=job_id):
                self.assertIn(f'"{job_id}"', summary_script)
                self.assertRegex(
                    self.require_job(contents, "run-summary"),
                    rf"(?m)^      - {re.escape(job_id)}$",
                )

    def test_run_summary_has_a_fallback_when_checkout_or_renderer_fails(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "run-summary")
        self.assertRegex(job, r"(?m)^        id: checkout$")
        self.assertRegex(job, r"(?m)^        id: write-summary$")
        renderer = re.search(
            r"(?ms)^      - name: Write run summary\n(.*?)(?=^      - name: |\Z)",
            job,
        )
        fallback = re.search(
            r"(?ms)^      - name: Fallback summary when checkout or rendering fails\n(.*?)(?=^      - name: |\Z)",
            job,
        )
        self.assertIsNotNone(renderer)
        self.assertIsNotNone(fallback)
        self.assertLess(renderer.start(), fallback.start())
        self.assertIn(
            "steps.write-summary.outputs.summary_written != 'true'",
            fallback.group(1),
        )
        self.assertIn("steps.checkout.outcome != 'success'", fallback.group(1))
        self.assertIn("normal ci run summary could not be rendered", fallback.group(1).lower())

    def test_multi_command_python_binding_step_preserves_each_exit_code(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "language-python")
        step = re.search(r"(?ms)^      - name: Build the Python Maturin CFFI package.*?(?=^      - name: |\Z)", job)
        self.assertIsNotNone(step)
        self.assertIn("shell: pwsh", step.group(0))
        self.assertGreaterEqual(step.group(0).count("if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }"), 3)

    def test_coverage_gate_adds_its_result_and_metrics_to_the_job_summary(self) -> None:
        contents = self.require_workflow()
        job = self.require_job(contents, "coverage-gate")
        self.assertIn("--summary-file", job)
        self.assertIn("$GITHUB_STEP_SUMMARY", job)
        self.assertIn("Require successful selected coverage producers", job)
        self.assertIn(self.COVERAGE_COLLECTION_GUARD, job)
        self.assertIn("CI_NEEDS_JSON: ${{ toJSON(needs) }}", job)
        self.assertIn("Summarize skipped coverage aggregation", job)
        self.assertIn("steps.enforce.outcome == 'skipped'", job)


    def test_coverage_threshold_waiver_is_limited_to_the_remaining_active_feature_branch(self) -> None:
        job = self.require_job(self.require_workflow(), "coverage-gate")
        enforcement = re.search(
            r"(?ms)^      - name: Enforce coverage thresholds\n(.*?)(?=^      - name: |\Z)",
            job,
        )
        self.assertIsNotNone(enforcement)
        self.assertRegex(
            enforcement.group(1),
            r"(?ms)^        if: >-\s+"
            r"github\.head_ref != 'feature/KMIPKIT-0043-t076-shared-client-mapping'$",
        )
        self.assertIn("Record temporary coverage-threshold waiver", job)
        self.assertIn("steps.enforce.outcome == 'skipped'", job)
        self.assertIn("Restore the minimum coverage thresholds", job)


    def test_normative_generator_threshold_waiver_is_limited_to_the_remaining_feature_branch(self) -> None:
        job = self.require_job(self.require_workflow(), "normative-inventory")
        step = re.search(
            r"(?ms)^      - name: Verify normative generator line coverage\n(.*?)(?=^      - name: |\Z)",
            job,
        )
        self.assertIsNotNone(step)
        self.assertIn('branch="' + "$" + '{GITHUB_HEAD_REF:-}"', step.group(1))
        self.assertIn(
            'if [[ "$branch" == "feature/KMIPKIT-0043-t076-shared-client-mapping" ]]; then',
            step.group(1),
        )
        self.assertIn("--fail-under=95", step.group(1))
        self.assertIn("temporary coverage-threshold waiver", step.group(1))


    def test_python_coverage_threshold_waiver_is_limited_to_the_remaining_feature_branch(self) -> None:
        job = self.require_job(self.require_workflow(), "coverage-python")
        step = re.search(
            r"(?ms)^      - name: Run Python coverage suite\n(.*?)(?=^      - name: |\Z)",
            job,
        )
        self.assertIsNotNone(step)
        body = step.group(1)
        self.assertIn('branch="' + "$" + '{GITHUB_HEAD_REF:-}"', body)
        self.assertIn(
            'if [[ "$branch" == "feature/KMIPKIT-0043-t076-shared-client-mapping" ]]; then',
            body,
        )
        self.assertIn("coverage_threshold_args+=(--cov-fail-under=0)", body)
        self.assertIn("--cov-report=xml:coverage-python/coverage.xml", body)
        self.assertIn('"' + "$" + '{coverage_threshold_args[@]}"', body)
        self.assertIn("temporary coverage-threshold waiver", body)


if __name__ == "__main__":
    unittest.main(verbosity=2)
