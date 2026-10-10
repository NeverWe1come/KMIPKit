# Feature Specification: KMIP 2.1 Get and Locate Operations

**Feature Branch**: `feature/KMIPKIT-0017-managed-object-retrieval`<br>
**Created**: 2026-10-09<br>
**Status**: Draft; inventory dispositions recorded, implementation approval pending<br>
**Input**: KMIPKit roadmap Phase D; the normative inventory identifies Get (§6.1.19) and Locate (§6.1.28) as unimplemented client-initiated operation families.

## Normative scope

This specification defines the Rust client models and execution path for exactly two client-to-server operations. The catalog links two Get requirement rows and thirteen Locate inventory rows to these operation sections. Of the Locate rows, five are server-only and one Group Member Default extraction is retired because the source has no independent normative keyword. Neither category is a client conformance obligation.

| Operation | Catalog element | Normative source | Payload and error tables | Catalog evidence record IDs | Official case labels recorded by the catalog |
| --- | --- | --- | --- | --- | --- |
| Get | KMIPKIT-ELEM-OP-C2S-GET | OASIS KMIP Specification v2.1 §6.1.19 | Tables 220–222 | KMIPKIT-TEST-CN01-2-68; KMIPKIT-TEST-CN01-2-69; profile record KMIPKIT-TEST-PROF-5-12-6-3 | TC-PKCS12-1-21; TC-PKCS12-2-21; profile case TL-M-3-21 |
| Locate | KMIPKIT-ELEM-OP-C2S-LOCATE | OASIS KMIP Specification v2.1 §6.1.28 | Tables 247–249 | KMIPKIT-TEST-CN01-2-62; KMIPKIT-TEST-CN01-2-63; KMIPKIT-TEST-CN01-2-64; KMIPKIT-TEST-CN01-2-65 | TC-MDO-2-21; TC-MDO-3-21; TC-OFFSET-1-21; TC-OFFSET-2-21 |

The linked requirement records are KMIPKIT-REQ-SPEC-6.1.19-001 and -002, and KMIPKIT-REQ-SPEC-6.1.28-001, -002, -004-001, -004-002, -007, -008-001, -008-002, -009-001, -009-002, -011, -012, -013-001, and -013-002. This is fifteen records total. Their actor, direction, and applicability are recorded individually in traceability.md; a link to an operation does not imply that every server obligation is implementable by a client library.

The pinned OASIS KMIP 2.1 source and reviewed catalog are authoritative. Usage Guide examples are informative. KMIPKIT-DEC-003 resolves KMIPKIT-DISC-015: lowercase “shall” in §6.1.19 is descriptive server guidance, while the uppercase key-format duties remain normative. KMIPKIT-DEC-004 resolves KMIPKIT-DISC-032: the case headings and hyperlink targets establish `TC-PKCS12-…` as the official IDs, and both linked XML fixtures are pinned. The visible `TC-PKCS-12-…` link text remains an editorial mismatch. Two of the seven linked fixtures are now available; the other five remain unavailable. Full official-case execution requires the other operations in those XML cases and later executable evidence; fixture availability alone does not constitute a pass or a profile conformance claim. KMIPKIT-DEC-005 classifies five Locate rows as server-only and retires the Group Member Default false extraction. Client behavior remains lossless forwarding and response preservation.

Shared identifier, generic TTLV, message, batch, response-result, decoder-limit, and error contracts remain owned by their existing specifications and are referenced rather than redefined here.

## Product boundaries and exclusions

