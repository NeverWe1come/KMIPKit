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
