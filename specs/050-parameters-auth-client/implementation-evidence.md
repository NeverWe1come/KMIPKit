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

## Full public-surface audit after the 0049 merge (2026-11-09)

The preceding 0050 audit summary is superseded for the combined branch. The
post-sync Rust export report now records 27,234 observed routes: 7,037 exact
manifest matches, 15,998 exact exclusions, and 4,199 routes missing from the
manifest. The scanner is complete and reports zero kind mismatches, conditional
mismatches, or other issues, but `matchesManifest` is false. Consequently,
`audit_rust_exports.py --check` exits nonzero and
`audit_shared_rust_inventory.py --check` reports unresolved missing routes.
Examples in the missing set include `ClientListener` APIs; the total is a
whole-tree inventory and must not be described as 4,199 KMIPKIT-0050-owned
routes. The pre-reconciliation report records exactly 226 0050-owned paths
(184 declarations and 42 trait exclusions), and none of those paths remain in
the current missing set. The remaining routes require owner-by-owner review
under the 0043 whole-tree manifest gate. KMIPKIT-0053 separately owns the
cross-language parity for 0050's APIs and is the planned final phase-5
reconciliation point; it does not own every missing route by default.

T043's command-execution scope is complete, but this audit is not a passing
release gate. The branch remains Draft until the final manifest reconciliation
and all required hosted checks pass. The old remote CI result was for a prior
branch head; it must be rerun after publishing the synchronized branch.
Existing untracked files and target directories were preserved; target sizes
were never inspected. There is no fresh coverage measurement or cross-platform
aggregate. Existing hosted CI, coverage, human review, and ADR-0012 hosted
candidate-test gates remain outstanding; the branch remains draft.

## Stacked CI formatting correction (2026-11-08)

- **Red:** `cargo +1.94 fmt --all --check` failed on this branch and hosted Core jobs for #142. Rustfmt required `mod kem` and `pub use kem` to precede the `key_*` declarations/exports in `crates/kmipkit-protocol/src/lib.rs`.
- **Green:** commit `8e3a27fb9` applies only rustfmt's ordering in that file; it does not change protocol behavior or public symbols.
- **Verification:** `cargo +1.94 fmt --all --check` passed after formatting. The same correction is included in the #143 branch so stacked descendants inherit it. Full hosted CI is still pending and is not claimed green by this local formatting check.

## Authentication documentation correction (2026-11-08)

- **Red:** the independent security review found that `Client::execute` rustdoc claimed the exchange never submits Authentication or Credential data, while the approved `ClientBatch::with_authentication` API explicitly adds caller-supplied Authentication to the Request Header.
- **Green:** the rustdoc now states that Authentication is included only when explicitly attached to the batch, is encoded once in the Request Header, and is not retained in client configuration. It also clarifies that KMIPKit does not generate or verify attestation evidence and that the server decides acceptance.
- **Refactor:** documentation-only correction; no wire behavior or API signature changed. `cargo +1.94 fmt --all --check`, the focused test `cargo +1.94 test -p kmipkit-client authentication_is_emitted_once_in_request_header_order_for_one_item` (1 passed), and `cargo +1.94 doc -p kmipkit-client --no-deps` all passed.

## Log direct-client requirement parity correction (2026-11-08)

- **Red:** hosted CI for PR #142 failed `test_every_verified_log_declaration_maps_to_log_facades_in_all_languages`: verified `Log` declarations include KMIPKIT-0050-FR-010 and FR-011 through the new direct `Client` methods, but the test's expected requirement set covered only the operation's KMIPKIT-0042 and KMIPKIT-0043 requirements. The same focused failure was reproduced locally before the fix.
- **Green:** the parity contract now checks operation-level requirements on the Log operation record separately from the complete requirement union on verified declarations, and allows the already implemented `Client::log_with_options` Rust declarations. No protocol behavior or manifest content changed.
- **Refactor:** named the two requirement scopes explicitly (`OPERATION_REQUIREMENTS`, `DIRECT_API_REQUIREMENTS`, and `VERIFIED_DECLARATION_REQUIREMENTS`) so operation conformance is not conflated with direct-client API requirements.
- **Verification:** `python -B -m unittest tools.api_manifest.tests.test_log_parity_manifest tools.api_manifest.tests.test_reconcile_0050_parameters_auth_client -v` passed (8 tests); `git diff --check` passed. The hosted workflow must rerun on this commit before PR #142 or its stacked descendants can be considered green.

## KMIP 3.0 KEM response correction (2026-11-08)

