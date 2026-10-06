# Implementation Plan: KMIP 2.1 Client Asynchronous Operations

**Branch**: `feature/KMIPKIT-0009-asynchronous-operations-spec` | **Date**: 2026-10-06 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification at `specs/009-asynchronous-operations/spec.md`.

## Summary

Add typed Poll, Cancel, Process, and Query Asynchronous Requests operation models and one-shot client follow-up for a Pending result. Correlation values remain opaque exact bytes; terminal Poll results expose the original operation's status/reason and payload semantics, with a generic payload on success and Result Reason without payload on Failure. Query Asynchronous Requests response remains generic TTLV while `KMIPKIT-DISC-039` is open. The implementation reuses KMIPKIT-0005 TTLV, KMIPKIT-0006 message and batch models, KMIPKIT-0007 execution, and ADR-0014's bounded synchronous transport exchange; it adds no transport, retry loop, or server behavior.

## Technical Context

**Language/Version**: Rust Edition 2024; MSRV 1.94.
**Primary Dependencies**: Existing `kmipkit-ttlv`, `kmipkit-protocol`, `kmipkit-client`, `kmipkit-transport`, `zeroize`, and current workspace dependencies. No new dependency is proposed.
**Storage**: No persistent storage. Pending owners and any copies created and retained by KMIPKit use zeroizing storage while a pending outcome/request exists. Caller-owned Query filter input storage remains the caller's responsibility.
**Testing**: Rust unit and integration tests, pinned-source-derived TTLV vectors, malformed input tests, property-based round trips, and deterministic fake-transport tests. No official operation-specific Test Case IDs are currently linked by the catalog.
**Target Platform**: Existing Rust workspace platforms; platform matrix remains owned by CI.
**Project Type**: Rust library workspace protocol/client feature.
**Performance Goals**: No new latency or throughput target. Each explicit follow-up makes no more than one existing synchronous exchange.
**Constraints**: 16 MiB default message limit, nesting depth 64, 100,000 elements; preserve configured `CodecLimits`; no automatic retries; no raw-body diagnostics; immutable pinned OASIS source; existing delivery-state, response-association, and zeroization rules remain binding.
**Scale/Scope**: Four client-initiated KMIP operations and explicit follow-up of an existing Pending outcome. No server-initiated operations or background processing.

## Constitution Check

| Principle | Check |
|---|---|
| I. Specification and traceability | Draft maps requirements to exact source clauses/tables and stable project IDs. Process request's missing catalog requirement ID and DISC-039 remain visible gates. |
| II. Test first and evidence based conformance | Future implementation tasks explicitly order Red, Green, Refactor commits. Derived tests will not be mislabeled official vectors. No conformance/profile claim is authorized by this plan. |
| III. One core, explicit language boundaries | This specification is Rust protocol/client only. C/Java/Python parity remains assigned to binding work. |
| IV. Secure defaults and lossless protocol handling | Exact opaque correlation bytes, redacted diagnostics, zeroizing owners, configured decode limits, and no retry are mandatory. |
| V. Human governed, reviewable changes | This draft and reviewer-owned checklists retain their review status; the direct human instruction authorizes autonomous implementation for this task without further approval prompts. Only a human may approve or merge the eventual PR. |

**Gate result**: No constitution exception is proposed. The direct human instruction authorizes autonomous implementation without further approval prompts, but does not claim formal approval of this draft plan or reviewer-owned checklists. Approval and OD-001/OD-002 review remain open. `KMIPKIT-DISC-039` blocks typed Query response mapping and a complete conformance claim, but does not block generic response preservation or Poll/Cancel/Process models.

## Design

### Operation and outcome model

- Add protocol value models for Poll, Cancel, Process, and Query Asynchronous Requests using the existing generic TTLV `Structure`/`Item` model and typed views. Known Cancellation Result values receive typed variants while retaining unknown raw Enumeration values.
- Extend the typed client request path to emit one selected asynchronous operation and use KMIPKIT-0007's existing request-header, response-ID association, size-limit, error, and delivery-state logic.
- Generalize pending-operation data so it retains the exact correlation bytes and enough request association to form a follow-up. Expose Pending values only through KMIPKIT-0007's explicit borrowed accessor; keep KMIPKit-owned storage zeroizing and do not create ordinary unzeroized duplicates. No `Debug`, `Display`, or error formatter may include the value.
- A Poll that remains Pending produces a caller-visible Pending outcome with no original-operation payload and the correlation value from that response. A terminal Poll exposes the original operation's status/reason and response payload semantics. Failure outcomes expose Result Reason and no payload; successful operation payloads are generic `Structure` values until a typed model exists. It is never decoded as `DiscoverVersionsResponse` unless the original operation is Discover Versions and its typed interpretation is explicitly supported.
- Cancel exposes its response echo and Cancellation Result. A mismatch between request and echoed correlation value is a protocol association error; unknown Cancellation Result values are preserved. Since §6.1.5 prohibits an asynchronous Cancel response, any Pending status in a Cancel response is a protocol error even if the original request permitted asynchronous results.
- Process is a separate request and result. For every non-Failure result, §8.6/Table 399 requires a Response Payload and §6.1.39/Table 279 defines that payload as empty; Failure has no payload. The client does not assert that a subsequent Poll completes, and it documents the possible original-batch effects when Batch Order Option is true or absent.
- Query Asynchronous Requests has optional correlation-value and operation filters. Its response is a generic TTLV tree while DISC-039 remains unresolved; do not add a guessed typed response model.

