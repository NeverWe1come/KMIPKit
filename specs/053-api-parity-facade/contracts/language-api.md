# KMIPKIT-0053 Language API Contract

## Shared behavior
- Rust remains the behavior source; foreign code calls Rust models, validation and client execution.
- Typed owners convert to/from generic TTLV without losing unknowns, repetitions, future raw values, order or vendor extensions.
- Diagnostics omit credentials, keys, payload values and KMIP bytes.
- C uses fixed-width values and opaque handles with release; Java uses AutoCloseable; Python supports close/context management.
- Operation, wire bytes, options, NotSent, async correlation, response presence, redaction and lifetime match Rust.

## C
Initialize outputs deterministically; document borrowed/consumed/owned inputs; panic-contain each export; never return pointers to temporary Rust values.

## Java
JNI transfers ownership once, releases partial handles on failure, and maps failures to sanitized exception categories. No toString or exception formats payload values.

## Python
Adopt handles once, release on conversion failure, provide idempotent close/context support, use existing synchronization and staging zeroization, and never return raw pointer integers.

## Rust
Keep existing source-compatible paths and the kmipkit-only facade. Do not expose low-level transport exchange. Response raw views borrow from their owner.
