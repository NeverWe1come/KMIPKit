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
| KMIPKIT-0009 | Poll, Cancel, Process, asynchronous follow-up and Query Asynchronous Requests | Approved for implementation by explicit maintainer approval dated 2026-10-29; see `specs/009-asynchronous-operations/approval-record.md`. Query response interpretation is recorded as KMIPKIT-DEC-015; the Process Table 278 catalog requirement and owner review checklists remain open | T035 maps the four explicit one-exchange operations across Rust, C, Java, and Python. Keep the generic Query response and exact correlation bytes; the manifest remains `declared` and no complete conformance claim is made while catalog traceability and reviewer gates remain open. |
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

`reports/shared-rust-inventory-2026-10-16.json` is the generated reconciliation artifact for the current manifest snapshot. It is derived from the complete Rust export audit and manifest input; run `tools/api_manifest/audit_shared_rust_inventory.py` to regenerate it or add `--check` to verify it. The full audit observes 9,532 declarations across the five published Rust crates. It maps 245 declarations, including five transport declarations under the bounded ADR-0014 Rust-only exception, and records 22 approved exclusions. Every mapped declaration has an exact published Rust path, an Approved/Accepted owner, and linked owner requirements. This inventory records exact declaration ownership; it does not mark a capability four-language verified before its parity tasks pass. Each crate's observed, mapped, and still-blocked counts are:

| Published Rust crate | Observed | Mapped | Blocked |
| --- | ---: | ---: | ---: |
| `kmipkit-ttlv` | 331 | 36 | 295 |
| `kmipkit-protocol` | 5,871 | 75 | 5,774 |
| `kmipkit-transport` | 365 | 46 | 319 |
| `kmipkit-client` | 1,035 | 66 | 969 |
| `kmipkit` facade | 1,930 | 22 | 1,908 |
| **Total** | **9,532** | **245** | **9,265** |

The approved shared capabilities mapped by this slice are generic TTLV model (KMIPKIT-0004, 30 Rust declarations), TTLV wire codec (KMIPKIT-0005, 6), vendor extension registry (KMIPKIT-0012, 84), Discover Versions request-item construction (KMIPKIT-0007, 1), FFI runtime metadata (KMIPKIT-0043, 10), client batch model (KMIPKIT-0007, 10), client execution (KMIPKIT-0007, 28), server-generated creation (KMIPKIT-0007, 9), API-level discovery (KMIPKIT-0043, 1), production transport (KMIPKIT-0013, 53), and opaque Register context (KMIPKIT-0044, 8). Five transport declarations retain the sole `rust-only-adr` classification under ADR-0014. Capability mappings are derived from retained declaration rows and checked against each language's exact published export names. The Discover Versions request-item builder is mapped in Rust, C, Java, and Python; this does not implement or unblock the operation's execution and response API. Opaque Register context remains declared, not parity-verified. No other declaration is classified Rust-only.

The `StructureView` type, its four-language view wrappers, and their handle/resource release methods are attributed to generic TTLV model requirements KMIPKIT-0004-FR-006, FR-009, FR-010, and FR-012. Registry recognition functions that return a view remain owned by KMIPKIT-0012. `ClientBatchItem` remains owned by KMIPKIT-0012 for extension attachment and inspection, with KMIPKIT-0007-FR-001 recorded for its typed request-item base contract. Its four Discover Versions builder exports have their own KMIPKIT-0007-owned capability and requirements FR-001/FR-002.

Thirty-two observed binding declarations remain blocked without signatures or language mappings: 18 C release functions have no callable Rust `Drop` export; 12 C/Java/Python TTLV helper declarations have no exact public Rust equivalent; and two Java/Python namespace wrappers map only to the Rust crate root, not an exported declaration. Their exact export names and reasons are preserved in the generated report and blocked manifest rows. The four `*_with_limits` names and the stale `ClientBatchItem` paths are not claimed as public Rust exports. The Java `RuntimeMetadata` class and Python `metadata` module are blocked only as namespace declarations; `RuntimeVersion`, version lookup, and capability query remain mapped under the T016 evidence.

KMIPKIT-0009 is approved for implementation, but its 257 declarations and four operation rows remain subject to exact mapping and parity evidence; KMIPKIT-0010 has no declaration or operation rows yet and remains a candidate gate. These counts are recorded independently from the Rust export totals. The rest of the observed Rust exports remain blocked in the manifest pending an approved owner, an exact mapping, or parity evidence; this inventory does not assert release readiness.
