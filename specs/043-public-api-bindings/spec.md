# Feature Specification: Full Public API Manifest and Binding Parity

**Feature Branch**: `feature/KMIPKIT-0043-public-api`

**Created**: 2026-10-10

**Status**: Approved for implementation

**Input**: Complete the versioned public API manifest and define functional parity for the Rust, C, Java, and Python 1.0.0 APIs.

## User Scenarios & Testing

### User Story 1 - Review the complete public contract (Priority: P1)

A maintainer or integrator inspects one reviewed manifest and can determine the complete 1.0 public capability, its stable requirement and OASIS references, how it maps to each language, and whether an operation API is ready or blocked by its operation specification.

**Why this priority**: The current manifest describes only the extension-registry slice. A complete release surface cannot be reviewed or generated until the full contract is explicit.

**Independent Test**: Validate the manifest against its strict schema, compare its operation inventory to the normative catalog, and verify every operation maps to exactly one OASIS section and an approved feature specification or an explicit blocked state.

**Acceptance Scenarios**:

1. **Given** the normative catalog, **When** a reviewer compares it with the 1.0 manifest, **Then** all 56 `client_1_0` KMIP 2.1 operations appear exactly once. The catalog's 57th client-initiated operation, Set Endpoint Role (§6.1.54), remains `client_1_1` and is excluded from 1.0.
2. **Given** an operation without an approved feature specification, **When** the manifest is reviewed, **Then** its API is marked blocked with its exact OASIS section and no invented feature number or signature.
3. **Given** a manifest declaration, **When** a reviewer follows its references, **Then** every public declaration has stable KMIPKit requirement IDs and every operation declaration identifies its exact KMIP 2.1 operation section.
4. **Given** the published package set, **When** a maintainer audits exported declarations, **Then** every public symbol is represented in the manifest or covered by a specific approved exclusion record.

---

### User Story 2 - Use equivalent capabilities in each language (Priority: P1)

An application developer uses Rust, C, Java, or Python and can access the same approved KMIPKit 1.0 capabilities while each language keeps its established idioms and lifecycle conventions.

**Why this priority**: Rust is the protocol implementation, while C is the shared foreign-language boundary. Java and Python are 1.0 product surfaces, not reduced adapters.

**Independent Test**: Run the same deterministic API fixtures through Rust, a compiled C consumer, Java 17, and Python 3.12; compare request semantics, typed results, TTLV preservation, error categories, and resource lifecycle.

**Acceptance Scenarios**:

1. **Given** an approved operation specification and its typed Rust API, **When** C, Java, and Python bindings are generated and exercised, **Then** each language provides semantically equivalent high-level, typed, and applicable generic-TTLV capabilities.
2. **Given** a batch with completed, failed, and permitted pending outcomes, **When** each language inspects it, **Then** ordering, per-item status, and asynchronous correlation bytes agree with the Rust result.
3. **Given** unknown standard enumeration values, bitmask bits, tags, or extension values accepted by the underlying model, **When** a caller round-trips them through a binding, **Then** the binding does not silently discard them.

---

### User Story 3 - Ship and load supported language packages safely (Priority: P2)

An application developer installs an official Java or Python package and loads its bundled native library without a runtime code download; a C consumer uses the documented ABI and ownership rules.

**Why this priority**: A binding is not usable parity if its native library cannot be located, loaded, or safely released using the documented package.

**Independent Test**: Build local package artifacts for the repository's supported platform matrix, load them in isolated offline consumer tests, and compile and call the C ABI from C.

**Acceptance Scenarios**:

1. **Given** an installed Java or Python artifact on a supported platform, **When** the package is imported or initialized without network access, **Then** its bundled ABI-matched native library loads or returns a redacted, actionable error.
2. **Given** a C caller using a library-owned handle, **When** it follows the declared release function, **Then** the handle is released exactly once and no platform allocator is used to free KMIPKit memory.
3. **Given** a native ABI mismatch or missing bundled binary, **When** the adapter initializes, **Then** it fails fast without downloading executable code.

### Edge Cases

- A catalog operation has no approved feature specification or its assigned specification conflicts with the 1.0 scope.
- A newly approved operation specification changes its canonical Rust declaration before the corresponding bindings are complete.
- A manifest declaration references a missing requirement ID, model, error category, or generated output.
- Two language names differ idiomatically while their semantics, accepted value ranges, ordering, or errors must remain equivalent.
- A C caller passes null, closed, stale, or wrong-kind handles, zero-length spans, oversized lengths, or a value outside the declared fixed-width range.
- A Java/Python wrapper is closed while a child view remains live, or the native library cannot be loaded for the current OS/architecture.
- An unknown enumeration or bitmask value is valid to preserve but is not known to the current convenience API.
- A generated output is missing, stale, redirected outside the fixed output set, or changes because of environment-specific ordering.
- A public item is exported by one of the published Rust crates or language distributions but is absent from the manifest, or is classified as Rust-only without an accepted ADR.
- Diagnostic, exception, or debug formatting encounters credentials, secret material, private keys, raw KMIP bodies, or TLS private-key data.
- An asynchronous response is pending; bindings must expose the established explicit follow-up API and must not introduce polling, waiting, or retries.

