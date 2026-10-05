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

### Independent security review: multiline evidence injection

The reviewer identified that a newline in an SPDX label could pass the
license-field filter and create a second log line. The dedicated Red test
failed on its one-line-output assertion with no test errors. The formatter
was first narrowed to ordinary spaces between SPDX tokens. A later scoped
review found that arbitrary `LicenseRef-*` values could still carry sensitive
text; the final formatter omits all license label spans, which also removes
the original line-injection path.

```text
python -X utf8 -m unittest scripts.tests.test_diagnostic_redaction -v
Red: 1 test FAILED (failures=1, errors=0)
Green: 1 test OK
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


### Documentation lifecycle review closure

A final scoped pass asked the FR-011/SC-006 contract to protect explicit removal of a resolved exception, in addition to renewal. Added assertions for removing the dependency/finding and both policy entries, and for removing both files' exception records. The documentation already contains both instructions.

```text
python -X utf8 -m unittest scripts.tests.test_workflow.WorkflowContractTests.test_dependency_policy_local_command_and_review_process_are_documented scripts.tests.test_requirement_traceability -v
Ran 2 tests ... OK
```

## Red evidence: T021 cargo-deny failure report

Added focused tests for failure findings, credential/token/message sentinels,
malformed JSON, untrusted field shapes, the piped formatter CLI, and the
PowerShell runner's structured-output contract. Before implementation, the
two formatter tests failed because `format_cargo_deny_diagnostics` did not
exist; there were no test errors.

```text
python -X utf8 -m unittest scripts.tests.test_dependency_policy.CargoDenyDiagnosticTests -v
Ran 2 tests ... FAILED (failures=2, errors=0)
```

## Green evidence: T021 cargo-deny failure report

The runner now requests cargo-deny's JSON format and never forwards raw
stdout, stderr, messages, URL paths, credentials, or secret query values. A
Python formatter extracts validated package coordinates and rule codes,
known advisory IDs, and a source host reference with the path redacted. It
omits license label text entirely. Missing or malformed diagnostics become a
generic safe report while the original cargo-deny exit code remains visible.
The actual 0.20.2 `rejected` license diagnostic was exercised in a temporary
fixture; the initial formatter produced package, version, source, and rule
fields while exposing the license label. The final formatter preserves the
rule and package context without printing the label. The fixture used offline
mode only for local diagnostic-shape validation; the repository runner
remains online.

```text
python -X utf8 -m unittest scripts.tests.test_dependency_policy.CargoDenyDiagnosticTests scripts.tests.test_dependency_policy.DependencyPolicyRunnerContractTests -v
Ran 13 tests ... OK

pwsh -NoProfile -Command '$tokens=$null; $errors=$null; [System.Management.Automation.Language.Parser]::ParseFile("scripts/Test-DependencyPolicy.ps1",[ref]$tokens,[ref]$errors) > $null; if ($errors) { $errors | Out-String; exit 1 }'
passed

cargo-deny 0.20.2 temporary GPL fixture
exit code 4; safely formatted `diag-fixture@0.1.0`, `license-rejected`, `GPL-3.0-only`, and `path:local`

python -X utf8 -m unittest discover -s scripts/tests -p "test_*.py" -v
Ran 105 tests ... OK (skipped=4)

git diff --check
passed
```

### Independent review follow-up: untrusted LicenseRef spans

An independent review showed that cargo-deny's license label spans can contain
untrusted `LicenseRef-*` text that matches generic SPDX syntax and could carry
credential-shaped values. A focused test was committed first and reproduced
the leak in the report. The formatter now omits all license spans; rule,
package, version, and redacted source context remain available.

```text
python -X utf8 -m unittest scripts.tests.test_dependency_policy.CargoDenyDiagnosticTests.test_failure_report_omits_untrusted_license_ref_labels -v
RED: Ran 1 test ... FAILED (failures=1, errors=0); report contained the sentinel

python -X utf8 -m unittest scripts.tests.test_dependency_policy.CargoDenyDiagnosticTests scripts.tests.test_dependency_policy.DependencyPolicyRunnerContractTests -v
GREEN: Ran 14 tests ... OK

python -X utf8 -m unittest discover -s scripts/tests -p "test_*.py" -v
Ran 109 tests ... OK (skipped=7)
```

The independent scoped re-review of `f241f38..350768f` marked the
LicenseRef finding **ADDRESSED** and found no regression in the fix. It
confirmed that report assembly does not read or emit license label spans and
that the PowerShell runner logs only the formatted report on cargo-deny
failure.

```text
python -m unittest -v scripts.tests.test_dependency_policy.CargoDenyDiagnosticTests
Ran 5 tests ... OK

