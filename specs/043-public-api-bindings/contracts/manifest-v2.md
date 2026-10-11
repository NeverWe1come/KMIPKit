# Public API Manifest v2 Contract

## Purpose and compatibility

`specification/api/public-api.json` remains the single reviewed manifest input. Version 1 is explicitly constrained to `registry-slice`; v2 expands the meaning to the complete KMIPKit 1.0 API and therefore uses `formatVersion: 2` and schema ID `urn:kmipkit:schemas:public-api-manifest:2`. V1 was not released and has no external compatibility guarantee. All v1 registry declarations and behavior are carried forward without loss.

The manifest is a repository build and review input. It is not downloaded, loaded, or interpreted by production clients at runtime.

## Required top-level records

The strict v2 schema requires these records and rejects undeclared fields. Stable package IDs are exactly `rust-kmipkit-ttlv`, `rust-kmipkit-protocol`, `rust-kmipkit-transport`, `rust-kmipkit-client`, `rust-kmipkit`, `c-kmipkit`, `java-kmipkit`, and `python-kmipkit`. The last three identify logical distribution families; their external coordinates and platform artifact names are read from their package metadata and are not invented by 0043.

| Record | Required content |
|---|---|
| `formatVersion` | Integer `2`. |
| `scope` | Exact value `full-1.0`. |
| `releaseLine` | Exact value `1.0.0`. |
| `catalog` | Repository-relative path and schema version of the reviewed normative catalog used for comparison. |
| `requirementIds` | Unique, lexicographically sorted stable feature requirements represented in the manifest. IDs may belong to 0043 or an owning approved feature. |
| `languages` | Exactly Rust, C, Java, and Python, in the schema-defined canonical order. |
| `publicPackages` | Exactly eight logical package entries with the IDs listed above: five published Rust crates, the C ABI distribution, the Java Maven artifact family, and the Python PyPI distribution family. Each records its language, package/crate identity, artifact family, synchronized version source, and supported target set. |
| `publicDeclarations` | Complete inventory of publicly reachable declarations for those packages. Every declaration has a stable ID, package ID, export kind/name, logical capability ID, requirement references, readiness state, and parity scope. Approved source is required for ready declarations; a blocked observed export may record a candidate owner and reason without a signature or mappings. |
| `approvedExclusions` | Array of explicit exclusions from the public declaration inventory; may be empty. Each exclusion identifies the exact symbol/package, reason, approving feature or ADR, and review evidence. No wildcard or package-wide exclusion is allowed. |
| `apiLevels` | Exactly high-level, typed-protocol, and generic-TTLV levels, with canonical IDs and requirement references. |
| `operations` | Exactly 57 unique `client_1_0` entries, ordered by OASIS §6.1 section number. Each identifies the exact source reference, catalog element ID, scope, owning approved feature (when available), API readiness, and declarations by language when ready. |
| `capabilities` | Shared client, batch, asynchronous, TTLV, credential, extension, error, lifecycle, and diagnostic capabilities with requirement IDs and four-language mappings. |
| `types` | Existing v1 type declarations plus all additional released API types, preserving complete Rust/C/Java/Python names, kind, ownership, and requirement references. |
| `functions` | Existing v1 function declarations plus all additional released functions, with ordered parameters, complete result type, error mappings, ownership, ordering, and all four language mappings. |
| `errorCategories` | Stable unique cross-language error/status/exception records; every function error reference resolves. |
| `limits` | All shared limit values with units, defaults, maxima, and one equivalent mapping in each language. |
| `security` | Redaction, zeroization, external-runtime-copy, and native-loading boundaries by language. Notes MUST describe guarantees and limits without secret examples. |
| `generatedOutputs` | Exactly the eleven schema-controlled output names and paths below; manifest text cannot add, remove, or redirect paths. |

## Fixed generated outputs

The v2 generator writes exactly these destinations. The six registry destinations preserve the current registry slice; the five full-API destinations cover all public declarations and parity fixtures. The generated C umbrella header is shared and contains both slices.

| Output name | Fixed destination | Content |
|---|---|---|
| `registryRustFfi` | `crates/kmipkit-ffi/src/extension_registry_generated.rs` | Existing generated extension-registry Rust FFI declarations. |
| `cHeader` | `bindings/c/include/kmipkit.h` | Complete public C declarations for the registry and full API. |
| `registryJavaApi` | `bindings/java/src/main/java/org/kmipkit/generated/ExtensionRegistryApi.java` | Existing generated registry Java declarations. |
| `registryJavaParityTests` | `bindings/java/src/test/java/org/kmipkit/generated/ExtensionRegistryParityFixtures.java` | Existing registry-specific Java parity fixtures. |
| `registryPythonApi` | `bindings/python/src/kmipkit/_generated/extension_registry.py` | Existing generated registry Python declarations. |
| `registryParityFixtures` | `tests/fixtures/extensions/generated/registry_parity.json` | Existing registry parity fixtures. |
| `rustFfi` | `crates/kmipkit-ffi/src/public_api_generated.rs` | Generated full public API declarations and repetitive operation shims; ownership, validation, errors, and panic containment remain reviewed code. |
| `javaApi` | `bindings/java/src/main/java/org/kmipkit/generated/PublicApi.java` | Generated full public API Java declarations/metadata. |
| `javaParityTests` | `bindings/java/src/test/java/org/kmipkit/generated/PublicApiParityFixtures.java` | Generated Java fixtures for the shared full-API parity corpus. |
| `pythonApi` | `bindings/python/src/kmipkit/_generated/public_api.py` | Generated full public API Python declarations/metadata. |
| `parityFixtures` | `tests/fixtures/api/generated/public_api_parity.json` | Language-neutral shared fixtures for complete API parity. |

