# Implementation Plan: KMIP 2.1 Encrypt and Decrypt Operations

**Branch**: `feature/KMIPKIT-0019-cryptographic-operations` | **Date**: 2026-10-09 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/019-cryptographic-operations/spec.md`

## Summary

Add typed KMIP 2.1 Encrypt and Decrypt request/response models and route them through the existing Rust client execution and TTLV paths. Keep each operation's payload contract distinct, preserve opaque byte strings and unknown values, reuse the current decoder limits, result/delivery reporting, and pending handling, and add operation-specific protocol/fake-transport tests plus bilingual examples. No local cryptographic implementation or runtime dependency is introduced.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94.

**Primary Dependencies**: Existing workspace crates `kmipkit-ttlv`, `kmipkit-protocol`, and `kmipkit-client`; KMIPKIT-0018's Recover operation is a blocking feature dependency for the archived-object sequence; no new runtime dependency.

**Storage**: None.

**Testing**: Cargo unit, integration, compile-fail, documentation, and property tests; source-backed derived vectors from the pinned OASIS sections; deterministic fake transport; full Rust workspace gates.

**Target Platform**: Linux, Windows, and macOS CI. The new code stays in platform-independent Rust crates.

**Project Type**: KMIP client library workspace.

**Performance Goals**: No new performance target. Continue to enforce the configured TTLV limits before allocation: 16 MiB message size, depth 64, and 100,000 elements by default.

**Constraints**: TTLV and KMIP 2.1 only; TLS 1.3/mTLS and existing production transports; one exchange per invocation; no automatic retry or polling; no local algorithm, key, size, mode, padding, IV, or nonce selection; redact and zeroize KMIPKit-owned operation bytes; preserve unknown tags, values, extensions, and result codes.

**Scale/Scope**: Exactly two of the 57 client-to-server KMIP 2.1 operations: Encrypt and Decrypt. Multipart calls remain caller-driven and stateless in KMIPKit.

## Constitution Check

- **Specification and traceability**: This plan covers exactly Encrypt and Decrypt, assigns 24 applicable catalog requirements plus the two operation elements, six shared operation-structure elements, and the Cryptographic Parameters attribute, and maps three OASIS test-case records. Shared batch contracts retain their existing owner and are explicitly inherited.
- **TDD and conformance**: Write operation-specific failing protocol and fake-transport tests first, preserve Red/Green/Refactor commits, and distinguish derived vectors from the three pinned official OASIS cases.
- **One Rust core**: Extend the existing protocol and client crates. Do not add another crate or language-specific implementation.
- **Security**: Use the existing redacted, zeroizing TTLV `Value` and protocol `SecretBytes` ownership contracts. Keep payload bytes out of Debug, Display, errors, and logs. Reuse bounded decoding and transport delivery classification.
- **Human governance**: No implementation starts from this draft. The draft PR remains subject to human review; agents do not approve, merge, or publish.
- **Source integrity**: Read only the checked-in OASIS source copy. Do not modify it or download/scrape it during builds. Regenerate the coverage report using the pinned repository tool.

**Gate status**: The ordinary single-part and multipart forms are bounded. While KMIPKIT-DISC-045 remains open, the typed client returns a local validation error before transmission for a one-request form with both Init and Final true; no OASIS Data-requiredness interpretation is selected. The catalog retains the Table 59 header extraction only as an excluded source-clause audit record because “REQUIRED” is a column heading, not a Cryptographic Parameters row value. All three assigned official fixtures are pinned; every in-scope Encrypt/Decrypt item must pass fixture-derived operation-item tests before implementation review. These item tests are not full official-case passes. Complete workflow execution remains a 1.0 interoperability gate after the other operations in each workflow are implemented and supported.

## Design Decisions and Alternatives

1. **Keep Encrypt and Decrypt request/response types separate.** Their tables differ, particularly Encrypt's optional returned IV and Authenticated Encryption Tag versus Decrypt's Authenticated Encryption Tag request input. A single permissive request type could accept a field in the wrong operation.
2. **Preserve request Data and parameters without narrowing.** Model request Data as Byte String, Enumeration, or Integer under §7.9 using redacted formatting; response Data is optional and uses Byte String when present in Tables 197/215. Keep Cryptographic Parameters ordered and generic, validate IV Length for recognized variable-IV modes and Tag Length for recognized GCM, preserve unknown values, and never infer remote object attributes.
3. **Keep multipart operation caller-driven.** Typed request models carry the table-defined Correlation Value, Init Indicator, Final Indicator, and optional Data. The client does not maintain stream state, move initial-only AAD/Tag, or issue follow-up requests.
4. **Reuse the existing client and result pipeline.** Add Encrypt/Decrypt variants to `ClientRequest`, typed completed responses, and the shared `PendingOutcome` path; do not introduce another transport, response parser, retry loop, or polling task.
5. **Keep server-owned object policy server-side.** In particular, §6.1.17 Usage Limits allocation remains a server action. The client preserves the server's Operation Failed/Permission Denied result and does not simulate allocations.
6. **Do not add a cryptographic dependency or provider abstraction.** KMIPKit sends opaque data to a server and transports returned bytes.

## Project Structure

### Documentation (this feature)

```text
specs/019-cryptographic-operations/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── traceability.md
├── contracts/
│   └── operation-payloads.md
└── checklists/
    ├── requirements.md
    └── protocol.md
