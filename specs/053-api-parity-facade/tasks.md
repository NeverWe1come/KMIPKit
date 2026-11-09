# Tasks: KMIPKIT-0053 Four-Language API Parity and Final Facade

**Input**: spec.md, plan.md, gap-matrix.md, contracts, traceability.md.
**Initial prerequisite**: Stacked on KMIPKIT-0052 / draft PR #123 when work began. The current canonical release-synchronized stack is PR #150 based on KMIPKIT-0052 PR #149, which is based on PR #143. See approval-record.md for the superseded #133 history. Scope and initial base approval are recorded there.

## Phase 1: Baseline and contracts
- [x] T001 Record the initial baseline commit, branch, predecessor PR #123, platform, and approval in specs/053-api-parity-facade/implementation-evidence.md; retain #123 as historical and identify the current #149/#150 stack there.
- [x] T002 Add manifest inventory tests for the 750 candidates, four capabilities, and unowned routes in tools/api_manifest/tests/test_reconcile_0053_api_parity.py.
- [x] T003 Add common ABI ownership, null-output, stale-handle, and redaction tests in crates/kmipkit-ffi/tests/support/api_parity_0053_tests.rs.
- [x] T004 Add clean external package consumers for Rust, C, Java, and Python in tests/consumers/0053-parity/.
- [x] T005 Review the language boundary and ownership contract in specs/053-api-parity-facade/contracts/language-api.md.

## Phase 2: US1 typed attributes (62 applicable definitions)
### Red
- [x] T006 [US1] Add C ABI tests for typed construction/read, invalid tag/type and ownership in crates/kmipkit-ffi/tests/support/typed_attribute_0053_tests.rs.
- [x] T007 [US1] Add Java typed owner/view, invalid tag/type, repeat/unknown value, and close tests in bindings/java/src/test/java/org/kmipkit/typed/TypedAttributeParityTest.java.
- [x] T008 [US1] Add Python 62-definition, generic roundtrip, repeat, redaction, and close tests in bindings/python/tests/test_typed_attribute_parity.py.
- [x] T009 [US1] Run the focused red tests and record expected missing-route failures in specs/053-api-parity-facade/implementation-evidence.md.
### Green and Refactor
- [x] T010 [US1] Add safe C ABI typed attribute construction/read/conversion over KMIPKIT-0048 definitions in crates/kmipkit-ffi/src/extension_registry/typed_attributes.rs. Current from-item wrappers only validate the generic TTLV tag/value kind.
- [x] T011 [US1] Add Java owners/views and JNI transfer for typed attributes in bindings/java/src/main/java/org/kmipkit/typed/ and bindings/java/native/kmipkit_jni.cpp, including typed scalar/structure constructors and readers.
- [x] T012 [US1] Add Python owners/readers and CFFI declarations in bindings/python/src/kmipkit/typed_attributes.py and bindings/python/src/kmipkit/_client_ffi.py, including typed scalar/structure constructors and readers.
- [x] T013 [US1] Verify all 62 attribute definitions, source value types, typed roundtrip, generic fallback, repeated values, and errors across four languages; record results in specs/053-api-parity-facade/implementation-evidence.md.
- [x] T014 [US1] Add deterministic mapping/reconciliation tests for every implemented typed attribute route in tools/api_manifest/reconcile_0053_api_parity.py and tools/api_manifest/tests/test_reconcile_0053_api_parity.py.
- [x] T015 [US1] Refactor shared attribute conversion/ownership helpers and rerun the C, Java, and Python typed attribute suites after typed field APIs are complete.

