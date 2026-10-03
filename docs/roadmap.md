# KMIPKit roadmap

The roadmap expresses dependency order. Each item becomes one or more approved
Spec Kit specifications before implementation.

## Phase 0: repository governance

- Constitution and Spec Kit installation (complete; maintainer owned).
- Community, security, contribution, ADR, and agent documentation.
- Branch protections, CODEOWNERS, DCO, and draft PR workflow.
- CI skeleton and pinned toolchains.

## Phase 1: stable foundation

Parallel feature development must wait until this phase is accepted.

- Cargo workspace skeleton (initialized); common automation remains pending.
- Normative source inventory and machine-readable catalog.
- Generated tags, types, enumerations, and value metadata.
- Lossless generic TTLV tree.
- Strict encoder and decoder with resource limits.
- Protocol error model and validation framework.
- Message headers, batch model, correlation, and unknown value preservation.
- Transport abstraction and deterministic fake transport.
- Raw TLS and HTTPS vertical skeleton with mTLS.
- Synchronous request/response client skeleton.
- Minimal stable C ABI vertical slice.
- Coverage, conformance, compatibility, and supply-chain CI gates.

## Phase 2: typed KMIP protocol

- Managed objects and attributes.
- Message credentials.
- All client initiated request and response types.
- Profile-specific validation.
- KMIP protocol asynchronous outcome model.
- Official and derived conformance fixtures.

Work is split into independent vertical specifications after shared models have
stabilized.

## Phase 3: complete public APIs

- Rust high level builders for every operation.
- Complete C ABI and generated header surface.
- Generic TTLV public escape hatch.
- Batch builder and per-item outcomes.
- Secret result types and lifecycle controls.
- Structured diagnostics.

## Phase 4: Java and Python

- JNI bridge and Java 17 API.
- CFFI and Python 3.12 API.
- High level, typed, and TTLV parity.
- Native package loading and installation tests.
- Maven Central and PyPI release pipelines in dry-run mode.

## Phase 5: interoperability and release hardening

- Automated integration with an external open source KMIP 2.1 server.
- Manual or automated integration with a second independent implementation.
- Server compatibility matrix.
- Fuzzing, Miri, sanitizers, and extended negative testing.
- Performance baselines and regression analysis.
- Independent security review.
- Complete English and Spanish documentation.
- Reproducible packages, SBOM, provenance, and signed artifacts.
- `1.0.0` release readiness review.

## Phase 6: after 1.0

- 1.1.0 server initiated operations.
- Additional idiomatic adapters, in order: Go, C++, C#, and JavaScript.
- Declarative vendor extension SDK.
- Optional compiled extension packages.
- JSON and XML encodings.
- Async/await client.
- Connection pools, explicit retry policies, and failover components.
- Additional TLS identity providers, platforms, and proxy support.
