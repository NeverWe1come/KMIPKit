# Implementation Plan: OASIS KMIP 2.0 Interoperability and Conformance (KMIPKIT-0070)

## 1. Architecture Approach

Building on `KMIPKIT-0069`:
- `VersionGate` already enforces operation spans: operations `0x36..=0x3B` are gated to `[V2_1, ...]`.
- In `kmipkit-protocol`:
  - Enhance `VersionGate` with attribute policy checks for KMIP 2.0: `validate_attribute_mutation(version, tag)`.
  - Add KMIP 2.0 validation tests.
- In `kmipkit-test-server`:
  - Add `ProtocolVersion` configuration on the server (e.g., `ServerConfiguration::with_protocol_version`, supporting `V2_0` or auto-negotiating).
  - When client sends a request with `ProtocolVersion::V2_0`, the test server validates the request under 2.0 rules and responds with `ProtocolVersion(2, 0)`.
  - Ensure `v1_compat.rs` remains untouched.
- In `kmipkit-client` / integration tests:
  - Add end-to-end tests configuring client with `ProtocolVersion::V2_0` executing queries, create, register, get, destroy, locate against `kmipkit-test-server`.
  - Add interoperability test suite with fixture scenarios for Cosmian KMS, Thales CipherTrust, and Utimaco ESKM under KMIP 2.0.

## 2. Red, Green, Refactor TDD Steps

1. **Commit 1 (Docs & Governance)**: Pin KMIP 2.0 OASIS documents, spec, plan, tasks, and traceability.
2. **Commit 2 (Red)**: Add tests for KMIP 2.0 operation gating, attribute policy constraints, and test-server 2.0 responses.
3. **Commit 3 (Green)**: Implement server 2.0 mode, attribute policy validation, and test harness execution.
4. **Commit 4 (Refactor)**: Code cleanup, clippy, formatting, and documentation.
