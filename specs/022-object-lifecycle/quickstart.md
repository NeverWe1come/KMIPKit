# Validation Quickstart: KMIPKIT-0036

This guide records the implementation validation path for Certify, Check, Obtain Lease, Re-certify, Re-key, Re-key Key Pair, Re-Provision, Register, and Revoke. It does not claim formal OASIS conformance, profile support, interoperability, or certification by itself.

## Prerequisites

- Start from the active `release/1.0.0` branch in the dedicated implementation worktree.
- Confirm the shared typed messages, batches, client execution, objects/attributes, common results, and secret storage remain available.
- Verify the pinned OASIS checksums and keep upstream files unchanged.
- For language parity, wait for KMIPKIT-0043's canonical manifest/mappings. Rust protocol and client work can proceed independently.

## Focused validation

Run each protocol test target and its fake-transport client target. Cover operation code, one exchange, batch/result association, failure delivery state, Pending passthrough, and no retry. Every test identifies the exact §6.1.x clause/table or requirement ID in [traceability.md](traceability.md).

Use pinned Test Case fixtures only when available. Re-key maps to KMIPKIT-TEST-CN01-2-76 through -87; the approved spec lists no direct Test Case IDs for the other eight operations. Label table-derived vectors accordingly.

## Repository gates

From the repository root, run focused cargo targets, then the repository checks:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features
python tools/normative_catalog/validate.py
python tools/normative_catalog/report.py --check
```

Run pinned generation-drift, security/dependency, and supported-platform CI checks per active repository automation. Do not claim a check passed without actual output.

## Release evidence

Record separate Red, Green, and Refactor commit IDs per operation in the draft PR. Include focused/full-suite commands, coverage and traceability results, generator status, binding parity state, unavailable fixture notes, risks, and actual test outputs. Update this guide with results only during implementation.

### Check (US2) maintainer clarification

- The maintainer approved applying §6.1.7 to failed Check responses: preserve and type each returned `Usage Limits Count`, `Cryptographic Usage Mask`, and `Lease Time`; a failed response still does not require `Unique Identifier`.
- Follow-up protocol and fake-client tests now exercise all three policy fields together on `Operation Failed`. `cargo test -p kmipkit-protocol policy_denial_response_returns_the_policy_value_without_a_success_identifier` and `cargo test -p kmipkit-client check_policy_denial_exposes_the_returned_mask_without_an_identifier` passed; focused Clippy, format, and diff checks passed.
- A failed Check response never exposes a returned `Unique Identifier` from the typed accessor, even if a nonconforming server includes one; the generic response TTLV still retains that field. §6.1.7 and Table 184 support the server-side omission. The regression test was observed red, then green, and the parser refactor was verified with all nine Check protocol tests (`f35e112c`, `1dc3b71f`, `6fc28305`).

### Re-Provision response interpretation (KMIPKIT-DISC-051)

- The maintainer approved following Table 314: the client permits `Unique Identifier` to be absent for CSR, certificate, and parameterless Re-Provision responses. If present, the identifier is preserved and typed. The §6.1.48 prose `SHALL` remains a server-side obligation. The decision is recorded as KMIPKIT-DEC-014 in the specification and normative catalog; pinned OASIS source files remain unchanged.

### Catalog and traceability closure (T035–T036)

- `specs/022-object-lifecycle/traceability.md` maps all 44 normative ledger IDs and each of the 53 Request/Response payload rows to canonical catalog elements, implementation paths, and exact executable Rust test symbols. It also maps the nine operation error tables to shared Result Status/Reason elements and tests.
- `python -B -m unittest tools.normative_catalog.tests.test_object_lifecycle_traceability -v` passes (2 tests). The audit checks the 44 requirement assignments, nine operation elements, all 53 payload rows, catalog element IDs, and every referenced test function.
- Full catalog suite: `python -B -m unittest discover -s tools/normative_catalog/tests -p 'test_*.py'` passes (209 tests, 7 platform skips).
- `python -B tools/normative_catalog/validate.py` passes (4 sources, 1,411 clauses, 4,044 records).
- Regenerated with `python tools/normative_catalog/report.py --repo-root . --write`; `python tools/normative_catalog/report.py --repo-root . --check` passes. Generated report changes are tool-produced.
- Coverage labels distinguish `verified` client behavior from `scoped_verified` obligations that rely on caller sequencing, server policy/state, or material use outside this library. No OASIS profile or official Test Case conformance claim is made.

### Re-key Key Pair (US6)

- RED protocol: `cargo test -p kmipkit-protocol rekey_key_pair_operation_tests --no-fail-fast` failed as expected because `RekeyKeyPairRequest`, `RekeyKeyPairResponse`, and `RekeyKeyPairError` were not yet exported.
- RED client: `cargo test -p kmipkit-client rekey_key_pair_execution_tests --no-fail-fast` failed as expected because the request type, `ClientRequest::RekeyKeyPair`, `ClientOperation::RekeyKeyPair`, and `Client::rekey_key_pair` were not yet implemented.
- GREEN focused: `cargo test -p kmipkit-protocol rekey_key_pair_operation_tests --no-fail-fast` and `cargo test -p kmipkit-client rekey_key_pair_execution_tests --no-fail-fast` both passed, six tests each.
- GREEN package regression: `cargo test -p kmipkit-protocol -p kmipkit-client` passed.
- Guide examples: `python scripts/test_user_guide_examples.py` compiled all 14 marked Rust examples.
- REFACTOR verification: both focused operation tests passed (seven protocol tests and six client tests); `cargo test -p kmipkit-protocol -p kmipkit-client`, `cargo clippy -p kmipkit-protocol -p kmipkit-client --all-targets --all-features -- -D warnings`, `cargo fmt --all --check`, and `git diff --check` passed.

### Re-Provision (US7)

- RED commit: `07e69112` (`test(022): add Re-Provision RED coverage`). `cargo test -p kmipkit-protocol reprovision_operation_tests --no-fail-fast` and the matching client command both failed because the Re-Provision protocol module did not yet exist.
- GREEN commit: `cd6456a1` (`feat(022): implement Re-Provision operation`). Focused protocol tests passed (7 tests) and fake-transport client tests passed (6 tests), including all three request forms, mutually exclusive object forms, optional/malformed result identifier, server failure, Pending correlation preservation, partial-write delivery state, and no retry.
- REFACTOR verifies the response-view projection through one constructor, retains the typed model and client dispatch, and updates the English lifecycle guide and public API architecture documentation.
- No direct OASIS Test Case is listed for Re-Provision; tests are derived from §6.1.48 Tables 313–315. `KMIPKIT-DISC-040` is applied; upstream source files remain unchanged.

### Register (US8)

- RED commit: `ec1c31f1` (`test(022): add Register RED coverage`). The protocol and client Register test targets failed as expected before the model and client operation existed.
- GREEN commit: `ff2f1581` (`feat(022): implement Register operation`). Added Table 291 ordered request encoding, Table 293 local required-attribute checks, typed Table 292 Unique Identifier response, failure/Pending passthrough, and one-exchange client methods.
- REFACTOR commit: `a6385456` (`refactor(022): finalize Register contract tests`). Added the ID Placeholder producer-to-Check batch case and verified a Certificate needs Certificate Length without a cryptographic Usage Mask.
- Verification passed: `cargo test -p kmipkit-protocol -p kmipkit-client` (package regression suite), focused Register protocol/client tests (11 and 5 tests), `cargo fmt --all --check`, `cargo clippy -p kmipkit-protocol -p kmipkit-client --all-targets --all-features -- -D warnings`, and `git diff --check`.
- No direct OASIS Test Case is listed for Register; tests are derived from §6.1.43 Tables 291–294. The Red/Green/Refactor commits use the maintainer-approved DCO sign-off; pinned upstream sources remain unchanged.

### Register Certificate signature-algorithm omission evidence

- Source basis: the pinned OASIS KMIP 2.1 Specification §2.1 defines a Certificate as a DER-encoded X.509 public-key certificate; §4.22 describes the Digital Signature Algorithm attribute for digitally signed objects; §6.1.43 Table 293 permits omission only when the information is encapsulated in the Certificate object; and §11.16 Table 447 maps SHA-256 with RSA Encryption to `0x00000005`.
- Fixture provenance: `crates/kmipkit-client/tests/fixtures/tc-pkcs12-step2-certificate.der` is a byte-for-byte extraction of the 557-byte Certificate Value in RequestMessage step 2 of the pinned `specification/oasis/kmip-2.1/fixtures/TC-PKCS12-1-21.xml`. The certificate's X.509 `signatureAlgorithm` OID is `1.2.840.113549.1.1.11` (SHA256withRSA). The upstream XML and pinned OASIS source copies were not modified.
- `register_client_omits_signature_algorithm_when_x509_certificate_encodes_it` verifies the captured Batch Item encodes Operation Register, Object Type Certificate, X.509 Certificate Type and length 557, omits the Digital Signature Algorithm attribute, and preserves the exact DER bytes. A small test-only DER reader checks the outer Certificate SEQUENCE and its second field's AlgorithmIdentifier OID; no parsing dependency was added.
- RED commit `dd75a46f`: the focused Rust client test passed (1 test); the named-evidence contract failed because the requirement row did not yet reference that test. GREEN commit `30df0690`: the catalog row adds the named reference while retaining `feature_spec=KMIPKIT-0036`, `status=verified`, and an empty official `test_case_ids` list. The named-evidence contract then passed; profile metadata and the coverage report were regenerated and their checks passed.
- The fake transport returns an Operation Failed response only to complete the client exchange. Assertions cover the request bytes and certificate input; they do not claim server acceptance, official Test Case execution, profile conformance, or certification. The generated coverage report continues to show no typed rule mapping for this requirement.
- Final verification after Refactor: the focused Rust test passed (1 test), named-evidence contract passed (1 test), object-lifecycle traceability tests passed (3 tests), catalog validation passed (4 sources / 1411 clauses / 4111 records), profile metadata check passed, coverage report check passed, `cargo fmt --all --check` passed, and `git diff --check` passed. `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings` remains blocked by the pre-existing redundant-closure lint in `crates/kmipkit-client/tests/unit/execute_lifecycle_tests.rs:192`; that unrelated test was not changed.
- Focused commands: `cargo test -p kmipkit-client --lib register_client_omits_signature_algorithm_when_x509_certificate_encodes_it -- --nocapture`; `python -m unittest tools.normative_catalog.tests.test_validate.CatalogValidationTests.test_register_certificate_dsa_omission_evidence_names_client_test_under_existing_owner -v`; `python -m unittest tools.normative_catalog.tests.test_object_lifecycle_traceability -v`; `python tools/normative_catalog/validate.py --repo-root .`; `python tools/normative_catalog/generate_profiles.py --check --repo-root .`; `python tools/normative_catalog/report.py --check --repo-root .`; `cargo fmt --all --check`; and `git diff --check`.

### Revoke (US9)

- RED commits: `997e030f` (`test(022): add Revoke RED coverage`), `2d045c45` (require §4.47/Table 123 nested Structure), and `b0323416` (require the §11.36/Table 470 Revoke operation code `0x13`). The latter tests failed against the prior `0x11` value.
- GREEN commit: `9884ded4` (`feat(022): implement Revoke operation`) adds the raw-preserving reason code, nested Table 123 request encoding, optional compromise date validation, typed success identifier, shared failure/Pending results, and the one-exchange client method. Follow-up fix `bb4f5387` aligns Revoke with Table 470 and corrects the pre-existing Archive code to `0x15`; both had collided at `0x13`. The request Debug output redacts the identifier and audit message.
- REFACTOR commit: `f7831af5` (`refactor(022): reuse shared Revoke payload helpers`) removes duplicate TTLV-tag, item, error, and identifier helpers while preserving behavior.
- Verification passed: focused Revoke protocol and client tests (10 and 6 tests), focused Archive protocol/client tests (12 and 4 tests), `cargo test -p kmipkit-protocol -p kmipkit-client` (420 client unit tests and all protocol tests passed), `cargo clippy -p kmipkit-protocol -p kmipkit-client --all-targets --all-features -- -D warnings`, `cargo fmt --all --check`, and `git diff --check`.
- No direct OASIS Test Case is listed for Revoke; these table-derived tests cover §6.1.44 Tables 295–297, §4.47/Table 123, and §11.48/Table 481. Pinned upstream sources remain unchanged.


### Final local verification (2026-10-11)

- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
- `cargo test --workspace --all-features --quiet` passed on the installed stable toolchain, including workspace unit, integration, and UI suites.
- `cargo +1.94 test --workspace --all-features --quiet` passed on this Windows host, including loopback transport tests.
- The earlier direct `cargo llvm-cov --workspace --all-features --summary-only` host summary reported 21,955/24,927 lines (88.08%). The repository release gate uses normalized CI reports from all supported platforms and the selected language scopes; see the authoritative aggregate result below.
- `python scripts/coverage_gate.py preflight --workspace . --status-out target/022-coverage-status.json` passed its source inventory check (99 Rust source files; 155 eligible files).
- `pwsh -File ./scripts/Test-DependencyPolicy.ps1` passed: the pinned cargo-deny policy, advisory database, exceptions, and lockfile hashes validate.
- `python -B -m unittest discover -s tools/normative_catalog/tests -p 'test_*.py'` passed (209 tests, 7 platform skips); `python -B tools/normative_catalog/validate.py` and `python -B tools/normative_catalog/report.py --check` passed.
- `python -B tools/api_manifest/generate.py --check` verified six generated files; `python -B tools/extension_fixtures/generate.py --check` passed. `tools.api_manifest.tests.test_extension_fixtures` passed 16 tests with 1 platform skip on Python 3.13.14 using the pinned `tools/api_manifest/requirements-test.txt` dependency.
- The English and Spanish lifecycle guide command compiled all 18 marked examples. Focused Check denial policy tests pass with the approved §6.1.7 behavior.
- Requirement traceability tests passed (2 tests). Feature traceability passed 15 tests, with 1 skipped because Windows denied symlink creation (`WinError 1314`). The threat-model and public API documentation corrections list all nine Rust operations and explicitly defer C/Java/Python operation parity to KMIPKIT-0043.
- `git diff --check` passed. The local host lacks CMake/CTest/Ninja and a native C compiler, so C-consumer, ASan, and libFuzzer checks require CI. Linux/macOS checks also require CI.

### Required CI aggregate (PR #91, run 38138339229)

- The run tested head `39c034067aab25c4b97bbbe6b0bb8e855b639538` against base `feeadde4acca1f222486e0d453f9e3e1773fb16c`. Every applicable check completed successfully, including Linux/macOS/Windows Rust core, coverage and C-consumer jobs; Windows C consumer; C FFI sanitizer; Java/JNI and Python bindings/coverage; dependency policy; documentation traceability; and extension-schema fuzz smoke.
- Replaying `python scripts/coverage_gate.py aggregate --workspace . --report-dir <downloaded coverage-* artifacts> --base feeadde4acca1f222486e0d453f9e3e1773fb16c --merge 39c034067aab25c4b97bbbe6b0bb8e855b639538 --scopes-json '["rust"]'` against all seven artifacts from that run passed: workspace 22,559/23,387 (96.46%, 90% minimum); `kmipkit-ttlv` 735/741 (99.19%, 95% minimum); `kmipkit-protocol` 10,143/10,474 (96.84%, 95% minimum); `kmipkit-transport` 3,713/3,902 (95.16%, 85% minimum); `kmipkit-ffi` 2,035/2,137 (95.23%, 85% minimum); changed production code 2,121/2,155 (98.42%, 95% minimum).
- A fresh local coverage attempt in this restricted Windows host stopped before report generation because three loopback DNS fixture tests were denied socket binding with OS error 10013. The supported-platform CI run above passed those gates and is the verification evidence for T037.
- T033 remains blocked on the canonical manifest and operation mappings from KMIPKIT-0043. T037 is complete based on the successful CI aggregate and platform checks. T038's independent QA/security review and draft PR evidence were completed in PR #91, which has since merged. No profile conformance or certification claim is made.
