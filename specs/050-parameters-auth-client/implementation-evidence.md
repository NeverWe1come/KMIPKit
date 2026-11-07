# KMIPKIT-0050 Implementation Evidence

## Branch and scope

- Branch: `feature/KMIPKIT-0050-parameters-auth-client`.
- Base: `feature/KMIPKIT-0049-managed-objects` (PR #116), as recorded in
  `approval-record.md`.
- At the initial 2026-11-07 audit snapshot, the latest synchronized predecessor
  was `28d209fd0` (PR #116), included through normal merge `c5e3051ef`.
- Scope: typed Cryptographic Parameters, request-scoped KMIP Authentication,
  direct Rust client methods, public API reconciliation, examples, and tests.

## Red, Green, Refactor commits

| Work group | Red | Green | Refactor |
|---|---|---|---|
| Cryptographic Parameters model | `c4028f6e6`, `1b5381e3c` | `823eb040c` | `d0d7d396b` (subsequent API cleanup) |
| Operation parameter setters | `50f0c045f` | `cc4cca605` | `d0d7d396b` |
| Request Header Authentication | `72bb37335`, `311374bb3` | `45dc27944` | `f000939e5` |
| Authentication TTLV ownership | `96f34ec0f` | `9e22b5326` | `a91267df7` |
| Complete direct Client API | `3c5927b3e` | `c3faccac9` | `d70a1071b` |
| Direct method transport parity | `3b8c5a025` | `f0a98a151` | `d0d7d396b`, `f34495eb3` |

`f34495eb3` extends the fake-transport matrix to exercise the default forms
of the direct methods as well as their `_with_options` forms, and checks that
the Cryptographic Parameters debug views do not reveal `P Source`.

## Verification

Successful Windows checks on this branch:

- `cargo fmt --all --check`.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`.
- `cargo test --workspace --all-features`.
- `cargo test --offline -p kmipkit-client --lib authentication_owner` passed
  both ADR-0012 candidate-callsite lifecycle tests: ownership survives a
  successful exchange and is zeroized afterward; a partial-write failure
  zeroizes the owner, reports the delivery outcome, and does not retry.
- After synchronizing to predecessor head `28d209fd0`, the same focused command
  passed again (2 tests); `cargo fmt --all --check` and `git diff --check` also
  passed.
- `cargo llvm-cov --workspace --all-features --locked --json --output-path coverage-raw-local.json`
  (full Windows workspace test and coverage instrumentation completed).
- `python scripts/coverage_gate.py normalize --workspace . --input coverage-raw-local.json --output coverage-local.json`.
- `cargo test -p kmipkit-protocol --all-features --test cryptographic_parameters_public`
  (9 tests passed).
- Focused direct-method parity and option-forwarding tests passed.
- `cargo test --offline -p kmipkit-client --all-features request_authentication_tests`
  (6 tests passed), including a pointer-identity regression test proving the
  caller-owned authentication TTLV tree is moved into the request without a
  deep clone.
- After the ownership fix, `cargo fmt --all --check` and
  `cargo clippy --offline -p kmipkit-client --all-targets --all-features -- -D warnings`
  passed. The Refactor commit `a91267df7` keeps the ownership move at the
  request-header construction boundary.
- `python -m unittest tools.api_manifest.tests.test_reconcile_0050_parameters_auth_client`
  (3 tests passed).
- `python tools/api_manifest/generate.py --check` (12 generated outputs current).
- `python tools/api_manifest/audit_rust_exports.py --check` (complete, matches
  manifest, no missing or unobserved Rust paths, zero issues).
- `python -B -m unittest scripts.tests.test_requirement_traceability -v`
  (2 tests passed).
- `python -B -m unittest discover -s tools/normative_catalog/tests -p 'test_feature_traceability.py' -v`
  (30 tests passed; one symlink-privilege-dependent test skipped by Windows).
- `python scripts/test_user_guide_examples.py` (22 Rust guide examples compiled).
- Manifest reconciler dry run: zero declarations and exclusions left to add.

Windows line coverage calculated from the normalized report using the exact
eligible-source inventory in `scripts/coverage_gate.py`. The repository's
documented generated-source exclusions were applied; this is a Windows-only
measurement, not the three-platform CI aggregate.

| Scope | Covered / executable lines | Result |
|---|---:|---:|
| Changed executable Rust lines | 305 / 318 (95.91%) | Meets 95% |
| `kmipkit-ttlv` | 735 / 741 (99.19%) | Meets 95% |
| `kmipkit-protocol` | 21,442 / 22,306 (96.13%) | Meets 95% |
| `kmipkit-transport` | 3,718 / 3,907 (95.16%) | Meets 85% |
| `kmipkit-ffi` | 12,505 / 14,258 (87.71%) | Meets 85% |
| Workspace total | 46,597 / 49,625 (93.90%) | Meets 90% |

The unfiltered `cargo llvm-cov --text` diagnostic previously showed 89.98% for
`kmipkit-protocol`; that denominator included generated files excluded by the
repository's exact documented coverage inventory. With the same exclusions
the coverage gate uses, the local Windows measurements exceed every Rust
threshold, including protocol and changed executable lines. The separate
instrumented C-consumer report and the Linux/macOS platform reports have not
been produced locally, so this does not establish that the complete hosted
coverage job passes.

Only Windows has been exercised locally. Linux and macOS checks and hosted CI
remain unverified; the repository's current GitHub billing issue may prevent
hosted CI from running. The Windows `aws-lc-sys` build emitted non-fatal
environment warnings about NASM and an optional compiler probe.

## Reviews and publication

- Independent QA review found no API-manifest, documentation, or parity
  discrepancy; its coverage concern was resolved by recalculating against the
  gate's documented generated-source exclusions.
- Independent security review and Codex Security diff scan found no confirmed
  security defect. The scan reviewed 15 changed files and recorded zero
  findings (scan `ab320995-27a7-4417-8bdc-17bed9db42a6`). Its follow-up review
  identified a local resource concern: caller-supplied `Authentication` was
  deep-cloned before encoder limits apply. The 0050 follow-up now moves that
  owned TTLV tree into the request without cloning; this removes the temporary
  amplification path. The follow-up Codex Security diff scan reviewed
  `crates/kmipkit-client/src/execute.rs`, recorded zero findings, and completed
  with full changed-file coverage (scan
  `e30feefc-b9b4-416c-b625-eec029f8eede`).
- Draft pull request: [#118](https://github.com/NeverWe1come/KMIPKit/pull/118),
  base `feature/KMIPKIT-0049-managed-objects`, head
  `feature/KMIPKIT-0050-parameters-auth-client`. The original PR-open head was
  `4f23fba09f791dbbd7f510188151a3683fef1dfb` (historical).
- ADR-0012 Decision 6 still requires hosted CI to run and pass these exact
  candidate-callsite tests before the production route can be merged, enabled,
  or released. CI was not queried under the maintainer's instruction during
  the billing issue, so this remains an integration gate despite the local
  pass.
- This evidence does not claim profile conformance, interoperability, human
  security approval, or release readiness.

## Current PR metadata and verification basis (2026-11-07)

- At the initial audit snapshot before either evidence refresh, the #118
  worktree was clean at `ecbd9c978c625ba3648e49f74149287b54ae6c09`, matching
  its local `origin` ref and `gh pr view 118` head. GitHub reported PR #118
  `OPEN`, draft, with base ref OID
  `28d209fd0a159376a8b7ca2b58c71704b37cc9a2` and head ref OID
  `ecbd9c978c625ba3648e49f74149287b54ae6c09`.
- PR #116's documentation-only evidence refresh was pushed as
  `77b3d91ca2dd5597e2eb3a8e89bef957115bd537`. This branch incorporates that
  parent through merge commit `79277e78f7484196ae2dd97c4d5d30ee4745a298`
  (parents: initial #118 head `ecbd9c9…` and refreshed #116 head `77b3d91…`).
  At this pre-push snapshot, the worktree is at that merge commit; GitHub still
  reports the not-yet-pushed #118 head `ecbd9c9…` and base ref OID `28d209fd…`.
- The full workspace verification listed above was recorded on the original
  #118 source snapshot `4f23fba09f791dbbd7f510188151a3683fef1dfb`. The
  0050-owned implementation and test paths are unchanged from that snapshot.
  After predecessor synchronization to `28d209fd0`, the focused ADR-0012
  candidate-callsite tests, formatting, and diff checks were rerun; the full
  workspace suite and coverage were not rerun on the combined stack.
- Compared with the original #118 source snapshot, the additional source/test
  paths are predecessor-owned: `crates/kmipkit-protocol/src/profile.rs`,
  `crates/kmipkit-protocol/tests/credential_contract.rs`,
  `crates/kmipkit-protocol/tests/support/profile_context_unit.rs`,
  `crates/kmipkit-protocol/tests/support/profile_unit.rs`, and
  `crates/kmipkit-protocol/tests/unit/get_attribute_list_tests.rs`. They are
  covered by the #049 workspace verification recorded against source snapshot
  `0a5f661c9a7964825dab597d4472f8edee2cca69`; comparison from that snapshot to
  #049's audited head `28d209fd0` contains documentation/evidence changes only.
- Since the previous #118 audit head `c30f3d4b1f013c40c3886a7aff2b32de4a128400`,
  the stack/evidence updates changed only documentation files, including the
  merge above. This refresh also changes documentation only; no #050
  implementation, test, or generated path is changed by it.
- Remote CI checks were not queried under the maintainer's billing instruction.

## Latest predecessor synchronization and follow-up (2026-11-07)

- This local branch now includes the latest #116 head `cbe09ce2f` through
  merge commit `1adb90894`. The merge carries the corrected Decrypt manifest
  attribution and regenerated parity fixtures from #115.
- After synchronization, the six focused Authentication tests passed, the
  focused Encrypt/Decrypt manifest module passed all 5 tests, and
  `python -B tools/api_manifest/generate.py --check --repo-root .` verified all
  12 generated outputs.
- The authentication ownership follow-up separately passed
  `cargo fmt --all --check`,
  `cargo clippy --offline -p kmipkit-client --all-targets --all-features -- -D warnings`,
  and the 6 focused Authentication tests. Its Red, Green, and Refactor commits
  are listed above. Full workspace tests and coverage have not been rerun on
  the combined latest stack.
- The follow-up Codex Security diff scan
  (`e30feefc-b9b4-416c-b625-eec029f8eede`) reviewed the changed
  `execute.rs`, found zero issues, and completed with full changed-file
  coverage. It was a static review; runtime tests are recorded separately.
- This refreshed local branch has not yet been pushed to PR #118. Hosted CI
  remains unqueried.


## Final reviewed predecessor synchronization (2026-11-07)

The final reviewed 0049 predecessor
`4958510050a9274d193a6606beb005287bedcd97` is included through normal merge
`062ed4e61`, starting from the original 0050 head
`fb4191e6de85c4bbac275cc67e421058b33dc4ad`. Earlier synchronization and PR
metadata above remain historical snapshots. The final 0049 follow-up changes
only evidence and T102 regression files; its runtime and manifest inputs are
identical to the audited `5f8536a93f4e753d80ed8ac9544ed3e14399be17` snapshot.

All 12,230 incoming declarations, 17,384 exclusions, 32 capabilities, and other
incoming manifest sections are preserved exactly. The official 0050 reconciler
restores 184 declarations and 42 exclusions, producing 12,414 declarations and
17,426 exclusions overall. All original 0050 exclusions and 180 of its 184
declarations are unchanged. Four Discover Versions direct-method C mappings
now correctly name `kmipkit_client_batch_item_discover_versions`; the old
snapshot incorrectly selected a metadata type as the constructor. Typed
parameters and request Authentication execution stay blocked for C, Java,
and Python parity under KMIPKIT-0053.

Integration exposed a prefix-selection defect in the official 0050 reconciler:
operation-owned C metadata and similarly prefixed Java/Python routes could be
selected before the constructor. The correction selects the exact constructor
name from the existing direct-method inventory and requires its same-operation
mapping already present in the reviewed manifest. It invents no language
mapping and adds no export. Separate development evidence:

| Phase | Commit | Result |
|---|---|---|
| Red | `93b31ac5d` | The 5-test 0050 reconciler module produced 4 expected assertion failures against the original code: C metadata and Java/Python lookalikes were selected, and a missing exact constructor did not fail closed. |
| Green | `f6f2efa4d` | The combined 0048/0049/0050 reconciler command below passed all 21 tests. |
| Refactor | `a67d56621` | Streamed candidates and documented the exact-constructor invariant. All 21 tests passed again; full real-input reconciliation reproduces the complete final manifest exactly, and the official current-report dry run adds zero rows. |

The shared test transport retains both options-observer entry points. The 0050
direct-method tests now expect the predecessor's documented resolved `Ttlv`
selection at the transport boundary while still comparing exact caller timeouts
and direct-versus-batch requests. The initial two expectation failures and the
subsequent five-test pass were observed before committing the resolution.
Authentication ownership, redaction, zeroization, delivery state, and one-exchange
behavior remain covered by the original tests and their fresh runs below.

Current reports live under `specs/050-parameters-auth-client/reports/`; the
paired shared-audit defaults and current-snapshot tests select those reports.
All 45 inherited 0047, 0048, and 0049 report files were verified byte-identical
to the predecessor. No historical report was overwritten. The official generator
regenerated all 12 outputs; no generated output was edited by hand.

Fresh Windows verification against the final combination:

- `python -B -m unittest discover -s tools/api_manifest/tests -p test_t102_*.py -q`
  passed all 399 tests in 141.061 seconds, without failures, errors, or skips.
  This is the single completed full T102 run against the final combination;
  the earlier superseded-predecessor run was stopped and supplies no pass claim.
- The exact combined API command below passed all 110 tests in 158.573 seconds,
  without failures, errors, or skips. The two T102 declaration fingerprints were
  recalculated against the combined manifest; operation fingerprints and
  normalization logic remain unchanged.
- `python -B -m unittest tools.api_manifest.tests.test_reconcile_0048_typed_attributes tools.api_manifest.tests.test_reconcile_0049_managed_objects tools.api_manifest.tests.test_reconcile_0050_parameters_auth_client -q`
  passed 21 tests: 14 ownership-filter regressions from 0048, 2 managed-object
  regressions from 0049, and 5 constructor regressions from 0050. This command
  passed again after the Refactor changes.
- `cargo test --offline --locked -p kmipkit-client --all-features --lib request_authentication_tests`
  passed 6 tests; `cargo test --offline --locked -p kmipkit-client --all-features --lib authentication_owner`
  passed both ADR-0012 candidate-callsite lifecycle tests.
- `cargo test --offline --locked -p kmipkit-client --all-features --lib direct_method`
  passed 5 tests; `cargo test --offline --locked -p kmipkit-protocol --all-features --test cryptographic_parameters_public`
  passed 9 parameter-contract tests.
- `cargo test --offline --locked -p kmipkit-protocol -p kmipkit-client --all-features --quiet`
  passed 698 client and 1,277 protocol unit tests, all selected integration
  tests, one client doctest, and three protocol doctests. Three explicitly
  ignored live-client integration tests remain ignored. This complete run
  preceded the final test-only 0049 correction; its exercised protocol/client
  Rust runtime and test sources are identical in the final tree. The focused
  tests above were rerun after the final merge.
- `cargo fmt --all --check` and
  `cargo clippy --offline --locked --workspace --all-targets --all-features -- -D warnings`
  passed on the final tree.
- `python -B tools/api_manifest/generate.py --check` verified all 12 outputs
  after the final predecessor merge.
- Official Rust audit `--check`: 22,655 observed routes, 6,825 exact manifest
  matches, 15,830 exclusions; zero missing, unobserved, kind, conditional, or
  scope issues. The pre-reconciliation snapshot found exactly 226 0050 routes
  missing and no other discrepancies. The report README records exact commands.
- Paired shared Rust audit `--check`: passed, with 3,697 mapped declarations
  and the current 0050 Rust report explicitly recorded as its source.
- Official binding audit `--check`: C 1,708 exact matches, Java 1,881, Python
  2,000. All inventories are complete with zero missing routes, unobserved
  declarations, kind mismatches, or issues.
- `python -B -m unittest scripts.tests.test_requirement_traceability -q`
  passed 2 tests. The stable post-generation feature-traceability run passed
  30 tests with one Windows symlink-privilege skip; its targeted typed-profile
  evidence test also passed separately.
- `python -B tools/normative_catalog/check_immutable_sources.py --base-sha fb4191e6de85c4bbac275cc67e421058b33dc4ad --repo-root .`
  passed: pinned upstream OASIS bytes are unchanged.

Exact combined API command:

```text
python -B -m unittest tools.api_manifest.tests.test_reconcile_0049_managed_objects tools.api_manifest.tests.test_reconcile_0048_typed_attributes tools.api_manifest.tests.test_reconcile_0050_parameters_auth_client tools.api_manifest.tests.test_t102_manifest_size_limit tools.api_manifest.tests.test_c_api_docs tools.api_manifest.tests.test_t102_response_view_mappings tools.api_manifest.tests.test_encrypt_decrypt_parity_manifest tools.api_manifest.tests.test_047_binding_audit_reconciliation tools.api_manifest.tests.test_t102_remaining_response_view_mappings tools.api_manifest.tests.test_parity tools.api_manifest.tests.test_catalog_parity tools.api_manifest.tests.test_t109_binding_export_audit_reconciliation tools.api_manifest.tests.test_shared_rust_inventory tools.api_manifest.tests.test_manifest_v2.PublicApiManifestV2SchemaTests.test_checked_in_manifest_inventories_all_rust_public_access_paths tools.api_manifest.tests.test_t102_certify_request_builder_mapping tools.api_manifest.tests.test_t102_optional_request_structure_mapping -q
```

Authored synchronization paths pass `git diff --check`. The complete predecessor
diff retains its two existing whitespace diagnostics: a final blank line in
`interop/c-kmip-validator/src/c_validator.c` and trailing whitespace in
`interop/shared/vectors/generate_vectors.py`. Neither is changed by this
resolution. Rust commands emitted the established non-fatal Windows AWS-LC
compiler-probe and missing NASM warnings.

No PR metadata was changed, no alias was created, and remote CI was not queried.
Existing untracked files and target directories were preserved; target sizes
were never inspected. There is no fresh coverage measurement or cross-platform
aggregate. Existing hosted CI, coverage, human review, and ADR-0012 hosted
candidate-test gates remain outstanding; the branch remains draft.