## Requirements

### Functional Requirements

- **KMIPKIT-0043-FR-001**: The project MUST maintain one reviewed, versioned, machine-readable manifest for the complete KMIPKit 1.0 public release surface: all published Rust crates, the C ABI/package, the Java package, and the Python package. The final 1.0 manifest MUST cover the high-level, complete typed-protocol, and generic-TTLV product levels and MUST retain the existing extension-registry declarations.
- **KMIPKIT-0043-FR-002**: The manifest operation inventory MUST contain each of the 56 KMIP 2.1 operations classified `client_1_0` by `specification/catalog/kmip-2.1.json` exactly once, with its exact OASIS KMIP Specification v2.1 §6.1.x reference and scope state. KMIP 2.1 defines 57 client-initiated operation types overall; Set Endpoint Role (§6.1.54) is explicitly `client_1_1` because its successful role transition requires bidirectional channel behavior outside the 1.0 boundary. The 1.0 manifest MUST NOT include it as a supported API.
- **KMIPKIT-0043-FR-003**: Each operation API declaration MUST be linked to an approved operation feature specification that defines its Rust protocol and client contract. If that specification is absent, unapproved, or scope-conflicting, the operation MUST remain explicitly blocked in the dependency matrix and MUST NOT receive an invented API signature or feature ID.
- **KMIPKIT-0043-FR-004**: Every public capability declaration MUST have a stable KMIPKIT-0043 requirement ID and identify the owning approved feature specification and requirement IDs when its behavior is owned elsewhere. No manifest declaration may refer to an unknown requirement, type, function, error category, limit, capability, or output.
- **KMIPKIT-0043-FR-005**: The manifest MUST record equivalent support across Rust, C, Java, and Python for every released product-facade capability, including high-level operations, typed request/response models, batch outcomes, protocol asynchronous follow-up, generic TTLV, errors, and applicable shared facilities. It MUST separately record the Rust-only low-level transport API as an explicit accepted architecture exception and MUST NOT expose raw request submission or arbitrary transport injection through the other facades.
- **KMIPKIT-0043-FR-006**: Rust MUST remain the protocol implementation and canonical source for language-neutral behavior. The manifest and bindings MUST NOT implement a second KMIP wire model or alter operation semantics owned by approved protocol specifications.
- **KMIPKIT-0043-FR-007**: The C surface MUST use the accepted stable ABI rules: `kmipkit_` symbols, fixed-width integers, explicit pointer lengths, opaque handles, explicit ownership and matching release functions, versioned public structures, stable status codes, per-thread detailed error access, and panic containment at every exported function. ABI additions MUST remain backward compatible within 1.x.
- **KMIPKIT-0043-FR-008**: The Java surface MUST target Java 17, use the accepted minimal JNI bridge and Gradle Kotlin DSL, provide idiomatic resource-safe wrappers and documented nullness, and preserve the semantics and error categories of the C ABI.
- **KMIPKIT-0043-FR-009**: The Python surface MUST target Python 3.12, use CFFI ABI mode and the accepted Maturin package flow, expose complete type annotations and `py.typed`, provide context-managed resources, and preserve the semantics and error categories of the C ABI.
- **KMIPKIT-0043-FR-010**: All four product facades MUST provide equivalent access to the three accepted API levels. Differences in spelling or idiomatic construction are permitted only when observable KMIP behavior, accepted inputs, defaults, results, ordering, error category, ownership, and security guarantees remain equivalent. Any public Rust-crate-only surface MUST be explicitly classified with its accepted ADR and excluded from facade parity only where that ADR defines the boundary.
- **KMIPKIT-0043-FR-011**: Bindings MUST preserve all unknown standard and vendor values preserved by their canonical Rust model, including raw enumeration values, unknown bitmask bits, ordered TTLV children, and valid extension values. A language adapter MUST NOT silently narrow an open value to a closed enum.
- **KMIPKIT-0043-FR-012**: Bindings MUST preserve ordered heterogeneous batch behavior and per-item outcomes. Protocol `Pending` results and opaque asynchronous correlation values MUST be surfaced consistently, and no binding may automatically poll, retry, wait, or execute another KMIP request.
- **KMIPKIT-0043-FR-013**: Public errors, statuses, exceptions, and diagnostics MUST map through a stable manifest-declared cross-language taxonomy. Formatting and default diagnostics MUST redact credentials, private keys, secret material, OTPs, tickets, raw KMIP bodies, and TLS private keys.
- **KMIPKIT-0043-FR-014**: The expanded manifest format MUST use a new format version and schema identifier because the existing version 1 schema fixes `scope` to `registry-slice`. Version 2 MUST state its complete 1.0 scope, required records, field semantics, ordering, cross-reference rules, and completion conditions. No 1.0 release artifact may treat an incomplete dependency matrix as complete API coverage.
- **KMIPKIT-0043-FR-015**: Manifest generation MUST be deterministic, offline, and restricted to a reviewed fixed set of output paths. Generation MUST validate the manifest and cross-references before writing; check mode MUST fail on missing or stale generated outputs. Generated files MUST NOT be hand-edited.
- **KMIPKIT-0043-FR-016**: Java and Python official packages MUST bundle the native library for the repository's supported platform matrix and MUST NOT download executable code at runtime. Package loading MUST reject ABI mismatch and MUST report failure without exposing secrets.
- **KMIPKIT-0043-FR-017**: The final manifest and test evidence MUST demonstrate 100% traceability for every public capability, 56/56 `client_1_0` operation inventory coverage, and semantic parity of every released capability across Rust, C, Java, and Python.
- **KMIPKIT-0043-FR-018**: API names, public documentation, errors, generated declarations, package metadata, and version identifiers MUST remain synchronized across the four languages. The user guides and API reference MUST describe actual released capability without claiming formal certification or profile support without the evidence required by the project.
- **KMIPKIT-0043-FR-019**: Each shared non-operation capability MUST link to a formally approved/accepted owning specification before it is declared available in any language. In particular, asynchronous follow-up APIs and profile-validation APIs MUST remain blocked until specifications 0009 and 0010 respectively are approved, their open source/scope gates are resolved, and their canonical Rust implementations exist.
- **KMIPKIT-0043-FR-020**: The manifest MUST enumerate the five published Rust crates named by ADR-0003 (`kmipkit-ttlv`, `kmipkit-protocol`, `kmipkit-transport`, `kmipkit-client`, and `kmipkit`) and all first-party C, Java, and Python distributions. It MUST exclude unpublished implementation/test-support crates from the supported API inventory. The `kmipkit-transport::Transport::exchange` surface MUST be included as Rust-crate-only under ADR-0003/ADR-0014, with its caller-owned raw-byte limits and no-re-export/no-facade-injection boundary documented.
- **KMIPKIT-0043-FR-021**: The manifest MUST enumerate every publicly reachable declaration in every package from FR-020, including public modules, types, variants, fields, traits, functions, methods, constants, macros, C ABI symbols, Java public API declarations, and Python package exports. Every declaration MUST link to one logical capability, stable requirement IDs, and its package mapping. A `blocked` declaration may record an observed export and candidate specification ID but MUST NOT advertise the capability, generate a new signature, or provide language mappings until its owner specification is approved; `declared` and `verified` declarations MUST link to an approved/accepted feature specification or accepted ADR with evidence. A declaration may be excluded only by a specific, reviewed exclusion record with its source and reason. Four-language product capabilities MUST map to equivalent behavior in all four languages. No other Rust-only exception is permitted without an accepted ADR that explicitly defines it.
- **KMIPKIT-0043-FR-022**: Every behavior or executable manifest/generator contract change performed under this specification MUST follow strict Red, Green, Refactor TDD. The failing test, passing implementation, and refactoring MUST be evidenced in distinct development commits, and the implementation PR MUST record the relevant commands and results.

