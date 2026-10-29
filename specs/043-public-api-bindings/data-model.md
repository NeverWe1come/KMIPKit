# Data Model: Public API Manifest v2 and Parity

## Manifest

The manifest is a versioned build-time contract with `formatVersion = 2`, `scope = full-1.0`, and `releaseLine = 1.0.0`. It contains stable requirement references and ordered collections for language IDs, the eight logical public package families, public declarations and explicit exclusions, API levels, operation inventory, capabilities, types, functions, error categories, limits, security notes, and generated outputs.

## Public Package

There is exactly one logical package record for each product package family: `rust-kmipkit-ttlv`, `rust-kmipkit-protocol`, `rust-kmipkit-transport`, `rust-kmipkit-client`, `rust-kmipkit`, `c-kmipkit`, `java-kmipkit`, and `python-kmipkit`. The first five identify Rust crates; the last three identify the C ABI, Java Maven artifact, and Python PyPI distribution families. Each record has a language, package/crate identity, distribution family, version source, and supported-target reference. The C family's repository-owned identity and target metadata come from `bindings/c/package.toml`; it defines the logical name `kmipkit-c` and does not establish publication. Java and Python coordinates and platform artifact names come from their respective package metadata. Package versions resolve to one synchronized release version. `kmipkit-ffi` and `kmipkit-test-support` are not separate public package families.

## Public Declaration

| Field | Meaning | Constraint |
|---|---|---|
| `declarationId` | Stable logical declaration ID. | Unique and never inferred from source order. |
| `packageId` | Package whose public surface exports the declaration. | Must resolve to `publicPackages`. |
| `kind` / `exportName` | Public module, type, variant, field, trait, function, associated item, constant, macro, C symbol, Java member, or Python export and its fully qualified name. | Must match the built/exported package surface. |
| `capabilityId` | Logical capability represented by this declaration. | Must resolve to a capability and its stable requirement IDs. |
| `approvalSource` | Approved owner feature specification or accepted ADR with evidence. | Required for declared/verified; linked specs must be formally approved/accepted; absent while blocked. |
| `declarationState` | Availability of the exported declaration. | `blocked`, `declared`, or `verified`; blocked entries may carry only an observed export name, candidate owner, and reason, with no generated signature or language mapping. |
| `parityScope` | `four-language` or `rust-only-adr`. | The latter is permitted only for the ADR-0003/ADR-0014 bounded low-level transport exchange and its response contract. |
| `candidateSpecId` / `blockedReason` | Optional candidate owner and explicit reason an observed declaration is blocked. | `blockedReason` is required while blocked; candidate ID is optional and never conveys approval or support. |
| `languageMappings` | Mapping to the logical behavior and relevant declarations in each applicable language. | Empty while blocked; four-language scope requires Rust/C/Java/Python when declared/verified; transport exception identifies only Rust and its accepted ADR. |

Every publicly reachable symbol is represented by one declaration or one exact, reviewed exclusion. An implementation that already exports a symbol for an unapproved operation records the observed item as `blocked`, with its candidate owner and reason; it does not expose or generate a new API signature. Exclusion records identify symbol, package, source, reason, and acceptance evidence; wildcard and whole-package exclusions are invalid. Rust, C, Java, and Python export audits compare the manifest to built artifacts or language package exports so that visibility changes cannot escape review.

## Operation Inventory Entry

| Field | Meaning | Constraint |
|---|---|---|
| `operationId` | Stable operation element ID from the reviewed normative catalog. | Unique; must resolve to one catalog operation. |
| `name` | OASIS operation name. | Exact catalog spelling. |
| `section` | OASIS KMIP 2.1 source section. | Exact §6.1.x; sorted numerically. |
| `scope` | Product scope classification. | `client_1_0` for the 56 in-scope entries; §6.1.54 Set Endpoint Role remains `client_1_1` and is excluded from the 1.0 manifest. |
| `featureSpecId` | Formally approved operation owner. | Required for declared/verified; absent while blocked. |
| `candidateSpecId` | Linked draft or implementation-authorized candidate owner. | Optional while blocked only; never conveys approval. |
| `approvalEvidence` | Repository-local evidence for the approved owner's active-release acceptance. | Required whenever `featureSpecId` is set. |
| `apiState` | API readiness. | `blocked`, `declared`, or `verified`. |
| `blockedReason` | Explanation of missing prerequisite. | Required only while blocked; no invented feature ID. |
| `declarations` | Language-specific public declaration IDs. | Empty only while blocked; all four languages required when declared. |
| `parityEvidenceIds` | Stable fixture/test evidence references. | Required and resolvable when verified. |

The API state is review metadata and is never exposed as a runtime trust input. A specification marked only as authorized for autonomous implementation does not satisfy the owner approval gate. A final release manifest requires every row `verified` and every owner formally approved/accepted.

## Capability

A capability has a stable ID, one or more KMIPKit requirement IDs, one or more applicable API levels, an owning or candidate feature ID or accepted ADR, `capabilityState` (`blocked`, `declared`, or `verified`), semantic description, parity scope, observable input/default/result/error/ordering behavior, security notes, and parity evidence references. `blocked` capabilities may name a candidate spec but are not advertised; `declared` requires formal owner approval and a canonical Rust API; `verified` additionally requires passing parity evidence for every language required by its parity scope. Language names can differ; observable behavior may not.

## Type and Function Declarations

V2 retains the full v1 declaration semantics and adds any fields required for the complete API. A type includes its logical identity/kind, requirement IDs, canonical Rust owner/path/name, C representation/ownership/release function, Java package/name/lifetime, and Python module/name/lifetime. A function includes a stable identity, requirement IDs, ordered parameters, complete result type, error categories, four language names/signatures, ordering, ownership, and behavior relevant to parity.

No declaration is valid unless all identifiers, referenced types/functions, status/error categories, limits, requirement IDs, and output paths resolve. Opaque handles cannot expose Rust memory layout or platform-allocator ownership.

## Error Category and Limit

An error category maps one stable logical ID to Rust error kind, C status value, Java exception type, Python exception type, delivery state if applicable, and redaction policy. A limit declares one semantic value with units, default, hard maximum where applicable, and a language field mapping. A caller must observe equivalent bound failures in all languages.

The generic codec limit records include defaults of 16 MiB per message, 64 Structure levels, and 100,000 elements, with configuration ranges and the depth hard cap inherited exactly from specifications 0004 and 0005.

## Parity Fixture

A fixture identifies one or more capabilities and requirements, language-neutral inputs, expected result/error/order/preservation behavior, resource cleanup, and the tests that consume it. Fixtures must contain no real credentials, keys, raw sensitive request bodies, or other secrets.

## Generated Output

An output is a schema-controlled repository-relative destination with a stable role and generation owner. The manifest cannot introduce arbitrary paths. Generated code/tests are committed review artifacts; handwritten Java/Python facade and resource-lifecycle behavior are not generated.
