# Implementation Plan: KMIP TTLV Wire Codec

**Branch**: `feature/KMIPKIT-0005-ttlv-wire-codec` | **Date**: 2026-10-04 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification for a bounded public TTLV decoder and a request-only private outbound encoder, following the generic model.

## Summary

Add canonical outbound TTLV encoding and bounded decoding for the eleven KMIP 2.1 Item Types represented by the KMIPKIT-0004 generic tree. Keep the public generic tree and bounded decoder in `kmipkit-ttlv`; implement and unit-test the private byte writer/owner in `kmipkit-client` without a production callsite. The first client feature/spec owns `Client::execute`, its closed typed request API, execute-owned `OperationEncodingPermit`, sole production writer callsite/mint site, and exact-one audit. KMIPKIT-0005 exposes no public `encode(&Item)` or general-purpose encoder. Apply OASIS big-endian headers, exact type-specific lengths, value widths, and padding. Preserve child order, unknown enum/mask bits, and accepted extension Tags. Check message, Structure depth, item count, and U32 Item Length before allocation. The private writer itself performs no transport I/O or operation/schema validation.

The 004 model implementation landed in `release/1.0.0` at `cf6c4c0d87c4de7dc159aba046a8fe5638ccc6bf`; implementation remains gated on approval of this feature, review/acceptance of proposed ADR-0011 for inbound Reserved Tags, and three human approvals for FR-013: ADR-0012 acceptance, approval of this feature specification, and approval of the enforceable boundary design. These approvals do not create a production path. The existing `AGENTS.md` §8 prohibition remains in force on the release branch until all three approvals are complete and the candidate client feature PR includes the sole production callsite plus its owner-through-transport integration test, which CI must pass before merge, enablement, or release.

## Technical Context

**Language/Version**: Rust Edition 2024, MSRV 1.94.
**Primary Dependencies**: Public `kmipkit-ttlv` generic model/bounded decoder and its existing pinned `zeroize` 1.9.0 dependency. After T001's approval gates, T002 adds direct client `dev-dependencies` on workspace `kmipkit-ttlv` and `zeroize` to compile the Red test harness. Only after T001 is satisfied may T003 promote those entries to normal client dependencies for the private encoder/owner; this direction is acyclic according to current manifests. KMIPKIT-0005 adds no production callsite; the first client feature/spec owns execute, permit, and the production invocation. No manifest is changed by this documentation proposal.
**Storage**: In-memory only; encoded output uses a dedicated owner that zeroizes its initialized encoded byte range before deallocation/drop. Spare or otherwise uninitialized `Vec` capacity is outside the guarantee unless explicitly initialized and cleanup is verified. Decoded values own their payloads through the approved model.
**Testing**: Focused codec unit tests, exact OASIS-derived vectors, malformed-input negatives, property-based model round trips, coverage, workspace checks, and fuzz targets after the parser surface stabilizes.
**Target Platform**: Rust workspace supported platforms.
**Project Type**: Public generic TTLV/decoder API in `crates/kmipkit-ttlv`; proposed private outbound encoder in `crates/kmipkit-client`.
**Performance Goals**: No throughput target is specified. Length and limit checks must be bounded and avoid body-sized allocation before validation.
**Constraints**: No unsafe code; no raw payload diagnostics; no automatic retries; all integer length arithmetic checked; no OASIS download/scraping; no hand-edited generated output. One item per call, no I/O or schema validation. Secret-bearing outbound TTLV is only a proposed conditional exception and is not authorized before three human approvals (ADR-0012 acceptance, feature approval, and boundary-design approval) plus the first client feature's passing request/owner integration test. KMIPKIT-0005 adds no production writer callsite or `Client::execute`.
**Scale/Scope**: 11 Item Types; 16 MiB default message cap; 64 Structure-level cap; 100,000 Items by default. See `research.md` for exact limit counting and configuration semantics.

## Constitution Check

### Pre-design gate

