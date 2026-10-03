# ADR-0002: KMIP 2.1 release scope

Status: Accepted
Date: 2026-10-03

## Context

The complete KMIP family includes multiple versions, encodings, directions,
transports, and API execution models. A useful first stable release needs a
precise claim.

## Decision

Version 1.0 implements all KMIP 2.1 client-initiated operations with TTLV, raw
TLS, HTTPS/HTTP 1.1, batch, and KMIP protocol asynchronous behavior. It exposes
synchronous APIs in Rust, C, Java, and Python. Version 1.1 adds server-initiated
operations. JSON, XML, Rust async/await, and older KMIP versions are later work.

## Consequences

The 1.0 completeness claim is precise and testable. Architecture boundaries
must permit both message directions later without exposing incomplete 1.1 APIs.

## Alternatives considered

Implementing every encoding and direction before 1.0 would delay feedback and
multiply test and binding work. Supporting older versions would complicate
models before the 2.1 foundation is proven.
