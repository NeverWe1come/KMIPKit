# KMIPKit language bindings

The bindings share the Rust implementation and C ABI. Start with the guide for
your language:

- [C](c/README.md)
- [Java](java/README.md)
- [Python](python/README.md)

Each binding uses the same documentation layout:

- `README.md`: introduction, first local example, and documentation links.
- `docs/installation.md`: prerequisites, source builds, and package output.
- `docs/native-library.md`: linking or loading native code and troubleshooting.
- `docs/testing.md`: local checks and shared fixtures.
- `examples/README.md`: available examples and how to run them.

The [architecture guide](../docs/architecture/ffi-and-bindings.md) covers shared
ownership and adapter responsibilities. The [user guides](../docs/README.md)
cover KMIP behavior. Keep shared explanations there and link to them from the
language guides.

Shared API contracts and fixtures live under `shared/`. Their presence and
package target declarations do not certify functional parity or KMIP conformance.