- **Normative review:** the pinned KMIP 3.0 Working Draft says Encapsulate's response is `Unique Identifier` + `Data` (§6.1.22, Table 318) and Decapsulate's response is only `Unique Identifier` (§6.1.15, Table 297). The shared secret belongs to the newly created server-managed Secret Data object; `Data` is the encapsulated output. The prior model incorrectly placed bytes under Key Material (`0x00420043`) in both responses.
- **Red:** commit `ffdf21c3c` adds assertions for exact response tags and absence of a shared-secret field/bytes in Debug. Before production changes, `cargo test -p kmipkit-protocol test_encapsulate_request_and_response -- --nocapture` failed with the actual Encapsulate tags `[Unique Identifier, Data, Key Material]` versus expected `[Unique Identifier, Data]`.
- **Green:** commit `73e7f1ec7` removes shared-secret fields/getters from both response types, serializes only the exact table fields, names the encapsulated output `data`, and uses the correct Data tag for the Decapsulate request. Commit `8fcb01c6e` adds server-test expectations; it failed before its fix because Encapsulate returned the input key identifier instead of a new Secret Data identifier. Commit `6fad2b833` creates and stores that Secret Data result server-side, returns only its identifier for Decapsulate, and corrects the server operation IDs to Table 583 (`Deactivate=0x40`, `Encapsulate=0x41`, `Decapsulate=0x42`).
- **Refactor/docs:** updated KMIPKIT-0050 FR-014/015 and `traceability.md`, corrected the KMIP 3.0 KEM description in KMIPKIT-0071, and added the pre-1.0 constructor/accessor migration guidance to `docs/development/kmip-3.0-api-migration.md`.
- **Focused verification:** `cargo test -p kmipkit-protocol kem_tests -- --nocapture` passed (3 tests); `cargo test -p kmipkit-test-server dispatcher_all_operations_and_error_handling -- --nocapture` passed (1 test). Both commands emitted the known non-fatal Windows aws-lc-sys/NASM warnings. Workspace-level checks for this correction are recorded below after completion.

## Binding audit snapshot refresh (2026-11-09)

- **Red:** `python -B -m unittest tools.api_manifest.tests.test_c_api_docs -q` failed because the checked-in C/Java/Python report and its Markdown table still described 1,708/1,881/2,000 declarations, while the merged manifest contains 1,733/1,906/2,025.
- **Green:** regenerated the JSON report using `python -B tools/api_manifest/audit_binding_exports.py --repo-root . --report specs/043-public-api-bindings/reports/binding-export-report-2026-11-07.json --check`; the official audit confirmed complete scans and exact matches for all three packages. The 0050 post-sync report was regenerated from the same current tree.
- **Refactor/docs:** updated the authored audit guide's status date, source range, and table to the regenerated counts, and clarified that a complete binding audit does not close the separate whole-tree Rust manifest gate.
- **Verification:** `python -B -m unittest tools.api_manifest.tests.test_c_api_docs -q` passed all 4 tests after the report and table refresh. `python -B tools/api_manifest/audit_binding_exports.py --repo-root . --report specs/043-public-api-bindings/reports/binding-export-report-2026-11-07.json --check` exited 0. The separate 97 focused manifest tests also passed.

## KMIP 3.0 client operation completion (2026-11-09)

- **Authorization and scope:** the maintainer-approved Phase 5 correction plan authorizes adding typed request/response payload models and direct client methods for the seven operations already catalogued by KMIPKIT-0071. The KMIP 3.0 Working Draft remains explicitly labeled a draft and is not the default protocol version.
- **Normative reconciliation:** the pinned source is `specification/oasis/kmip-3.0/upstream/kmip-spec-v3.0.html`, SHA-256 `a7d5e58e63acd9be33fab08b76039fc8db4e072fdbdf4a280ea5b743bae5682b`. Parsed payload tables confirm Create Credential 277–278 (Credential Type, Attributes, and one defined credential object; Unique Identifier response), Create Group 280–281 (Attributes; Unique Identifier response), Create User 290–291 (Attributes; Unique Identifier response), Deactivate 293–294 (Unique Identifier, optional Deactivation Reason/Date; Unique Identifier response), Decapsulate 296–297 (Unique Identifier, optional Cryptographic Parameters, Data, optional Attributes; new Secret Data Unique Identifier response), Encapsulate 317–318 (Unique Identifier, optional Cryptographic Parameters/Attributes; new Secret Data Unique Identifier and Data response), and Obliterate 374–375 (Unique Identifier request; empty response payload). Table 583 assigns Create Group=`0x3C`, Obliterate=`0x3D`, Create User=`0x3E`, Create Credential=`0x3F`, Deactivate=`0x40`, Encapsulate=`0x41`, and Decapsulate=`0x42`; `0x43` is unassigned.
- **Red:** `cargo test -p kmipkit-client --lib kmip30_operation_surface_tests --no-run` fails as expected before implementation because Create Credential, Create Group, Create User, and Obliterate request models, all seven `ClientOperation` variants, and direct methods are absent. The corrected VersionGate test verifies all seven codes and rejects `0x43`; this uses the already-correct exact table, not inferred numeric-range support.
- **Green:** commit `b1fd28664` adds the typed payload models, all seven `ClientRequest`/`ClientOperation` cases, direct methods and `_with_options` methods, typed response dispatch, and operation-version gating through the existing batch path. The KMIP 3.0 response parsers follow the pinned tables; Decapsulate rejects a response `Data` field and neither KEM response nor diagnostics expose the shared secret.
- **Refactor:** the follow-up refactor commit records the exact operation-table source, separates request/response fixtures into focused tests, updates FR-016/017 traceability, and marks T046/T047 complete. T048 remains open only for versioned manifest reconciliation of the KMIP 3.0 public API routes, assigned to KMIPKIT-0053 together with C/Java/Python parity. The checked-in v2.1 manifest still inventories its 56 client operations; it does not yet claim KMIP 3.0 four-language parity.
- **Focused verification:** `cargo test -p kmipkit-protocol --lib --quiet` passed (1,341 tests); `cargo test -p kmipkit-client --lib --quiet` passed (735 tests); `cargo test -p kmipkit-test-server dispatcher_all_operations_and_error_handling -- --nocapture` passed (1 test). `cargo test --workspace --all-features --quiet` passed (exit code 0). `cargo fmt --all --check`, `cargo clippy -p kmipkit-protocol -p kmipkit-client --all-targets --all-features -- -D warnings`, the KMIPKIT-0050 manifest reconciliation test (5 tests), and `python -B tools/api_manifest/generate.py --check` (12 generated outputs) passed. Windows AWS-LC compiler-probe/NASM warnings were non-fatal. Hosted CI has not yet been confirmed green for the pushed head.

