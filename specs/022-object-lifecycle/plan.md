# Implementation Plan: KMIP 2.1 Object Lifecycle and Certificates

**Branch**: `feature/KMIPKIT-0036-object-lifecycle` | **Date**: 2026-10-11 | **Spec**: [spec.md](spec.md)

**Input**: Approved feature specification at `specs/022-object-lifecycle/spec.md`.

## Summary

Add typed client support for Certify, Check, Obtain Lease, Re-certify, Re-key, Re-key Key Pair, Re-Provision, Register, and Revoke using existing KMIP 2.1 TTLV, object, message, batch, result, and transport contracts. Keep payload rules tied to the exact OASIS clauses and tables in the approved specification. Preserve unknown values and generic response data, apply only client-side validation backed by the requirement ledger, perform one explicit exchange per call, and leave server policy and object-state decisions to the server and caller.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94; C11, Java 17, and Python 3.12 through the approved binding surface.

**Primary Dependencies**: Existing KMIPKit workspace crates and pinned generators only; no new runtime dependency is expected.

**Storage**: N/A.

**Testing**: Protocol unit tests from pinned OASIS request/response/error tables; malformed and conditional-field tests; fake-transport client tests; TTLV round-trip/property tests; language parity fixtures and existing C/Java/Python consumer suites; catalog validation and generated-output checks.

**Target Platform**: Existing Linux, Windows, and macOS CI matrix.

**Project Type**: Synchronous Rust client/protocol library with C ABI, Java JNI, and Python CFFI product facades.

**Performance Goals**: No new latency target or local cryptographic work. Each explicit operation call performs at most one request/response exchange and never retries or polls automatically.

**Constraints**: KMIP 2.1 over TTLV only; client-initiated operations; existing typed batch and delivery-state semantics; unknown-value preservation; redaction and zeroization for KMIPKit-owned secrets; no server implementation, remote policy checks, or local certificate/key algorithms.

**Scale/Scope**: Nine operation request/response/error forms; 44 stable client requirement IDs in the approved ledger; all applicable request and response table fields plus listed error result reasons.

## Constitution Check

- Exact OASIS clauses and payload/error tables are assigned in [spec.md](spec.md): PASS.
- Approved 1.0.0 client, TTLV, transport, TLS, API-level, and no-local-crypto boundaries are unchanged: PASS.
- Unknown values, common result metadata, Pending/correlation, and generic TTLV remain inspectable: PASS.
- Local validation is limited to client requirements decidable from supplied request data; remote policy and object state are not inferred: PASS.
- Red, Green, and Refactor are planned as separate development commits per operation story: PASS.
- Secret redaction, zeroization reuse, no retries, and delivery-state behavior use shared contracts: PASS.
- Rust operation models can be implemented without KMIPKIT-0043. Four-language manifest/binding mapping follows the canonical public API scope in approved KMIPKIT-0043; that gate does not block standalone Rust protocol work: PASS.
- No upstream-source, constitution, accepted ADR, or generated-file hand-edit is planned: PASS.

## Design

### Dependency and sequencing gate

The active release base supplies the TTLV codec, generic ordered structures, typed messages and batches, common result handling, client execution, transports, object/attribute models, credentials, and redacted/zeroizing byte ownership. Confirm those APIs at implementation start and reuse them; do not add parallel message, result, secret, or attribute abstractions.

The nine operation modules and protocol tests have independent payload ownership. Their client registrations share `crates/kmipkit-client/src/execute.rs`, so dispatcher changes are serialized by one owner. No operation in this feature requires another operation here first.

KMIPKIT-0043 owns the versioned public manifest and four-language capability mapping. Rust request/response models and typed client methods are independently implementable. C/Java/Python declarations, generated outputs, and parity completion follow the canonical API and the corresponding 0043 operation tasks after that work is available. Do not invent binding signatures or edit the 0043 manifest from this feature.

### Protocol models

Add one module per operation under `crates/kmipkit-protocol/src/`: `certify.rs`, `check.rs`, `obtain_lease.rs`, `recertify.rs`, `rekey.rs`, `rekey_key_pair.rs`, `reprovision.rs`, `register.rs`, and `revoke.rs`. Export documented request, response, and sanitized model-error types from `lib.rs`.

Reuse generic TTLV, object, attribute, certificate, credential, result, and secret-value representations. Requests preserve optional-field absence and table child ordering. Responses retain the common result and original generic response item so unknown fields remain available. CSR, certificate, key/object, and credential material is sensitive; reuse the existing zeroizing owner and redacted formatting.