git diff --check f241f38..350768f
passed
```

## Convergence evidence: T019

Assessed the approved specification, plan, tasks, current implementation, and
constitution. The review covered 13 functional requirements, 8 success
criteria, 10 acceptance scenarios, 6 edge cases, 9 design decisions, and all
5 constitution principles. No untracked buildable gap remained, so no
convergence task was appended. The existing T017, T018, and T020 entries
continue to track full verification, independent final review, and draft PR
creation.

## Red/Green evidence: T022 offline cargo-deny fixtures

Red commit `a0668a3` added the original pinned-tool negative fixtures. The
first independent review found that Cargo fixture creation could reach the
network and that expected findings could mask plain-text or warning output.
Green commit `edebb8c` made every fixture Cargo invocation use both
`CARGO_NET_OFFLINE=true` and `--offline`, tightened the structured-output
parser, and added guards against online Cargo calls. The scoped re-review of
`a0668a3..edebb8c` marked both findings addressed with no regressions.

The wildcard case has a distinct Red/Green pair. Red commit `436fd64` runs the
fixture with `wildcards = "allow"`; the test failed because cargo-deny
returned success with no wildcard finding. Green commit `a4fb756` restores the
fixture's denied-wildcard policy. The test asserts the exact `wildcard` rule,
declaring package `fixture-root@0.1.0`, the manifest requirement `*`, and the
locked dependency `fixture-wildcard@1.0.0`. The scoped re-review of
`436fd64..a4fb756` found no issues.

```text
python -X utf8 -m unittest scripts.tests.test_cargo_deny_fixtures -v
Ran 15 tests ... OK

python -X utf8 -m unittest discover -s scripts/tests -p 'test_*.py' -v
Ran 123 tests ... OK (skipped=4)

pwsh -NoProfile -File scripts/tests/Test-Wsl.ps1
All PowerShell WSL tests passed.

pwsh -NoProfile -File scripts/Test-DependencyPolicy.ps1
Verified cargo-deny 0.20.2; root and fuzz checks passed after online RustSec
refresh at ef6173cbc5c50ec8166f9a5b28f07834144373ee
2026-10-03T10:14:03+02:00; root and fuzz Cargo.lock SHA256 values unchanged.
```

The four Python skips on Windows are directory-symlink cases requiring
privileges unavailable to the current user. CI covers those cases on Linux
and macOS. The candidate scan passed with cargo-deny 0.20.2 when supplied via
`CARGO_DENY`.

## Local verification evidence: T017 progress

On the Windows host, the following CI-equivalent checks passed: `cargo fmt
--all --check`; Clippy and workspace tests with `--locked`; and
`cargo doc --workspace --all-features --no-deps --locked`. The full Rust test
command completed with no failed tests or doctests. The normative catalog
validated 4 sources, 1,411 clauses, and 4,021 records; the 167 catalog tests,
immutable-source check, source-candidate audit, and both generated-output
checks passed. `git diff --check` and both lockfile immutability checks passed.

The Windows LLVM JSON report normalized successfully and measured TTLV/
protocol at 97.46%, transport/FFI at 100%, and workspace at 97.31%. Linux,
macOS, and the three-platform aggregate remain for PR CI; T017 remains open
until those required checks and the remaining repository gates are recorded.

## Convergence closure evidence: T024-T026

T024 has a distinct Red/Green pair. Red commit `3da4e06` added an executable
traceability expectation for FR-012 while the CSV still pointed to an
exception-source test; the traceability suite failed on that mismatch. Green
commit `1119bb5` maps the requirement to
`CargoDenyDiagnosticTests.test_failure_report_retains_allowlisted_finding_fields_and_redacts_secrets`,
which asserts the safe diagnostic fields and secret omission.

T025 has a distinct Red/Green pair. Red commit `43ddeb5` added cargo-deny
fixtures for absent and invalid SPDX license metadata and failed because
private packages were configured to be ignored. Green commit `80c08b6`
explicitly sets `licenses.private.ignore = false`; both fixtures then assert
the exact `licenses` / `unlicensed` finding and package/version. The pinned
cargo-deny fixture suite passed all 17 tests.

T026 has a distinct Red/Green pair. Red commit `0759592` changed the FR-013
and SC-008 traceability contract to require the new executable invariant;
the suite failed because that method did not exist. Green commit `0b03f8a`
implements it and wires it into `scripts/Test-DependencyPolicy.ps1` after the
runner verifies the pinned cargo-deny binary and refreshes RustSec. It runs
both root and fuzz graphs offline with `--locked`, snapshots each lockfile's
bytes and complete `name@version` set, and requires both snapshots to remain
identical. The same commit adds a runner contract test for this invocation.

Independent QA re-review on 2026-10-06 closed all three P2 findings and
recorded the results in `checklists/policy-review.md`; no additional
actionable issue was found. Its focused command set `CARGO_DENY` to
cargo-deny 0.20.2 and passed 21 tests.

Fresh Windows verification on 2026-10-06:

```text
pwsh -NoProfile -File scripts/Test-DependencyPolicy.ps1
Dependency policy checks passed; Cargo.lock and fuzz/Cargo.lock SHA256 hashes are unchanged.
RustSec root and fuzz: ef6173cbc5c50ec8166f9a5b28f07834144373ee
2026-10-03T10:14:03+02:00

