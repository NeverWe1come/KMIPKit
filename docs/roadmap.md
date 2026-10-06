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
- `KMIPKIT-0007-client-execution`: paired response validation; mixed synchronous/asynchronous response handling under `KMIPKIT-REQ-SPEC-8-003-002`; outbound Asynchronous Indicator validation under Table 432; assigned outbound Batch Error Continuation values under §9.6 and Table 435; response-size enforcement and large-response policy under `KMIPKIT-REQ-SPEC-9.12-001-002/-003`; and ADR-0002's KMIP 2.1-only request/response version enforcement, documented as the §9.16 scope exception `KMIPKIT-DISC-022`. `KMIPKIT-DISC-043` records the §9.6/Table 435 value-range discrepancy separately from `KMIPKIT-DISC-001`; delegated decision `KMIPKIT-DEC-002` accepts assigned values for the initial client while raw Enumeration values remain preserved. Acceptance tests cover a deterministic fake-transport batch containing completed and Pending items, accepting Pending only when the request indicator permits asynchronous results; a property check accepting protocol-version pairs iff they equal `(2, 1)`, plus representative non-2.1 mismatches and signed-32-bit boundary cases; assigned and extension-range Asynchronous Indicator acceptance, assigned Batch Error Continuation acceptance, rejection of Batch Error Continuation extension-range outbound values under the project policy, raw-value preservation, exact/over-limit response-size boundaries, and configured limits for operations likely to return large responses. The set of operations likely to return large responses awaits the Phase 2 operation inventory.
- Minimal stable C ABI vertical slice.
- Coverage, conformance, compatibility, and supply-chain CI gates.
- `KMIPKIT-0011-dependency-policy-gates`: finite reviewed Cargo license/source
  policy, advisory checks for both root and fuzz workspaces, exact expiring
  exceptions, and pull-request/scheduled CI. The approved specification and
  implementation were merged by PR #37 at `d4582e2`; release signing, SBOM,
  provenance, and branch protection remain separate.

## Phase 2: typed KMIP protocol

- Managed objects and attributes.
- `KMIPKIT-0008-credentials-attestation`: credential construction and message credential models; authentication policy; Device Identifier; hashed-password algorithm/timestamp/default rules; conditional credential/header fields; and attestation, including truthful Attestation Capable Indicator behavior. Add exact-clause tests for credential variants and required/conditional fields, authentication policy, Device Identifier handling, hash/default behavior, and indicator truthfulness against supported Attestation Credential creation capability.
- `KMIPKIT-0009-asynchronous-operations`: client-initiated Poll/Cancel/Process models and explicit pending-operation follow-up using the exact Asynchronous Correlation Value bytes. Tests verify byte-for-byte preservation in both Poll and Cancel requests and no automatic polling or retry. It also owns `KMIPKIT-DISC-039`/§6.1.41 Query Asynchronous Requests response mapping: review the exact normative source conflict, document and test the decision, and assume no resolution in KMIPKIT-0006.
- `KMIPKIT-0010-profile-conformance`: profile-specific requirement applicability, defaults, validation, and claims; each claimed profile must have exact OASIS Profile clause mapping and passing executable tests before any support claim.
- All client initiated request and response types.
- Profile-specific validation.
- KMIP protocol asynchronous outcome model.
- `KMIPKIT-0012-vendor-extension-registry`: immutable per-client registry,
  typed validated extension values, and generated Rust/C/Java/Python adapters.
  It implements ADR-0007's 1.0 extension commitment; dynamic executable
  plugins remain deferred. Complete it before public API parity is frozen.
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