### Key Entities

- **Public API manifest**: Versioned review input describing scope, requirements, operation inventory, language mappings, types, functions, errors, limits, security constraints, capability state, and fixed generated outputs.
- **Public package**: One logical published API package: one of the five Rust crates, the C ABI distribution, the Java artifact family, or the Python distribution family, with a synchronized version and release target.
- **Public declaration**: A publicly reachable symbol or declaration in a public package, including its package, kind, exported name, logical capability, owning requirement/specification, language mapping, and parity classification.
- **Operation inventory entry**: One client-initiated KMIP 2.1 operation with exact OASIS section, catalog identity, approved owning feature specification (if any), and explicit readiness state.
- **Capability**: A named observable API behavior at one or more API levels and its equivalent Rust, C, Java, and Python mappings.
- **API declaration**: A stable type, function, parameter, result, error, ownership rule, or capability query mapped to the four public language surfaces.
- **Parity fixture**: A language-neutral input and expected observable behavior used to compare the public bindings.
- **Generated output**: A checked-in source or test artifact derived deterministically from the reviewed manifest and generator.
- **Binding resource**: A library-owned client, result, view, buffer, or secret wrapper with explicit close/release and parent-lifetime semantics.

## Success Criteria

### Measurable Outcomes

- **KMIPKIT-0043-SC-001**: The v2 schema accepts the final manifest and rejects missing, duplicate, out-of-scope, unresolved, or cross-reference-invalid API records.
- **KMIPKIT-0043-SC-002**: An automated catalog comparison reports exactly 56 unique `client_1_0` operation entries, with 56 exact §6.1.x references and no out-of-scope operation classified as supported. Set Endpoint Role remains cataloged as `client_1_1`.
- **KMIPKIT-0043-SC-003**: Every operation has an approved dependency or is explicitly marked blocked; at release completion, zero operation entries remain blocked.
- **KMIPKIT-0043-SC-004**: The parity matrix has no missing Rust, C, Java, or Python mapping for any released capability and no semantic mismatch in results, ordering, accepted values, errors, resource lifecycle, or redaction.
- **KMIPKIT-0043-SC-005**: Two consecutive manifest generation checks on the same commit produce byte-identical outputs, and a deliberately stale generated output is detected by check mode.
- **KMIPKIT-0043-SC-006**: A C consumer compiles and calls the ABI; Java 17 and Python 3.12 consumer tests load their local packaged native library offline and exercise representative success, error, closure, and ABI-mismatch paths.
- **KMIPKIT-0043-SC-007**: Requirement traceability for this feature and all released API declarations is 100%; no unsupported profile or certification claim is introduced.
- **KMIPKIT-0043-SC-008**: Every shared capability has a formally approved owner and four-language evidence before release; no capability with an unapproved or unresolved owner is represented as available.
- **KMIPKIT-0043-SC-009**: Every declared public symbol in a published Rust crate or first-party language distribution is accounted for by the manifest or an explicit approved exclusion; the only Rust-only capability is identified with the accepted ADR and is absent from foreign facades.
- **KMIPKIT-0043-SC-010**: The package inventory contains exactly the five published Rust crates plus the official C, Java, and Python distribution families; every public declaration audit resolves to a manifest entry or an approved exclusion, and every distribution reports the synchronized release version.
- **KMIPKIT-0043-SC-011**: The implementation history contains separate, ordered Red, Green, and Refactor commits for every behavior or executable contract change; the PR records commands and outcomes for the repository's applicable checks.