| Principle | Result | Evidence / condition |
|---|---|---|
| I. Specification and traceability | Pass for design; completion gated | Stable FR/NR IDs and exact clauses are recorded. Implementation and executable verification links remain required. |
| II. Test first and evidence based conformance | Pass with mandatory TDD | Tasks require separate Red, Green, and Refactor commits, OASIS vectors, malformed inputs, and coverage evidence. |
| III. One core, explicit language boundaries | Pass | Rust-only codec over the common model; FFI and language adapters are out of scope. |
| IV. Secure defaults and lossless handling | Pass with policy gates | Input lengths and limits are checked before allocation; model values and order are preserved. ADR-0011 proposes rejection of Reserved Tags. ADR-0012 proposes a request-only outbound TTLV exception; the current prohibition remains in force on the release branch until all three FR-013 approvals are complete and the candidate client feature PR includes the sole production callsite plus its owner-through-transport integration test, which CI must pass before merge, enablement, or release. KMIPKIT-0005 has no production writer callsite. `max_structure_depth` is caller-configurable from 0 to the model cap of 64. |
| V. Human governed, reviewable changes | Pass with hard gates | Dedicated worktree and branch. No code until this spec is approved and dependencies/policies are resolved. Only a human approves or merges the PR. |

### Post-design gate

No architecture boundary change is approved by this draft. The public TTLV tree and bounded decoder remain in `kmipkit-ttlv`; KMIPKIT-0005 implements and tests the private byte-producing writer/owner but adds no `Client::execute`, permit, or production writer callsite. The future private path is proposed to use an execute-owned permit and a closed set of concrete typed requests, with no public `encode(&Item)`, raw-body input, caller-implementable conversion trait, or general-purpose encoder. The first client feature/spec owns the caller API, permit type/private constructor, execute-only writer callsite and mint site, parent-restricted writer access, and exact-one audit. It also owns the request/owner integration test; the candidate first-client feature PR must contain both the sole production callsite and its owner-through-transport integration test. CI must run and pass that test against the candidate callsite before merge, enablement, or release; until then, the release branch must contain no production callsite or secret-bearing send. The decoder operates on bounded slices and the private encoder on a generic Item supplied by unit tests in this feature. The design does not claim OASIS profile conformance or schema validity. Proposed ADR-0011 recommends rejection of received Reserved Tags and must be accepted before implementing that decoder branch. Proposed ADR-0012 is a separate human-acceptance gate; the current prohibition on secret-bearing serialization remains in force. If reviewers reject the private request-only boundary or it cannot be enforced, do not approve or implement a secret-bearing request path.

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
├── lib.rs                  # public generic tree; cfg(test) Red module before promotion
├── codec_red_tests.rs      # cfg(test) Red decoder seam/tests; no pre-T006 public API references
└── codec/
    ├── mod.rs              # public decoder facade, limits, and errors; no encoder export
    ├── decoder.rs          # checked slice parser and Structure decode
    └── limits_tests.rs     # cfg(test) Red limits seam/tests; no pre-T010 public API references
crates/kmipkit-client/src/
├── lib.rs                  # private wire_encoder module declaration
└── wire_encoder.rs         # private uncalled writer, zeroizing owner, and unit tests; no permit/callsite here
crates/kmipkit-ttlv/tests/
├── codec_api.rs            # public decode API visibility after T006
├── codec_negative.rs       # malformed, unsupported, and decoder-limit cases
└── codec_limits_api.rs     # public CodecLimits/getter/decode_with_limits API after T010
specification/catalog/kmip-2.1.json # requirement-to-spec/code/test references
```

**Structure Decision**: Keep the public generic tree and bounded decoder in `kmipkit-ttlv`. T005/T009 Red cases call only test-local candidate seams and fixtures behind `#[cfg(test)]`; stubs deliberately fail behavioral assertions, so the Red target does not import or call production decoder/`CodecLimits` symbols that do not yet exist. Client encoder limit Red cases use a private fixture/borrowed limits-view seam rather than public `CodecLimits`. T006/T010 promote those seams to production APIs and add external-crate tests for public visibility; T010 also adapts the exact borrowed `CodecLimits` instance to the private client writer and verifies no clone/reconstruction. T003 implements the writer as private code with no production caller; it does not implement an `OperationEncodingPermit`, `Client::execute`, or their type/constructor/mint/callsite. The first client feature/spec owns those parts, moves/refactors the writer under the execute-owning module, requires an execute-owned permit at a parent-restricted entry point (`pub(super)` or equivalent; never `pub(crate)`), and audits exactly one production mint site and callsite. Its execute input must be a closed set of concrete typed KMIP requests, not a caller-supplied `Item`, raw body, or caller-implementable conversion trait. The candidate first-client feature PR must contain both the sole production callsite and its owner-through-transport integration test. CI must run and pass that test against the candidate callsite before merge, enablement, or release; until then, the release branch must contain no production callsite or secret-bearing send. Do not export an encoder or encoded owner; generated catalog files remain generated from reviewed inputs. This is proposed documentation only and does not alter source or manifests.