## Phase 3: US2 managed objects and key structures
### Red
- [x] T016 [US2] Add compiled C object/key/material/wrapping roundtrip and release tests in bindings/c/tests/managed_object_parity_consumer.c.
- [x] T017 [US2] Add Java object dispatch/material/extension/redaction/close tests in bindings/java/src/test/java/org/kmipkit/typed/ManagedObjectParityTest.java.
- [x] T018 [US2] Add Python object dispatch/material/extension/redaction/context tests in bindings/python/tests/test_managed_object_parity.py.
- [x] T019 [US2] Run focused red consumers and record failures in specs/053-api-parity-facade/implementation-evidence.md.
### Green and Refactor
- [x] T020 [US2] Add C ABI typed field construction/read APIs for nine objects, KeyBlock, KeyValue, material and wrapping forms in crates/kmipkit-ffi/src/extension_registry/managed_objects.rs; current APIs primarily validate generic Items.
- [x] T021 [US2] Add Java typed field constructors/readers, closeable owners, fallback values and JNI conversion in bindings/java/src/main/java/org/kmipkit/managed/ and bindings/java/native/kmipkit_jni.cpp.
- [x] T022 [US2] Add Python typed field constructors/readers, closeable owners, fallback values and CFFI conversions in bindings/python/src/kmipkit/managed_objects.py and bindings/python/src/kmipkit/_client_ffi.py.
- [x] T023 [US2] Verify typed object/key field values, formats, order, unknown fallback, failure cleanup, and secret cleanup; record results in specs/053-api-parity-facade/implementation-evidence.md.
- [x] T024 [US2] Refactor common managed-value conversion and rerun the C, Java, and Python managed-object suites after field APIs are complete.

## Phase 4: US3 Cryptographic Parameters and Authentication
### Red
- [x] T025 [US3] Add C Table 59/Auth-header tests in crates/kmipkit-ffi/tests/support/parameters_auth_0053_tests.rs and bindings/c/tests/authentication_consumer.c.
- [x] T026 [US3] Add Java parameter/auth tests in bindings/java/src/test/java/org/kmipkit/typed/ParametersAuthenticationParityTest.java.
- [x] T027 [US3] Add Python parameter/auth tests in bindings/python/tests/test_parameters_authentication_parity.py.
- [x] T028 [US3] Run red tests and record expected missing routes in specs/053-api-parity-facade/implementation-evidence.md.
### Green and Refactor
- [x] T029 [US3] Add C ABI typed parameters and batch Authentication via Rust in crates/kmipkit-ffi/src/extension_registry/cryptographic_parameters.rs and client_abi.rs.
- [x] T030 [US3] Add Java parameter models, batch Authentication, and JNI lifecycle in bindings/java/src/main/java/org/kmipkit/ and bindings/java/native/kmipkit_jni.cpp.
- [x] T031 [US3] Add Python parameter models, batch Authentication, and CFFI declarations in bindings/python/src/kmipkit/ and bindings/python/src/kmipkit/_client_ffi.py.
- [x] T032 [US3] Verify all 18 fields, future enums, request-header order, batch scope and no global credentials in the consumer tests; record results in `specs/053-api-parity-facade/implementation-evidence.md`.
- [x] T033 [US3] Refactor generated parameter/auth routes and capability contracts in tools/api_manifest/reconcile_0053_api_parity.py.

## Phase 5: US4 foreign validation
### Red
- [x] T034 [US4] Add C validation issue/report and redaction tests in crates/kmipkit-ffi/tests/support/request_validation_0053_tests.rs.
- [x] T035 [US4] Add Java validation/no-write tests in bindings/java/src/test/java/org/kmipkit/ValidationParityTest.java.
- [x] T036 [US4] Add Python validation/no-write tests in bindings/python/tests/test_validation_parity.py.
- [x] T037 [US4] Run red tests and record expected missing report accessors in specs/053-api-parity-facade/implementation-evidence.md.
### Green and Refactor
- [x] T038 [US4] Expose Rust ValidationReport/Issue through the C ABI in crates/kmipkit-ffi/src/extension_registry/request_validation.rs.
- [x] T039 [US4] Add Java validation iteration and request entry points in bindings/java/src/main/java/org/kmipkit/ and bindings/java/native/kmipkit_jni.cpp.
- [x] T040 [US4] Add Python validation report and request entry points in bindings/python/src/kmipkit/ and bindings/python/src/kmipkit/_client_ffi.py.
- [x] T041 [US4] Verify issue parity, generic schema behavior, NotSent, zero writes, valid wire-byte stability, and secret redaction in `crates/kmipkit-ffi/tests/support/request_validation_0053_tests.rs`, `bindings/java/src/test/java/org/kmipkit/ValidationParityTest.java`, and `bindings/python/tests/test_validation_parity.py`.
- [x] T042 [US4] Refactor shared report mapping and rerun the validation tests in crates/kmipkit-ffi/tests/, bindings/java/src/test/, and bindings/python/tests/.