## Scope and Dependencies

The complete 1.0 target is the 56 KMIP 2.1 operations classified `client_1_0` in the normative catalog and the three API levels defined in `docs/design/project-definition.md` and ADR-0006. Set Endpoint Role (§6.1.54) remains cataloged as `client_1_1` and outside this release scope. This feature defines the versioned public contract, manifest/generator rules, four-language mapping, parity verification, and supported binding packaging. It does not restate or implement KMIP operation semantics. Each operation API is added only from its separately approved operation specification and its implemented Rust contract.

The active release catalog currently links all 56 `client_1_0` operations to owner specifications. Forty-one operations have Approved/Accepted owners (0007, 0014, 0017, 0021, and 0036–0042); 15 remain linked to unapproved candidates (0009, 0016, 0018, 0019, and 0020) and remain blocked in the API manifest. Set Endpoint Role (§6.1.54) remains `client_1_1` per the catalog and is not a 1.0 operation dependency. This feature does not assign future IDs.

## Assumptions

- The approved project definition, constitution, ADR-0006, ADR-0004, and current active `release/1.0.0` branch remain authoritative.
- The manifest is a build-time/review artifact, not a runtime capability document loaded from an untrusted source.
- Manifest v1 has not been released as a public compatibility contract; its registry declarations are migrated without semantic loss into v2.
- The existing fixed generator destinations and its offline behavior are extended only through this approved specification.
- Existing approved operation specifications remain authoritative for operation behavior and canonical Rust signatures; an unapproved draft in another worktree is not a dependency contract.
- Native package OS and architecture coverage is exactly the matrix already supported by project CI and packaging documentation; this feature adds no platform.

## Out of Scope

- KMIP operation protocol behavior, request/response field rules, or state transitions already owned by operation specifications.
- Adding new cryptographic algorithms, server-initiated operations, another language, another transport, another encoding, or a new TLS policy.
- Local cryptographic operations, automatically chosen algorithms, automatic polling, retries, server capability caching, or profile/certification claims.
- Runtime manifest downloads, executable plugins, changing the normative catalog format for unrelated reasons, or editing pinned OASIS sources.
- Publishing a package or release; local package build/load and dry-run consumer verification remain in scope.
