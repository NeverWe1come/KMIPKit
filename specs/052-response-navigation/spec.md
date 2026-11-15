# Feature Specification: Typed Response Access and TTLV Navigation

**Feature Branch**: `feature/KMIPKIT-0052-typed-responses`

**Created**: 2026-11-06

**Status**: Approved for implementation by the maintainer's KMIPKIT-0047–0053 plan.

**Input**: User-approved Phase 5 plan, specifically KMIPKIT-0052: typed readers for all 56 supported client-initiated operation results, a uniform raw view of the Response Payload, and navigation of nested standard and vendor structures.

## User Scenarios & Testing

### User Story 1 - Read typed results consistently (Priority: P1)

An application developer can read the typed result of any in-scope operation without falling back to ad hoc TTLV parsing for fields already modeled by KMIPKit.

**Why this priority**: Typed results are the primary client API and must cover every operation in the 1.0 client scope.

**Independent Test**: A compile-time and runtime matrix constructs or decodes one result for each of the 56 supported client-initiated operation identities and proves the matching typed response accessor is available.

**Acceptance Scenarios**:

1. **Given** a response for any supported operation, **When** the application uses the corresponding typed response reader, **Then** it obtains the response model for that operation.
2. **Given** a Pending result or a Failure without Response Payload, **When** the application asks for payload data, **Then** the API reports absence and preserves the existing asynchronous or error result semantics.

### User Story 2 - Inspect complete response TTLV (Priority: P1)

An application developer can inspect the decoded Response Payload as a read-only ordered TTLV tree, including fields not yet represented by typed accessors.

**Why this priority**: Servers may return optional, future, or vendor-specific fields that applications need to inspect without losing the typed API.

**Independent Test**: For response fixtures with required, optional, repeated, unknown, and vendor fields, assert that `.raw()` exposes the corresponding tree in original order and returns absence when no payload is present.

**Acceptance Scenarios**:

1. **Given** a decoded response with a Response Payload, **When** the application requests `.raw()`, **Then** it gets a read-only `StructureView` owned by the response lifetime.
2. **Given** a decoded response with repeated fields, unknown allocated tags, future values, or nested `0x54xxxx` tags, **When** the application traverses the raw tree, **Then** source order, repetitions, type, and value remain available at every depth.
3. **Given** a response whose Result Status legitimately has no Response Payload, **When** the application requests `.raw()`, **Then** it gets `None` without a fabricated empty structure.

### User Story 3 - Navigate nested typed structures and extensions (Priority: P1)

An application developer can reach the raw node for nested Structures such as Managed Object, Key Block, Attribute Set, and Server Information, and can traverse from the payload root to any descendant.

**Why this priority**: A vendor extension may appear deep inside an otherwise standard object and must remain inspectable.

**Independent Test**: Place distinct `0x54xxxx` fields at multiple depths, including inside Key Block and Server Information, and verify the path, source order, item type, and value from Rust, C, Java, and Python consumers.

**Acceptance Scenarios**:

1. **Given** a typed nested Structure backed by a response, **When** the caller obtains its raw node or traverses from the response root, **Then** each Structure descendant is reachable without converting or flattening the tree.
2. **Given** a nested vendor tag, **When** the caller reads it through each supported language API, **Then** the tag and value are preserved and exposed as generic TTLV.

### User Story 4 - Manage raw views safely across language boundaries (Priority: P1)

An application developer can use raw response access from Rust, C, Java, and Python with explicit lifetime or ownership rules and without accidental payload logging.

**Why this priority**: Response payloads can include key material and other sensitive values; raw inspection must not weaken existing lifetime, redaction, or zeroization guarantees.

**Independent Test**: Compile and execute real consumers in all four languages, verify handle or wrapper release, and assert Debug/error output does not contain response payload canaries.

**Acceptance Scenarios**:

