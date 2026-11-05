# KMIPKIT-0049 Local Validation Guide

This guide describes the compiling Rust consumer that the implementation
must provide. It becomes executable as soon as the typed models land; the
test task keeps the code sample in sync with the real API.

## Construct a plaintext key object

1. Create explicit Key Format Type and Cryptographic Algorithm values.
2. Build Key Material using the caller's chosen format; do not infer an
   algorithm or key size from the bytes.
3. Add optional typed Attributes to Key Value when the chosen KMIP structure
   carries them.
4. Wrap Key Value in Key Block, then in the selected managed-object type.
5. Convert to an Item and pass it to the typed Register or Import helper. The
   Item carries its object-specific Structure tag (such as `Symmetric Key`),
   separate from the Object Type Enumeration; the request sends that Item
   directly without an additional wrapper.

The compiled example must assert exact child order and tag/value types from
the pinned OASIS table and verify the request through the fake transport.

## Construct wrapped material

1. Supply wrapped bytes and Key Wrapping Data, including required key
   identifiers and caller-selected parameters.
2. Keep the wrapping method, key information, IV/counter/nonce, and encoding
   option explicit.
3. Verify that the resulting Key Block carries a Byte String Key Value and
   preserves the supplied wrapping metadata.

KMIPKit does not perform the wrapping operation or retry the request.

## Validate locally

Run the focused Rust tests, API manifest generator check, export audit, format,
Clippy, and protocol tests specified in
`implementation-evidence.md`. After local implementation checks, run the
workspace coverage gate on the supported Windows toolchain. Hosted matrix
checks remain separate evidence and must be recorded when billing permits.
