# Implementation Plan: KMIP 2.1 Query and Ping

**Branch**: `feature/KMIPKIT-0020-query-ping` | **Date**: 2026-10-09 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/020-query-ping/spec.md`

## Summary

Add typed Rust Ping and Query models and explicit client methods on top of the existing TTLV, message/batch, execution, result, and transport foundations. Query will expose its complete request and response shape while preserving open enumeration values, optional/repeated fields, nested Items, and server-reported capability data. Ping will send and receive empty operation payloads. The feature makes no profile or server-health claim.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94

**Primary Dependencies**: Existing KMIPKit workspace crates only; no new dependency expected

**Storage**: N/A

**Testing**: Rust unit/integration tests, source-derived TTLV vectors, fake client transport; normative catalog validator and deterministic coverage report generator

**Target Platform**: Existing Linux, Windows, and macOS CI matrix

**Project Type**: Rust client library with protocol crate

**Performance Goals**: No new latency or throughput target; one explicit exchange per call

**Constraints**: KMIP 2.1 TTLV, existing TLS 1.3/mTLS transports, bounded decoding, no automatic retry, preserve unknown and vendor values

**Scale/Scope**: Two client operations, 14 standard Query Function values plus extensions, all Query response members from Table 283

## Constitution Check

- Exact OASIS 2.1 references and stable catalog requirement IDs are present: PASS.
- Client-initiated TTLV scope and existing transport boundaries are unchanged: PASS.
- Strict Red, Green, Refactor commits are planned for implementation: PASS.
- Unknown values, nested Items, result/delivery evidence, and redaction are preserved: PASS.
- Generated catalog report will be regenerated with its pinned tool: PASS.
- No implementation begins as part of this draft specification PR: PASS.

## Design

### Protocol model

- Add operation-specific Query and Ping request/response models under `crates/kmipkit-protocol/src/`.
- Query request stores an ordered non-empty list of Query Function values and optional Object Groups. Object Groups contains zero or more repeated Object Group attributes; each attribute carries its Text String value (§6.1.40, Table 282; §7.23, Table 375; §4.35, Tables 99–100).
- Query response exposes Table 283 members with exact occurrence rules. Scalar enumerations use the existing open-value model. Complex nested structures and unknown Items remain available as structurally valid `kmipkit_ttlv::Item` values.
- The Query decoder accepts both response forms described by §6.1.40 and Table 283, whose empty-payload and required Protection Storage Masks statements conflict. Keep `KMIPKIT-DISC-045` open and do not make a server-conformance claim until an approved resolution exists.
- Ping request and response operation payloads contain no fields.
- Conversion validates required top-level structure and correct item types without narrowing unknown values.

### Client execution

- Add typed variants and response conversion to the existing client dispatch in `crates/kmipkit-client/src/execute.rs`.
- Add explicit `Client::query` and `Client::ping` conveniences following existing operation methods; both use the common batch/exchange path exactly once.
- Preserve common success/failure, result reason/message, batch correlation, and delivery state. Neither operation adds asynchronous correlation handling; do not retry, poll, or issue hidden follow-up operations.
- No transport implementation or language binding changes.

### Normative catalog

- Assign Query, Ping, Query Function, its 14 named values and extension range, Object Groups structure/member and Object Group attribute, and the two client requirement IDs to KMIPKIT-0020.
- Record the contradictory Query response-shape text as open `KMIPKIT-DISC-045`; do not assign an interpretation to the normative server obligation.
- Keep server-only clause dispositions and existing test/profile mappings intact.
- Regenerate `specification/catalog/coverage-report.md`; do not edit generated output manually.
- Record exact test paths and references in `traceability.md` when implementation exists. No code/test refs are fabricated in this spec-preparation PR.

## Project Structure

### Documentation (this feature)

```text
specs/020-query-ping/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
├── traceability.md
├── checklists/
│   ├── protocol.md
│   └── requirements.md
└── contracts/
    └── public-rust.md
```

### Source code (implementation phase)

```text
crates/kmipkit-protocol/src/query.rs
crates/kmipkit-protocol/src/ping.rs
crates/kmipkit-protocol/src/lib.rs
crates/kmipkit-protocol/tests/unit/query_operation_tests.rs
crates/kmipkit-protocol/tests/unit/ping_operation_tests.rs
crates/kmipkit-client/src/execute.rs
crates/kmipkit-client/src/lib.rs
crates/kmipkit-client/tests/unit/query_execution_tests.rs
crates/kmipkit-client/tests/unit/ping_execution_tests.rs
crates/kmipkit-protocol/tests/query_ping_guide_examples.rs
specification/catalog/kmip-2.1.json
specification/catalog/coverage-report.md  # generated by report.py
specs/020-query-ping/traceability.md
```

**Structure Decision**: Extend the existing protocol and client crates with one module per operation; keep shared transport, message, TTLV, and error code unchanged.

## Risks and mitigations

- **Large heterogeneous Query response**: test each Table 283 member, cardinality, and nested-item round trip independently; retain unknown Items.
- **Unavailable official fixture evidence**: use source-derived vectors and accurately label their provenance; do not claim official fixture passes or profile conformance.
- **Server/client obligation confusion**: map the two applicable client requirements separately; document response selection and precedence as server obligations, while testing lossless client behavior.
- **Release base changes**: synchronize with the active release branch before review and rerun catalog, formatting, lint, test, coverage, and security checks.

## Complexity Tracking

No constitution violation or architecture change is proposed.