# Implementation evidence: dependency policy gates

## Red evidence

The test-only Red changes were committed in `2285fc80c4d1e2f401ec627ca229ed10439544f2` before policy implementation. After correcting the fixtures to resolve workspace-inherited versions and normalize Cargo source URLs, the focused suites produced behavioral failures with no test errors:

| Command | Result before Green | Interpretation |
|---|---:|---|
| `python -X utf8 -m unittest scripts.tests.test_dependency_policy` | 33 tests; 42 assertion failures; 0 errors; 0 skips | Expected failures for the missing validator, runner, manifests, and policy behavior. |
| `python -X utf8 -m unittest scripts.tests.test_workflow` | 14 tests; 6 assertion failures; 0 errors | Expected failures for the missing pull-request policy job, scheduled policy integration, and documentation contract. |

The FR-013 candidate-scan fixture isolated the fuzz metadata findings: the fuzz package had no recognized license and its local `kmipkit-ttlv` path dependency had no version requirement. The source-policy fixture also confirmed that Cargo metadata's `git+` source notation must be normalized for the cargo-deny configuration while the exception register retains the exact metadata source.

The Red tests were committed separately from policy implementation. This evidence file was added after the first Green commit; the test commit and Green commit boundaries remain distinct and their order is visible in Git history.

## Green evidence: T006

Commit `8c2db15b8b3e1f427cafd796754978e55e084eed` changed only `.cargo/deny.toml` and `fuzz/Cargo.toml`.

Focused checks run on the Green commit:

```text
python -X utf8 -m unittest scripts.tests.test_dependency_policy.DependencyPolicyApiTests.test_fuzz_package_declares_apache_license_and_matching_local_crate_version
Ran 1 test ... OK

python -X utf8 -m unittest scripts.tests.test_dependency_policy.DependencyPolicyApiTests.test_fuzz_candidate_scan_no_longer_reports_metadata_policy_findings
Ran 1 test ... OK

cargo deny --manifest-path Cargo.toml --workspace --all-features --locked check
advisories ok, bans ok, licenses ok, sources ok

cargo deny --manifest-path fuzz/Cargo.toml --workspace --all-features --locked check
advisories ok, bans ok, licenses ok, sources ok
```

Both online cargo-deny invocations exited successfully. The reviewed allowlist contains five exact license IDs; cargo-deny emitted only its expected `license-not-encountered` warnings for IDs not used in each individual graph. No finding was suppressed or waived.

The focused fuzz candidate scan changed from the Red findings above to passing. Independently recomputed SHA-256 values were unchanged for `Cargo.lock` (`08cbbb0bbfb0db6e567eec2db83d2b63788cc6ffada8531be0d50033a8ff0231`) and `fuzz/Cargo.lock` (`ea34d89d36fa78841f0b1c63064726f09a353f62e68725cc7cdc99d32c6c7778`). SHA-256 values for the sorted resolved `name@version` entries were also unchanged: root `ecd3e1d80d5a53059247b7ab58b73ec6c890c325c0313518417989e8d4a83041` and fuzz `b34e0f7e8407b8154d4ee1e5acceb04dc147c539ef28040a44d26714772cfc4`.

At this point the complete policy suite still reports 42 expected assertion failures and the workflow suite still reports 6 expected assertion failures; these cover tasks not implemented by T006. Both suites have zero test errors. They are not claimed as passing.

An independent review of T006 found no blocking issue. The reviewer noted that Cargo interprets `version = "0.1.0"` as a caret requirement rather than an exact pin; it resolves to the existing local package under both tested toolchains and matches the task's requirement to specify the local crate version. The unused-license warnings above are expected for a shared allowlist covering the union of the root and fuzz graphs.

## Green evidence: T007

T007 adds the version-1 exception register with no default waivers and a validator for exact rule/package/version findings, required rationale and mitigation, reviewer separation, durable approval evidence, review/expiry dates, human-reviewed cargo-deny license clarifications, exact immutable Git revisions, and bidirectional correspondence with waiver fields. Source-less metadata packages must resolve to canonical member manifests within the root/fuzz workspace union; lexical and canonical path checks reject checkout escapes and symlink escapes. Diagnostics omit untrusted URL parser input and normalize invalid checkout-root failures.

Fresh Windows checks after Green:

