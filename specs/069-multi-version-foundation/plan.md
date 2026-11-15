# Implementation Plan: Multi-Version Modern KMIP Foundation (KMIPKIT-0069)

## 1. Architectural Strategy

The implementation introduces protocol-version awareness into `KMIPKit` across six functional areas:
1. **Governance & Decision Records**: Amend the Constitution (v1.3.0) and establish ADR-0022.
2. **Protocol Version & Gate**: Add `ProtocolVersion` canonical constants, `VersionSpan`, `VersionGate`, and `VersionedCodec` in `kmipkit-protocol`.
3. **Execution Wire Dynamism**: Parameterize `build_request_header`, `build_async_request_message`, and `execute.rs` to encode and enforce the resolved effective version.
4. **Client API Facade**: Add `.with_protocol_version` and `.force_kmip_version` to `ClientConfiguration`, `RequestOptions`, and `ClientBatch` in `kmipkit-client` and `kmipkit`.
5. **C ABI & Multi-Language Bindings**: Expose version getters and setters in `kmipkit-ffi` and implement Java and Python parity.
6. **Verification & Quality Gates**: Unit, negative, boundary, and regression tests ensuring 100% Zero-Breakage on KMIP 2.1.

## 2. Test-Driven Development (TDD) Strategy

Strict Red $\to$ Green $\to$ Refactor cycle:
- **Red Phase 1**: Failing unit tests for `ProtocolVersion` constants, comparison, `VersionSpan`, and `VersionGate` local validation.
- **Green Phase 1**: Implement protocol types, spans, and gate logic.
- **Red Phase 2**: Failing unit tests in `kmipkit-client` verifying that request headers encode the overridden version and that `VersionGate` blocks disallowed requests pre-send.
- **Green Phase 2**: Implement `ClientConfiguration`, `RequestOptions`, and `ClientBatch` version resolution and wiring.
- **Red Phase 3**: Failing binding and parity tests for C, Java, and Python.
- **Green Phase 3**: Implement FFI functions and language binding adapters.
- **Refactor Phase**: Clippy, rustfmt, documentation, and coverage gate verification.