- Encoding is TTLV only, as required by the approved 1.0 product boundary.
- The scope is exactly Get and Locate. It does not implement Recover, Activate, Destroy, Register, other object lifecycle operations, cryptographic processing, or server-initiated operations.
- A caller may use Locate to find archived object identifiers when the request includes the Archived Storage indicator. Obtaining an archived object requires the separate Recover and Get operations. This feature returns Locate results and does not perform or claim the Recover follow-up.
- Get returns the server's KMIP object structure as a complete, ordered generic TTLV value. KMIPKit preserves object contents, including unknown tags and enumeration values, but does not parse key material into a local cryptographic provider or perform wrapping, unwrapping, format conversion, PKCS#12 construction, or certificate-chain discovery.
- The Get response object is secret-bearing. Its contents must remain redacted from diagnostics and be cleared when KMIPKit-owned TTLV storage is dropped, according to the existing TTLV secret-storage contract. Runtime copies made by callers remain outside that guarantee and must be documented.
- The client does not locally search objects, emulate server attribute matching, sort results, infer the server's ID Placeholder state, or retry a request. It preserves server-returned values and order.
- C ABI, Java, Python, and high-level public builders remain in the later API-parity specifications. This feature does not claim cross-language parity.
- No changes to protocol version, TLS policy, transport boundaries, immutable OASIS source copies, or public language scope are included.

## User Scenarios & Testing

### User Story 1 — Retrieve one managed object (Priority: P1)

As a KMIPKit caller, I can request one object by its Unique Identifier, optionally specifying a key format or wrapping parameters, so I can receive the server-held object without changing or interpreting its cryptographic contents locally.

**Why this priority**: Retrieving a previously created or located object is a core client workflow and exercises the secret-bearing response path.

**Independent Test**: A controlled KMIP endpoint accepts Get requests with explicit and placeholder-based identifiers and returns object payloads for representative object types. Results preserve the response Object Type, Unique Identifier, complete object value, operation status, and wire order; diagnostics do not reveal payload values.

**Acceptance Scenarios**:

1. **Given** a Get request with a Unique Identifier, **When** it is encoded, **Then** the identifier is preserved exactly and precedes the optional request fields in Table 220 order.
2. **Given** a Get request without a Unique Identifier in a batch where the server ID Placeholder is applicable, **When** it is sent, **Then** the client preserves the omission and does not synthesize an identifier.
3. **Given** optional Key Format Type, Key Wrap Type, Key Compression Type, or Key Wrapping Specification values, **When** the request is encoded, **Then** each supplied value and its presence are preserved in Table 220 order without local key conversion or wrapping.
4. **Given** a successful Get response, **When** it is decoded, **Then** the required Object Type, Unique Identifier, and Any Object are exposed, and the object subtree remains lossless, ordered, redacted, and zeroizing under the existing TTLV ownership guarantees.
5. **Given** a server returns an operation-specific Get error, **When** the result is processed, **Then** its Result Status, applicable Result Reason, and Result Message are preserved without retry or payload leakage.
6. **Given** the caller requests PKCS#12 or a wrapped key, **When** the server returns the response, **Then** KMIPKit preserves the returned KMIP object and explicit request options but does not construct or inspect the cryptographic container locally.

### User Story 2 — Find managed objects (Priority: P1)

As a KMIPKit caller, I can ask the server to locate objects by attributes and optional search constraints, so I can receive matching identifiers while leaving authoritative matching and object state on the server.

**Why this priority**: Locate connects object creation and attribute operations to later retrieval without maintaining a local copy of server state.

**Independent Test**: Controlled responses cover constrained and empty-attribute searches, optional limits and offsets, storage masks, group-member selectors, zero, one, and repeated identifiers, optional Located Items, and operation errors. The client preserves response order and never performs local matching.

**Acceptance Scenarios**:

1. **Given** a Locate request, **When** it is encoded, **Then** the required Attributes Structure is present, including when empty, and Maximum Items, Offset Items, Storage Status Mask, Object Group Member, then Attributes are emitted in the Table 247 order with caller presence and values preserved.
2. **Given** Offset Items is explicitly zero, **When** the request is encoded, **Then** zero remains represented; the server's normative behavior treats it as equivalent to omission.
3. **Given** an empty Attributes Structure, **When** Locate is executed, **Then** the client sends it unchanged to request a match-all search and does not enumerate objects locally.
4. **Given** structured, repeated, date-range, Cryptographic Usage Mask, or Usage Limits attributes, **When** they are supplied as search criteria, **Then** their exact generic TTLV structures, values, repetitions, and order are preserved for server-side matching.
5. **Given** Object Group Member is Group Member Fresh or Group Member Default, **When** Locate is executed, **Then** the exact Enumeration is transmitted and the client does not create or select group members locally.
6. **Given** Storage Status Mask is omitted, **When** Locate is executed, **Then** omission is preserved. OASIS defines online-only results as server behavior when the field is omitted; the client neither infers a mask nor filters identifiers returned by the server, including archived or destroyed identifiers if a server returns them.
7. **Given** a successful response with zero, one, or multiple Unique Identifier items and optional Located Items, **When** it is decoded, **Then** every value and its wire order are retained, including an empty result payload, without filtering values based on expected server-side status rules.
8. **Given** multiple identifiers are returned, **When** the result is exposed, **Then** KMIPKit preserves the server's order and does not sort or deduplicate it. Descending object-creation order is a server requirement; the client preserves the order received and does not claim server conformance.
9. **Given** Locate is batched with later operations, **When** the server resolves ID Placeholder behavior for zero, one, or multiple results, **Then** KMIPKit preserves the submitted batch and each returned result without predicting or modifying server placeholder state.
10. **Given** the response identifies an archived object, **When** a caller wants its contents, **Then** this feature exposes the identifier and documents that Recover followed by Get is required; it does not imply Recover is part of this feature.

### User Story 3 — Preserve results and protect object contents (Priority: P1)

As a KMIPKit caller, I can inspect operation results and retain future or vendor-defined values without exposing secret object contents in diagnostics, so interoperability and key confidentiality are preserved.

**Why this priority**: Get can return private key material, while Locate may return extension values unknown to this release.

**Independent Test**: Verification exercises unknown Object Type, Key Format Type, Storage Status Mask bits, generic object fields, repeated response identifiers, malformed payloads, decoder limits, redaction, zeroization, Pending results permitted by the shared client contract, and delivery-state reporting.

**Acceptance Scenarios**:

1. **Given** an allocated but unknown tag, Enumeration, bitmask bit, or future object type, **When** it passes the shared generic TTLV constraints, **Then** its exact raw value and position are preserved.
2. **Given** any Get request or response object contains byte strings, text, or nested structures, **When** it is formatted for diagnostics or dropped, **Then** payload content is not formatted and KMIPKit-owned payload storage is zeroized under the existing TTLV guarantee.
3. **Given** a malformed, truncated, or over-limit response, **When** decoding fails, **Then** the shared decoder rejects it within configured size, depth, and element limits and errors do not contain raw KMIP payloads.
4. **Given** a transport failure before sending, after possible transmission, or during response reception, **When** execution returns, **Then** the existing delivery-state classification is preserved and the client does not retry automatically.
5. **Given** an allowed asynchronous Pending result, **When** it is returned, **Then** KMIPKit preserves it through the shared client outcome model and performs no automatic polling.

## Edge Cases

- Get omits Unique Identifier because the request uses the KMIP ID Placeholder; the omission is not replaced with a local value.
- A successful Get response omits, duplicates, or mistypes a required Table 221 field; the typed conversion fails with a redacted protocol error.
- Get returns an unrecognized Object Type or a nested object containing allocated extension tags; generic values remain lossless and are not interpreted as a local algorithm.
- A caller requests a format that requires a server conversion. The request remains explicit; the client does not claim the server supports an unlisted conversion or rewrite the requested format.
- PKCS#12, wrapping, compression, private keys, certificates, or Secret Data appear in Get input/output; all contents remain redacted in diagnostics and zeroized when the KMIPKit-owned TTLV tree is dropped.
- Locate uses no attributes, an empty Attributes Structure, an optional zero offset, an omitted maximum, or multiple attributes; all forms retain their exact request presence and ordering.
- Locate returns no matches, omits Located Items, returns Located Items despite an omitted Offset Items, or repeats Unique Identifier fields; the response shape remains lossless.
- Locate returns archived or destroyed identifiers without the corresponding Storage Status Mask indicator; the client exposes the received values, while the server-side SHALL NOT obligations are classified separately in the catalog.
- Locate contains date attributes as one value, a two-instance range, or a maximum Date value; the client preserves each form and leaves match evaluation to the server.
- Locate contains a structured attribute with only selected fields, a Cryptographic Usage Mask, Usage Limits, Group Member Fresh, or Group Member Default; the exact source-defined request is sent without client-side candidate evaluation.
- A batch follows Locate with an operation that relies on ID Placeholder. The request is not split or automatically retried, and result handling does not assume a unique match.
- A response is Pending, failed, malformed, or larger than configured limits; shared result, error, and delivery contracts remain in force.