python -B -m unittest discover -s scripts/tests -p 'test_*.py' -v
Ran 127 tests ... OK (skipped=22)

pwsh -NoProfile -File scripts/tests/Test-Wsl.ps1
All PowerShell WSL tests passed.
```

The discovery run skips cargo-deny fixtures when the exact binary is not
provided in its environment and skips privilege-dependent Windows symlink
cases. The dedicated policy runner supplies the verified binary and passed
all 17 cargo-deny fixtures. T017 remains open for required Linux/macOS and
aggregate CI results and the remaining repository gates.

## Pull request workflow trigger investigation: 2026-10-06

PR #37 is open as a draft against `release/1.0.0`, and GitHub reports its
merge state as clean. The active `CI` workflow includes `release/**` in its
`pull_request` base-branch filter, Actions is enabled for the repository, and
the commits in this branch contain no recognized CI-skip annotation. The
GitHub Actions API reports no run for a `pull_request` event and no check runs
for the PR head. The only run for head `7f4db22` is run `37381749747`, a `push`
event created before PR #37 existed; it failed with zero jobs because the
workflow has no `push` trigger. PRs #35 and #36 targeting this same release
branch had successful pull-request runs.

GitHub's documented default `pull_request` activity types include `opened`,
`synchronize`, and `reopened`; GitHub also documents merge conflicts and skip
annotations as conditions that suppress these runs. The observed PR state,
workflow filter, and commit messages do not match those exclusions. No
repository-side trigger mismatch has been identified. The available API does
not expose the delivery record for the missing webhook event, so its cause
remains unverified and cannot safely be attributed to a repository setting.
After this evidence update is pushed, inspect the resulting `synchronize`
event once. T017 stays open unless the required Linux, Windows, macOS, and
aggregate checks complete successfully.

References: [GitHub Actions pull_request event](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#pull_request)
and [workflow trigger troubleshooting](https://docs.github.com/en/actions/how-tos/troubleshoot-workflows#triggering-event-conditions).

## Review convergence: workflow parsing and exception suppression

The initial trigger investigation above was superseded by run annotations from
GitHub. Run `37381749747` reports an invalid workflow file at
`.github/workflows/ci.yml:237`: the unquoted inline `run` scalar contained
`: ` in `echo "Active release ref: $ACTIVE_RELEASE_REF"`. The next diagnostic
push, commit `cbb4e13`, produced run `37383185830`; GitHub again identified a
workflow-file failure and created no jobs. The adjacent scanned-commit command
had the same YAML scalar issue. Both inline commands are now quoted as complete
YAML scalars.

The regression test was added first in Red commit `be35a27`. The focused test
failed on workflow line 237, and the prior scheduled-report assertion also
failed because it permitted the invalid scalar. Green commit `1aff744` quotes
both commands. Refactor commit `28d1f40` centralizes inline-run scalar
inspection and reports all unsafe scalars together. Verification passed:
`python -m unittest scripts.tests.test_workflow -v` (16 tests), followed by
`python -m unittest discover -s scripts/tests -p 'test_*.py'` (128 tests,
22 environment-dependent skips). These commits are in the PR branch; full
platform CI is still required to close T017.

Independent security review found that `validate_exception_config` accepted
`bans.skip-tree` without matching it to an exact exception record. Cargo-deny
documents this setting as suppressing duplicate findings for a crate and its
direct/transitive dependency tree. The checked-in configuration did not use
it, but a future unregistered value could bypass exact duplicate-waiver
matching. Red commit `0f66eb8` added a regression test and failed because no
`PolicyError` was raised. Green commit `4d2eb05` rejects malformed or
non-empty `bans.skip-tree`, updates FR-007, task T027, compliance traceability,
and the security guide. Verification passed: the new focused test,
`python -m unittest scripts.tests.test_dependency_policy -v` (51 tests, 2
environment-dependent skips), and the full script suite (129 tests, 22
environment-dependent skips). Cargo-deny's behavior is described in its
[bans configuration documentation](https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html#the-skip-tree-field-optional).

The security reviewer must re-review commit `4d2eb05` and complete the
reviewer-owned checklist. T017 remains open until CI runs successfully on
Linux, Windows, macOS, and the coverage aggregate; T020 remains open until
the final release-base update and review package are complete.
