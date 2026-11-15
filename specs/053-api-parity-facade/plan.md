# Implementation Plan: KMIPKIT-0053 Four-Language API Parity and Final Facade

**Branch**: feature/KMIPKIT-0053-parity-facade | **Date**: 2026-11-06 | **Spec**: spec.md

## Summary
Complete the C, Java, and Python adapters assigned to KMIPKIT-0047 through KMIPKIT-0052, using Rust as the only implementation of protocol/model/validation behavior. Reconcile all 750 candidate declarations and four capability records, preserve the one-crate Rust facade, and provide runnable English/Spanish examples.

## Technical Context
- Rust Edition 2024, MSRV 1.94; C fixed-width opaque-handle ABI; Java JNI; Python CFFI.
- No new protocol or cryptographic implementation; use existing Rust models and KMIPKit FFI.
- Verify with Cargo tests/Clippy, compiled C consumers, Gradle/JNI tests, pytest, manifest generation, and exact export audits.
- Existing supported Linux/Windows/macOS targets; report actual local/hosted evidence independently.
- Limits, redaction, zeroization, TLS, ABI, compatibility, and single-core requirements remain unchanged.
- Scale: 62 direct attributes, 9 object types/key structures, 18 Table 59 parameters, one-message Authentication, 56 operations, and 750 blocked public routes.

## Constitution Check
Specification and traceability: PASS. TDD: required before each implementation group. One Rust core: PASS; all protocol/validation work calls Rust. Security: PASS only when handle ownership, redaction, zeroization, and generic fallback tests pass. Human governance: work remains a feature branch and draft PR; no merge/release. Coverage and hosted CI are temporarily deferred under direct maintainer instruction, not waived.

## Design and Execution
1. Record exact package/capability/operation gaps and executable baselines.
2. Implement macro-lots sequentially: A) typed attributes and managed objects; B) cryptographic parameters and message Authentication; C) validation, all-operation/response parity, packages, docs, and final manifest.
3. Each lot uses separate Red/Green/Refactor commits. Focused tests run per increment; run the pinned manifest generator once at each lot boundary.
4. Foreign wrappers call Rust; they do not duplicate schema/validation logic. Unknown/future/vendor data stays reachable through generic TTLV.
5. C uses fixed-width values and opaque handles with panic containment; Java uses AutoCloseable; Python uses close/context managers. KMIPKit-owned copies zeroize at last release.
6. Final checks run local full tests, generator/export audits, package examples, security diff review, and traceability. Coverage/hosted checks remain visible as outstanding.

## Normative and API Decisions
Use exact source tables and behavior in 0047-0052. Cryptographic Parameters follow KMIP v2.1 §4.16 Table 59; Authentication follows §8.2 Table 395 and is once per Request Message. Validation uses KMIPKIT-0051's shared Rust validator and never guesses an unknown schema. Response absence/order/vendor/ownership follow KMIPKIT-0052. No client-global credentials, new algorithms, server operations, or low-level Transport exchange through the public facade.

## Project Structure
- crates/kmipkit-ffi/src/extension_registry/: C ABI and shared handle operations.
- crates/kmipkit-ffi/tests/ and bindings/c/tests/: ABI and compiled C consumers.
- bindings/java/native/ and bindings/java/src/: JNI and public Java wrappers.
- bindings/python/src/kmipkit/ and bindings/python/tests/: CFFI and public Python wrappers.
- specification/api/public-api.json and tools/api_manifest/: reviewed input, reconciliation, generators and audits.
- docs/user-guide/en/ and docs/user-guide/es/: runnable bilingual usage.
- specs/053-api-parity-facade/: spec, contracts, evidence, traceability and tasks.

## Complexity Tracking
No architecture boundary changes. The size comes from equivalent four-language public APIs; deterministic generators and macro-lots reduce repeated work without relaxing verification.
