# Research: Full Public API Manifest and Binding Parity

**Feature**: [KMIPKIT-0043](spec.md)

## R1 - Expand the current manifest with a breaking format version

**Decision**: Make the complete public contract manifest v2 with `scope: full-1.0` and a new schema identifier. Preserve every registry declaration from v1.

**Evidence**: `specification/api/public-api.schema.json` fixes `formatVersion` to 1 and `scope` to `registry-slice`; its description says breaking field or meaning changes require a new version/schema identifier. `specs/012-vendor-extension-registry/contracts/public-api-manifest.md` also limits that feature's contract to the registry surface. The manifest is unreleased, so compatibility migration is repository-local.

## R2 - Use the checked-in catalog as the complete operation inventory

**Decision**: Compare the manifest's operation rows directly against `specification/catalog/kmip-2.1.json`; require exact set equality with the 56 entries marked `client_1_0`, in OASIS section order. Preserve the catalog's Set Endpoint Role (§6.1.54) row as `client_1_1` and exclude it from the 1.0 API.

**Evidence**: The active release catalog contains 56 operation elements with `scope_state: client_1_0` and a 57th client-initiated operation, Set Endpoint Role, explicitly classified `client_1_1` because the successful role transition requires bidirectional channel behavior outside the 1.0 boundary. The current 1.0 matrix links all 56 operations to owners: 41 operations have Approved/Accepted owners and 15 remain linked to unapproved candidate specs. The updated owner statuses are recorded in `contracts/operation-dependencies.md`.

## R3 - Keep operation semantics in their approved feature specifications

**Decision**: KMIPKIT-0043 owns the cross-language declaration and parity framework, not request-field semantics or operation behavior. Each operation API entry advances only after its separate feature specification is formally approved/accepted for implementation and its canonical Rust implementation is available. An instruction authorizing implementation of a draft does not by itself change that draft's approval status.

**Evidence**: The repository requires one approved specification per implementation change. The catalog and the 1.0 project definition distinguish normative coverage from feature ownership. Draft worktrees contain conflicting reuse of IDs 0022–0028; a branch/worktree draft is not an approved contract or safe source for an API signature.

## R4 - Use Rust as the semantic source and C ABI as the shared foreign boundary

**Decision**: Rust provides the canonical protocol/client behavior; C exposes the stable opaque-handle ABI; Java and Python adapt that ABI without defining protocol behavior of their own.

**Evidence**: Accepted ADR-0006, `docs/architecture/ffi-and-bindings.md`, and constitution principle III specify Rust, opaque C handles, minimal JNI, CFFI ABI mode, and semantic parity.

## R5 - Preserve semantics, not identical spelling

**Decision**: Track equivalent observable capabilities, accepted input ranges, defaults, ordering, results, errors, lifetimes, and security properties; allow idiomatic names and construction per language.

**Evidence**: ADR-0006 states that the language APIs remain idiomatic and parity concerns semantics rather than spelling. `docs/architecture/ffi-and-bindings.md` defines language-specific resource management and package loading.

## R6 - Treat generated outputs as fixed review artifacts

**Decision**: The v2 schema/generator validates the whole contract, uses only fixed repository destinations, works offline, and supports deterministic write/check modes. Handwritten lifecycle and idiomatic adapter code remains reviewed source.

**Evidence**: ADR-0004, `docs/development/api-manifest.md`, the current manifest schema, and AGENTS.md prohibit manual edits to generated output and require deterministic checked-in generation without OASIS scraping.

## R7 - Keep package loading offline and on the existing platform matrix

**Decision**: Verify packaged Java/Python native loading without network access on the platform/architecture matrix already supported by project CI. Do not add platforms or runtime binary downloads.

**Evidence**: `docs/architecture/ffi-and-bindings.md` says official packages bundle native binaries and never download executable code at runtime. AGENTS.md forbids runtime downloads and requires TLS/secret safeguards.

## R8 - Inventory the complete published package and declaration surface

**Decision**: The manifest inventories the five published Rust crates plus the logical C ABI, Java Maven, and Python PyPI distribution families. It accounts for every publicly reachable declaration with a capability and approved source or an exact reviewed exclusion. The sole Rust-only capability is the bounded `kmipkit-transport::Transport::exchange` contract and `TransportResponse`, explicitly accepted under ADR-0003/ADR-0014; all other four-language product capabilities require semantic parity.

**Evidence**: ADR-0003 identifies the five published Rust crates and distinguishes unpublished FFI/test-support crates. `docs/design/project-definition.md` defines the C, Maven Central, PyPI and synchronized-version distribution families. ADR-0014 and `docs/architecture/public-api.md` document the low-level transport exception, caller-owned raw bytes, no facade re-export and prohibition on arbitrary transport injection into the typed client. No additional Rust-only exception is inferred.

## Normative source

The protocol source is **Key Management Interoperability Protocol Specification Version 2.1**, OASIS Standard, 14 December 2020, pinned at `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`. Each operation declaration cites its exact operation section from §6.1. The catalog links request/response fields and other normative clauses to their exact source sections; those clauses remain owned by the corresponding operation/model specifications. Informative Usage Guide text does not create a 0043 requirement.
