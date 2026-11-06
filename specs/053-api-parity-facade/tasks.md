# Tasks: KMIPKIT-0053 Four-Language API Parity and Final Facade

**Input**: spec.md, plan.md, gap-matrix.md, contracts, traceability.md.
**Prerequisite**: Stacked on KMIPKIT-0052 / draft PR #123. Scope and base are approved in approval-record.md.

## Phase 1: Baseline and contracts
- [x] T001 Record baseline commit, branch, predecessor PR #123, platform, and approval in specs/053-api-parity-facade/implementation-evidence.md.
- [x] T002 Add manifest inventory tests for the 750 candidates, four capabilities, and unowned routes in tools/api_manifest/tests/test_reconcile_0053_api_parity.py.
- [x] T003 Add common ABI ownership, null-output, stale-handle, and redaction tests in crates/kmipkit-ffi/tests/support/api_parity_0053_tests.rs.
- [ ] T004 Add clean external package consumers for Rust, C, Java, and Python in tests/consumers/0053-parity/.
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
- [ ] T046 [US5] Add only evidenced missing C operation/option/result routes in crates/kmipkit-ffi/src/extension_registry/client_abi.rs.
- [ ] T047 [US5] Add only evidenced missing Java operation/options/response/async accessors in bindings/java/src/main/java/org/kmipkit/ and bindings/java/native/kmipkit_jni.cpp.
- [ ] T048 [US5] Add only evidenced missing Python operation/options/response/async accessors in bindings/python/src/kmipkit/.
- [ ] T049 [US5] Verify all 56 operations, delivery/async behavior, KMIPKIT-0052 raw semantics, and nested vendor TTLV in `crates/kmipkit-ffi/tests/support/response_parity_0053_tests.rs`, `bindings/java/src/test/`, and `bindings/python/tests/`.
- [ ] T050 [US5] Refactor repeated operation fixtures from the reviewed operation catalog in tools/api_manifest/tests/ and rerun all four-language operation tests.

## Phase 7: US6 package docs and closure
- [ ] T051 [US6] Add runnable external consumers and quickstart examples in tests/consumers/0053-parity/ and specs/053-api-parity-facade/quickstart.md.
- [ ] T052 [US6] Update English typed binding, ownership, validation, authentication, and response guidance in docs/user-guide/en/.
- [ ] T053 [US6] Update Spanish equivalents and external-runtime copy limits in docs/user-guide/es/.
- [ ] T054 Complete exact four-language mappings or justified per-route exclusions for the 750 candidates in tools/api_manifest/reconcile_0053_api_parity.py and specification/api/public-api.json.
- [ ] T055 Run the pinned generator and assert all capability/declaration states in tools/api_manifest/generate.py and tools/api_manifest/tests/test_reconcile_0053_api_parity.py.
- [ ] T056 Update specs/053-api-parity-facade/gap-matrix.md and traceability.md only where executable evidence verifies closure.
- [ ] T057 Run format, Clippy, full local tests, C/JNI/Python consumers, generator and export audits; record coverage/remote status in specs/053-api-parity-facade/implementation-evidence.md.
- [ ] T058 Review OASIS sources, all 750 declarations, security effects and QA acceptance; complete specs/053-api-parity-facade/checklists/normative-requirements.md.
- [ ] T059 Confirm distinct Red/Green/Refactor commits, push branch, open draft PR against feature/KMIPKIT-0052-typed-responses, and record URL in specs/053-api-parity-facade/implementation-evidence.md.

## Converged work from implementation review
- [x] T060 [US1] Add named typed constructors/readers for all applicable KMIPKIT-0048 attributes and typed `AttributeSet` insert/read/iteration across C, Java, and Python, retaining generic Item access and repeated values.
- [x] T061 [US1] Add behavior tests that round-trip each attribute's actual value, not only its tag and TTLV type, through the typed foreign APIs.
- [x] T062 [US2] Add typed constructors/readers for the nine managed-object fields and nested KeyBlock/KeyValue/KeyMaterial/wrapping structures across C, Java, and Python, retaining unknown TTLV children.
- [x] T063 [US2] Add behavior tests for object/key field values, key format/material variants, wrapping metadata, ownership, and cleanup guarantees across C, Java, and Python.
- [ ] T064 [US5] Expose normal Query's typed fields in C, Java, and Python with raw access retained for unknown/vendor fields and exact order/repetition preservation.
- [ ] T065 [US5] Replace route-only checks with assertions of encoded operation identity for every synchronous factory and concrete typed response values for each operation family.

## Dependencies
T001–T005 precede the user stories. US1 and US2 stabilize typed value/secret patterns; US3 follows; US4 reuses the shared Rust validator; US5 consumes the prior request/response bindings; US6 closes packages, docs, manifest and PR evidence. Keep each lot sequential on this branch.

## Strategy
Tests first and expected Red failures are recorded before implementation. Each lot receives separate Green and Refactor commits; run focused checks after every increment and the full local suite at closure. Coverage and hosted CI remain visible release gates.
