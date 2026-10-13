# Quickstart: Validate Opaque Object Register context

This guide describes implementation acceptance scenarios. The Rust protocol and fake-transport examples are runnable on this feature branch. C, Java, and Python parity commands become runnable after KMIPKIT-0043's public API manifest is integrated into `release/1.0.0` and the generated binding changes are added here.

## Generic opaque registration

1. Construct the Opaque Object tag with a Structure value containing Opaque Data Type (Enumeration) and Opaque Data Value (Byte String).
2. Build a Register request with no crypto-context selection and no crypto attributes.
3. Confirm the serialized request preserves the caller's child fields and Attributes and performs one exchange.

Expected result: local validation succeeds and the server receives the ordinary Table 291 request.

## Managed cryptographic opaque registration

1. Construct the same valid Opaque Object.
2. Select Managed Cryptographic Object context.
3. Supply Cryptographic Usage Mask, Cryptographic Algorithm, and Cryptographic Length.
4. Confirm all supplied attribute values and order are preserved in the Table 291 request.

Expected result: local validation succeeds without adding a context marker to the wire.

## Negative preflight cases

Repeat with each of these mutations:

- Remove Cryptographic Usage Mask.
- Remove Cryptographic Algorithm.
- Remove Cryptographic Length.
- Supply Algorithm without Length or Length without Algorithm.
- Use the wrong outer Object tag, remove Opaque Data Type, or encode it with a non-Enumeration TTLV type.
- Remove Opaque Data Value or encode it with a non-Byte String TTLV type.
- Select Opaque Object context for another Object Type.

Expected result: each request fails before the transport writes any bytes and reports NotSent.

## Verification commands

- Run `cargo test -p kmipkit-protocol --lib register_operation_tests` and `cargo test -p kmipkit-client --lib register_execution_tests` on the implementation branch.
- Run the generated C, Java, and Python parity tests after the public API manifest is integrated.
- Run the repository-required format, Clippy, workspace tests, coverage, security, and generated-artifact checks.
