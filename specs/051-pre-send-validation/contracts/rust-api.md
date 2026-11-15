# KMIPKIT-0051 Rust API Contract

This contract describes the additive public behavior; final item names are fixed in the reviewed implementation and API manifest.

## Validation report

The protocol API exposes a `ValidationReport` with a stable, deterministic list of `ValidationIssue`s and an `is_valid()` query. `is_valid()` is false when any issue has error severity; warnings do not make a request invalid. Each issue exposes a stable code, schema-relative path, severity, and source reference. No issue exposes or formats the invalid value or raw TTLV. An explicitly requested but unsupported schema produces a value-free error-severity `UnsupportedSchema` issue; generic validation without an operation-schema identity checks only generic TTLV constraints.

## Typed requests

All 56 typed client-initiated request payload types support borrowed validation through one consistent public method contract. Validation does not consume or clone the request. The API inventory records operation types that are already invariant-safe and proves that construction or the existing serializer enforces their OASIS constraints.

## Generic structures

Generic structures are validated only against a caller-supplied, explicitly supported schema identity. With no operation-schema identity, the validator checks only generic TTLV constraints. An explicitly requested but unsupported schema is reported as an error and is not guessed from numeric tags. Unknown values and fields accepted by generic/extension paths remain accessible and unchanged; the report must not claim an unrecognized field conforms to a closed OASIS structure.

## Client execution

`ClientBatch` exposes the same validation report used by execution. `Client::execute` runs it before message encoding or transport exchange. Error-severity findings return the existing sanitized client validation error with `RequestDeliveryState::NotSent`; no transport exchange or write occurs. Warnings do not block.

## Compatibility

Existing request constructors, batch methods, generic TTLV access, wire order, and successful bytes remain available and unchanged. No server policy, profile-conformance claim, transport option, retry, or server-initiated operation is added.
