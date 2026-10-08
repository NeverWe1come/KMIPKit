# Implementation Plan: KMIP 2.1 Attribute Operations

**Branch**: `feature/KMIPKIT-0016-attribute-operations` | **Date**: 2026-10-08 | **Spec**: [spec.md](spec.md)

**Input**: [Feature specification](spec.md)

## Summary

Add typed Rust request/response models and deterministic client execution for the seven KMIP 2.1 attribute operations. Reuse the bounded generic TTLV value model and the accepted typed message, result, transport, and delivery-state contracts. Preserve each operation's distinct semantics, full Attribute identity/value, future values, wire order, and operation-specific errors. No new runtime dependency is planned.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94.

**Primary Dependencies**: Existing `kmipkit-protocol`, `kmipkit-client`, `kmipkit-ttlv`, `kmipkit-transport`, and workspace `zeroize` dependencies; no new runtime dependency.

**Storage**: None. Attribute changes are requests to a remote KMIP server; KMIPKit MUST NOT keep a local mirror.

**Testing**: Red/Green/Refactor Rust tests; table-derived request/response vectors; pinned OASIS test cases where applicable; negative malformed-input tests; generic TTLV property roundtrips; deterministic fake-transport execution tests; `cargo llvm-cov` at repository gates.

**Target Platform**: Linux, Windows, and macOS; Rust 1.94 and stable.

**Project Type**: Rust library workspace.

**Performance Goals**: No additional serialization pass, avoid unnecessary copies of attribute values, and enforce existing 16 MiB message, depth 64, and 100,000 element limits before allocation.

**Constraints**: Exact OASIS table fields/cardinality; no implicit changes or attribute selection; no automatic retry; preserve pending/delivery-state semantics; redact attribute values from Debug/errors; retain exact names and unknown values on allocated/accepted extension tags, while rejecting Reserved outbound tags and Adjustment Type values; obey the 95% protocol/new-code coverage gates; do not modify upstream OASIS files.

**Scale/Scope**: Seven client-initiated operations and their shared attribute values, limited to TTLV and the Rust protocol/client surface in this specification.

## Constitution Check

- **Specification and traceability**: Partial for this draft. Every operation has an exact v2.1 section/table citation and FR-001 through FR-014 map to planned verification. Approval remains blocked until Phase B maps applicable catalog requirements/test cases and operation elements to this feature; implementation must complete code/test refs.
- **Test-first and evidence-based conformance**: Pass as a plan. Tests are first and recorded in a distinct Red commit; Green and Refactor are separate commits. Official test cases are claimed only for the actual complete case exercised.
- **One core, explicit language boundaries**: Pass. Rust owns protocol behavior. C, Java, Python, and high-level builders are later API-parity work.
- **Secure defaults and lossless handling**: Pass. Existing decoder bounds, generic unknown-value preservation, redaction, and no-retry policy remain in force.
- **Human-governed changes**: Pass. This plan is a draft for review. No operation implementation starts until the specification PR is approved/merged, its catalog approval blocker is closed, and the dependency gates below are satisfied. Only a human may approve or merge the PR.
- **Product boundaries**: Pass. TTLV, KMIP 2.1, TLS 1.3/mTLS, and client-initiated operation limits remain unchanged.

### Implementation Gate

The approved KMIPKIT-0014 specification defines the shared `AttributeEntry` model used by creation and attribute operations, but its implementation has not yet landed on the release branch. Code for this feature MUST wait until the approved KMIPKIT-0014 implementation is merged; KMIPKIT-0016 MUST NOT redefine or modify that shared contract. The KMIPKIT-0013 transport and KMIPKIT-0015 integration branches also touch the client/transport baseline; this feature MUST branch from the updated release after those dependencies are integrated. The KMIPKIT-0016 specification itself MUST be approved and merged, and the catalog approval blocker must be closed, before implementation begins. If a dependency changes the agreed public contract, revise and re-review this specification before implementation.

## Project Structure

### Documentation (this feature)

```text
specs/016-attribute-operations/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── traceability.md
├── contracts/public-rust.md
├── checklists/requirements.md
├── checklists/protocol.md
└── tasks.md
```

### Source Code

```text
crates/kmipkit-protocol/src/attribute.rs
crates/kmipkit-protocol/src/{add_attribute,adjust_attribute,delete_attribute}.rs
crates/kmipkit-protocol/src/{get_attributes,get_attribute_list,modify_attribute,set_attribute}.rs
crates/kmipkit-protocol/tests/unit/attribute_*_tests.rs
crates/kmipkit-client/src/execute.rs
crates/kmipkit-client/tests/unit/attribute_execution_tests.rs
specification/catalog/kmip-2.1.json
specification/catalog/coverage-report.md       # regenerated output only
specs/016-attribute-operations/traceability.md
```

**Structure Decision**: Extend the existing protocol crate with one shared attribute model and one module per operation. Extend the current client request/response and execution path, rather than introducing a second writer. Keep protocol conformance tests beside the protocol crate and fake-transport tests beside client execution. Update the checked-in catalog input and regenerate generated reports in the same implementation PR.

## Design Decisions and Alternatives

- Preserve each operation as a distinct typed request. A generic `AttributeMutation` API was rejected because it could hide the differences among add, adjust, delete, modify, and set.
- Represent an Attribute as exact name plus its complete generic TTLV value. Reconstructing tags from names was rejected. Generic unknown Enumeration values remain preservable, but outbound Items must use assigned/accepted §11.56 tags and typed Adjustment Type must use assigned or Table 429 extension values; unallocated/Reserved values cannot be emitted.
- Keep Adjust Attribute as a server-side operation carrying `Adjustment Type` and optional `Adjustment Value`. Computing the result locally was rejected because the client lacks authoritative server state and KMIP defines attribute and object-state checks at the server.
- Leave attribute policy and state transition outcomes to the server; preserve its Result Reason. Local policy emulation was rejected because it can diverge from server state and object-specific requirements.
- Reuse existing message execution, transport, decoder limits, and result handling. A new runtime dependency or parallel transport path is unnecessary.

## Complexity Tracking

No architectural boundary change, additional workspace crate, or new runtime dependency is planned. Any change to the shared `AttributeEntry` contract or client writer requires a reviewed specification/ADR update before code changes.
