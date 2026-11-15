# Research: KMIP 2.1 Remote RNG Operations

## Decision 1: Use the existing typed operation pipeline

Implement two request/response models admitted through the existing Rust client batch and single-exchange dispatch. This preserves common transport, error, Pending, delivery, and response handling.

## Decision 2: RNG Retrieve forwards a length and returns opaque secret bytes

Use the catalog Integer for required Data Length and required Byte String for response Data. Do not generate randomness locally, interpret the returned bytes, or impose an exact response length absent an explicit normative rule.

## Decision 3: RNG Seed forwards caller bytes and preserves a zero response length

Use required request Data bytes and required response Data Length. OASIS §6.1.50 and stable requirement KMIPKIT-REQ-SPEC-6.1.50-001 say a client SHALL NOT treat a response indicating unused seed input as an error. Therefore zero is a valid decoded result.

## Decision 4: Apply shared secrecy and asynchronous behavior

Treat seed input and returned RNG output as secret. Redact formatting/errors and zeroize KMIPKit-owned buffers. Preserve Success/Failure/Pending, correlation, unknown values, and delivery state without retry or implicit Poll.

## Decision 5: Stage bindings behind KMIPKIT-0043

Rust protocol/client modules do not wait on the public API feature. Manifest mapping, generated signatures, C/Java/Python adapters, and capability parity wait for 0043's canonical mapping to integrate.

## Evidence reviewed

Pinned OASIS Specification v2.1 §§6.1.49–6.1.50, Tables 316–321, §11.36, approved spec 027, and current client/protocol/secret contracts. The catalog links no direct official test case for either operation.
