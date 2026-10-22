# Implementation Plan: Full Public API Manifest and Binding Parity

**Branch**: `feature/KMIPKIT-0043-public-api` | **Date**: 2026-10-11 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/043-public-api-bindings/spec.md`

## Summary

Expand the unreleased registry-only API manifest to strict v2 for the complete 1.0 public contract, inventory all five published Rust crates and the C/Java/Python distribution families, account for every public declaration, and build a catalog-checked inventory of all 56 `client_1_0` operations. KMIP 2.1 defines 57 client-initiated operation types overall; Set Endpoint Role (§6.1.54) remains `client_1_1` because its role transition requires bidirectional channel behavior outside the 1.0 boundary. Define/test equivalent Rust, C, Java 17, and Python 3.12 product capabilities at the three approved API levels. The manifest owns mappings and generation metadata; each operation's approved feature specification remains the source of its Rust operation contract and semantics. Common manifest, shared API, and adapter work proceeds in phases; an operation's language surface is gated on its owner specification and canonical Rust implementation. ADR-0003/ADR-0014's low-level transport contract is explicitly inventoried as the only Rust-only capability.

## Technical Context

**Language/Version**: Rust Edition 2024 / MSRV 1.94; C11; Java 17; Python 3.12.

**Primary Dependencies**: Existing workspace and pinned manifest toolchain; generated C ABI from `kmipkit-ffi`; minimal JNI; CFFI ABI mode; Gradle Kotlin DSL; Maturin. Add no runtime dependency solely for manifest validation or generation unless already approved.

**Storage**: None. The checked-in JSON manifest and schema are development inputs only.

**Testing**: Manifest schema/generator contract tests; catalog comparison; Rust unit and parity tests; compiled C consumer tests; Java 17 JNI tests; Python 3.12 CFFI/type/package tests; supported-platform offline package-load tests; ABI compatibility and deterministic regeneration checks.

**Target Platform**: Exactly the operating-system and architecture matrix already supported by repository CI/package policy. No new target is introduced.

**Project Type**: Multi-language protocol library with Rust core and native C ABI.

**Performance Goals**: No new KMIP runtime path or protocol computation. Adapter overhead and native loading must not introduce unbounded copies or unbounded handle retention; existing operation/transport budgets remain owned by their feature specifications.

**Constraints**: One canonical Rust protocol model; no operation-semantic duplication; stable 1.x C ABI; four-language semantic parity; no runtime network downloads; fixed generator destinations; secret redaction and explicit ownership; all 56 in-scope operations required at 1.0 release completion; Set Endpoint Role remains deferred to 1.1.

**Scale/Scope**: Eight logical published package families, every publicly reachable declaration, three API levels, four languages, 56 in-scope operations, current shared facilities and existing registry manifest declarations.

## Constitution Check

- Approved Spec Kit specification required: satisfied; human approval is recorded in `approval-record.md`.
- Worktree and branch from active `release/1.0.0`: satisfied; current branch is `feature/KMIPKIT-0043-public-api` created at the active release head.
- OASIS authority and exact references: operation inventory cites applicable KMIP Specification v2.1 §§6.1.1–6.1.57; §6.1.54 Set Endpoint Role is explicitly deferred by the catalog to 1.1.
- Rust core and four-language parity: preserved; C stays the foreign-language ABI boundary, Java and Python remain thin adapters.
- Security, ownership, no-retry, and generated-file constraints: preserved and included in acceptance.
- No constitution or accepted ADR boundary change: none.

## Architecture and Phases

### Phase A - Manifest v2 and complete inventory

Add a strict v2 schema and expand the existing generator. Preserve v1 registry declarations and fixed outputs. Inventory the eight logical public package families and every exported declaration; cross-check exactly 56 catalog client_1_0 operation entries, exact §6.1 references, stable requirement IDs, blocked/declared/verified operation status, accepted parity scope, and fixed generated destinations. Keep the catalog's 1.1 Set Endpoint Role record out of the 1.0 manifest. Common contract/generator work does not wait for per-operation specifications.

### Phase B - Canonical Rust API and stable C ABI mapping

Inventory canonical declarations across all five published Rust crates, then map approved shared API levels, cross-language error/status categories, limits, resource ownership, capability queries, batch outcomes, and generic TTLV capabilities. T076 supplies the missing C ABI for validated 0013 transport/client construction, typed batch execution, and common ordered outcomes. Record its observed C exports only as blocked `publicDeclarations` until Java/Python mappings exist; defer four-language type/function signatures and generated public headers, and do not advertise C-only readiness. The in-repo ABI consumer may use test-local prototypes that are not public API evidence. Keep Poll/Cancel/Process and profile-validation APIs blocked until their owner specifications are approved. Extend the manifest only with canonical declarations backed by implemented approved Rust APIs. Once all facade mappings exist, generate mechanical C declarations and keep hand-reviewed lifecycle/validation code in `kmipkit-ffi`. Preserve existing C symbols and ABI behavior. The low-level transport exchange is recorded separately with its accepted ADR scope; it is neither re-exported nor made injectable into the typed client.

### Phase C - Java and Python adapters

Implement idiomatic Java 17/JNI and Python 3.12/CFFI facades over the stable C ABI. Add close/context-manager semantics, native loading, typing, and shared parity fixtures. T077 completes the local three-level API taxonomy across all facades, without network calls and distinct from Discover Versions. Each shared capability is exposed only when its Rust/C contract exists.

### Phase D - Operation-specific binding slices

For every operation in the dependency matrix, use its formally approved feature specification and canonical Rust implementation as a prerequisite. Add the manifest mapping and C/Java/Python accessors, operation tests, documentation, and traceability. After T021/T024 and the Encrypt, MAC, Sign, and Get Usage Allocation mappings (T033, T040, T049) exist, T078 adds the approved 025 caller-owned local Usage Allocation guard across all facades; it remains distinct from those operation request/response mappings. The 15 operations whose linked owners remain unapproved stay blocked; all 56 operation rows now have an explicit owner link. Set Endpoint Role remains outside the 1.0 API per the accepted 025 scope disposition. The final release gate requires zero blocked rows.

### Phase E - Complete parity and packaging gate

Compile and call a C consumer; run Java/Python parity and offline package loading on each currently supported platform; compare common fixtures and errors; regenerate outputs and prove a clean check; validate compatibility, traceability, docs, and release metadata.

## Project Structure

### Documentation (this feature)

```text
specs/043-public-api-bindings/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
├── traceability.md
├── checklists/requirements.md
└── contracts/
    ├── manifest-v2.md
    ├── operation-dependencies.md
    ├── shared-dependencies.md
    └── parity-matrix.md