```

### Source Code

```text
crates/kmipkit-protocol/src/encrypt.rs
crates/kmipkit-protocol/src/decrypt.rs
crates/kmipkit-protocol/src/operation_data.rs
crates/kmipkit-protocol/src/lib.rs
crates/kmipkit-protocol/tests/unit/encrypt_tests.rs
crates/kmipkit-protocol/tests/unit/decrypt_tests.rs
crates/kmipkit-protocol/tests/unit/operation_data_tests.rs
crates/kmipkit-test-support/src/oasis_xml.rs
crates/kmipkit-test-support/tests/unit/oasis_xml_tests.rs
crates/kmipkit-client/src/execute.rs
crates/kmipkit-client/tests/unit/encrypt_execution_tests.rs
crates/kmipkit-client/tests/unit/decrypt_execution_tests.rs
crates/kmipkit-client/tests/unit/execute_test_support.rs
docs/user-guide/en/client-execution.md
docs/user-guide/es/ejecucion-cliente.md
specification/catalog/kmip-2.1.json
specification/catalog/coverage-report.md  # regenerated by the pinned report tool
```

**Structure Decision**: Follow the existing one-module-per-operation protocol pattern (`create.rs`, `create_key_pair.rs`, and `create_split_key.rs`) and the existing closed `ClientRequest` / `ClientOperation` execution pipeline in `execute.rs`. Keep operation payload parsing adjacent to the protocol model and fake-transport assertions in the client test suite. Do not edit generated Rust tags by hand; no tag-generator change is expected because the normative tags are already in the catalog.

## Phase 0: Research

Resolve field ownership and validation from pinned Tables 196–198 and 214–216, §4.16, §4.58, and §§7.3, 7.4, 7.8, 7.9, 7.14, and 7.17. Confirm existing TTLV byte values redact formatting and zeroize owned storage, determine how the client maps Pending and result failures, and verify the fixture contents, 28 in-scope request-response pairs, and the deterministic test-only substitutions for the symbolic values in all three assigned official test cases. Record each decision and rejected alternative in `research.md` before implementation planning is handed off.

## Phase 1: Design

Document operation entities, the Data/multipart validity matrix, result-shape rules, and validation in data-model.md; exact public request/response field contracts in contracts/operation-payloads.md; and repeatable checks in quickstart.md. Add traceability.md mapping the 24 assigned applicable catalog requirements, two operation elements, six shared operation-structure elements, the Cryptographic Parameters attribute, inherited batch requirements, and three OASIS test-case records to planned source paths and test names. The specification PR maps feature_spec in the canonical catalog and regenerates coverage-report.md; implementation and verification references are completed with the implementation PR.

## Constitution Recheck

- No architectural boundary or workspace dependency changes.
- Each request and response owns or borrows byte material only through redacted/zeroizing types; runtime-owned copies outside KMIPKit remain outside this crate's guarantee.
- Single-part unframed Data is required; initial/final multipart Data may be omitted and a middle part requires Data. While KMIPKIT-DISC-045 remains unresolved, the typed client returns a local validation error before transmission for a one-request Init=true/Final=true form, without selecting Data requiredness.
- Every applicable operation and shared source clause has an explicit owner or inherited dependency; all 24 applicable requirements are traceable, and the Table 59 header extraction is retained only as an excluded source-clause audit record.
- Do not implement while this draft is awaiting review and approval.

## Complexity Tracking

No constitution violations, new dependency, crate, transport, language boundary, or local algorithm implementation is planned.
