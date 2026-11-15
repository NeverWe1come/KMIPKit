# ADR-0002: KMIP 2.1 release scope

Status: Superseded by ADR-0018
Date: 2026-10-03

## Context

The complete KMIP family includes multiple versions, encodings, directions,
transports, and API execution models. A useful first stable release needs a
precise claim.

## Decision

This decision established the original 1.0 scope. Its encoding scope was
superseded on 2026-11-05 by [ADR-0018](0018-multi-encoding-1-0-scope.md).
Version 1.0 includes client-initiated operations, batch, and KMIP protocol
asynchronous behavior, with synchronous APIs in Rust, C, Java, and Python.
Version 1.1 adds server-initiated operations. Rust async/await and older KMIP
versions remain later work.

## Consequences

The original 1.0 completeness claim was precise and testable. ADR-0018 expands
its encoding set and preserves the client-initiated 1.0 boundary.

## Alternatives considered

Implementing every encoding and direction before 1.0 would delay feedback and
multiply test and binding work. Supporting older versions would complicate
models before the 2.1 foundation is proven.