```text
python -m py_compile scripts/dependency_policy.py
passed

python -m unittest -v scripts.tests.test_dependency_policy.DependencyExceptionTests scripts.tests.test_dependency_policy.DependencyPolicyApiTests
Ran 30 tests ... OK (skipped=1)
```

The one skip is the directory-symlink escape test: this Windows account lacks the privilege required to create directory symlinks (`WinError 1314`). The path validation itself also has lexical escape tests and canonical resolution checks. A prior fresh run exposed that Python's `cp1252` default could not decode cargo-deny UTF-8 output in the candidate-scan test; the test now requests UTF-8 and replaces invalid bytes. The same test passes without setting environment overrides.

Both locked all-feature Cargo metadata commands completed for the root and fuzz manifests without platform filters. Running the validator against those generated metadata files printed `dependency policy metadata and exception register are valid` and exited zero. `git diff --check` passed. The independent T007 review found no blocking issue and confirmed the checkout-root diagnostic regression is closed. It recorded a non-blocking observation that the validator synchronizes waiver fields but does not independently pin every baseline setting; T006's reviewed cargo-deny configuration remains responsible for those baseline values, outside T007/T011's waiver-synchronization contract.

## Green evidence: T008

T008 added the PowerShell dependency-policy runner and was exercised with the committed runner-contract tests first. The initial Red run was:

```text
python -m unittest scripts.tests.test_dependency_policy.DependencyPolicyRunnerContractTests
Ran 8 tests ... FAILED (failures=8)
```

All eight failures asserted that `scripts/Test-DependencyPolicy.ps1` was missing; there were no test errors. After implementation, the focused runner, API, and exception suites passed:

```text
python -m unittest -v scripts.tests.test_dependency_policy.DependencyPolicyRunnerContractTests scripts.tests.test_dependency_policy.DependencyPolicyApiTests scripts.tests.test_dependency_policy.DependencyExceptionTests
Ran 38 tests ... OK (skipped=1)
```

The one skip is the directory-symlink escape test: this Windows account lacks the required symlink privilege (`WinError 1314`). PowerShell parser validation passed. The full Windows runner passed on `x86_64-pc-windows-msvc` with cargo-deny 0.20.2, unfiltered root/fuzz metadata validation, and separate online root and fuzz workspace checks:

```text
pwsh -NoLogo -NoProfile -File scripts/Test-DependencyPolicy.ps1
```

Each successful workspace scan verified the RustSec remote and emitted its database commit and ISO timestamp:

| Workspace | RustSec remote | Commit | Commit timestamp |
|---|---|---|---|
| root | `https://github.com/RustSec/advisory-db` | `ef6173cbc5c50ec8166f9a5b28f07834144373ee` | `2026-10-03T10:14:03+02:00` |
| fuzz | `https://github.com/RustSec/advisory-db` | `ef6173cbc5c50ec8166f9a5b28f07834144373ee` | `2026-10-03T10:14:03+02:00` |

The runner confirmed both lockfiles remained unchanged:

| Lockfile | SHA-256 |
|---|---|
| `Cargo.lock` | `08cbbb0bbfb0db6e567eec2db83d2b63788cc6ffada8531be0d50033a8ff0231` |
| `fuzz/Cargo.lock` | `ea34d89d36fa78841f0b1c63064726f09a353f62e68725cc7cdc99d32c6c7778` |

## Red evidence: T011 diagnostic identifiers

Added `DependencyExceptionTests.test_exception_config_mismatch_reports_rule_and_exception_id_without_secrets` before changing validator behavior. It covers missing advisory, license clarification, Git source, and duplicate waiver surfaces, asserting the rule name, stable exception ID, and absence of a secret-bearing URL value from diagnostics.

```text
python -m unittest -v scripts.tests.test_dependency_policy.DependencyExceptionTests.test_exception_config_mismatch_reports_rule_and_exception_id_without_secrets
Ran 1 test ... FAILED (failures=4)
```

All four subtests failed behaviorally because the corresponding diagnostics omitted the exception ID; the test had zero errors. The source mismatch diagnostic did not echo the configured credential sentinel.

## Green evidence: T011 diagnostic identifiers