```

### Source Code (repository root)

```text
specification/api/                 # reviewed manifest and strict schema
tools/api_manifest/                # deterministic generator and contract tests
crates/kmipkit-ffi/                # reviewed ABI lifecycle and generated declarations
bindings/c/                        # public C header and C consumer tests
bindings/java/                     # Java 17 facade, JNI, package tests
bindings/python/                   # Python 3.12 facade, CFFI, package tests
tests/fixtures/                    # shared cross-language parity fixtures
docs/architecture/                 # API reference and binding documentation
specification/compliance/          # requirement traceability evidence
```

**Structure Decision**: Extend the existing manifest/generator and language surfaces in place. The manifest remains the single API-contract input; handwritten language facades remain source-controlled and are checked against it. Public declaration audits cover published exports and the official distribution families; unpublished FFI and test-support crates are excluded. No new independent protocol or wire-model project is created.

## Dependency and Merge Gates

- Base branch and dependency source: active `release/1.0.0`.
- Shared APIs depend on an approved owner and implementation recorded in `contracts/shared-dependencies.md`; 0009 async follow-up and 0010 profile-validation APIs remain blocked in their current draft state.
- The public-package inventory is fixed to five Rust crates and the C, Java, and Python distribution families; actual publication coordinates are read from package metadata rather than invented here. Every export must map to an approved capability or an exact reviewed exclusion.
- `kmipkit-transport::Transport::exchange` is the sole Rust-only API exception under accepted ADR-0003/ADR-0014; no raw request or arbitrary-transport injection is added to the typed facade.
- Operation APIs depend on the owning approved operation specification plus its canonical Rust request/response/client contract.
- The 15 operations without a formally approved owner contract are explicit release blockers; their candidate specifications are linked in the dependency matrix. They are not silently deferred out of 1.0.
- A feature-specific operation API can be merged only with the corresponding approved contract and tests. KMIPKIT-0043's complete 1.0 gate remains red until all 56 in-scope operations and all shared APIs are verified across four languages.
- Follow the repository's required Red, Green, Refactor commit evidence and draft-PR workflow during implementation; this documentation-preparation task does not modify production code.

## Complexity Tracking

No constitution or accepted architecture exception is required. The 56-operation `client_1_0` matrix is a release dependency register, not a second owner of KMIP semantics; it records the 57th KMIP operation, Set Endpoint Role, as deferred to 1.1 by the normative catalog.
