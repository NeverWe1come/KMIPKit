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