`validate_exception_config` now includes relevant registered exception IDs in advisory, license clarification, Git source, and duplicate waiver mismatch diagnostics. A config-only waiver with no matching register record receives a rule-specific diagnostic stating that no registered exception ID exists. Diagnostics derive identifiers from validated register entries and do not include configured values, URLs, credentials, or secrets. Bidirectional matching and exact waiver scope are unchanged.

```text
python -m unittest -v scripts.tests.test_dependency_policy.DependencyExceptionTests.test_exception_config_mismatch_reports_rule_and_exception_id_without_secrets
Ran 1 test ... OK

python -m unittest -v scripts.tests.test_dependency_policy.DependencyExceptionTests
Ran 20 tests ... OK

python -m unittest -v scripts.tests.test_dependency_policy.DependencyPolicyApiTests
Ran 11 tests ... OK (skipped=1)

python -m py_compile scripts/dependency_policy.py
passed

git diff --check
passed
```

The API-suite skip is the Windows directory-symlink escape test, unavailable because this account lacks the required symlink privilege (`WinError 1314`). A supplemental in-memory check exercised config-only advisory, license clarification, Git source, and duplicate waivers with an empty register; all four diagnostics named the relevant rule, stated that no registered exception ID exists, and did not expose the secret-bearing URL sentinel.

## Red evidence: T011 duplicate configured waivers

Added `test_duplicate_configured_waivers_report_registered_exception_ids_without_values` for duplicated advisory, license clarification, Git source, and duplicate waiver entries, plus `test_unregistered_secret_bearing_git_waiver_names_rule_without_echoing_value` for reverse-direction safe diagnostics. The duplicate waiver case and config-only source case already behaved correctly. The three remaining duplicate-config subcases exposed missing register IDs: the advisory branch fell back to claiming no ID existed, while duplicate license and Git entries failed in early branches without ID-aware diagnostics.

```text
python -m unittest -v scripts.tests.test_dependency_policy.DependencyExceptionTests.test_duplicate_configured_waivers_report_registered_exception_ids_without_values scripts.tests.test_dependency_policy.DependencyExceptionTests.test_unregistered_secret_bearing_git_waiver_names_rule_without_echoing_value
Ran 2 tests ... FAILED (failures=3)
```

The failure was behavioral with zero test errors. The config-only secret-bearing Git source check passed and confirmed the configured URL was not echoed.

## Green evidence: T011 duplicate configured waivers

Duplicate advisory values now resolve to matching register entries before diagnostics are assembled. Duplicate license clarifications and Git source entries include their matching registered IDs; unregistered duplicated values retain rule-specific no-ID wording. Duplicate waiver diagnostics retain their previously correct register ID. No configured values are included in these diagnostics.

```text
python -m unittest -v scripts.tests.test_dependency_policy.DependencyExceptionTests.test_exception_config_mismatch_reports_rule_and_exception_id_without_secrets scripts.tests.test_dependency_policy.DependencyExceptionTests.test_duplicate_configured_waivers_report_registered_exception_ids_without_values scripts.tests.test_dependency_policy.DependencyExceptionTests.test_unregistered_secret_bearing_git_waiver_names_rule_without_echoing_value
Ran 3 tests ... OK

python -m unittest -v scripts.tests.test_dependency_policy.DependencyExceptionTests
Ran 22 tests ... OK

python -m unittest -v scripts.tests.test_dependency_policy.DependencyPolicyApiTests
Ran 11 tests ... OK (skipped=1)

python -m py_compile scripts/dependency_policy.py
passed

git diff --check
passed
```

The single API-suite skip remains the Windows directory-symlink escape test, unavailable because this account lacks the required privilege (`WinError 1314`).


## Green evidence: T009 pull-request and scheduled policy jobs

The workflow Red contract from T004/T013 initially had six behavioral assertion failures and no test errors. The Green workflow adds a read-only policy job to every supported pull request, routes same-repository Linux PRs to the established Linux ARM64 runner and fork PRs to GitHub-hosted Linux, and adds a separate daily schedule job. The scheduled job checks out the explicit `ACTIVE_RELEASE_REF` (`release/1.0.0`), prints the scanned commit, then reports root and fuzz RustSec revision evidence through the shared runner. Both jobs use the exact pinned checkout action and no secrets or write permissions. Separate PR and schedule jobs ensure the scheduled job checks out only its configured release ref while PR checks still evaluate their merge commit.

