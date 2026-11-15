# Implementation Plan: Typed Managed Objects and Key Material

**Branch**: `feature/KMIPKIT-0049-managed-objects` | **Date**: 2026-11-05 | **Spec**: [spec.md](spec.md)

**Input**: `specs/049-managed-objects/spec.md`

## Summary

Add typed Rust construction and borrowed inspection for all nine KMIP 2.1
managed-object structures and their Key Block, Key Value, Key Material, and
wrapping structures. Keep generic TTLV and existing request APIs source
compatible; use the existing recursively zeroizing `Value` owner for all
KMIPKit-owned payloads. Canonical field order is derived from the pinned
OASIS tables, never from numeric tags or setter call order. No cryptographic
operations are introduced.

## Technical Context

**Language/Version**: Rust Edition 2024; MSRV 1.94
**Primary Dependencies**: Existing `kmipkit-protocol`, `kmipkit-ttlv`, catalog, and manifest tooling; no new runtime dependencies planned
**Storage**: In-memory TTLV trees only; no persistent storage
**Testing**: Rust unit/integration tests, pinned KMIP test vectors, fake transport, rustdoc example compilation, API manifest and export audits
**Target Platform**: Existing Rust workspace platforms; local final verification is Windows and hosted matrix remains required when available
**Project Type**: Rust protocol library
**Performance Goals**: Typed construction and views remain linear in the number of TTLV nodes; impose no additional copy for borrowed inspection
**Constraints**: TTLV only, KMIP 2.1 only, client-initiated 1.0 operations; no implicit cryptographic choices or operations; preserve unknown and vendor data; remain within configured TTLV depth/element limits; preserve caller-visible existing APIs
**Scale/Scope**: 9 object structures; all Key Block/Key Value/wrapping fields; 9 transparent key-material formats; Rust only in 0049, with C/Java/Python parity assigned to 0053

### Design decisions

1. Model the nine known object structures as typed constructors and borrowed
   views. Unknown Object Type values retain a generic `Item` and their raw
   Object Type instead of being rejected.
2. Distinguish plaintext Key Value Structures from wrapped Key Value Byte
   Strings in the typed API. Keep Key Format Type caller-selected and open to
   future/extension values.
3. Add typed structures for all transparent material formats in §§3.4–3.12;
   retain generic byte/structure fallback for Raw, Opaque, PKCS1, PKCS8,
   ECPrivateKey, and extension formats.
4. Reuse `Item`/`Value` as the single payload ownership boundary. New typed
   views borrow from their source, typed owners consume TTLV values, and no
   typed clone silently duplicates secret material.
5. Add typed-to-generic composition methods and additive Register/Import
   request helpers. Existing constructors, operation validation, wire fields,
   delivery state, and `GetResponse` generic access remain unchanged.
6. Treat PGP Key Block as opaque data as permitted/recommended in §2.4. KMIPKit
   is not PGP-aware and does not decompose keys into separate objects.
7. Requirements dependent on server policy or capabilities remain server
   decisions. `validate()` and universal pre-send rejection remain assigned
   to 0051; typed constructors enforce only their own source-defined structure.

## Constitution Check

| Principle | Result | Evidence/decision |
|---|---|---|
| Specification and traceability | PASS | Approved 0049 specification; all operative catalog requirement IDs for §§2.1–3.12 are mapped in `traceability.md`. |
| Test first and evidence-based conformance | PASS | Tasks require failing tests before implementation and separate Red, Green, Refactor commits; no conformance claim is made. |
| One Rust core and explicit boundaries | PASS | Typed behavior lives in `kmipkit-protocol`; C/Java/Python parity stays assigned to 0053. |
| Secure defaults and lossless handling | PASS | Generic fallbacks preserve unknown data; owned payloads reuse recursive zeroization; no secret formatting or implicit crypto. |
| Human-governed changes | APPROVED EXCEPTION | The maintainer directly authorized the 0047–0053 feature stack from the #104 branch. This branch starts at #115 and its PR targets 0048, then the stack is moved to `release/1.0.0` after preceding integrations. No release branch is changed directly. |
| Immutable OASIS inputs | PASS | Only the pinned source and catalog are read; no upstream OASIS file is modified. |
| Generated-artifact rules | PASS | Public manifest outputs will be regenerated only with `tools/api_manifest/generate.py`; no generated file will be hand-edited. |

## Phase 0: Research

See [research.md](research.md). Source review confirms that the object
structures, nested members, requiredness, and encodings are present in the
catalog but lack public typed model types. Existing TTLV storage already
preserves order and unknown values and recursively zeroizes owned payloads.

## Phase 1: Design and Contracts

See [data-model.md](data-model.md), [contracts/managed-objects.md](contracts/managed-objects.md), and [quickstart.md](quickstart.md). The gap matrix and stable catalog requirements are in [gap-matrix.md](gap-matrix.md) and [traceability.md](traceability.md).

## Constitution Check (post-design)

PASS with the same approved branch-base exception. The design adds no new
crate, external runtime dependency, unsafe code, network behavior, or
cryptographic provider. The public surface is additive and keeps generic
access available. Scope dependencies are explicit: 0048 supplies typed
attributes; 0051 supplies full validation reports; 0052 supplies response
navigation; 0053 supplies non-Rust parity.

## Project Structure

### Documentation (this feature)

```text
specs/049-managed-objects/
├── approval-record.md
├── checklists/requirements.md
├── checklists/normative-requirements.md
├── contracts/managed-objects.md
├── data-model.md
├── gap-matrix.md
├── implementation-evidence.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
├── tasks.md
└── traceability.md
```

### Source Code

```text
crates/kmipkit-protocol/src/
├── lib.rs                 # public exports
├── managed_object.rs     # nine object owners/views and unknown fallback
├── key_block.rs          # Key Block and Key Value forms
├── key_material.rs       # opaque and transparent material models
├── key_wrapping.rs       # wrapping and key-information structures
├── register.rs           # additive typed request constructor
└── import.rs             # additive typed request constructor

crates/kmipkit-protocol/tests/
├── unit/managed_object_tests.rs
├── unit/key_block_tests.rs
├── unit/key_material_tests.rs
├── unit/key_wrapping_tests.rs
├── unit/managed_object_security_tests.rs
└── managed_object_guide_examples.rs

docs/user-guide/{en,es}/managed-objects.md
specification/api/public-api.json  # reviewed input; regenerate output via official tool
```

**Structure Decision**: Keep domain structs in the protocol crate near the
existing request models and generic TTLV types. Separate object envelopes,
key material, and wrapping structures so source tables and secret ownership
can be audited independently. Operation request files receive only additive
typed helpers.

## Complexity Tracking

No constitution exception beyond the maintainer-approved stacked branch base.
The number of typed structures is required by the full KMIP 2.1 object scope;
the generic fallback avoids duplicating the protocol tree or closing future
values.