Output names and paths are constants in the generator and exact values in the schema; the JSON manifest repeats those values for review. The generator MUST reject any mismatch, duplicate destination, traversal, or symlink escape. The two Rust output files remain separate so existing registry generation and its reviewed behavior remain intact.

## Operation readiness

Every inventory row includes:

- `operationId`: the catalog operation element ID.
- `name` and `section`: the exact catalog name and OASIS Specification v2.1 §6.1.x section.
- `scope`: `client_1_0` for these 57 entries.
- `ownerDependency`: the exact linked owner from `contracts/operation-dependencies.md`, represented as `approved`, `candidate`, or `unassigned`. This is dependency metadata only; it does not declare the operation API available.
- `featureSpecId`: an approved feature ID, or absent when none is approved.
- `candidateSpecId`: optional linked draft/implementation-authorized feature ID, permitted only while blocked; it records a possible owner but conveys no approval.
- `approvalEvidence`: a repository-relative status/approval record and active-release acceptance reference; required whenever `featureSpecId` is set.
- `apiState`: `blocked`, `declared`, or `verified`.
- `blockedReason`: required only for `blocked`; it identifies the missing/unapproved operation contract, not an invented ID.
- `declarations`: stable declaration IDs grouped by Rust, C, Java, and Python. This list is empty only while blocked.
- `parityEvidenceIds`: stable fixture/test references; required when `apiState` is `verified` and each reference must resolve to checked-in evidence.

`blocked` means the operation is in the 1.0 product target but its binding API cannot be declared or implemented under 0043 yet. `ownerDependency` records the dependency matrix's linked owner and whether its checked-in release-line status is approved, a candidate, or unassigned; it MUST NOT be treated as API readiness. A blocked row may have a `candidateSpecId` if a draft exists, but MUST NOT set `featureSpecId` or be presented as supported capability. For this contract, an owner is approved only when its checked-in status explicitly says Approved or Accepted for implementation and records human acceptance on the active release line. A status that only grants autonomous implementation does not qualify. `declared` means an approved operation specification and canonical Rust surface exist and all four mappings are defined but parity verification is incomplete. `verified` means all four mappings and required tests pass. A 1.0 release manifest MUST have all 57 rows `verified` and zero blocked rows.

The generator MUST reject a transition to `declared` unless the `featureSpecId` resolves to an approved/accepted feature, its approval evidence is recorded, and all referenced Rust declarations and requirement IDs exist. It MUST reject a transition to `verified` unless each language has a mapping and the parity evidence references resolve. During phased implementation, blocked inventory rows are permitted in the review manifest; `--check --require-complete` MUST reject any manifest that is not fully verified.

## Capability and declaration mappings

Each released capability maps to one or more of the three API levels and contains:

- stable capability and requirement IDs;
- `ownerSpecId` plus `approvalEvidence`, or `ownerAdrId` plus accepted evidence for an ADR-owned capability, or `candidateSpecId` and `blockedReason` while blocked;
- `capabilityState`: `blocked`, `declared`, or `verified`, with the same approval and four-language evidence gates as operation readiness;
- canonical Rust declaration paths and owning crate;
- C symbol/type names and status/ownership behavior;
- Java package/type/method names and resource lifecycle;
- Python module/type/function names and resource lifecycle;
- observable input ranges, effective defaults, result behavior, error category, ordering, and security notes;
- test fixture or verification identifiers for each language.

Every exported declaration is represented by exactly one `publicDeclarations` record or one exact `approvedExclusions` record. Declaration kinds cover public modules, types, enum variants, public fields, traits, functions, associated functions/methods/constants/types, macros, C ABI symbols, Java public/protected members, and Python package exports. For Rust, every externally reachable `pub` item counts even if marked `doc(hidden)`. For C, audit the public headers and exported ABI symbols. For Java, audit public/protected members of the published API packages; package-private implementation details are not public. For Python, the package's explicit `__all__` and documented public exports define the public surface; underscore-prefixed implementation modules are private unless explicitly exported. Rust mappings identify the owning published crate and fully qualified item; C mappings identify the exported symbol and header declaration; Java mappings identify the published package/type/member; Python mappings identify the package/module/export. One logical declaration may map to different idiomatic names across languages, but must resolve to a capability record. Declaration visibility changes are manifest changes and are checked against the built package exports. All external package versions resolve to the single `releaseLine` version; all official supported target variants belong to the package-family record rather than separate language APIs.

