# Register Opaque Object context contract

## Caller behavior

A Register request for Object Type Opaque Object starts in generic opaque context. The caller may select Managed Cryptographic Object context before request execution.

The context selector is local client metadata. It is never encoded as a KMIP field. The Object Type remains the standard Opaque Object Enumeration, the Attributes remain the caller's ordered Attributes structure, and the Object remains the standard Table 4 structure.

## Validation

All Opaque Object requests require:

- The required Any Object item has the Opaque Object tag for Object Type value 8 (OASIS §11.34/Table 468) and is represented as a Structure (OASIS §2.3/Table 4).
- Opaque Data Type child represented as Enumeration.
- Opaque Data Value child represented as Byte String.
- Unrecognized child items remain intact in their caller-supplied order; their values are not interpreted.

With Managed Cryptographic Object context, direct Attributes also require:

- Cryptographic Usage Mask.
- Cryptographic Algorithm.
- Cryptographic Length.

Algorithm and Length must appear together. Opaque Data Value is never interpreted as a Key Block.

## Failure and transport contract

Selecting Opaque Object context for another Object Type, omitting or mistyping a required Table 4 child, or omitting a required crypto attribute fails locally. The high-level call reports NotSent and performs zero transport writes. The error is sanitized and contains no opaque payload or raw KMIP body.

## Binding parity

Each supported language binding exposes the same default, opt-in choice, attribute-preservation behavior, and preflight error semantics. Public names and generated declarations are determined by the accepted API manifest.
