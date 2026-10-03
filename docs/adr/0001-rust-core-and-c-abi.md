# ADR-0001: Rust core and stable C ABI

Status: Accepted
Date: 2026-10-03

## Context

KMIPKit must provide one KMIP implementation to many language ecosystems,
remain memory safe, and support native performance.

## Decision

Implement protocol, transport, client, and high level behavior in Rust. Expose
a versioned C ABI using opaque handles, fixed-width values, explicit ownership,
stable error codes, panic containment, and `kmipkit_` symbols. Concentrate all
project unsafe code in the FFI crate.

## Consequences

KMIP semantics remain centralized. Native packaging and ABI compatibility
become long-term responsibilities. Foreign adapters translate idioms and
resources instead of protocol rules.

## Alternatives considered

Independent native implementations would drift. A network service would add a
deployment boundary. WebAssembly would not provide the desired native ABI and
TLS integration for the initial targets.