## Requirements

### Functional Requirements

- **KMIPKIT-0017-FR-001**: KMIPKit MUST expose typed Get and Locate client requests and responses using OASIS KMIP v2.1 §§6.1.19 and 6.1.28, Tables 220–222 and 247–249.
- **KMIPKIT-0017-FR-002**: Each model MUST preserve the Table-defined required, optional, repeatable, and ordered fields, including the difference between an omitted field and a present empty Structure or zero value.
- **KMIPKIT-0017-FR-003**: Get MUST support the optional Unique Identifier, Key Format Type, Key Wrap Type, Key Compression Type, and Key Wrapping Specification fields without synthesizing caller values.
- **KMIPKIT-0017-FR-004**: A successful Get result MUST expose Object Type, Unique Identifier, and the complete Any Object TTLV Structure without parsing or transforming cryptographic material. The object payload MUST remain redacted from formatting and use the existing zeroizing TTLV ownership contract.
- **KMIPKIT-0017-FR-005**: Get format handling MUST follow §6.1.19's key-format capabilities and restrictions. KMIPKit MUST preserve explicit format and wrapping options and MUST NOT perform unrequested key conversion, wrapping, unwrapping, PKCS#12 construction, or certificate-chain processing.
- **KMIPKIT-0017-FR-006**: Locate MUST encode the required Attributes Structure and optional Maximum Items, Offset Items, Storage Status Mask, and Object Group Member fields according to Table 247. An explicitly supplied zero Offset Items and an empty Attributes Structure MUST remain representable.
- **KMIPKIT-0017-FR-007**: Locate MUST preserve generic search attributes, including repeated and partially specified structured values, date ranges, Cryptographic Usage Mask values, and Usage Limits values, without evaluating them against local state.
- **KMIPKIT-0017-FR-008**: Locate MUST expose optional Located Items and zero or more repeated Unique Identifier response fields as received, preserving wire order, including an empty response payload.
- **KMIPKIT-0017-FR-009**: Locate MUST represent the online, archived, and destroyed search indicators without inventing a default mask. If the mask is omitted, omission is retained and documented as the online-object default. The client MUST NOT filter or normalize returned identifiers.
- **KMIPKIT-0017-FR-010**: Batch execution MUST leave ID Placeholder state to the server, preserve Locate results, and avoid splitting, auto-follow-up, retry, or local uniqueness assumptions. Archived object retrieval remains Recover followed by Get and is outside this feature's implementation scope.
- **KMIPKIT-0017-FR-011**: Get and Locate responses MUST preserve their operation-specific Result Status, Result Reason, and Result Message and reuse the shared Pending and delivery-state contracts without automatic polling or retry.
- **KMIPKIT-0017-FR-012**: Malformed or over-limit object payloads MUST be rejected under the configured generic TTLV limits before unbounded allocation, and sanitized diagnostics MUST omit raw payload and secret contents.
- **KMIPKIT-0017-FR-013**: Every normative requirement confirmed as applicable to the client MUST link from the catalog to this specification, implementation, and executable verification. Server-only and retired rows must be distinguished from client behavior. Official fixtures and any remaining availability gaps MUST be recorded without claiming a pass before full-case execution.

### Key Entities

