# Shared Capability Dependency Matrix

This matrix records the shared non-operation specifications whose public types or behaviors can appear in the full 1.0 API manifest. “Approved” here means the current release-line spec explicitly records Approved/Accepted status and its approval/acceptance evidence. Approval to implement a draft does not satisfy that gate.

Status snapshot: active `release/1.0.0` base `fd6b7782`, checked 2026-10-10. Revalidate these gates at implementation start.

| Feature | Shared capability area | Current recorded status | KMIPKIT-0043 gate |
|---|---|---|---|
| KMIPKIT-0003 | Core types, errors, stable values | Approved under direct maintainer authorization | Map only implemented Rust declarations; verify four-language error/value mappings. |
| KMIPKIT-0004 | Generic TTLV model and unknown-value preservation | Approved; design and implementation merged | Map the accepted generic-TTLV level and its exact limits/losslessness. |
| KMIPKIT-0005 | TTLV wire codec | Approved under recorded delegated authorization | Map only the public codec surface actually accepted by 0005; preserve its sole-send-callsite gate. |
| KMIPKIT-0006 | Message and batch model | Approved for implementation under delegated authorization | Map typed message/batch outcomes when the canonical implementation is present. |
| KMIPKIT-0007 | Typed client execution and Discover Versions | Approved for implementation with approval record | Map the approved client behavior and Discover Versions; keep explicit execution and delivery-state semantics. |
| KMIPKIT-0008 | Credentials and attestation models | Approved for implementation with approval record | Map approved in-memory models only; preserve its no-credential-send boundary. |
| KMIPKIT-0009 | Poll, Cancel, Process, asynchronous follow-up and Query Asynchronous Requests | Draft; autonomous implementation authorization is explicitly not spec approval; DISC-039 remains relevant | Keep asynchronous follow-up APIs blocked until formal approval, open source gates are resolved, and the Rust API exists. Pending outcome behavior already owned by another approved contract may be mapped only to that approved scope. |
| KMIPKIT-0010 | Profile applicability, validation, and support claims | Draft | Keep profile-validation APIs and profile capability claims blocked until formal approval and evidence. |
| KMIPKIT-0011 | Dependency policy and supply-chain constraints | Approved | Apply required dependency metadata/checks; do not add a user-facing capability beyond the approved contract. |
| KMIPKIT-0012 | Vendor extension registry and registry API slice | Approved for implementation under standing maintainer authorization | Preserve registry declarations and behavior while migrating v1 to v2. |
| KMIPKIT-0013 | Production TLS/HTTPS transport configuration | Accepted | Map only public transport construction/configuration already accepted and implemented; preserve TLS and no-injection boundaries. |

## Accepted ADR-defined public declaration

The public Rust-only exchange declaration is not a KMIP operation and therefore has no operation feature-spec owner. Its accepted authority is ADR-0003 plus ADR-0014: `kmipkit-transport::Transport::exchange` and the associated `TransportResponse` contract are part of the published Rust crate compatibility surface. The manifest MUST identify this as the sole `rust-only-adr` parity scope and cite the ADR acceptance evidence. Its caller-owned request bytes, caller-supplied response cap, delivery-aware errors, response redaction/zeroization, no facade re-export, and no arbitrary transport injection into `kmipkit-client::Client::execute` remain exactly as those ADRs define them. This record does not unblock or redefine any KMIP operation API.

## Use in the manifest

For a formally approved/accepted feature, the capability record references `featureSpecId`, requirement IDs, and repository-local approval evidence. For a draft, the record may keep a `candidateSpecId` and `blockedReason`, but it is not an available capability. A missing implementation keeps the capability declared-but-not-verified as appropriate; it never justifies advertising functionality that is not present.

The 0009 and 0010 gates are independent of the 56-operation `client_1_0` inventory and apply to the corresponding shared async/profile capabilities. At 1.0 release review, every in-scope shared capability must have an approved owner and passing parity evidence. Do not infer that a capability is ready merely because an existing spec number appears in a catalog, roadmap, implementation branch, or merged code PR.

## T013 audited declaration slice

The manifest currently has 101 cross-language declaration rows whose Rust mapping paths begin with `kmipkit_ttlv::`: 76 generic-model rows are owned by approved KMIPKIT-0004 and 25 `CodecLimits` rows are owned by approved KMIPKIT-0005. These rows are distributed across the Rust, C, Java, and Python package families; they are not the count of declarations in the Rust TTLV package. The `rust-kmipkit-ttlv` package itself has 95 rows: 24 `declared` and 71 `blocked`. The four-language mappings remain declared pending parity verification. Each row links to applicable owner requirement IDs and KMIPKIT-0043 inventory requirements. Existing type and function rows retained from the KMIPKIT-0012 registry slice remain unchanged.

This is a bounded reconciliation, not completion of the five-crate inventory audit. Other observed declarations remain blocked where their approved owner mapping or implementation/parity evidence has not been reconciled. Draft-owned KMIPKIT-0009 and KMIPKIT-0010 surfaces remain blocked, and ADR-0014 remains the only Rust-only exception.
