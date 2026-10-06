# Cross-Language Ownership Contract

## Rust

Rust callers own constructed values. ValidatedExtensionValue owns its TTLV payload and redacts default formatting. Borrowed views cannot outlive the owner. KMIPKit zeroizes the initialized/current payload allocation on drop under the existing TTLV Value contract.

## C

C receives fixed-width stable symbols with kmipkit_ prefix and opaque handles. Constructors return stable error codes and either a complete handle or no handle. The API documents retain/release/drop ownership and does not expose payload bytes through errors or logs. Unsafe code is isolated in kmipkit-ffi and each unsafe block states its safety invariant.

## Java

Java 17 uses the JNI bridge. Native handles have explicit close/lifecycle semantics and errors map to stable categories. JNI/runtime copies may outlive the native owner and are outside KMIPKit zeroization guarantees.

## Python

Python 3.12 uses CFFI/Maturin. Native handles have deterministic close/context-manager semantics. Python-managed bytes/objects are outside KMIPKit zeroization guarantees. Errors do not include payload reprs.

## Parity

The same definitions, malformed schemas, matching/mismatch values, explicit Criticality Indicator selections, secret-bearing payloads, and unknown fields are exercised in every adapter. Adapters produce the same success/failure category, extension identity, normalized generic TTLV structure, outbound Message Extension fields, metadata, and preservation result.