A public declaration has `declarationState`: `blocked`, `declared`, or `verified`. A blocked entry can inventory a symbol already observed in source or an artifact, with only its exact export name, package, `candidateSpecId`, and `blockedReason`; it carries no new API signature, language mappings, or support claim. `approvalSource` is absent while blocked and required for other states. `declared` requires an Approved/Accepted owner source, the canonical Rust declaration, and all mappings required by `parityScope`; `verified` additionally requires passing evidence for every applicable language and an export audit. A blocked KMIP operation may be represented at the operation row without a public symbol record if no export exists yet. Existing exports tied to a draft operation are recorded as blocked and are not generated into the C/Java/Python APIs.

`parityScope` is `four-language` or `rust-only-adr`. `four-language` requires semantically equivalent capability mappings across all four languages. `rust-only-adr` is permitted only for the bounded `kmipkit-transport::Transport::exchange` and associated response contract defined by ADR-0003/ADR-0014; it MUST record those ADRs and their accepted evidence. This exception retains caller-owned request-byte and response-limit semantics, is not re-exported by `kmipkit`, and MUST NOT create an arbitrary-transport constructor or raw-byte path in `kmipkit-client::Client::execute`. No exclusion or Rust-only classification may be inferred from implementation convenience.

The package inventory is a set of logical distribution families, not a list of individual platform files. The C family covers its documented CMake/pkg-config and static/shared artifacts; the Java family covers its Maven Central API and platform-native JARs; the Python family covers its PyPI wheels and source distribution. All package versions resolve to the synchronized release version. `kmipkit-ffi` and `kmipkit-test-support` are not published and MUST NOT appear as supported packages.

Names may differ by language. The listed observable behavior may not. A language-specific convenience wrapper may compose canonical calls but MUST NOT add protocol behavior, implicit cryptographic choices, retries, waits, or capability assumptions.

Shared capability records for asynchronous follow-up (0009) and profile validation (0010) remain blocked while their current owner specs are drafts. A capability's presence in an accepted project target or API-level taxonomy does not mark it available. At 1.0 release completion, every in-scope four-language capability must be `verified` with a formally approved/accepted owner and resolvable evidence in all four languages; the only Rust-only capability is verified against the accepted ADR scope.

The existing v1 `types`, `functions`, `errorCategories`, `limits`, `security`, and `generatedOutputs` declarations remain represented. Existing registry API behavior, including request ordering, criticality, generic TTLV limits, and extension value preservation, MUST NOT regress during migration.

## Ordering, validation, and generation

- JSON object key order has no semantic effect. Array order is significant and schema-defined for languages, API levels, operations, types, functions, parameters, errors, limits, and generated destinations.
- Operations are sorted by numeric OASIS section. Requirement ID arrays and reference arrays that are declared unordered are unique and sorted.
- The schema validates shapes; the generator additionally validates cross-record references, exact catalog coverage, ready-state requirements, symbol uniqueness, fixed-width value ranges, and permitted output destinations.
- Catalog comparison MUST fail on a missing, duplicate, renamed, out-of-scope, or extra operation. A catalog change that changes the set of 57 requires an approved product-scope/specification change before the public manifest changes.
- Generator behavior is offline, deterministic, and independent of filesystem traversal order, timestamps, locale, environment-specific paths, or network access.
- Check mode compares bytes and never repairs or rewrites outputs. Write mode atomically writes only the eleven fixed outputs listed above.
- Generated source and test files are never edited by hand. Handwritten Java/Python facades, lifecycle handling, validation, and package loaders remain reviewed source and are covered by parity tests.

## Completion gates

The feature and release gate require all of the following:

1. The operation dependency matrix contains 57/57 exact entries.
2. Every operation dependency has an approved, in-scope feature specification.
3. All 57 operation APIs are verified across Rust, C, Java, and Python; their owner specifications are approved/accepted.
4. All API levels and shared capabilities have four-language mappings, except only the ADR-defined Rust transport capability.
5. All eight logical package families are inventoried and version-synchronized; public symbol audits have no untracked export and no unapproved exclusion.
6. Manifest schema validation, catalog cross-check, generator tests, generated-file check, consumer tests, compatibility checks, and 100% traceability pass.
7. Local Java/Python package tests load the native library without network access on every platform required by the current project matrix.

The generic codec defaults exposed by the accepted TTLV specifications are 16 MiB maximum message bytes, 64 nested Structure levels, and 100,000 items. Message-size and item-count limits remain configurable within the owning codec contract; Structure depth cannot exceed 64. The manifest records these same effective values and configuration ranges for all language surfaces.
