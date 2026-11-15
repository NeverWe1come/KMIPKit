# Implementation Plan: KMIP TTLV Wire Codec

**Branch**: `feature/KMIPKIT-0005-ttlv-wire-codec` | **Date**: 2026-10-04 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification for a bounded public TTLV decoder and a request-only private outbound encoder, following the generic model.

## Summary

Add canonical outbound TTLV encoding and bounded decoding for the eleven KMIP 2.1 Item Types represented by the KMIPKIT-0004 generic tree. Keep the public generic tree and bounded decoder in `kmipkit-ttlv`; implement and unit-test the private byte writer/owner in `kmipkit-client` without a production callsite. The first client feature/spec owns `Client::execute`, its closed typed request API, execute-owned `OperationEncodingPermit`, sole production writer callsite/mint site, and exact-one audit. KMIPKIT-0005 exposes no public `encode(&Item)` or general-purpose encoder. Apply OASIS big-endian headers, exact type-specific lengths, value widths, and padding. Preserve child order, unknown enum/mask bits, and accepted extension Tags. Check message, Structure depth, item count, and U32 Item Length before allocation. The private writer itself performs no transport I/O or operation/schema validation.

The 004 model implementation landed in `release/1.0.0` at `cf6c4c0d87c4de7dc159aba046a8fe5638ccc6bf`. KMIPKIT-0005, ADR-0011, ADR-0012, the feature specification, and the request-only boundary are approved under the delegated maintainer authorization recorded in `approval-record.md`. The first-client production callsite remains gated on its candidate owner-through-transport integration test passing CI; until then the release branch contains no production callsite or secret-bearing send.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94.
**Primary Dependencies**: Public `kmipkit-ttlv` generic model/bounded decoder and its existing pinned `zeroize` 1.9.0 dependency. Before any `kmipkit-client/Cargo.toml` change, complete `specs/005-ttlv-wire-codec/dependency-review.md` for every newly direct client dependency (including internal crates and test-only dependencies) and obtain independent review. The record must cover capability, alternatives, maintenance, security history, MSRV, license, platform support, transitive cost, and exact package versions/features. The existing 004 `zeroize` review may be referenced only where version, features, and scope match; the client-use rationale must be documented in this feature's record. The independent review is complete; any checklist disposition remains with its reviewer. After the dependency-review gate and T001, T002 may add direct client `dev-dependencies` on workspace `kmipkit-ttlv` and `zeroize` to compile the Red test harness, plus the reviewed test-only dependencies. Only after T001 is satisfied may T003 promote the reviewed `kmipkit-ttlv` and `zeroize` entries to normal client dependencies for the private encoder/owner; before promotion, confirm the independent review covers that scope and the exact versions/features, and update and re-review the record first if they differ. This direction is acyclic according to current manifests. KMIPKIT-0005 adds no production callsite; the first client feature/spec owns execute, permit, and the production invocation. No manifest is changed by this documentation proposal.
**Storage**: In-memory only; encoded output uses a dedicated owner that zeroizes its initialized encoded byte range before deallocation/drop. Spare or otherwise uninitialized `Vec` capacity is outside the guarantee unless explicitly initialized and cleanup is verified. Decoded values own their payloads through the approved model.
**Testing**: Focused codec unit tests, exact OASIS-derived vectors, malformed-input negatives, property-based model round trips, coverage, workspace checks, and fuzz targets after the parser surface stabilizes.
**Target Platform**: Rust workspace supported platforms.
**Project Type**: Public generic TTLV/decoder API in `crates/kmipkit-ttlv`; private outbound encoder in `crates/kmipkit-client`.
**Performance Goals**: No throughput target is specified. Length and limit checks must be bounded and avoid body-sized allocation before validation.
**Constraints**: No unsafe code; no raw payload diagnostics; no automatic retries; all integer length arithmetic checked; no OASIS download/scraping; no hand-edited generated output. One item per call, no I/O or schema validation. ADR-0012, this feature, and the enforceable request-only boundary are approved under `approval-record.md`; the first-client production callsite remains prohibited until its owner-through-transport integration test passes CI. KMIPKIT-0005 adds no production writer callsite or `Client::execute`.
**Scale/Scope**: 11 Item Types; 16 MiB default message cap; 64 Structure-level cap; 100,000 Items by default. See `research.md` for exact limit counting and configuration semantics.

## Constitution Check

### Pre-design gate

