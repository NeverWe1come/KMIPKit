# Cross-Language Ownership Contract

## Rust

Rust callers own constructed values. ValidatedExtensionValue owns its TTLV payload and redacts default formatting. Borrowed views cannot outlive the owner. KMIPKit zeroizes the initialized/current payload allocation on drop under the existing TTLV Value contract.

The generic TTLV surface uses owned `TtlvValue`, `TtlvItem`, and `TtlvStructure` values for construction. `TtlvValueView`, `TtlvItemView`, and `TtlvStructureView` are read-only borrowed views; foreign-language view wrappers retain a strong reference to their owning wrapper and expose no mutation or payload ownership transfer. Scalar getters return copies. Variable-length payloads are read as indexed octets so a caller can preserve the exact bytes without a manifest-level variable-output buffer.

## C

C receives fixed-width stable symbols with kmipkit_ prefix and opaque handles. Constructors return stable error codes and either a complete handle or no handle. Functions reject NULL handles and live KMIPKit handles of the wrong declared kind with `invalid_input` before dereference. Every non-NULL input handle must be live and have the declared type; use-after-release, dangling pointers, and foreign pointers violate the caller precondition and cannot be safely probed. The API documents retain/release/drop ownership and does not expose payload bytes through errors or logs. Unsafe code is isolated in kmipkit-ffi and each unsafe block states its safety invariant.

C-owned value handles and C view handles have different semantics: a view handle is library-owned and must be released, but is a read-only view rather than a payload copy. Each C view pins/retains its backing value or structure until its release function drops that reference; the caller may release the original owner while the view remains live. Variable-length inputs use `const uint8_t *` plus `uint64_t` byte length and are bounded before any read or copy; variable-length view outputs use length/index accessors.

## Java

Java 17 uses the JNI bridge. Native handles have explicit close/lifecycle semantics; wrappers reject use of a closed handle with the stable `invalid_input` category before JNI. The JNI bridge zeroizes its temporary native byte buffer after constructing BigInteger, TextString, or ByteString values. Copies retained by Java arrays and objects are owned by the Java runtime and remain outside KMIPKit zeroization guarantees.

## Python

Python 3.12 uses CFFI/Maturin. Native handles have deterministic close/context-manager semantics; wrappers reject use of a closed handle with the stable `invalid_input` category before CFFI. Python-managed bytes/objects are outside KMIPKit zeroization guarantees. Errors do not include payload reprs.

## Parity

The same definitions, malformed schemas, matching/mismatch values, explicit Criticality Indicator selections, secret-bearing payloads, and unknown fields are exercised in every adapter. Adapters produce the same success/failure category, extension identity, normalized generic TTLV structure, outbound Message Extension fields, metadata, and preservation result.
