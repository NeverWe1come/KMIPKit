# Architecture decision records

ADRs preserve why KMIPKit chose an architectural direction. Accepted records
are immutable except for clerical corrections. A later ADR supersedes an old
one and links both directions.

## Status values

- Proposed
- Accepted
- Rejected
- Superseded by ADR-NNNN

## Index

- [ADR-0001: Rust core and stable C ABI](0001-rust-core-and-c-abi.md)
- [ADR-0002: KMIP 2.1 release scope](0002-kmip-21-release-scope.md)
- [ADR-0003: Layered Cargo workspace](0003-layered-cargo-workspace.md)
- [ADR-0004: Normative catalog and generated code](0004-normative-catalog-and-generation.md)
- [ADR-0005: Synchronous transports and TLS policy](0005-transport-and-tls.md)
- [ADR-0006: Three-level public API and bindings](0006-public-api-and-bindings.md)
- [ADR-0007: Vendor extension model](0007-vendor-extension-model.md)
- [ADR-0008: SDD, TDD, branches, and agents](0008-development-workflow.md)
- [ADR-0009: Conformance and pinned OASIS sources](0009-conformance-and-oasis-sources.md)
- [ADR-0010: Tag allocation precedence for generic TTLV](0010-tag-allocation-precedence.md)
- [ADR-0011: Reject received Reserved TTLV Tags](0011-reserved-tag-decoding-policy.md) (Accepted)
- [ADR-0012: Conditional caller-requested TTLV wire encoding](0012-caller-requested-wire-encoding-policy.md) (Accepted)
- [ADR-0013: Client extension registry ownership](0013-client-extension-registry-ownership.md) (Accepted)
- [ADR-0014: Public low-level transport exchange contract](0014-public-transport-exchange-contract.md) (Accepted)
- [ADR-0015: Tokio worker for synchronous TLS and HTTPS transports](0015-asynchronous-transport-worker.md) (Accepted; resolver decision superseded by ADR-0016)
- [ADR-0016: Preserve native system name-resolution policy](0016-native-system-name-resolution.md) (Accepted)
- [ADR-0017: Single-dependency Rust facade](0017-single-dependency-rust-facade.md) (Accepted)
- [ADR-0018: KMIP 2.1 multi-encoding scope for 1.0](0018-multi-encoding-1-0-scope.md) (Accepted)
- [ADR-0019: File-backed TLS hot-reload](0019-file-backed-tls-hot-reload.md) (Proposed)
- [ADR-0020: TLS 1.3 session resumption without 0-RTT and HTTPS idle reconnect](0020-tls13-session-resumption-and-keepalive.md) (Accepted)
- [ADR-0021: Server-to-Client operations and role inversion scope for KMIPKit](0021-server-to-client-operations-scope.md) (Accepted)
- [ADR-0022: Multi-version modern KMIP architecture](0022-multi-version-modern-kmip-architecture.md) (Accepted)
- [ADR-0023: Cloud KMS stateless sidecar proxy architecture](0023-cloud-kms-stateless-proxy.md) (Accepted)
- [ADR-0024: Complete fluent API boundary](0024-complete-fluent-api-boundary.md) (Proposed)

## Template

```text
# ADR-NNNN: Title
Status: Proposed
Date: YYYY-MM-DD

## Context
## Decision
## Consequences
## Alternatives considered
```