- **Get Request**: Optional Unique Identifier and the optional key format, wrap type, compression type, and wrapping specification values requested by the caller.
- **Retrieved Object**: The required response Object Type, Unique Identifier, and ordered generic KMIP object Structure. Its contents are secret-bearing and remain opaque to KMIPKit's transport/client layer.
- **Locate Query**: Required Attributes Structure plus optional maximum count, offset, storage mask, and group-member selector.
- **Locate Result**: Optional total Located Items count and an ordered zero-or-more list of Unique Identifier values.
- **Operation Result**: Shared operation status, optional reason/message, Pending outcome, and request delivery state.

## Success Criteria

### Measurable Outcomes

- **KMIPKIT-0017-SC-001**: Both operation request and response models represent every field in Tables 220–222 and 247–249, including operation-specific error fields, with the specified presence, type, repetition, and order.
- **KMIPKIT-0017-SC-002**: Fake-transport tests demonstrate exact request preservation and one-exchange behavior for all Get and Locate success, error, Pending, omission, empty, repeated, and batch-placeholder cases in scope.
- **KMIPKIT-0017-SC-003**: Tests demonstrate lossless preservation of a returned object containing nested values, unknown allocated tags/enumerations, and repeated fields, while developer diagnostics contain none of the sentinel object payloads.
- **KMIPKIT-0017-SC-004**: Tests cover every source-linked OASIS case whose fixture is available; each missing fixture has a traceable local derived vector and is explicitly not reported as an official-case pass.
- **KMIPKIT-0017-SC-005**: Every client-applicable requirement has a catalog-to-spec-to-code-to-test path; server-side obligations are explicitly distinguished and never presented as behavior implemented by the client.
- **KMIPKIT-0017-SC-006**: Formatting, lint, documentation, workspace test, coverage, and supported-platform CI checks pass the repository's applicable gates before the implementation PR is review-ready.

## Inventory Dispositions

- KMIPKIT-DISC-015 is resolved by KMIPKIT-DEC-003: the §6.1.19 lowercase “shall” paragraph describes server PKCS#12 output and does not create a separate RFC 2119 client duty. This feature preserves caller options and opaque server output without local container validation.
- KMIPKIT-DISC-032 is resolved by KMIPKIT-DEC-004: in Test Cases §§2.68–2.69, section headings and link targets identify `TC-PKCS12-…`; the visible `TC-PKCS-12-…` text is inconsistent. Both target XML files are pinned by URL and checksum. Their presence is not an official-case pass.
- KMIPKIT-DISC-044 is resolved by KMIPKIT-DEC-005: KMIPKIT-REQ-SPEC-6.1.28-004-002, -008-001, -009-001, -009-002, and -012 are server-only. KMIPKIT-REQ-SPEC-6.1.28-008-002 is retired because the Group Member Default sentence has no separate normative keyword. The client preserves request/result values but does not enforce server behavior.

These dispositions do not authorize the client to invent, filter, or normalize server data. Implementation and conformance claims still require executable evidence and human specification approval.

## Assumptions

- The active `release/1.0.0` branch contains the approved bounded TTLV model, lossless generic value representation, common result/error model, client execution path, batch model, async outcome handling, and production transport needed by this feature.
- Get's response object can be represented by the existing generic TTLV Structure; the operation feature will not introduce a second object-value tree or secret-storage mechanism.
- The server is authoritative for Locate matching, object group policy, maximum-result caps, identifier ordering, and ID Placeholder state. The client preserves requests/results and does not emulate the server.
- Only OASIS client-to-server operations are in this feature. Test-case mappings sourced from profiles do not constitute a profile support claim.
- The specific first 1.0 API surface remains Rust protocol/client; C, Java, and Python parity are separate planned specifications.

## Out of Scope

- Recover and the automatic archived-object workflow.
- Typed object-body or cryptographic algorithm APIs, local key import/export conversion, PKCS#12 generation, key wrapping/unwrapping, and certificate-chain computation.
- Local object persistence/search, server implementation, response reordering, request retries, automatic polling, JSON/XML, and non-Rust bindings.
- Formal KMIP profile certification or a general server interoperability claim.
