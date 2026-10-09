# Feature Specification: Server-Generated Object Creation

**Feature Branch**: feature/KMIPKIT-0014-attribute-schema-correction

**Created**: 2026-10-07

**Status**: Accepted for implementation. Human maintainer acceptance is recorded by the merge of [PR #53](https://github.com/NeverWe1come/KMIPKit/pull/53) into `release/1.0.0` (merge commit `a2572084d3d6f42019bed13234598e6d77478796`). Source and catalog corrections were merged in [PR #57](https://github.com/NeverWe1come/KMIPKit/pull/57) (merge commit `55c0c0cc42a6422cee72183a3532fc1e169f297c`). The implementation base contains both commits. No separate GitHub review event is claimed.
**Input**: KMIPKit roadmap Phase D: complete client-initiated KMIP 2.1 operation families.

## Normative scope

This specification adds typed support for three server-generated object creation operations:

| Operation | Normative source | Catalog element |
| --- | --- | --- |
| Create | OASIS KMIP Specification v2.1 §6.1.8, Tables 186–188 | KMIPKIT-ELEM-OP-C2S-CREATE |
| Create Key Pair | OASIS KMIP Specification v2.1 §6.1.9, Tables 189–192 | KMIPKIT-ELEM-OP-C2S-CREATE-KEY-PAIR |
| Create Split Key | OASIS KMIP Specification v2.1 §6.1.10, Tables 193–195 | KMIPKIT-ELEM-OP-C2S-CREATE-SPLIT-KEY |

Shared message and result behavior follows OASIS KMIP Specification v2.1 §§8.1–8.6, 9.1–9.2, 9.5–9.9, 9.12–9.13, 9.16, and 9.19–9.21 and their applicable tables. Operation payload fields follow §§6.1.8–6.1.10, Tables 186–195. Attribute groups follow §§5.1–5.4, Tables 157–160: their members are direct §4 Object Attribute TTLV items and may repeat. The separate §4.60 Vendor Attribute structure is defined by Table 150 and requires Vendor Identification. The immutable local OASIS copy and checked-in catalog are authoritative; the Usage Guide is informative.

The catalog assigns only these three operation elements and the applicable existing §6.1.9 and §6.1.10 requirements to KMIPKIT-0014; it does not claim full 1.0 operation coverage. The §2.8/Table 9 Split Key object requirement is not assigned as a Create Split Key request obligation. The pinned OASIS KMIP Test Cases work product lists `TC-CREATE-SD-1-21` (§2.12). Its byte-identical XML is available at `specification/oasis/kmip-2.1/fixtures/TC-CREATE-SD-1-21.xml`; its source URL, version, date, and SHA-256 are recorded in `specification/oasis/kmip-2.1/SOURCES.md`. The catalog retains the Secret Data object association and marks the fixture `available`; availability is not conformance evidence. The XML contains Create and Get request/response batch items. This feature uses only the Create request/response item; Get remains out of scope, so its executable fixture is a clearly labeled Create-only derivation and MUST NOT be reported as passing the complete official test case. No direct Create Key Pair or Create Split Key cases appear in the work product. Other table-derived tests remain labeled separately from official OASIS test-case evidence.

### Normative requirements carried by this feature

- KMIPKIT-REQ-SPEC-6.1.9-001-001: MAY distribute attributes shared by both keys through Common Attributes.
- KMIPKIT-REQ-SPEC-6.1.9-001-002: MAY supply differing attributes through Private Key Attributes and Public Key Attributes.
- KMIPKIT-REQ-SPEC-6.1.9-006 and KMIPKIT-REQ-SPEC-6.1.9-007: SHALL use identical values for the attributes identified by Table 191 across the private and public keys.
- KMIPKIT-REQ-SPEC-6.1.10-001: MAY identify an existing cryptographic object to split by including its Unique Identifier.

Table-required fields, optionality, cardinality, and operation-specific errors are binding requirements even where the catalog represents them as table structure rather than as a separate SHALL/MUST record. The implementation change must map every applicable catalog clause, requirement, and protocol element to a stable feature requirement, implementation location, and executable test.

### Explicit client policy

KMIPKIT-0014-FR-015 is a KMIPKit client restriction: for Polynomial Sharing Prime Field, the caller supplies Prime Field Size explicitly and KMIPKit does not choose or synthesize it. This is not an OASIS requirement that the Create Split Key request contain the field. Section 2.8, Table 9 concerns the Split Key object; §6.1.10, Table 193 marks the request field optional.

## Product boundaries and exclusions

- Encoding is TTLV only. Only the three named client-to-server operations are added.
- Requests use the existing typed client and common batch/message path. Each explicit invocation performs at most one exchange. Pending outcomes follow the existing asynchronous contract; no background polling, retry, scheduling, or server-initiated operation is added.
- Cryptographic algorithm, length, domain parameters, key usage, split method, threshold, and protection/storage choices are explicit caller inputs when the cited tables require them. KMIPKit MUST NOT choose them implicitly.
- Unknown values remain lossless in generic TTLV. Typed outbound values follow the accepted catalog policy for assigned values and registered extensions.
- Standard operation failures, including Operation Not Supported, are returned as KMIP results. Their representation does not claim a server supports the operation.
- This feature does not add Get, Locate, Register, Import, Export, lifecycle transitions, attribute operations, cryptographic processing, credential/session operations, C ABI methods, Java/Python bindings, JSON/XML, or server support.
- The server performs key generation and split-key creation. KMIPKit is not a local cryptographic algorithm provider.

## User Scenarios & Testing

### User Story 1 — Create one managed object (Priority: P1)

A caller explicitly requests the server to create one object of a selected KMIP Object Type with caller-selected attributes and creation options. On success, the caller receives the returned Object Type and Unique Identifier.

**Why this priority**: Create is the smallest server-generated object request and exercises the operation path without returning key material.

**Independent Test**: A deterministic fake transport exercises a Create-only TTLV fixture derived from the first batch item of the locally pinned `TC-CREATE-SD-1-21` XML. The second Get item is out of scope and is not executed or counted as full official-case coverage. A separate table-derived case verifies other Table 186–188 fields, typed results, and one-exchange behavior.

**Acceptance Scenarios**:

1. Given a valid Create request, when encoded, then Object Type and all supplied fields follow Table 186 structure and cardinality, including its required outer Attributes structure.
2. Given a Create request with no attribute members, when encoded, then the required Attributes structure is present and empty, as permitted by §5.1 and Table 157.
3. Given a successful response, when decoded, then Object Type and Unique Identifier are returned exactly as encoded in Table 187.
4. Given an operation failure, when processed, then KMIP status, reason, and message are preserved and the request is not retried.
5. Given an unrecognized value inspected through generic TTLV, when decoded, then its raw value remains available without reinterpretation.

### User Story 2 — Create a public/private key pair (Priority: P1)

A caller explicitly requests a key pair and supplies common and key-specific attributes. The response identifies both created keys.

**Why this priority**: Key-pair creation is a central managed-key workflow, and its attribute-sharing rules are normative and independently testable.

**Independent Test**: Table-driven tests encode and decode Tables 189–190, verify request-side Table 191 consistency, and prove attribute groups and repeated values are preserved for the server to apply the Table 189 union rule.

For each Table 191 attribute, the server applies Private Key Attributes and Public Key Attributes in preference to Common Attributes under §6.1.9. For request validation, the supplied effective value for each key is its key-specific value when present, otherwise its Common Attributes value. If both supplied effective values are absent, the caller has not selected that value and KMIPKit does not invent one. If only one effective value is present, or both are present but differ, KMIPKit rejects the request. Equal effective values are accepted, including equal key-specific overrides that differ from the Common Attributes value. KMIPKit preserves all three groups and never copies values between them.

**Acceptance Scenarios**:

1. Given a key-pair request, when attributes are assigned, then Common Attributes, Private Key Attributes, and Public Key Attributes remain distinguishable; every repeated value is preserved in its group, and the server applies the union rule after Table 189.
2. Given any Table 191 attribute, when the request is validated, then the key-specific value takes precedence over the Common Attributes value for that key; if both effective values are absent the caller selection remains absent; if only one is present the request is rejected; equal effective values are accepted even when both key-specific overrides differ from Common Attributes; differing effective values are rejected.
3. Given a successful response, when decoded, then both required Unique Identifiers retain their private/public meaning.
4. Given unsupported cryptographic parameters or a server policy rejection, when returned as a KMIP result, then no replacement parameters are selected.
5. Given an operation failure, when processed, then the KMIP Result Reason required by §6.1.9.1 Table 192 is preserved by the shared result model.

### User Story 3 — Create split-key parts (Priority: P2)

A caller explicitly requests a split-key operation using the method, part count, threshold, and optional controls defined in Table 193. The caller may identify an existing cryptographic object to split. On success, the caller receives all returned Unique Identifiers in wire order.

**Why this priority**: Split-key creation is a distinct KMIP capability with repeated results and threshold controls; full representation prevents truncation and data loss.

**Independent Test**: Deterministic tests cover Table 193 fields, the optional input identifier, one and multiple Table 194 identifiers, malformed response cardinalities, and Table 195 errors.

**Acceptance Scenarios**:

1. Given a split-key request with an existing Unique Identifier, when encoded, then that exact value is sent; when omitted, the client does not invent one.
2. Given a successful response with repeated Unique Identifier fields, when decoded, then every identifier and its wire order are preserved.
3. Given invalid thresholds, unsupported methods, or a missing input object, when the server returns an operation error, then that result, including the Table 195 Result Reason, is preserved without retry or local cryptographic fallback.
4. Given supplied attributes that disagree with an identified input key, when processing follows §6.1.10, then the model does not claim the supplied values override that key's attributes.
5. Given a request that may return the repeated identifiers from Table 194, when executed, then the request advertises a Maximum Response Size no greater than the client's configured local response-byte limit; an exact-limit response is accepted and a response one byte over the local limit is rejected before decoder entry even if the peer ignores the advertised value.
6. Given Split Key Method Polynomial Sharing Prime Field, when the request is validated, then KMIPKit's explicit client policy requires the caller's Prime Field Size value and preserves it; Table 193 still defines the request field as optional, and KMIPKit invents no value.
7. Given a Create Split Key request, when encoded, then its required Attributes structure is present and may be empty under §5.1 and Table 157.

### Edge Cases

- Omitted optional fields remain distinct from present-empty collections where the tables permit both. Create and Create Split Key each include the required outer Attributes structure and it may contain no members under §5.1; optional Create Key Pair attribute structures may be absent or present-empty.
- Duplicate, unknown, or extension values are not silently collapsed by generic TTLV handling. Attribute groups preserve ordered direct §4 attribute TTLV items and their tags and typed values. A Vendor Attribute is the distinct §4.60 Table 150 structure, including its required Vendor Identification. Typed validation rejects only conditions required by the normative tables or accepted policy.
- Missing, malformed, or wrong-type response fields are rejected before returning typed success.
- A successful KMIP status without the table-required payload is a protocol error; a Failure status is not treated as successful creation.
- Create Split Key response identifiers are repeated values. The response requires at least one and may repeat; no uniqueness rule is invented beyond the pinned text.
- Pending is accepted only under the existing asynchronous rules and is never turned into an automatic Poll.
- Caller-provided cryptographic choices and raw message bodies are never logged or included in errors.

## Requirements

### Functional Requirements

- **KMIPKIT-0014-FR-001**: The client MUST expose typed request and response models for Create, Create Key Pair, and Create Split Key, each associated with its exact KMIP operation identifier.
- **KMIPKIT-0014-FR-002**: Create requests and responses MUST preserve required, optional, repeated, and ordered fields in §6.1.8, Tables 186–187. Create operation failures MUST preserve the Result Reasons in Table 188 through the shared KMIP result contract.
- **KMIPKIT-0014-FR-003**: Create Key Pair requests and responses MUST preserve fields and multiplicity in §6.1.9, Tables 189–190. Common, private, and public attributes MUST remain distinguishable and every repeated value MUST be preserved in its group. KMIPKit MUST NOT merge or reorder groups; the server applies the Table 189 union semantics when creating the keys.
- **KMIPKIT-0014-FR-004**: For each attribute identified by Table 191 and requirements KMIPKIT-REQ-SPEC-6.1.9-006 and -007, request validation MUST apply the §6.1.9 precedence independently for each key: a key-specific value overrides Common Attributes, otherwise the Common Attributes value applies. If both resulting supplied values are absent, they remain unspecified; if exactly one is present or both differ, the request MUST be rejected; if both are equal, it is accepted. KMIPKit MUST NOT copy values between key-specific groups or invent an omitted value.
- **KMIPKIT-0014-FR-005**: Create Split Key requests and responses MUST preserve fields, optionality, and repeated Unique Identifier values in §6.1.10, Tables 193–194. Failures MUST preserve Table 195 Result Reasons. The optional input Unique Identifier MUST be represented and encoded exactly when supplied, as required by KMIPKIT-REQ-SPEC-6.1.10-001. When input-key attributes take precedence under §6.1.10, the client MUST NOT report conflicting request attributes as authoritative effective key attributes.
- **KMIPKIT-0014-FR-006**: All cryptographic choices required for a request MUST be explicit caller inputs. KMIPKit MUST NOT infer algorithms, lengths, domain parameters, key usage, split method, threshold, or protection/storage policy.
- **KMIPKIT-0014-FR-007**: All three operations MUST use the existing typed request path and common batch association/validation rules, perform at most one transport exchange per invocation, and preserve delivery state and redacted errors.
- **KMIPKIT-0014-FR-008**: The operations MUST honor the asynchronous indicator rules. For a permitted Pending response with its required correlation value, the client MUST preserve the originating operation identity and exact KMIP result in an operation-agnostic Pending outcome. It MUST NOT model Pending as a Discover Versions result, automatically Poll, Process, retry, or claim completion.
- **KMIPKIT-0014-FR-009**: KMIPKit-owned request/response buffers MUST retain existing zeroization behavior. Public Debug output and error context for operation models MUST redact attribute-item values; Debug, Display, errors, logs, and diagnostics MUST NOT reveal private key material, raw KMIP bodies, or secret-bearing values.
- **KMIPKIT-0014-FR-010**: Every applicable requirement and table field MUST link the pinned OASIS section/table, a stable feature requirement, implementation location, and executable test. Task T001 MUST record applicable official OASIS Test Case IDs and pin the byte-identical `TC-CREATE-SD-1-21` source XML. The Create batch item may have a separately labeled Create-only TTLV derivation with executable evidence; because the official case also contains an out-of-scope Get item, KMIPKit MUST NOT claim the complete official test case passes. Table-derived tests MUST be labeled separately and MUST NOT be represented as official vectors.
- **KMIPKIT-0014-FR-011**: The change MUST NOT edit pinned OASIS copies or generated files manually, claim profile support or certification, broaden the 1.0 boundary, or add non-TTLV encodings, server-initiated operations, language bindings, or local cryptographic algorithms.
- **KMIPKIT-0014-FR-012**: Create Split Key responses may contain a repeated list of Unique Identifiers (Table 194), so each request batch containing Create Split Key MUST include the §9.12 Maximum Response Size field. Set it to the smaller of the configured local response-byte limit and the largest KMIP Integer value; never advertise a size greater than the local limit. This is KMIPKit's application of the §9.12 recommendation for potentially large replies. The client MUST independently enforce its local response limit and reject an over-limit response before TTLV decoder entry even if the peer ignores the advertised value.
- **KMIPKIT-0014-FR-013**: Create Key Pair operation failures MUST preserve the Result Reasons defined by §6.1.9.1 Table 192 through the shared KMIP result contract.
- **KMIPKIT-0014-FR-014**: Attribute groups MUST preserve their direct §4 Object Attribute TTLV items, including each item's tag and typed value, repeated items, and wire order, as defined by §§5.1–5.4, Tables 157–160. Unknown attribute tags MUST remain representable and round-trip without tag synthesis or narrowing. The Vendor Attribute is the distinct §4.60 Table 150 structure and MUST include Vendor Identification. Validation for recognized attributes MUST follow the applicable catalog and operation rules.
- **KMIPKIT-0014-FR-015**: As an explicit KMIPKit client restriction, a Create Split Key request using Polynomial Sharing Prime Field MUST receive Prime Field Size as an explicit caller input and MUST NOT choose or synthesize it. This restriction is not attributed to OASIS: §6.1.10 Table 193 marks the request field optional, while §2.8 Table 9 describes the Split Key object.

### Key Entities

- **Object Type**: KMIP object category selected for server-side creation.
- **Creation Attributes**: Caller-supplied attributes, including separate common/private/public sets for key-pair creation.
- **Split-Key Parameters**: Caller-supplied split method, part count, threshold, optional input identifier, and table-defined controls.
- **Unique Identifier**: Server-assigned identifier returned for each created object; split-key responses may repeat this field.
- **Operation Result**: KMIP status, reason, message, and operation payload associated with the request.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All three request/response payloads round-trip every field defined by their cited tables, including required outer attribute structures, permitted empty structures, optional fields, and repeated values.
- **SC-002**: Negative tests reject missing, malformed, duplicate, or incorrectly typed mandatory fields while preserving permitted unknown values through generic TTLV.
- **SC-003**: Create Key Pair tests prove the Table 191 request-consistency rules and verify repeated attribute values remain intact in each group for the server-side union behavior.
- **SC-004**: Create Split Key tests preserve one or more returned identifiers without truncation or reordering.
- **SC-005**: Every normative requirement in scope has 100% traceability from pinned source through this spec to implementation and executable tests before implementation review.
- **SC-006**: Operation tests prove at most one exchange per invocation, keep raw message bodies and secret-bearing values out of diagnostics, and preserve operation failures without automatic retry.
- **SC-007**: A Create Split Key execution includes a peer-visible Maximum Response Size no greater than the configured local limit; an exact-limit response is accepted and a response one byte over is rejected before decoder entry even if the peer ignores the advertised value.
- **SC-008**: The English and Spanish client guides include Rust examples for Create, Create Key Pair, and Create Split Key, and a repository check compiles every example marked `rust,kmipkit-test` from both guides.

## Assumptions

- The KMIPKIT-0005 TTLV codec and KMIPKIT-0006 message/batch model remain the wire foundation.
- The existing typed client execution and KMIPKIT-0009 asynchronous result handling are extended without reopening accepted semantics.
- A server may reject any operation or parameters according to its implementation and policy; support is not implied by the client model.
- This is a Rust protocol/client slice. A later API parity feature will expose equivalent capabilities through C, Java, and Python.
- Exact field rules and enumerated values come from the pinned OASIS v2.1 source and catalog, not server examples or the Usage Guide.
- Create Split Key is treated as a potentially large response because Table 194 permits a repeated Unique Identifier list whose size scales with the requested split parts. `Client::execute` already receives the local response-byte limit and uses it as the source of the peer-visible maximum for batches containing this operation.

## Clarification record

No unresolved clarification is required for this specification. The bounded operation set is Create, Create Key Pair, and Create Split Key because they are the server-generated creation family selected from the 1.0 roadmap. Request fields, response cardinality, attribute grouping, and operation-specific errors are determined by the cited pinned OASIS tables. Create Split Key input-key attribute precedence is taken directly from §6.1.10. Pending behavior, one-exchange delivery reporting, zeroization, and unknown-value preservation reuse accepted KMIPKIT-0007/0009/architecture contracts. All other operations and language bindings are explicitly excluded and remain assigned to later specifications.