The existing workflow contract had one false-negative assertion: it expected a literal shell command for cargo-deny installation, while the runner safely passes an argument vector through `ProcessStartInfo`. The assertion now verifies the exact `0.20.2` version variable and ordered install arguments (`install --locked --version <version> cargo-deny`) without requiring shell-concatenated execution.

```text
python -X utf8 -m unittest scripts.tests.test_workflow -v
Ran 14 tests ... OK
```

## Red evidence: T015 normative traceability

Added `scripts/tests/test_requirement_traceability.py` before the CSV. It extracts every FR/SC identifier from the approved specification and requires one CSV row per identifier, existing safe repository-relative configuration and implementation paths, and a reference to a real `unittest.TestCase.test_*` method.

```text
python scripts/tests/test_requirement_traceability.py -v
Ran 1 test ... FAILED (failures=1)
```

The expected behavioral failure was that `specification/compliance/requirements/KMIPKIT-0011.csv` did not exist; there were no test errors. Test-only Red commit: `54a298aa5d028ecf1c62eaeb76fae96fff3e1ef8`.

## Green evidence: T015 normative traceability

Added `specification/compliance/requirements/KMIPKIT-0011.csv` with exactly 21 rows covering FR-001 through FR-013 and SC-001 through SC-008. Each row identifies a TOML/JSON/YAML configuration location, a policy script or workflow location, and an executable Python test method. The workflow schedule is wired to the active release as part of the same Green change.

```text
python -X utf8 -m unittest scripts.tests.test_requirement_traceability -v
Ran 1 test ... OK

git diff --check
passed
```


## Refactor evidence: T016

Refactor commit `f75a901` consolidates the two duplicated JSON findings-file read/type-check branches in `main` into one path. It preserves the existing order and messages for malformed finding data, non-source exceptions without `--findings`, and findings supplied without registered exceptions; policy matching is unchanged. Red and Green feature commits remain distinct: baseline contract Red `2285fc8`, T009/T015 Green `91ee32b`, traceability Red `54a298a`, and T011 Red/Green pairs `af431b3`/`81c0675` and `609933b`/`9d5210e`.

The complete Python contract suite passed before the Refactor commit (98 tests, four skips) and after it:

```text
python -X utf8 -m unittest discover -s scripts/tests -p 'test_*.py' -v
Ran 98 tests ... OK (skipped=4)
```

The skips are environment-conditional coverage checks and the Windows directory-symlink privilege check; no test failed. `git diff --check` passed.


## Independent review follow-up: T009/T015 acceptance and traceability

The first independent review found that the scheduled job did not print `ACTIVE_RELEASE_REF`, and that FR-004, FR-011, and SC-006 pointed to tests that did not exercise their stated acceptance criteria. The review otherwise passed the PR trigger, runner routing, read-only permissions, and daily checkout behavior.

Added test-only Red commit `0ce8856`. The focused workflow and traceability suites produced five behavioral assertion failures and no errors: the schedule did not report its active ref, the tool-upgrade review instruction was absent, and the three traceability rows linked to different test methods than the requirement-specific contracts.

Green now prints both the active release ref and scanned commit. The security guide requires a reviewed tool upgrade to update the exact runner pin, local documentation, and independent review together. The CSV links FR-004 to the runner's online RustSec evidence test, and FR-011/SC-006 to a documentation contract that checks the reproducible command, tool version, both workspaces, and review process.

```text
python -X utf8 -m unittest scripts.tests.test_workflow scripts.tests.test_requirement_traceability -v
Ran 16 tests ... OK

git diff --check
passed
```


### Independent review follow-up: exception lifecycle coverage

The second scoped review confirmed T009's release-ref output and the semantic test links for FR-004, FR-011, and SC-006. It found that the FR-011 documentation contract still did not assert the exception-review lifecycle. The policy guide already documented exact package/version scope, rationale, mitigation, distinct owner and reviewer, expiry, approval evidence, and renewal/removal. Expanded the existing documentation contract to assert those points and whitespace-normalize prose before checking it. No policy documentation behavior needed correction.

```text
python -X utf8 -m unittest scripts.tests.test_workflow scripts.tests.test_requirement_traceability -v
Ran 16 tests ... OK

git diff --check
passed
```