## Design Phases

### Phase 0 — Prerequisite confirmation and normative review

- Confirm the source digest and clauses in the pinned local OASIS copy.
- PR #14 is merged; verify the actual public model API and accepted ADR-0010 on the updated release base.
- Record `KMIPKIT-REQ-SPEC-10.1.2-001` as follow-on typed-protocol scope: the generic codec preserves supplied child order but cannot validate operation schemas. Keep the 1.0 traceability gate open until every applicable client 1.0 Structure has approved typed-spec ownership, implementation, and executable order-verification references.
- Obtain review/acceptance of proposed ADR-0011 resolving `KMIPKIT-DISC-037`; update the catalog decision reference before coding.
- Require three human approvals before KMIPKIT-0005 tasks rely on FR-013: accept proposed ADR-0012, approve this feature, and approve the enforceable boundary design. These approvals do not implement a client request path or satisfy the transport gate. The first client feature/spec defines the caller-facing API, permit type/private constructor, execute-only mint/writer callsite, exact-one audit, and integration test for typed input, permit enforcement, owner lifetime through partial writes and transport success/error return, post-return drop/zeroization, §8 failure-delivery-state reporting, and no automatic retry. KMIPKIT-0005 creates no production writer callsite; the first client feature PR must include the sole production callsite and its owner-through-transport integration test together. CI must pass that test against the candidate callsite before merge, enablement, or release; until then, the release branch must have neither the callsite nor a secret-bearing send. If review rejects the boundary or it cannot be enforced, do not approve or implement a secret-bearing request path.
- Do not add or use client production dependencies on `kmipkit-ttlv` or `zeroize` before T001 passes. T002 may add these as direct dev-dependencies only for the Red test harness; T003 promotes them to normal dependencies only after the implementation gates are satisfied.

### Phase 1 — Contracts and data invariants

- Define immutable public `CodecLimits` with read-only getters, public per-call decoder limits and errors, exact one-item decode, and the proposed future permit-gated writer/owner boundary in `contracts/rust-ttlv-codec.md`. The KMIPKIT-0005 writer receives the same borrowed limits instance in its private unit-test seam. The first client feature/spec defines how the caller passes it into `Client::execute` and owns the production permit/callsite.
- Define Item Length/padding accounting for each Item Type in `data-model.md`.
- Keep FR-013 conditional on all three human approvals and the first client feature's passing request/owner integration test. KMIPKIT-0005 must add no production invocation or permit mint site. Never expose a generic `encode(&Item)` call as evidence of caller-requested operation intent; the first client spec must define the closed typed request and permit creation site before a production callsite exists.
- Keep operation field-order/schema checking and transport framing outside this crate.

### Phase 2 — Encoder (strict TDD)

- Write failing exact byte vectors for all eleven types and nested Structures, plus synthetic U32 maximum/one-over output-size planner boundaries without multi-gigabyte allocation.
- After the three approvals, implement and unit-test the private writer and zeroizing owner, with default/configured per-call byte/depth/count and U32 preflight, bounded length planning, complete output reservation, canonical encoding, and no fallible exits after payload copying begins. Do not add a production callsite, `Client::execute`, or permit in KMIPKIT-0005. The first client feature/spec owns the callsite and permit. Its candidate PR must include the sole production callsite and its owner-through-transport integration test together; CI must pass that test against the candidate callsite before merge, enablement, or release. Until then, the release branch must have neither the callsite nor a secret-bearing send.
- Refactor the writer for one focused responsibility, document invariants, and verify checked arithmetic and error redaction.

