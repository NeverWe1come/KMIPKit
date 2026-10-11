# Implementation Plan: KMIP 2.1 Policy and Usage Controls

**Branch**: `feature/KMIPKIT-0039-policy-controls` | **Date**: 2026-10-11 | **Spec**: [spec.md](spec.md)

**Input**: Approved feature specification in `specs/025-policy-controls/spec.md`.

## Summary

Add typed client request/response models and explicit one-exchange execution for Get Constraints (§6.1.22, Tables 229–231), Get Usage Allocation (§6.1.23, Tables 232–234), Set Constraints (§6.1.52, Tables 325–327), Set Defaults (§6.1.53, Tables 328–330), and client-to-server Set Endpoint Role (§6.1.54, Tables 331–333). Preserve common KMIP results, Pending correlation, unknown values, absent fields, and transport delivery evidence. Add the approved caller-supplied local usage-allocation guard for typed Encrypt, MAC, and Sign calls. Reuse existing TTLV, object, result, batch, client, transport, and secret-safe error contracts; no local cryptographic algorithm or remote state inference is introduced.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94; C, Java, and Python parity through the approved public API manifest.

**Primary Dependencies**: Existing KMIPKit workspace crates and pinned generation tools. No new runtime dependency is planned.

**Storage**: N/A. A usage-allocation guard is caller-owned in-memory context and is not persisted.

**Testing**: Protocol model and TTLV roundtrip tests; malformed/unknown-value tests; deterministic one-exchange fake-transport tests; §6.1.23 guard and delivery-state tests; compiled C ABI and Java/Python parity tests after the canonical manifest slice is approved; official fixtures only where the pinned catalog identifies an available fixture.

**Target Platform**: Existing Linux, Windows, and macOS matrix; Rust core with C, Java, and Python public surfaces.

**Project Type**: KMIP 2.1 TTLV client library.

**Performance Goals**: No new throughput target. A guarded call makes one explicit request exchange and performs no local cryptographic computation.

**Constraints**: KMIP 2.1, client-to-server operations only, TTLV only, current typed batch and delivery-state contracts, no retry or automatic Poll, exact source citations, preserve open enums and unknown TTLV, no server policy inference. The §6.1.23 guard accounts only caller-supplied unit costs routed through one local context; it is not server-consumption evidence.

**Scale/Scope**: Five KMIP operations and one local usage-allocation guard. No server-side Set Endpoint Role operation (§6.2.5), no local policy engine, no new cryptographic provider, and no raw-body execution path.

## Constitution Check

- Approved product boundary: KMIP 2.1, TTLV, client initiated, existing transports and Rust/C/Java/Python parity only: **PASS**.
- Exact normative references: operation tables and §6.1.23 requirement IDs are enumerated in `spec.md` and `traceability.md`: **PASS**.
- Unknown tags, enum values, bitmask bits, and extensions remain lossless: **PASS**.
- No local cryptography, automatic retry/Poll, unrequested server policy, or hidden object/session state: **PASS**.
- The local usage guard uses only explicit caller inputs and existing delivery evidence; it does not infer remote consumption or external use: **PASS**.
- TDD Red, Green, and Refactor evidence is required in distinct commits for each operation story and the guard: **PASS**.
- KMIPKIT-0043 gates the cross-language manifest/adapters/generated parity slice only; it does not block standalone Rust protocol models or Rust client work: **PASS**.
- No pinned OASIS upstream file or generated file is edited by hand: **PASS**.
- No implementation-blocking ambiguity remains after the maintainer-approved §6.1.23 guard interpretation recorded in the approved specification: **PASS**.

## Normative Scope and Dependencies

| Work | Exact source | Requirement/contract |
| --- | --- | --- |
| Get Constraints | Specification v2.1 §6.1.22, Tables 229–231; Constraints §7.7 | Empty request, required Constraints response, preserve operation errors. |
| Get Usage Allocation | Specification v2.1 §6.1.23, Tables 232–234; Usage Limits §4.59/Table 148; Unit §11.60/Table 491 | Optional Unique Identifier; required Usage Limits Count; required response Unique Identifier; apply four catalog requirement IDs through the approved local caller guard. |
| Set Constraints | Specification v2.1 §6.1.52, Tables 325–327; Constraints §7.7 | Required Constraints request, empty success payload, preserve errors; follow approved DISC-020 interpretation. |
| Set Defaults | Specification v2.1 §6.1.53, Tables 328–330; Defaults Information §7.11 | Optional Defaults Information; its absence requests removal of all Object Defaults; empty success payload; follow approved DISC-020 interpretation. |
| Set Endpoint Role | Specification v2.1 §6.1.54, Tables 331–333; Endpoint Role §11.19 | Required open Enumeration on request and response; only client-to-server direction. |
| Shared response contract | Constitution and existing batch/client contracts | Preserve Result Status/Reason/Message where permitted, Pending/correlation, unknown extensions, and delivery state. One explicit exchange; no retry or implicit Poll. |

The approved user-facing Rust/client interfaces for Encrypt, MAC, and Sign are the integration points for the guard. Reuse only the typed request path available on the active release baseline; do not reimplement those operation payloads. If a specific typed request is not present on that baseline, sequence its guard integration after its approved operation model lands. This is a sequencing dependency on typed request availability, not on the separate manifest/bindings work.

KMIPKIT-0043 is a dependency only for adding these operations and the guard to the canonical public manifest, generating language surfaces, and completing C/Java/Python parity. Rust models, typed dispatch, fake-transport tests, and Rust guard accounting can proceed independently of 0043. Cross-feature edits to `crates/kmipkit-client/src/execute.rs`, `crates/kmipkit-client/src/lib.rs`, and the facade exports must be integrated serially with other operation work.

## Design