| Principle | Result | Evidence / condition |
|---|---|---|
| I. Specification and traceability | Pass for design; completion gated | Stable FR/NR IDs and exact clauses are recorded. Implementation and executable verification links remain required. |
| II. Test first and evidence based conformance | Pass with mandatory TDD | Tasks require separate Red, Green, and Refactor commits, OASIS vectors, malformed inputs, and coverage evidence. |
| III. One core, explicit language boundaries | Pass | Rust-only codec over the common model; FFI and language adapters are out of scope. |
| IV. Secure defaults and lossless handling | Pass with policy gates | Input lengths and limits are checked before size-driven allocation; model values and order are preserved. Accepted ADR-0011 rejects received Reserved Tags. Accepted ADR-0012 permits only the specified private, uncalled codec writer in this feature; the first-client candidate integration test must pass CI before a production callsite can be merged, enabled, or used to send secret-bearing data. |
| V. Human governed, reviewable changes | Pass with recorded delegated authorization | The maintainer delegated spec and ADR approval for this work in the authorization recorded in `approval-record.md`. Work remains on the feature branch; only a human may approve or merge its PR. |

### Post-design gate

The approved design keeps public generic TTLV values and bounded decoding in `kmipkit-ttlv`; KMIPKIT-0005 implements/tests the private byte writer/owner without a production callsite. It adds no `Client::execute`, permit, or production writer callsite. The future private path uses an execute-owned permit and a closed set of concrete typed requests, with no public `encode(&Item)`, raw-body input, caller-implementable conversion trait, or general-purpose encoder. The first client feature/spec owns the caller API, permit type/private constructor, execute-only writer callsite/mint site, parent-restricted writer access, and exact-one audit. It also owns the request/owner integration test; its candidate PR must include the sole production callsite and owner-through-transport test, and CI must pass that test before merge, enablement, or release. Until then, the release branch has no production callsite or secret-bearing send. The decoder operates on bounded slices; the private encoder receives generic Items only from unit tests here. The design does not claim OASIS profile conformance or schema validity. ADR-0011 and ADR-0012 are accepted under `approval-record.md`.

## Project Structure

### Documentation

```text
specs/005-ttlv-wire-codec/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/rust-ttlv-codec.md
├── checklists/requirements.md
├── checklists/design.md
└── tasks.md
```

Related repository decision records: `docs/adr/0011-reserved-tag-decoding-policy.md` and `docs/adr/0012-caller-requested-wire-encoding-policy.md`.

### Source and verification

```text
crates/kmipkit-ttlv/src/
├── lib.rs                  # public generic tree; codec module is declared below
└── codec/
    ├── mod.rs              # public decoder facade, limits, and errors; no encoder export
    ├── decoder.rs          # checked slice parser and Structure decode
crates/kmipkit-ttlv/tests/support/
├── codec_red_tests.rs      # cfg(test) Red decoder seam/tests; no pre-T006 public API references
├── codec_limits_tests.rs   # cfg(test) Red limits seam/tests; no pre-T010 public API references
└── decoder_internal_tests.rs # direct tests for checked spans and defensive boundaries
crates/kmipkit-client/src/
├── lib.rs                  # private wire_encoder module declaration
└── wire_encoder.rs         # private uncalled writer and zeroizing owner; no permit/callsite here
crates/kmipkit-client/tests/support/
└── wire_encoder_tests.rs   # private writer unit tests
crates/kmipkit-ttlv/tests/
├── codec_api.rs            # public decode API visibility after T006
├── codec_negative.rs       # malformed, unsupported, and decoder-limit cases
└── codec_limits_api.rs     # public CodecLimits/getter/decode_with_limits API after T010
specification/catalog/kmip-2.1.json # requirement-to-spec/code/test references
```

**Structure Decision**: Keep the public generic tree and bounded decoder in `kmipkit-ttlv`. T005/T009 Red cases call only test-local candidate seams and fixtures behind `#[cfg(test)]`; stubs deliberately fail behavioral assertions, so the Red target does not import or call production decoder/`CodecLimits` symbols that do not yet exist. Client encoder limit Red cases use a private fixture/borrowed limits-view seam rather than public `CodecLimits`. T006/T010 promote those seams to production APIs and add external-crate tests for public visibility; T010 also adapts the exact borrowed `CodecLimits` instance to the private client writer and verifies no clone/reconstruction. T003 implements the writer as private code with no production caller; it does not implement an `OperationEncodingPermit`, `Client::execute`, or their type/constructor/mint/callsite. The first client feature/spec owns those parts, moves/refactors the writer under the execute-owning module, requires an execute-owned permit at a parent-restricted entry point (`pub(super)` or equivalent; never `pub(crate)`), and audits exactly one production mint site and callsite. Its execute input must be a closed set of concrete typed KMIP requests, not a caller-supplied `Item`, raw body, or caller-implementable conversion trait. The candidate first-client feature PR must contain both the sole production callsite and its owner-through-transport integration test. CI must run and pass that test against the candidate callsite before merge, enablement, or release; until then, the release branch must contain no production callsite or secret-bearing send. Do not export an encoder or encoded owner; generated catalog files remain generated from reviewed inputs. This approval/plan phase changed no source or manifest; implementation is recorded by the tasks below.

