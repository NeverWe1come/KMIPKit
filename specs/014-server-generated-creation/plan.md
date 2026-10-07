# Implementation Plan: Server-Generated Object Creation

**Branch**: feature/KMIPKIT-0014-create-operations | **Date**: 2026-10-07 | **Spec**: spec.md

**Input**: specs/014-server-generated-creation/spec.md

## Summary

Add typed KMIP 2.1 Create, Create Key Pair, and Create Split Key requests and responses to the Rust protocol and typed-client path. Preserve every table-defined field, cardinality, order, and result. Keep cryptographic choices explicit, one exchange per invocation, errors redacted, and response models backed by the existing generic message owner.

## Technical Context

**Language/Version**: Rust 2024, MSRV 1.94

**Primary Dependencies**: Existing kmipkit-protocol, kmipkit-client, kmipkit-ttlv, kmipkit-transport, and zeroize workspace dependencies; no new runtime dependency is planned.

**Storage**: None.

**Testing**: Rust unit tests, table-derived TTLV fixtures, malformed-input tests, property roundtrips where appropriate, and deterministic client fake transport.

**Target Platform**: Linux, Windows, and macOS; Rust 1.94 and stable.

**Project Type**: Rust library workspace.

**Performance Goals**: No additional serialization pass or unbounded allocation; existing 16 MiB/depth-64/100,000-item defaults remain authoritative.

**Constraints**: All outbound model checks precede encoding; no implicit cryptographic parameters; no automatic retry or polling; no raw-body errors or logs; redact AttributeEntry values from public Debug/error context; preserve unknown generic values; send the §9.12 peer response limit for batches containing Create Split Key while enforcing the local byte cap; coverage gates and 100% requirement traceability apply.
**Scale/Scope**: Three client-initiated operations and their operation payloads, limited to TTLV and Rust APIs in this feature.

## Constitution Check

- Specification and normative source clauses are cited in spec.md and will be connected to implementation/tests in the conformance matrix.
- Implementation uses separate Red, Green, and Refactor commits and adds malformed-input and deterministic fake-transport tests.
- Rust remains the sole implementation core. C/Java/Python exposure is deferred to the API parity feature.
- Existing TTLV decoder limits, unknown-value preservation, secret redaction, zeroization, TLS boundary, and no-retry policy remain unchanged. Table 194 permits repeated Create Split Key response identifiers, so the operation applies the existing `Client::execute` local response-byte limit as the peer-visible §9.12 Maximum Response Size, capped to the largest KMIP Integer value.
- Scope stays within the approved KMIP 2.1 client-initiated TTLV 1.0 boundary.
- Sequencing gate: implementation changes in execute.rs must wait until KMIPKIT-0010, KMIPKIT-0012, and KMIPKIT-0013 have landed because they own overlapping client execution and extension paths. KMIPKIT-0012 must also land before the 1.0 public API manifest is frozen. This feature adds Rust operation APIs only; the public API manifest and generated C/Java/Python surfaces belong to the later API parity specification. This spec may be reviewed in parallel; code work must not conflict with those branches.

## Project Structure

    crates/kmipkit-protocol/src/{create,create_key_pair,create_split_key}.rs
    crates/kmipkit-protocol/tests/unit/{create,create_key_pair,create_split_key}_tests.rs
    crates/kmipkit-client/src/execute.rs
    crates/kmipkit-client/tests/unit/{create,create_key_pair,create_split_key}_execution_tests.rs
    specification/catalog/kmip-2.1.json
    specification/catalog/coverage-report.md (regenerated only)
    specs/014-server-generated-creation/{contracts,data-model,quickstart,research,traceability}.md

**Structure Decision**: Extend the existing protocol model crate with one module per operation; connect those typed payloads and operation-agnostic Pending outcomes to the existing closed request enum and single execute writer after the overlapping client work has merged. Update the checked-in catalog input and regenerate the report in the same implementation PR.

## Complexity Tracking

No architectural boundary change or new runtime dependency is planned. The operation models use the existing TTLV value union and preserve wire order. Any proposal to add a new dependency, public raw-item conversion, or a second client writer requires a reviewed spec/ADR change before implementation.
