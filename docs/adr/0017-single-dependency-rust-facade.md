# ADR-0017: Single-dependency Rust facade

Status: Accepted
Date: 2026-11-05

## Context

KMIPKit publishes layered Rust crates for protocol models, TTLV, transport, and client execution. This separation supports reuse and independent testing (ADR-0003), but forces application consumers to declare several KMIPKit crates directly when they need the complete supported client API. The top-level `kmipkit` package is the intended high-level entry point and already depends on those layers.

## Decision

The `kmipkit` crate re-exports the complete protocol and TTLV public namespaces and exposes the client and transport configuration types needed to construct its supported client. Existing root exports remain available. The facade does not re-export `Transport::exchange`, raw response byte types, or concrete low-level transport adapters. The generic TTLV model and bounded decoder are public; generic outbound encoding remains unavailable and typed request encoding continues through the client execution boundary.

All newly reachable declarations remain subject to the exact Rust export inventory in KMIPKIT-0043. Alias records map to the existing logical capability and retain its four-language mapping semantics; declarations without distinct cross-language behavior receive individual reviewed exclusions. This ADR does not introduce a Rust-only parity exception or new protocol capability.

## Consequences

Rust consumers can depend directly on `kmipkit` alone for the supported client surface. The facade has a larger semver surface and the API manifest must track every reachable path. Existing layer crates remain independently publishable and usable. The export audit and manifest generator remain release controls.

## Alternatives considered

- Keep `kmipkit` minimal and require several direct crate dependencies: rejected because it makes the high-level package insufficient for an ordinary client consumer.
- Re-export every layer at the root: rejected because it creates a flat, collision-prone namespace; named `protocol` and `ttlv` namespaces preserve crate boundaries.
- Expose arbitrary transport exchange through the facade: rejected because it bypasses typed client execution and widens the raw-byte contract.