## Design Phases

### Phase 0 — Prerequisite confirmation and normative review

- Confirm the source digest and clauses in the pinned local OASIS copy.
- PR #14 is merged; verify the actual public model API and accepted ADR-0010 on the updated release base.
- Record `KMIPKIT-REQ-SPEC-10.1.2-001` as follow-on typed-protocol scope: the generic codec preserves supplied child order but cannot validate operation schemas. Keep the 1.0 traceability gate open until every applicable client 1.0 Structure has approved typed-spec ownership, implementation, and executable order-verification references.
- ADR-0011 is accepted under `approval-record.md`; T001 records accepted decision `KMIPKIT-DEC-001` against `KMIPKIT-DISC-037` with the exact OASIS KMIP Specification v2.1 §11.56 reference and validates/regenerates the catalog report before T006. T012 adds only applicable implementation/test traceability for normative requirements assigned to this codec.
- Resolve `KMIPKIT-0005-OD-001` as bounded input/limit preflight plus fallible decoder-owned scratch/payload reservations; explicitly document that allocation failure inside existing model `Box::new`/`Vec::push` constructors may abort. KMIPKIT-0005 does not change model constructors.
- The delegated maintainer authorization records acceptance of ADR-0012, this feature specification, and the enforceable private request-only boundary in `approval-record.md`. It permits only the private uncalled writer/owner in KMIPKIT-0005. The first client feature/spec defines the caller API, execute-owned permit, sole production writer callsite/mint site, exact-one audit, and integration test for typed input, permit enforcement, owner lifetime through partial writes and transport success/error return, post-return zeroization, §8 failure-delivery-state reporting, and no automatic retry. Its candidate PR must include the sole callsite and test, and CI must pass before merge, enablement, or release. Until then, the release branch has no production callsite or secret-bearing send.
- Before any `crates/kmipkit-client/Cargo.toml` change, complete and independently review `specs/005-ttlv-wire-codec/dependency-review.md` for every newly direct client dependency, including internal crates and test-only dependencies. Cover capability, alternatives, maintenance, security history, MSRV, license, platform support, transitive cost, and exact versions/features. Reuse the 004 `zeroize` review only where version, features, and scope match, and document its client-use rationale. The independent review is complete; any checklist disposition remains with its reviewer. Do not add or use client production dependencies on `kmipkit-ttlv` or `zeroize` before T001 passes. After the dependency-review gate and T001, T002 may add the reviewed direct dev-dependencies for its Red harness; T003 promotes only reviewed `kmipkit-ttlv` and `zeroize` to normal dependencies after the implementation gates are satisfied.

### Phase 1 — Contracts and data invariants

- Define immutable public `CodecLimits` with read-only getters, public per-call decoder limits and errors, exact one-item decode, and the approved future permit-gated writer/owner boundary in `contracts/rust-ttlv-codec.md`. The KMIPKIT-0005 writer receives the same borrowed limits instance in its private unit-test seam. The first client feature/spec defines how the caller passes it into `Client::execute` and owns the production permit/callsite.
- Define Item Length/padding accounting for each Item Type in `data-model.md`.
- The FR-013 approvals and enforceable request-only boundary are recorded in `approval-record.md`. KMIPKIT-0005 adds no production invocation or permit mint site. Never expose a generic `encode(&Item)` call as evidence of caller-requested operation intent; the first client spec defines the closed typed request and permit creation site before a production callsite exists.
- Keep operation field-order/schema checking and transport framing outside this crate.

### Phase 2 — Encoder (strict TDD)

- T002 writes failing exact byte vectors for all eleven types and nested Structures, private encoder byte/depth/count preflight boundaries, a test-observed no-copy failure case, and synthetic U32 maximum/one-over output-size planner boundaries; all use bounded fixtures rather than giant trees or allocations.
- After T001, implement and unit-test the private writer and zeroizing owner, with default/configured per-call byte/depth/count and U32 preflight, bounded length planning, complete output reservation, canonical encoding, and no fallible exits after payload copying begins. Do not add a production callsite, `Client::execute`, or permit in KMIPKIT-0005. The first client feature/spec owns the callsite and permit. Its candidate PR must include the sole production callsite and its owner-through-transport integration test together; CI must pass that test against the candidate callsite before merge, enablement, or release. Until then, the release branch must have neither the callsite nor a secret-bearing send.
- Refactor the writer for one focused responsibility, document invariants, and verify checked arithmetic and error redaction.

