# ADR-0006: Three-level public API and bindings

Status: Accepted
Date: 2026-10-03

## Context

Beginners need a simple API while protocol experts and vendor integrations need
complete control. Four 1.0 language surfaces must remain equivalent.

## Decision

Expose high level builders, complete typed KMIP models, and generic valid TTLV.
Use a generated opaque-handle C ABI. Build Java 17 with a minimal JNI bridge
and Gradle Kotlin DSL. Build Python 3.12 with CFFI and Maturin. Generate
mechanical surfaces and hand-design idiomatic facades.

## Consequences

The library can hide KMIP structure without hiding capability. Parity becomes a
release gate. Native packaging, lifecycle, errors, typing, and compatibility
must be tested independently in every language.

## Alternatives considered

Only a generic API would expose too much complexity. Only a high level API
would block extensions and uncommon operations. Fully hand-written bindings
would drift; fully generated bindings would be awkward.