Apply only locally decidable client rules recorded by stable requirement IDs. Examples include requiring the public key's explicit Unique Identifier when Certify omits Certificate Request (KMIPKIT-DEC-013), conditional Certificate Request Type, Revoke Compromise Occurrence Date prohibition, required Register fields, explicit Re-Provision input forms, and operation date/attribute rules. Do not preflight, resolve ID Placeholders locally, infer server-held attributes, or enforce remote policy. Preserve listed error Result Reasons as common result values, including unknown/future values.

Check's recommended batch options are opt-in: callers can express Batch Order Option=true and Stop or Undo with the existing `ClientBatch` builder. Do not rewrite caller-supplied options or hide those choices in the single-item convenience method.

### Client execution

Add each operation to the existing typed request/response dispatch and operation-code map in `crates/kmipkit-client/src/execute.rs`, with explicit synchronous convenience methods following neighboring operations. Each method uses the current encoder/transport/result path and makes at most one exchange. Typed batch callers keep per-item order/correlation, Pending status, delivery evidence, and generic response access. No automatic retry, Poll, Re-Provision follow-up Get, lease tracking, or certificate/key algorithm is introduced.

### Catalog, traceability, and bindings

Map every applicable requirement and mandatory/conditional payload row to existing normative catalog records or stable spec-scoped IDs where required. Update the source catalog and feature traceability in the implementation PR, then run the pinned generator; never hand-edit generated language code or coverage reports. Coordinate catalog/report edits with concurrent operation work because they are shared integration files.

Bind all four languages only through KMIPKIT-0043's canonical manifest. Its operation mapping tasks for this feature are T043, T044, T056, and T058–T063. Four-language completion is a release requirement, but not a prerequisite for Rust models.

### Test strategy

- Build source-derived positive and negative TTLV vectors for every applicable request, success, and error table in [spec.md](spec.md).
- Cover conditional/optional fields, field order, open/unknown values, generic unknown children, duplicate or malformed fields, and response cardinality.
- Use fake transports to check each operation code, one exchange, batch/result association, Failure and Pending passthrough, delivery state, and no retry.
- Redact sentinel CSR, certificate, key/object, and credential bytes from Debug, Display, and errors; verify owned secret bytes reuse zeroizing storage.
- Re-key official catalog cases are KMIPKIT-TEST-CN01-2-76 through -87. Other operations have no direct case IDs in the approved inventory. Use local pinned fixtures only when present; otherwise label tests table-derived and do not claim an official fixture pass.
- Run formatter, Clippy, focused/workspace tests, coverage, generator drift, catalog, security, and supported-platform CI checks during implementation review.

## Project Structure

### Documentation (this feature)

```text
specs/022-object-lifecycle/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
├── traceability.md
└── contracts/
    └── public-rust.md
```

### Source code (implementation phase)

```text
crates/kmipkit-protocol/src/certify.rs
crates/kmipkit-protocol/src/check.rs
crates/kmipkit-protocol/src/obtain_lease.rs
crates/kmipkit-protocol/src/recertify.rs
crates/kmipkit-protocol/src/rekey.rs
crates/kmipkit-protocol/src/rekey_key_pair.rs
crates/kmipkit-protocol/src/reprovision.rs
crates/kmipkit-protocol/src/register.rs
crates/kmipkit-protocol/src/revoke.rs
crates/kmipkit-protocol/src/lib.rs
crates/kmipkit-protocol/tests/unit/*_operation_tests.rs
crates/kmipkit-client/src/execute.rs
crates/kmipkit-client/src/lib.rs
crates/kmipkit-client/tests/unit/*_execution_tests.rs
docs/user-guide/en/lifecycle-and-certificates.md
docs/user-guide/es/operaciones-ciclo-vida-y-certificados.md
specification/catalog/kmip-2.1.json
specification/catalog/coverage-report.md  # regenerate with the pinned tool
```

**Structure Decision**: Follow the existing one-module-per-operation protocol pattern and closed typed client dispatcher. Operation modules/tests can be authored independently; protocol exports, client dispatch, catalog, and generated report are shared serialized integration points. Four-language mapping belongs to KMIPKIT-0043.

## Risks and mitigations

- **Shared dispatcher contention**: serialize edits to `execute.rs` and client exports; keep operation modules/tests isolated.
- **Remote policy mistaken for local validation**: validate only caller-provided data and the approved client requirement ledger.
- **Sensitive material in diagnostics**: reuse zeroizing owners and test sentinel redaction.
- **Unavailable official fixtures**: derive vectors from exact pinned tables and report fixture availability honestly.
- **Concurrent catalog edits**: coordinate catalog/report generation; never hand-edit generated output.
- **0043 is not yet integrated**: continue Rust protocol/client work; hold only manifest-backed language mapping and parity closure until its canonical manifest is available.

## Complexity Tracking

No constitution violation or accepted architecture change is proposed.