### Phase 3 — Decoder (strict TDD)

- Write failing tests for valid OASIS vectors and all malformed boundaries before implementation.
- Parse one complete item with checked offsets, parent bounds, supported type lengths, UTF-8/Boolean checks, and resolved Tag policy.
- Refactor bounded Structure traversal and allocation behavior as approved under `KMIPKIT-0005-OD-001`; reject trailing bytes and never expose raw input. Property tests compare against a canonicalized expected model: empty Big Integer values are excluded, unaligned Big Integer octets are minimally sign-extended to an eight-byte multiple, and already aligned Big Integer octets remain exact.

### Phase 4 — Limits and cross-cutting validation

- T002 establishes private encoder Red cases before T003. After T010, add getter-value tests for default and constructed limits, exact-boundary and one-over decoder tests, and adapter tests verifying the private writer sees the same per-operation borrowed `CodecLimits` instance. Keep synthetic encoder boundary tests in the T002/T003/T011 sequence; do not create them after their implementation.
- Add bounded property tests; in T012 add only applicable normative requirement code/test traceability (T001 already records `KMIPKIT-DEC-001` and closes `KMIPKIT-DISC-037`); then update documentation, security review, platform CI, and coverage records.
- Run repository-required fmt, Clippy, workspace test, and coverage automation after the tests exist and the implementation is complete.

## Dependencies and Gates

- KMIPKIT-0004 implementation PR #14 and KMIPKIT-0003 core types/errors are merged into the release ancestry.
- ADR-0010 is Accepted in the updated release tree.
- ADR-0011 is accepted under `approval-record.md`; T001 records accepted decision `KMIPKIT-DEC-001` against `KMIPKIT-DISC-037` with the exact OASIS KMIP Specification v2.1 §11.56 reference and validates/regenerates the catalog report before T006. T012 adds only applicable implementation/test traceability for normative requirements assigned to this codec.
- The three FR-013 decisions are recorded in `approval-record.md`. They authorize only KMIPKIT-0005 private unit-tested writer/owner code; they do not authorize a release-branch production callsite or send before the first-client candidate integration test passes CI.
- The first client feature/spec defines the exact public request/limit API, the permit type/private constructor owned by the `Client::execute` module, execute-only mint and writer callsite, parent-restricted access, exact-one audit, and the request/owner integration test. Its candidate PR must include the sole production callsite and test; CI must pass before merge, enablement, or release. The release branch must not have a production callsite or secret-bearing send until then. No public `encode(&Item)` API is exposed.
- Depth is configurable from 0 to the generic model's hard maximum of 64; exceeding 64 requires a separately reviewed model change.

## Risks and Mitigations

- **Length confusion across padding types**: Maintain an explicit per-type table in `data-model.md`; tests assert both the header length and complete encoded extent.
- **Allocation denial of service**: Check total input size and declared/cumulative lengths before value allocation; enforce item/depth counters incrementally.
- **Loss of wire padding bytes**: Specify semantic canonicalization and zero-fill output; do not promise byte identity for accepted noncanonical padding.
- **Reserved-tag policy**: Accepted ADR-0011 rejects received Reserved Tags. T001 records this decision against the catalog discrepancy before T006; never fold Reserved Tags into unknown extension preservation.
- **Secret-bearing wire policy**: ADR-0012 and the request-only boundary are accepted under `approval-record.md`. KMIPKIT-0005 may implement only a private uncalled writer/owner. The first-client candidate PR must include the sole production callsite and owner-through-transport integration test, and CI must pass before merge, enablement, or release; until then, no release-branch production callsite or secret-bearing send is allowed. Keep the policy limited to temporary outbound TTLV for an explicit typed caller-requested operation; preserve diagnostic, logging, formatting, general-purpose serialization, persistence, and arbitrary inbound raw-byte exclusions.
- **Model API integration**: Reconcile implementation with the merged 004 API and its accepted ADR-0010; do not code against guessed interfaces.

## Complexity Tracking

No constitution exception or new architecture layer is part of this approved feature. ADR-0011 and ADR-0012 record the accepted project policies; empty Big Integer and U32 Item Length behaviors remain explicit project constraints.