1. **Given** a live Rust response, **When** `.raw()` is called, **Then** its borrowed view cannot outlive the response owner.
2. **Given** a C, Java, or Python response view, **When** the consumer releases or closes it, **Then** the owned copy is released and zeroized according to KMIPKit's ownership boundary.
3. **Given** a response containing secret canary bytes, **When** response and view objects are formatted for Debug or an error is produced, **Then** the canary is absent.

## Edge Cases

- Result Status Failure may omit Response Payload under KMIP v2.1 §8.6 Table 399; the raw accessor must return absence in that case.
- A response tree may contain repeated fields, unknown standard tags, vendor tags, future Enumeration values, and nested Structures.
- The decoder retains the decoded tree but discards TTLV padding bytes. `.raw()` does not promise byte-for-byte reproduction of the network message.
- The whole response can fail message decoding before typed response objects exist; this feature does not expose malformed partial trees.
- Response data may contain sensitive values. Explicit raw access is permitted, while automatic formatting and errors remain redacted.
- Existing asynchronous operation outcomes and their correlation-value handling must remain unchanged; their existing raw payload APIs are included in the parity audit.

## Requirements

### Functional Requirements

- **KMIPKIT-0052-FR-001**: The public client API MUST provide a typed reader for each of the 56 supported client-initiated operation results, including asynchronous control and result flows; existing typed response APIs MUST remain source-compatible.
- **KMIPKIT-0052-FR-002**: Each typed response MUST provide a uniform raw Response Payload view when that payload is present; the view MUST represent the decoded TTLV tree, not reconstructed network bytes.
- **KMIPKIT-0052-FR-003**: The payload accessor MUST return absence when the operation result has no Response Payload. It MUST NOT fabricate an empty payload or infer one from the operation name.
- **KMIPKIT-0052-FR-004**: The raw tree MUST preserve child order, repeated children, tags, item types, values, and nested Structures, including unknown allocated tags and vendor extensions.
- **KMIPKIT-0052-FR-005**: Callers MUST be able to navigate to any descendant Structure or Item from the Response Payload root. Typed nested Structure models in scope MUST expose their source node or make it directly reachable through the same generic tree.
- **KMIPKIT-0052-FR-006**: The Response Header MUST expose a uniform read-only raw Structure view that retains unknown fields and source order.
- **KMIPKIT-0052-FR-007**: Rust, C, Java, and Python MUST expose equivalent response-payload and nested-TTLV inspection capability. Rust views MUST be borrowed from the owning response; foreign-language owned handles MUST have explicit release/close semantics.
- **KMIPKIT-0052-FR-008**: Debug output and automatically generated errors MUST NOT disclose response payload values, raw response bytes, credentials, or secret key material.
- **KMIPKIT-0052-FR-009**: KMIPKit-owned response-tree copies that may contain sensitive payloads MUST be zeroized when released. Runtime limitations for external language copies MUST be documented.
- **KMIPKIT-0052-FR-010**: The implementation MUST preserve existing response decoding, Result Status semantics, asynchronous correlation handling, error redaction, and public ABI compatibility.
- **KMIPKIT-0052-FR-011**: Every normative response-structure requirement MUST cite the exact KMIP v2.1 section and table and have a stable traceability ID linked to implementation and executable tests. Project-specific API/lifetime decisions MUST use separate project decision IDs.
- **KMIPKIT-0052-FR-012**: The API manifest and generated Rust, C, Java, Python, test, and documentation outputs MUST be reconciled using the pinned generator; generated files MUST NOT be hand-edited.
- **KMIPKIT-0052-FR-013**: Rust Get and Export responses MUST provide typed access to a successful managed object and, when that object defines one, its Key Block. These accessors MUST preserve the original response tree, retain unknown and vendor children in their typed copies, return absence for non-success results or objects without a Key Block, and keep failures payload-free and redacted.

### Normative Source References