### Protocol models

Implement one focused protocol module per operation under `crates/kmipkit-protocol/src/`: `get_constraints.rs`, `get_usage_allocation.rs`, `set_constraints.rs`, `set_defaults.rs`, and `set_endpoint_role.rs`. Keep nested Constraints and Defaults Information as ordered generic `Structure` values so unrecognized members are preserved. Represent Endpoint Role and Usage Limits Unit as open raw Enumeration values. Encode Usage Limits Count as KMIP Long Integer (`i64`) per Specification §4.59, Table 148, and preserve the caller's requested value. Operation error results use the common result model rather than operation-local error enums that discard unknown values.

### Client execution

Add explicit `ClientRequest` variants, typed response views, and one-item convenience methods for the five operations through the existing shared executor. All methods use the same validation, message building, bounded codec, transport, response matching, result handling, and delivery-state reporting as existing operations. Set Endpoint Role means only client request/response exchange; it does not add server-initiated behavior. No automatic retry, Poll, metadata lookup, or client policy mutation is added.

### Caller-supplied usage-allocation guard

Add a non-cloneable caller-owned context created only from a successful Get Usage Allocation exchange plus its exact requested count, the response's resolved Unique Identifier, and caller-known Usage Limits Unit. The response does not echo the count or unit, so both remain explicit inputs to the context constructor. An omitted request identifier is bound to the concrete identifier returned by the server.

The guard accepts one typed Encrypt, MAC, or Sign request per execution and a caller-supplied positive unit cost. It rejects a missing/placeholder/mismatched Unique Identifier, zero cost, or cost exceeding the context's currently available local allowance before transport. The caller supplies costs; byte count is never inferred from Data length, and Object/extension-unit costs are not inferred from operation names. Unknown unit enumeration values remain representable. Decrypt, MAC Verify, Signature Verify, and Hash are not guard targets because §4.59 and §6.1.23 distinguish protection from processing protected data.

Reserve the cost before the exchange. Release it only when the client has `NotSent` evidence. Keep it reserved when delivery is `PossiblySent` or `ResponseStarted`, or when a complete Success, Failure, or Pending response arrives. This avoids exceeding the known local quota after an ambiguous request, without claiming the server consumed it. Ordinary unguarded execution, generic TTLV, another process/client, and outside actions are not counted. The caller is responsible for routing each known Usage-Limited protection use through the chosen guard. The feature cannot inspect object metadata, activation/protect-stop dates, or server-wide usage.

### Validation and forward compatibility

Validate required fields and structures before writing bytes, assigning `NotSent` to local validation failures. Preserve optional absence and field order. Unknown Enumeration values and extension members pass through structurally valid TTLV. Preserve server Failure, Pending, permitted Result Message, correlation bytes, and opaque result data without inventing status. Malformed network responses use the shared safe protocol error and strongest delivery evidence.

### Cross-language parity

After the canonical API contract in KMIPKIT-0043 is approved and merged, add manifest entries and corresponding C ABI, Java, and Python adapters, then regenerate and test outputs using the pinned repository generator. Generated header and binding metadata are never edited manually. The guard is part of parity only if the 0043 manifest contract can expose the same explicit caller-provided object, unit, count, cost, and delivery behavior; do not weaken the Rust contract or create language-specific hidden tracking.

### TDD and review gates

For each of the five operations and for the guard, retain distinct Red, Green, and Refactor development commits. Red commits introduce focused failing protocol/client tests, Green commits implement the smallest passing change, and Refactor commits preserve behavior while improving structure. Each stage records exact focused command/output. Cross-language work gets its own Red/Green/Refactor evidence after 0043 is available. Complete traceability and documentation before review.

## Project Structure

### Documentation (this feature)

```text
specs/025-policy-controls/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── traceability.md
├── quickstart.md
├── contracts/
│   ├── operations.md
│   └── usage-allocation-guard.md
└── tasks.md
```

### Implementation and verification locations

```text
crates/kmipkit-protocol/src/{get_constraints,get_usage_allocation,set_constraints,set_defaults,set_endpoint_role}.rs
crates/kmipkit-protocol/src/lib.rs
crates/kmipkit-protocol/tests/unit/{get_constraints,get_usage_allocation,set_constraints,set_defaults,set_endpoint_role}_tests.rs
crates/kmipkit-client/src/{execute,lib}.rs
crates/kmipkit-client/tests/unit/{get_constraints,get_usage_allocation,set_constraints,set_defaults,set_endpoint_role}_execution_tests.rs
crates/kmipkit-client/tests/unit/usage_allocation_guard_tests.rs
crates/kmipkit/src/lib.rs
crates/kmipkit-protocol/tests/attribute_operations_guide_examples.rs
specification/catalog/kmip-2.1.json
specification/api/public-api.json
crates/kmipkit-ffi/src/lib.rs
bindings/c/include/kmipkit.h (generated)
bindings/c/tests/policy_controls.c
bindings/java/native/kmipkit_jni.cpp
bindings/java/src/main/java/org/kmipkit/PolicyControls.java
bindings/java/src/test/java/org/kmipkit/PolicyControlsParityTest.java
bindings/python/src/kmipkit/policy_controls.py
bindings/python/tests/test_policy_controls.py
docs/user-guide/en/policy-controls.md
docs/user-guide/es/controles-de-uso.md
docs/development/kmip-interoperability.md
```

**Structure Decision**: Extend the existing protocol/client/facade layers. Operation payloads stay isolated; the guard belongs in the Rust client layer because it combines caller allocation context, typed request identity, and transport-delivery evidence. Use the existing catalog, manifest, generators, and test fake. Keep shared dispatcher edits coordinated and language artifacts behind 0043.

## Complexity Tracking

No constitution violation or additional service/crate is proposed.
