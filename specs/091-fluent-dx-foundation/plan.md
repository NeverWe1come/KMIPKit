# Implementation Plan: KMIPKit Fluent API Foundation

**Branch**: `feature/KMIPKIT-0091-fluent-dx`

**Base**: `release/1.0.0` at `9692057c9ade474fb0c4e0b7130539da368eb794`

**Spec**: [spec.md](spec.md)

**Status**: Draft; implementation is gated on human approval of the spec.

## Summary

Add a four-language fluent path for Create Symmetric Key, Get by Unique
Identifier, and Destroy by Unique Identifier. Rust owns semantic construction
and validation. The existing typed requests and client execution methods stay
the only network route; C exposes adapters through the stable ABI, and Java
and Python provide idiomatic wrappers.

## Technical Context

- **Languages**: Rust 2024, C ABI, Java 17/JNI, Python 3.12/CFFI.
- **Dependencies**: Existing KMIPKit protocol/client/FFI crates, generated
  named-value and attribute catalogs, and binding wrappers.
- **Storage**: None.
- **Testing**: Rust unit and integration tests, compiled C consumers, Java/JNI
  tests, Python tests, and the existing loopback TLS 1.3/mTLS fixture.
- **Target**: Existing KMIPKit-supported platforms and consumers.
- **Constraints**: Preserve public APIs and stable ABI; KMIP 2.1 default;
  no arbitrary TTLV request transmission, automatic polling, or retries;
  update manifest and generated exports with the generator.
- **Scope**: Foundation delivery only. Later operation families each require
  their own approved specification.

## Constitution Check

- **Spec gate**: This draft must be approved before production changes.
- **Normative traceability**: `traceability.md` maps each requirement to its
  KMIP/project source, planned implementation location, and verification.
- **TDD**: Record focused Red, Green, and Refactor evidence in separate
  DCO-signed commits.
- **Architecture**: Protocol semantics remain in Rust; FFI unsafe code stays
  within `kmipkit-ffi`; foreign bindings reuse the existing C ABI.
- **Security**: Use typed execution routes, existing zeroizing owners and
  redaction policy; keep cryptographic policy explicit.
- **Review**: Run all required checks and remote CI before requesting human
  review. Human approval/merge remains a maintainer action.

## Project Structure

```text
crates/kmipkit-protocol/src/                 # fluent attribute/request construction
crates/kmipkit-protocol/tests/               # semantic validation tests
crates/kmipkit-client/src/                   # bound request builders/dispatcher
crates/kmipkit-client/tests/                 # execution and typed outcome tests
crates/kmipkit-ffi/src/                      # stable C ABI adapters
bindings/c/include/                           # public C declarations
bindings/java/src/main/java/org/kmipkit/      # Java facade and builders
bindings/python/src/kmipkit/                  # Python facade and builders
tests/consumers/0053-parity/                  # four-language TLS journey
specification/api/public-api.json             # generated public API manifest
```

**Structure Decision**: Extend the existing Rust workspace, C ABI, and foreign
binding packages. Do not introduce another protocol implementation or
transport boundary.

## Execution Sequence

See [tasks.md](tasks.md) for Red/Green/Refactor tasks. The detailed
implementation checklist is maintained in
`docs/superpowers/plans/2026-11-10-kmipkit-0091-fluent-dx-foundation.md`.