### Phase 3 — Decoder (strict TDD)

- Write failing tests for valid OASIS vectors and all malformed boundaries before implementation.
- Parse one complete item with checked offsets, parent bounds, supported type lengths, UTF-8/Boolean checks, and resolved Tag policy.
- Refactor bounded Structure traversal and fallible allocation; reject trailing bytes and never expose raw input. Property tests compare against a canonicalized expected model: empty Big Integer values are excluded, unaligned Big Integer octets are minimally sign-extended to an eight-byte multiple, and already aligned Big Integer octets remain exact.

### Phase 4 — Limits and cross-cutting validation

- Add getter-value tests for default and constructed limits, then exact-boundary and one-over tests for decoder and private encoder byte, depth, and element limits at defaults and configured values; verify the private writer sees the same per-operation borrowed instance. Test the private encoder's U32 ceiling with synthetic size-planner inputs and no giant allocation.
- Add bounded property tests, normative catalog traceability, documentation, security review, platform CI, and coverage records.
- Run repository-required fmt, Clippy, workspace test, and coverage automation after the tests exist and the implementation is complete.

## Dependencies and Gates

- KMIPKIT-0004 implementation PR #14 and KMIPKIT-0003 core types/errors are merged into the release ancestry.
- ADR-0010 is Accepted in the updated release tree.
- Proposed ADR-0011 needs review/acceptance for received Reserved Tags; the branch remains gated for that path.
- Three human approvals are required before relying on FR-013: ADR-0012 acceptance, approval of this feature specification, and approval of the enforceable boundary design. These do not satisfy the integration gate or create a production path.
- The first client feature/spec defines the exact public request/limit API, the permit type/private constructor owned by the `Client::execute` module, execute-only mint and writer callsite, parent-restricted access, exact-one audit, and the request/owner integration test. The first client feature PR must include the sole production callsite and its owner-through-transport integration test together. CI must pass that test against the candidate callsite before merge, enablement, or release; the release branch must have neither the callsite nor a secret-bearing send until then. If review rejects this boundary or it cannot be enforced, do not approve or implement a secret-bearing request path. No public `encode(&Item)` API is proposed.
- Depth is configurable from 0 to the generic model's hard maximum of 64; exceeding 64 requires a separately reviewed model change.

## Risks and Mitigations

- **Length confusion across padding types**: Maintain an explicit per-type table in `data-model.md`; tests assert both the header length and complete encoded extent.
- **Allocation denial of service**: Check total input size and declared/cumulative lengths before value allocation; enforce item/depth counters incrementally.
- **Loss of wire padding bytes**: Specify semantic canonicalization and zero-fill output; do not promise byte identity for accepted noncanonical padding.
- **Reserved-tag policy**: Keep Reserved-tag decoding gated until proposed ADR-0011 is reviewed; never fold it into unknown extension preservation.
- **Secret-bearing wire policy**: Keep the current prohibition in force on the release branch until all three human approvals are complete and the candidate client feature PR includes the sole production callsite together with its owner-through-transport integration test, which CI must pass before merge, enablement, or release. KMIPKIT-0005 adds no production callsite. Keep the proposal limited to temporary outbound TTLV for an explicit typed caller-requested operation, owned in a private zeroizing buffer through the transport write, with initialized encoded bytes zeroized before owner deallocation/drop; spare or otherwise uninitialized `Vec` capacity is outside the guarantee unless explicitly initialized and cleanup is verified; preserve all diagnostic, logging, formatting, general-purpose serialization, persistence, and arbitrary inbound raw-byte exclusions.
- **Model API integration**: Reconcile implementation with the merged 004 API and its accepted ADR-0010; do not code against guessed interfaces.

## Complexity Tracking

No constitution exception or new architecture layer is approved by this draft. Reserved-tag handling and the narrowly proposed secret-bearing wire policy remain separate formal review gates; the empty Big Integer and U32 Item Length behaviors are explicit project constraints in this draft.