### Transport and lifecycle boundaries

- Every public follow-up action maps to at most one call to the existing `Transport::exchange`; Poll Pending never recurses or loops.
- Every Asynchronous Correlation Value is sensitive diagnostic data, including values supplied as Query filters. Caller-owned Query input storage remains the caller's responsibility; redact all such values, zeroize KMIPKit-owned copies under ADR-0014, and do not create ordinary unzeroized duplicates. Pending values remain accessible only through the explicit borrowed accessor.
- Transport/protocol errors preserve NotSent, PossiblySent, or ResponseStarted evidence established by KMIPKIT-0007 and ADR-0014. No request is resent automatically.
- Correlation bytes are copied into the encoded request only for the duration/ownership boundary defined by ADR-0014; owner drop zeroizes the initialized bytes that KMIPKit owns. Tests use sentinels at request and pending-owner boundaries.
- A production TLS/HTTPS constructor remains the responsibility of the transport/configuration feature. KMIPKIT-0009 tests the private fake-transport client seam and does not make transport injection public.

### Requirement-to-code/test plan

| Requirement | Planned code location | Planned verification |
|---|---|---|
| FR-001, FR-003–FR-007 | `crates/kmipkit-protocol/src/{poll,cancel,process,query_async_requests}.rs`; protocol module exports | Model and exact TTLV field/order tests; operation-level pending/completed/error vectors |
| FR-002, FR-008–FR-010 | `crates/kmipkit-client/src/execute.rs` or a focused async execution module; existing private encoder/response decoder | Fake-transport one-exchange tests, borrowed accessor and no-duplicate checks, zeroization, exact-byte correlation sentinels, pending lifecycle tests, terminal Poll Failure, rejected Cancel Pending, response association/error delivery tests |
| FR-006–FR-007, FR-009 | `crates/kmipkit-protocol/src/query_async_requests.rs` and generic response view | Filter encoding, sentinel redaction, encoded request-owner zeroization, caller-owned buffer boundary, and opaque response round-trip; no Table 286 typed mapping |
| FR-009 | `crates/kmipkit-client/tests/unit/` plus current zeroization test-support seams | Debug/Display/error/log sentinel checks, borrowed-only Pending access, no ordinary unzeroized duplicate, encoded request-owner drop observation for Cancel/Query success and exchange-error paths, Query request-drop-before-exchange AST assertion, and caller-owned input boundary |
| FR-012 | `specification/catalog/kmip-2.1.json` workflow is out of scope here; tests remain in protocol/client suites | Before release, catalog-owner workflow must assign/verify a stable requirement for Table 278's client field; feature tests cite this feature FR until then |

Exact source links and clause IDs are in `spec.md`. The direct human instruction authorizes autonomous implementation for this task; it does not close source/catalog gates or claim reviewer approval.

## Project Structure

### Documentation (this feature)

```text
specs/009-asynchronous-operations/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
└── checklists/
    ├── requirements.md
    ├── protocol-review.md
    └── implementation-readiness.md
```

### Source Code (repository root)

```text
crates/
├── kmipkit-protocol/src/       # operation models and generic payload views
├── kmipkit-protocol/tests/     # source-derived operation/model tests
└── kmipkit-client/
    ├── src/                    # one-shot follow-up through typed execution
    └── tests/                  # fake-transport and lifecycle tests
```

**Structure Decision**: Keep wire-level operation models in `kmipkit-protocol`; keep exchange, pending owner, batch association, and delivery state in `kmipkit-client`. Do not add a new crate or public transport injection path.

## Test and implementation sequence

1. **Authorization record**: the direct human instruction authorizes autonomous implementation without further approval prompts. This does not represent reviewer approval of the spec or reviewer-owned checklists; their review status remains open. The source and catalog dispositions remain explicit and unchanged.
2. **Red**: add derived table-driven and malformed-input tests first; add exact-byte and fake-transport lifecycle tests; commit these failing tests separately with DCO sign-off.
3. **Green**: implement the minimal typed model and one-shot client execution to satisfy only those tests; commit separately.
4. **Refactor**: consolidate duplicated response parsing, preserve module boundaries, document public APIs and update traceability/tests; commit separately.
5. **Converge and gates**: run convergence, update any discovered tasks, run format, Clippy, workspace tests/docs, coverage thresholds, immutable-source/catalog checks, dependency/security checks, and CI matrix. Resolve failures without changing scope silently.
6. **Review**: independent QA and security reviews, then a terminal-created draft PR. No self-approval or merge.

Tests must include the §6.1.38 special Pending Poll shape (no payload but Pending correlation), borrowed-only Pending access with no ordinary unzeroized duplicate, successful and failed terminal original-operation outcomes, rejected Pending Cancel response, Cancel echo/result, Process empty payload on every non-Failure result (including Pending) and no payload on Failure, Query opaque response and sensitive filters, and exact byte requests. Query tests distinguish caller-owned input storage from zeroized KMIPKit-owned copies. Property tests must include arbitrary binary correlation bytes within `CodecLimits`; do not impose a token format or an unsupported empty-value restriction.

## Complexity Tracking

No constitution violation or new architectural pattern is proposed. The generic response path is required to avoid inventing a Query response schema while a primary-source discrepancy remains open.