- OASIS KMIP Specification v2.1 §6.1, operation-specific Response Payload tables for each supported operation. The exact operation-to-table mapping is recorded in `gap-matrix.md`.
- OASIS KMIP Specification v2.1 §8.5, Table 398 (Response Header), for response-header fields.
- OASIS KMIP Specification v2.1 §8.6, Table 399 (Response Batch Item), for Response Payload presence relative to Result Status.
- OASIS KMIP Specification v2.1 §10.1.2 (Type), Structure encoding order statement.
- OASIS KMIP Specification v2.1 §9.13 (Message Extension) and §11.56 (Tag Enumeration), for vendor extension semantics and the `0x540000`–`0x54FFFF` extension tag allocation. Preservation at arbitrary payload depth is an explicit KMIPKit API requirement and does not reinterpret the extension.
- OASIS KMIP Specification v2.1 §6.1.19 Table 221 (Get Response Payload), §3.1 Table 12 (Key Block Object Structure), and §7.37 Table 389 (Server Information Structure) for nested-structure examples. Exact labels are verified against the pinned Standard in `gap-matrix.md` before implementation.
- OASIS KMIP Specification v2.1 §6.1.38, Table 276, for Poll request and asynchronous response behavior. Poll has no separate response-payload table in the pinned source; the operation result's optional/required payload behavior follows its defined asynchronous response and §8.6 Table 399.
- OASIS KMIP Specification v2.1 §6.1.41, Tables 285–287, for Query Asynchronous Requests. The pinned Standard's Table 286 caption says “PKCS#11 Response Payload” while its content describes an “Asynchronous Request”; this feature preserves the decoded payload generically and assigns no disputed semantics to it.

## Success Criteria

### Measurable Outcomes

- **KMIPKIT-0052-SC-001**: The response inventory maps 56/56 supported operation identities to a typed reader, payload-presence rule, exact OASIS table, and executable test.
- **KMIPKIT-0052-SC-002**: Every in-scope response with a decoded Response Payload exposes the same source-ordered TTLV tree, and every in-scope result without one returns absence.
- **KMIPKIT-0052-SC-003**: Rust, C, Java, and Python consumers each successfully navigate a vendor-tagged nested Structure in both Key Block and Server Information while preserving repeated fields and value types.
- **KMIPKIT-0052-SC-004**: Response ownership and release tests pass in all four languages, and response canary values are absent from Debug and error output.
- **KMIPKIT-0052-SC-005**: All new public routes have exact manifest declarations or narrowly justified exclusions; generated output is clean under `generate.py --check`.
- **KMIPKIT-0052-SC-006**: Rust Get and Export results expose validated managed-object and Key Block models without losing access to the original decoded TTLV tree; tests prove extension preservation, absent accessors for non-key objects, and redaction of secret payloads.

## Assumptions

- The maintainer-approved Phase 5 plan authorizes the scope of this feature and the stacked base on `feature/KMIPKIT-0051-pre-send-validation`, whose ancestry begins at the KMIPKIT-0043 branch.
- KMIP v2.1 is the source of truth. The response payload table, Result Status, and operation-specific response structures determine whether payload is required, optional, or absent.
- `.raw()` means a borrowed view of KMIPKit's decoded ordered TTLV tree. It does not reproduce discarded padding or claim byte-for-byte equality with network input.
- Generic TTLV is the fallback for unknown and vendor data. Unknown fields are not reinterpreted or rejected by this inspection API.
- Where a typed response already owns a generic response item, the implementation may reuse that owner if the ownership and zeroization contract is equivalent; it must avoid exposing an unredacted `Debug` path.

## Out of Scope

- Changing KMIP wire encoding, response acceptance rules, transport framing, TLS, retries, or server-side behavior.
- Reconstructing exact received bytes or TTLV padding.
- Adding client-initiated operations outside the 56-operation product scope or server-initiated operations planned for 1.1.
- Claiming profile conformance, certification, or server interoperability from response navigation tests.
- JSON/XML encodings and additional language adapters beyond Rust, C, Java, and Python.

## Clarifications

No critical ambiguities remain: the maintainer-approved Phase 5 plan establishes the language set, optional payload behavior, decoded-tree semantics, nested vendor access, ownership, and redaction boundaries.
<!-- retry-149 -->
<!-- retry-149-2 -->
