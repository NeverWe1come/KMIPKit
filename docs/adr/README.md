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
- [ADR-0011: Reject received Reserved TTLV Tags](0011-reserved-tag-decoding-policy.md) (Proposed)

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