## Phase 6: US5 operation and response parity (56 operations)
### Red
- [x] T043 [US5] Generate operation/typed-response compile matrices for all 56 entries in tools/api_manifest/tests/test_0053_operation_binding_matrix.py.
- [x] T044 [US5] Add async pending/correlation and raw payload presence tests in crates/kmipkit-ffi/tests/support/response_parity_0053_tests.rs, bindings/java/src/test/, and bindings/python/tests/.
- [x] T045 [US5] Run matrix red tests and record missing routes in specs/053-api-parity-facade/implementation-evidence.md.
### Green and Refactor
- [x] T046 [US5] Audit C operation/option/result routes; the executable 56-operation matrix found no additional C routes to add.
- [x] T047 [US5] Audit Java operation/options/response/async accessors; the executable 56-operation matrix found no additional Java routes to add.
- [x] T048 [US5] Audit Python operation/options/response/async accessors; the executable 56-operation matrix found no additional Python routes to add.
- [x] T049 [US5] Verify all 56 operations, delivery/async behavior, KMIPKIT-0052 raw semantics, and nested vendor TTLV in `crates/kmipkit-ffi/tests/operation_parity_0053.rs`, `crates/kmipkit-ffi/tests/support/async_operation_abi_tests.rs`, `crates/kmipkit-ffi/tests/support/query_ping_abi_tests.rs`, `bindings/java/src/test/`, and `bindings/python/tests/`.
- [x] T050 [US5] Derive operation fixtures from the reviewed normative catalog and rerun the four-language operation matrices.

## Phase 7: US6 package docs and closure
- [x] T051 [US6] Add runnable external consumers and quickstart examples in tests/consumers/0053-parity/ and specs/053-api-parity-facade/quickstart.md.
- [x] T052 [US6] Document English typed binding, ownership, validation, authentication, response, and Query guidance in `docs/user-guide/en/typed-bindings-and-response-data.md`.
- [x] T053 [US6] Document the Spanish equivalents and external-runtime copy limits in `docs/user-guide/es/enlaces-tipados-validacion-respuestas.md`.
- [x] T054 Complete exact four-language mappings or justified per-route exclusions for the 750 candidates in `tools/api_manifest/reconcile_0053_api_parity.py` and `specification/api/public-api.json`.
- [x] T055 Run the pinned generator and assert all capability/declaration states in `tools/api_manifest/generate.py` and `tools/api_manifest/tests/test_reconcile_0053_api_parity.py`.
- [x] T056 Update `specs/053-api-parity-facade/gap-matrix.md` and traceability only where executable evidence verifies closure.
- [x] T057 Run format, Clippy, full local tests, C/JNI/Python consumers, generator and export audits; record coverage/remote status in specs/053-api-parity-facade/implementation-evidence.md.
- [x] T058 Review OASIS sources, all 750 declarations, security effects and QA acceptance; complete specs/053-api-parity-facade/checklists/normative-requirements.md.
- [x] T059 Confirm distinct Red/Green/Refactor commits, push the initial branch, open PR #133 against `feature/KMIPKIT-0052-typed-responses`, and record the URL in specs/053-api-parity-facade/implementation-evidence.md. PR #133 was later closed as superseded; canonical review-ready PR #150 is based on PR #149. The release-alias branch initially shared implementation head `47e6aaee4` with #133 and includes the earlier #149 update through merge commit `8407ed352` plus its final T022 evidence through merge commit `0b2169cd0`.

## Converged work from implementation review
- [x] T060 [US1] Add named typed constructors/readers for all applicable KMIPKIT-0048 attributes and typed `AttributeSet` insert/read/iteration across C, Java, and Python, retaining generic Item access and repeated values.
- [x] T061 [US1] Add behavior tests that round-trip each attribute's actual value, not only its tag and TTLV type, through the typed foreign APIs.
- [x] T062 [US2] Add typed constructors/readers for the nine managed-object fields and nested KeyBlock/KeyValue/KeyMaterial/wrapping structures across C, Java, and Python, retaining unknown TTLV children.
- [x] T063 [US2] Add behavior tests for object/key field values, key format/material variants, wrapping metadata, ownership, and cleanup guarantees across C, Java, and Python.
- [x] T064 [US5] Expose normal Query's typed fields in C, Java, and Python with raw access retained for unknown/vendor fields and exact order/repetition preservation.
- [x] T065 [US5] Replace route-only checks with assertions of encoded operation identity for every synchronous factory and concrete typed response values for each operation family.
- [x] T066 [US2] Correct transparent EC Key Format Type dispatch to the hexadecimal values in KMIP 2.1 §11.25 Table 461; add rejection tests for malformed 0x14/0x15 structures and preserve reserved 0x0E/0x0F as generic values in Rust, C, Java, and Python.

