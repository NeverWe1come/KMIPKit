# KMIPKIT-0043 T049 Get Usage Allocation Implementation Plan

> **For agentic workers:** This plan is being executed inline in the existing T049 worktree. Keep the Red, Green, and Refactor evidence in distinct signed-off commits.

**Goal:** Expose the approved KMIP 2.1 Get Usage Allocation request and response across Rust, C, Java, and Python with matching result, delivery, ownership, and redaction behavior.

**Architecture:** Reuse the canonical `GetUsageAllocationRequest` and `GetUsageAllocationResponse` in the Rust protocol/client crates. Add typed constructors and response accessors to the existing T076 C ABI, then map those declarations through the handwritten Java and Python facades. Use the pinned public API manifest generator for all generated surfaces.

**Tech Stack:** Rust Edition 2024, C ABI, Java 17/JNI, Python 3.12/CFFI, the Python standard-library manifest generator, Rust/C/Java/Python test suites.

**Spec:** `specs/043-public-api-bindings/tasks.md` T049; approved operation contract `specs/025-policy-controls/spec.md`; OASIS KMIP Specification v2.1 §6.1.23 Tables 232–234 and §7.40 Table 392.

## Global Constraints

- Keep the change within KMIP 2.1, TTLV, client-initiated 1.0 operations, and the existing Rust/C/Java/Python parity boundary.
- Preserve the exact signed 64-bit Usage Limits Count, optional Unique Identifier, and server-resolved response identifier.
- Preserve Success, Pending, Failure, unknown values, correlation bytes, and request-delivery state without automatic Poll or retry.
- Do not implement the local Usage Allocation guard (T078), or modify Encrypt/Decrypt or other operation mappings.
- Do not modify generated files by hand or the immutable OASIS upstream copies.
- Keep caller data ownership explicit and sensitive diagnostics redacted.

---

### Task 1: Prove the mapping gap (Red)

**Files:**
- Create focused operation-mapping tests for manifest readiness, C ABI construction/response access, Java facade behavior, and Python facade behavior.
- Modify `bindings/c/tests/client_abi_consumer.c` to compile and call the typed C operation surface.
- Test the existing fake-transport operation contract from `crates/kmipkit-client/tests/unit/get_usage_allocation_execution_tests.rs` unchanged.

**Interfaces:**
- Request constructor: C receives optional `kmipkit_unique_identifier_v1_t` plus `int64_t usage_limits_count`.
- Response accessor: C returns a present flag and the resolved identifier only for a successful typed result.
- Java uses `ClientBatchItem.getUsageAllocation(UniqueIdentifier, long)` and an owned typed response value.
- Python uses `client_batch_item_get_usage_allocation(UniqueIdentifier | None, int)` and a typed response value.

- [x] Add the assertions before production code and run focused Rust, C, Java, Python, and manifest tests.
- [x] Confirm failures identify the missing mapping declarations or adapter behavior.
- [x] Commit only the failing tests as the signed-off Red change.

**Red evidence:** `cargo test -p kmipkit-ffi get_usage_allocation --lib` failed to compile at the absent request constructor and response getter. The C consumer compile with MSVC `/W4 /WX` failed with C4013 for both missing symbols. `bindings/java/gradlew.bat test --no-daemon` failed test compilation for the absent builder and typed response mapping. Python 3.12 `pytest` ran five new cases; all failed on the missing builder/response/BatchItemResult mapping. `python -B -m unittest tools.api_manifest.tests.test_get_usage_allocation_parity_manifest -v` failed because the manifest still marks the operation blocked and generated parity omits it. `cargo fmt --all` and `git diff --check` pass.

### Task 2: Add the typed mapping (Green)

**Files:**
- Modify `crates/kmipkit-ffi/src/extension_registry.rs` and `crates/kmipkit-ffi/src/extension_registry/client_abi.rs` for typed request construction and success-only response access.
- Modify `bindings/java/src/main/java/org/kmipkit/extensions/ClientBatchItem.java`, `ClientBatchItemResult.java`, `bindings/java/src/main/java/org/kmipkit/NativeExtensionRegistry.java`, and `bindings/java/native/kmipkit_jni.cpp` for the JNI mapping and owned result.
- Modify `bindings/python/src/kmipkit/_client_ffi.py`, `extensions.py`, and `client.py` for the CFFI mapping and owned result.
- Update `specification/api/public-api.json` with approved declarations and T049 parity mappings; run `python tools/api_manifest/generate.py --write` for generated outputs.

**Interfaces:**
- Reuse the T076 batch options and common result metadata for Pending, Failure, correlation, and delivery evidence.
- Do not add operation-specific transport behavior, polling, retries, or local accounting.

- [x] Implement only what the Red tests require.
- [x] Run the focused ABI, C consumer, JNI/Java, CFFI/Python, and manifest checks.
- [x] Commit implementation and generated output as the signed-off Green change (`1a3bfdc9`).

### Task 3: Reconcile documentation and evidence (Refactor)

**Files:**
- Update `specs/043-public-api-bindings/tasks.md`, `implementation-evidence.md`, `traceability.md`, operation dependency documentation, and the public API documentation in English and Spanish.
- Refine tests and adapters without changing the approved behavior.

- [x] Run all focused checks again, then the required workspace formatting, lint, tests, coverage, and generated-output checks available in this environment; results are in `specs/043-public-api-bindings/implementation-evidence.md`.
- [x] Verify T049 is complete, the operation manifest row is verified, and no T078 or unrelated-operation scope changed.
- [x] Commit documentation, traceability, and cleanup as the signed-off Refactor change.
