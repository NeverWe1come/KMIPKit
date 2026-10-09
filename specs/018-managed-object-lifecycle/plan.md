# Implementation Plan: KMIP 2.1 Managed-Object State Transitions

**Branch**: `feature/KMIPKIT-0018-managed-object-lifecycle` | **Date**: 2026-10-09 | **Spec**: [spec.md](spec.md)

**Input**: Bounded feature specification covering Activate, Archive, Destroy, and Recover for the KMIP 2.1 client.

## Summary

Add typed Rust request/response models and client execution for four client-initiated lifecycle operations, using their exact KMIP 2.1 payload tables. Reuse the existing TTLV, message, batch, result, delivery-state, async, and transport contracts. Assign the normative catalog records to KMIPKIT-0018 and regenerate the coverage report. Tests use local OASIS-derived vectors and deterministic fake transport; no server behavior is simulated and no automatic follow-up is added.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94.

**Primary Dependencies**: Existing KMIPKit workspace crates only; no new dependency is planned.

**Storage**: None. The client does not keep a local object-state database.

**Testing**: `cargo test` unit and fake-transport tests; OASIS-derived request/response vectors; malformed and negative TTLV inputs; property or round-trip checks where they add coverage; normative catalog validator and generated-report check.

**Target Platform**: Linux, Windows, and macOS through existing CI.

**Project Type**: Rust client library protocol and execution crates.

**Performance Goals**: No new performance target; operations add no extra network exchanges beyond the caller's explicit request.

**Constraints**: TTLV only; KMIP 2.1; client-initiated operations only; default decoder limits remain 16 MiB, depth 64, and 100,000 elements; TLS 1.3 with mTLS remains unchanged; no automatic retry, Poll, or follow-up Get; all upstream OASIS files remain immutable.

**Scale/Scope**: Exactly four operations and their table-defined request/response payloads, common KMIP result behavior, batch correlation, Recover Pending integration, documentation, and normative traceability.

## Constitution Check

| Principle | Status | Evidence / gate |
| --- | --- | --- |
| I. Specification and traceability | Pass for design | This draft cites pinned KMIP 2.1 sections/tables and catalog IDs. Implementation remains gated on human review/approval; changed code must map every applicable client requirement to a test. |
| II. Test first and evidence-based conformance | Pass for design | Tasks require distinct Red, Green, and Refactor commits, source-derived negative vectors, fake-transport checks, and honest official-test limitations. |
| III. One Rust core and explicit language boundaries | Pass | This is a Rust protocol/client slice. FFI and Java/Python APIs are deferred to the later parity specifications; no unsafe code is introduced. |
| IV. Secure defaults and lossless protocol handling | Pass | Reuses decoder bounds, generic TTLV preservation, redacted errors, secret ownership, TLS and no-retry policy. No protocol or transport policy changes. |
| V. Human governed, reviewable changes | Pass | Dedicated branch/worktree from `release/1.0.0`; draft PR only; only a human approves or merges. No generated output is edited manually. |

No ADR or architecture boundary change is required. Implementation MUST NOT begin until this specification is approved and merged into the active release branch.

## Phase 0: Research Decisions

See [research.md](research.md). The pinned source and release catalog confirm the four request/response/error table ranges, the optional request and required successful-response Unique Identifier shape, its encodings in §4.58 Tables 145–146, the tag assignment in §11.56, the Archive and Recover client MAY requirements, and the server-only classification of Activate and Destroy prose clauses. Recover reuses the existing asynchronous outcome model; no additional state machine is needed.

The catalog has no requirement-specific official Test Cases IDs for the Archive and Recover client requirements. Use derived vectors and report that limitation; do not claim official-case passes. Activate and Destroy operation-table behavior still requires executable coverage even though their prose clauses are server-only.

## Phase 1: Design

### Data Model

See [data-model.md](data-model.md). Each request preserves an optional Unique Identifier, including omission and valid existing batch-placeholder use. Each successful typed response exposes its required Unique Identifier. Recover's Pending outcome uses the shared operation-agnostic result with exact correlation bytes. Failure and transport delivery evidence use existing shared types.

### Public Contract

See [contracts/public-rust.md](contracts/public-rust.md). Add closed request/response variants and typed single-operation entry points following the existing Create and attribute-operation conventions. The generic `Client::execute` batch path remains the shared one-exchange implementation. This PR does not update generated C, JNI, or CFFI surfaces; those are later API-parity work.

### Traceability and Generated Artifacts

`traceability.md` maps each operation element, each applicable client requirement, and each relevant source table to its acceptance criteria and planned tests. In the checked-in normative catalog, assign the four operation elements and the three applicable client requirement records to `KMIPKIT-0018`; link the Archive and Recover requirement records to their operation elements. Keep Activate and Destroy server-only clauses out of the client requirement set. Regenerate `specification/catalog/coverage-report.md` with the pinned report generator and verify it with `--check`.

### Project Structure

```text
crates/kmipkit-protocol/src/{activate,archive,destroy,recover}.rs
crates/kmipkit-protocol/src/lib.rs
crates/kmipkit-client/src/execute.rs
crates/kmipkit-client/src/lib.rs
crates/kmipkit-protocol/tests/unit/*_operation_tests.rs
crates/kmipkit-client/tests/unit/*_execution_tests.rs
crates/kmipkit-protocol/tests/support/lifecycle_fixtures.rs
docs/user-guide/en/lifecycle-operations.md
docs/user-guide/es/operaciones-ciclo-vida.md
specification/catalog/kmip-2.1.json
specification/catalog/coverage-report.md  # generated; never edit by hand
specs/018-managed-object-lifecycle/
```

The implementation may adapt file names to established workspace conventions, but must keep each operation model and its tests independently understandable.

### Quickstart Validation

Follow [quickstart.md](quickstart.md) after implementation. The initial verification includes focused protocol and client tests, the catalog validator/report check, and then the full cross-platform CI/coverage gates. These commands are validation instructions, not evidence that implementation has already passed.

## Complexity Tracking

No constitution violations or new architectural components are planned.