## Converged validation export inventory
- [x] T067 [US4] Add Red tests for the exact 934-candidate validation inventory, its current Rustdoc identities, and parity between Rust schema enum variants and foreign schema-string dispatch.
- [x] T068 [US4] Map the 12 direct validation aliases and record exact exclusions for the other 922 routes with deterministic preparation, approved evidence, and generated-manifest reconciliation.
- [x] T069 [US4] Exercise all request and nested schema identifiers through the Python binding; preserve `SetEndpointRole` as an accepted-but-unsupported descriptor identity and verify `Unsupported`.
- [x] T070 [US6] Refresh T102 declaration, operation, and manifest fingerprints after the latest synchronized stack, update exact totals, and record final audit evidence in the specification artifacts.
- [x] T071 [US4] Add Red tampering regressions for fabricated source commits and substituted Rust exports; pin candidate preparation to the exact audited Git commit and compare every candidate with its Git manifest source.
- [x] T072 [US6] Refresh the binding-export audit documentation from the final C, Java, and Python compiled audit report and verify the documented counts against the checked-in report.
- [x] T073 [US6] Update prior manifest regressions to reflect the synchronized verified state of the three Create operations while retaining their declared route state.
- [x] T074 [US6] Add a bounded manifest-size regression for the 56 MiB limit and the full inventory's indented test serialization; raise the pinned cap only enough to admit this inventory.
- [x] T075 [US6] Preserve per-record `evidencePaths` in isolated generator-test repositories and relocate generated-output evidence into fixture paths without creating checked output destinations.
- [x] T076 [US6] Apply the normative standard-attribute catalog parity check to manifests that claim the ready `typed_object_attributes` capability, while permitting empty partial-development fixtures and rejecting tag rows without that capability.
- [x] T077 [US6] Run the complete manifest suite with its pinned test dependency, final generator/reconciliation and four-language export audits, workspace quality checks, bindings, consumers, and independent review; record exact local and remote release-gate status. Local Windows and Linux x86_64 checks pass, including Windows Java coverage; hosted CI is intentionally not run during the maintainer-reported billing issue. The macOS Rust report and 1.0 interoperability/security release gates remain external.
- [x] T078 [US5] Converge Cancel, Poll, Process, and Query Async in the approved parity contract: add operation-checked typed readers and contract tests in C, Java, and Python, then correct the public API manifest mappings so each binding points to the actual typed reader rather than generic metadata; retain raw TTLV access and verify operation identity and response fields per binding. Follow-up: export Python `managed_objects` from the package root and `__all__`; `test_generated_api_manifest::test_managed_objects_module_is_exported_from_the_package` covers that public import path. The English and Spanish guides now describe T078 as implemented.
- [x] T079 [US5, US6] Extend the four external consumers to execute Ping through `Client.execute` against a loopback-only, TLS 1.3/mTLS test responder built from the existing disposable `login-mtls` fixtures; assert the typed Ping result and decoded generic Response Payload, and clearly exclude this responder from interoperability evidence.
- [x] T080 [US6] Reconcile inherited C ABI routes that are now represented by ready four-language functions: remove only the exact superseded C-only exclusions, add exact ready `c-symbol` declarations so the pinned generator emits both public prototypes, and verify the generated header, compiled ABI, and release manifest contain each symbol exactly once.

- [x] T081 [US6] Correct double-encoded OASIS section and en-dash citations in the affected Rust trait-exclusion evidence, preserving the exact audited declaration paths; regenerate the manifest-derived Java parity fixtures and verify the 92 focused T102 contract tests.
- [ ] T082 [US6] Run the complete script-contract matrix on this corrected head and verify the fresh hosted workflows; resolve any remaining contract or CI failures before marking this PR review-ready.

## Dependencies
T001–T005 precede the user stories. US1 and US2 stabilize typed value/secret patterns; US3 follows; US4 reuses the shared Rust validator; US5 consumes the prior request/response bindings; US6 closes packages, docs, manifest and PR evidence. Keep each lot sequential on this branch.

## Strategy
Tests first and expected Red failures are recorded before implementation. Each lot receives separate Green and Refactor commits; run focused checks after every increment and the full local suite at closure. Coverage and hosted CI remain visible release gates.
