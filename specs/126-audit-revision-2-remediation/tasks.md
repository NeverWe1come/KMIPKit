# Tasks: KMIPKIT-0126 Remediation of Second Independent Audit Findings

- [x] T001: Author specification, plan, tasks, and traceability for KMIPKIT-0126.
- [x] T002: (Red/Green/Refactor) Fix `v2_1_tags` in `kmip-versioned-tags.json` and `v21_tags` in `generate_attribute_policy.py`; regenerate `attribute_policy.rs` and verify unit tests.
- [x] T003: (Red/Green/Refactor) Add `Split Key Polynomial` to `kmip-versioned-tags.json` & `kmip-versioned-values.json`; make `generate_wire_names.py` fail closed for enums without families; regenerate wire names and named values.
- [x] T004: (Red/Green/Refactor) Add verified OASIS KMIP 3.0 enumeration deltas to `kmip-versioned-values.json` and regenerate catalog artifacts.
- [x] T005: (Red/Green/Refactor) Pass `versioned_tags` in `generate_attribute_types.py` to emit all 99 typed attributes; regenerate Rust/C/Java/Python files; update parity tests and assertions from 62 to 99.
- [x] T006: (Red/Green/Refactor) Correct hex tags and operation names in `VersionGate::validate_attribute_mutation`, `kmip20_tests.rs`, and `specs/070-kmip-2-0-interoperability/spec.md`.
- [x] T007: (Red/Green/Refactor) Implement typed batch response readers for the 7 KMIP 3.0 operations across Rust, C, Java, and Python; update `docs/architecture/public-api.md` and `public-api.json`.
- [x] T008: (Red/Green/Refactor) Update `interop/java-kmip-validator/library-pin.json` and unblock KMIP 2.0 live routes in `Main.java` and tests.
- [x] T009: (Red/Green/Refactor) Audit Rev 3: Implement batch response readers for 7 KMIP 3.0 operations in C ABI, JNI/Java, Python, and update `public-api.md`.
- [x] T010: (Red/Green/Refactor) Audit Rev 3: Update `generate_wire_names.py` to traverse `v3_0_promoted_attributes` and validate `bitmask_family` for Integer encoding.
- [x] T011: (Red/Green/Refactor) Audit Rev 3: Add `Password` (7) and `Certificate` (8) to `CredentialType` across Rust, C, Java, and Python.
- [x] T012: (Red/Green/Refactor) Audit Rev 3: Fix boolean condition and normative `source_always_required` in `generate_attribute_policy.py` and `kmip-versioned-tags.json`; expose versions in typed attributes.
- [x] T013: (Red/Green/Refactor) Audit Rev 3: Reclassify async operations in `JavaRouteInventory` and fix spec 070 §2 operation names.
- [x] T014: Run full workspace checks (format, clippy, unit tests, bindings tests, interop validator, catalog validation).
