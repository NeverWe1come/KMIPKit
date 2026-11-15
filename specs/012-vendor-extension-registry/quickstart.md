# KMIPKit Vendor Extension Registry Quickstart

This is the implementation verification contract; commands become runnable as the feature is implemented.

## Prerequisites

- Rust 1.94 and the repository-pinned C toolchain.
- Java 17 and Gradle.
- Python 3.12 with CFFI/Maturin development extras.
- A built KMIPKit native library.
- No KMIP server is required for registry/schema unit and fake-transport scenarios.

## Scenarios

1. Create a client configuration with vendor identifier, extension name/version, compatibility range, exact discriminator, and data-only TTLV schema.
2. Construct two TTLV subtrees, validate each through the registry, choose an explicit Criticality Indicator for each request use, attach both sealed values in order to one Rust `ClientBatchItem`, and verify that fake transport observes two serialized Message Extension structures in that same order.
3. Decode a fake response containing a matching Message Extension and inspect both the typed projection and unchanged generic subtree.
4. Send an unknown critical extension and assert KMIPKIT-0007 rejects the message; send an unknown non-critical extension and assert it remains losslessly inspectable.
5. Register two client definitions and assert registry isolation and deterministic list/map metadata.
6. Repeat shared fixtures through C, Java, and Python and compare normalized outcomes.
7. Include secret payload data and assert errors/debug/log output never reveal it.
8. Verify omission is impossible at the typed request API and that KMIPKit never selects a criticality default.
9. Inspect a payload containing two distinct registered discriminator paths whose complete schemas both pass; assert all four adapters return an unrecognized generic value with no registration-order winner.
10. Configure ExtensionRegistryLimits below defaults and above defaults only for fields whose default is below the hard maximum; for every field, exercise default, hard maximum, and hard maximum plus one, then confirm identical deterministic outcomes without partial state or overflow in Rust, C, Java, and Python.
11. Send a secret-bearing typed extension through fake transport with multiple partial writes, then exercise pre-send, partial-write, and response-started failures; assert zeroizing-owner lifetime/drop, delivery state, no retry, and redacted diagnostics.
12. Use a wide payload with a registered discriminator at the final child, then repeat a path tag; assert the shared index finds the unique final match, treats the repeated-tag path as non-matching, preserves original child order, and stays within the comparison budget in all four adapters.
13. Use a repeated enum field with the largest allowed-value set and ordered repeated fields with the largest edge set; assert binary membership bounds, one order-edge pass, stable errors, and equivalent outcomes across Rust, C, Java, and Python.

## Validation commands

After implementation, run these focused checks from the repository root:

- cargo test -p kmipkit-protocol extension
- cargo test -p kmipkit-client extension
- cargo test -p kmipkit-ffi extension
- python tools/api_manifest/generate.py --check
- python -m pytest bindings/python/tests
- ./gradlew -p bindings/java test
- cargo fmt --all --check
- cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
- cargo test --workspace --all-features --locked

A real C consumer and shared cross-language fixtures are mandatory; a Rust-only pass does not close this feature.