## Changed-code coverage remediation (2026-11-09)

- **Red:** hosted run `42568647774`, job `129006019991`, failed the 90% changed-Rust coverage gate at 812/977 executable changed lines (83.1%). Coverage artifacts identified unexercised KMIP 3.0 response-view/display/debug and Cryptographic Parameters builder/accessor paths. This was a real coverage failure, not an infrastructure failure.
- **Green tests added:** `all_kmip30_requests_use_the_table_583_wire_code_and_preserve_typed_response` now exercises each KMIP 3.0 request's redacted Debug path, typed response view, typed KMIP 3.0 response view, response-view Debug, outcome Display, and direct batch-item response accessor. `every_table_59_builder_setter_and_typed_reader_roundtrips` exercises all 18 Table 59 setters/readers, P Source borrowing, Debug redaction shape, and preservation of a vendor extension.
- **Focused local verification:** Red commit `d521c529e` adds the KMIP 3.0 response-view coverage and Green commit `c6e8ddeaf` adds the Table 59 builder/accessor coverage. `cargo test --offline -p kmipkit-client --lib kmip30_operation_surface_tests` passed (5 tests); `cargo test --offline -p kmipkit-protocol --test cryptographic_parameters_typed_setters every_table_59_builder_setter_and_typed_reader_roundtrips` passed (1 test); `cargo fmt --all --check` and `cargo clippy --offline -p kmipkit-client -p kmipkit-protocol --all-targets --all-features -- -D warnings` passed. Windows AWS-LC compiler-probe/NASM warnings were non-fatal.
- **Follow-up Red/Green:** the first coverage follow-up raised local changed-Rust coverage to 881/977 (90.17%). Added `all_seven_kmip30_direct_client_methods_follow_the_configured_version`, which exercises all seven direct methods under the default KMIP 2.1 gate (`NotSent`, zero exchanges) and with explicit KMIP 3.0 options.
- **Refactor:** hosted SonarCloud analysis first reported 4.35% duplication on new code (272 lines; maximum 3%). Refactor commits `1ec8784dd` and `e53df263f` centralize KMIP 3.0 response parsing and KEM request builders without excluding code from analysis. The latest SonarCloud analysis for `e53df263f` passes at **2.88% duplication (179 lines)**.
- **Current local verification:** `cargo llvm-cov --offline --workspace --all-features --json --output-path coverage-raw.json` passed the full workspace test suite. Recomputing changed Rust lines against the exact #142 base `b6cc0d28` and current feature head `e53df263f` gives **853/927 (92.02%)**, above the 90% gate; the 74 uncovered executable lines are `crates/kmipkit-client/src/execute.rs:2673,3622,8371`; `crates/kmipkit-protocol/src/create.rs:90`; `crates/kmipkit-protocol/src/cryptographic_parameters.rs:179,385,390,399,405,412,470,477,484,502-505`; `crates/kmipkit-protocol/src/kem.rs:45-46,158,236,251-255,384,399,428,432,437,441,444,447,452,464,468,484,489-490`; and `crates/kmipkit-protocol/src/kmip30_operations.rs:72-74,81,85-87,98,102,185-188,190-191,200-202,227,229-231,237,239-241,285,406,485,492,520,579,598,613`.
- **Verification commands:** `cargo test --offline -p kmipkit-protocol --all-features` passed (1,341 library tests and all protocol integration tests); `cargo fmt --all --check` and `cargo clippy --offline -p kmipkit-protocol --all-targets --all-features -- -D warnings` passed. Windows AWS-LC compiler-probe/NASM and linker warnings were non-fatal. The current-head hosted CI run is still running; keep T050 open until all required checks finish successfully.
