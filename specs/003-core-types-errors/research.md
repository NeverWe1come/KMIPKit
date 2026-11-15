# Research: KMIP Shared Error Contract

## Decision 1: Keep types in existing workspace layers

Decision: Put KMIP result semantics in kmipkit-protocol, delivery evidence in kmipkit-transport, aggregation in kmipkit-client, and supported Rust exports in kmipkit.

Rationale: This preserves the accepted layer graph and prevents transport from owning KMIP operation semantics. A new common crate would expand the accepted architecture without being needed for the limited shared contract.

Alternatives considered: Put all errors in protocol (would pull KMIP operation concepts into transport) or add a workspace error crate (unnecessary dependency and release surface).

## Decision 2: Use open numeric wrappers for protocol enumerations

Decision: Retain raw 32-bit Enumeration values and offer known-value interpretation through the generated normative catalog.

Rationale: Consumers need exact forward-compatible values. Hardcoded parallel tables would diverge from the catalog, which is the approved generation input.

Alternatives considered: Closed Rust enums (cannot retain unknown values) or manually duplicated numeric constants (violates the generation rule).

Dependency: The generated normative catalog from KMIPKIT-0002 must be merged before implementation. The current release tip does not include that PR; raw values alone are not enough to meet all defined-value and validation requirements.

## Decision 3: Separate display from explicit diagnostic inspection

Decision: Preserve safe cause categories, but discard arbitrary source text and payloads before retention. Expose only safe category wrappers through the public source chain; omit Result Message, secrets, and raw bodies from default Display and library-generated logs.

Rationale: Arbitrary error and server text is untrusted and may echo sensitive material. Retaining an arbitrary `Error` as a public source would leave that text reachable even if outer Display/Debug implementations redact it. Safe cause categories preserve the failure layer without retaining untrusted strings or payloads.

Alternatives considered: Including full cause/message text in Display (unsafe default), exposing arbitrary original causes through `source()` (still exposes secret-bearing text), or discarding all cause categories (loses useful diagnostic evidence).

## Decision 4: Report delivery evidence without retry policy

Decision: Report NotSent, PossiblySent, or ResponseStarted for local request-handling failures without a complete KMIP operation result. Never infer retry safety.

Rationale: An application can make an informed decision without the library repeating a potentially state-changing request.

Alternatives considered: Automatic retries (outside accepted product scope) or one ambiguous state for all failures (not useful to callers).

## Dependency and validation

No external information is required beyond the pinned OASIS KMIP 2.1 source and approved repository architecture. OASIS KMIP Specification v2.1 §§9.17–9.19 and §§11.46–11.47 define the result fields and assigned values. The checked-in copy remains authoritative for implementation.
