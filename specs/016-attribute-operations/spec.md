# Feature Specification: KMIP 2.1 Attribute Operations

**Feature Branch**: `feature/KMIPKIT-0016-attribute-operations`
**Created**: 2026-10-08
**Status**: Draft for independent human review
**Input**: KMIPKit roadmap Phase D: implement the client-initiated attribute-operation family for KMIP 2.1.

## Normative scope

This specification covers exactly seven client-to-server operations:

| Operation | Catalog element | Normative source | Request/response/error tables |
| --- | --- | --- | --- |
| Add Attribute | `KMIPKIT-ELEM-OP-C2S-ADD-ATTRIBUTE` | OASIS KMIP Specification v2.1 §6.1.2 | Tables 167–169 |
| Adjust Attribute | `KMIPKIT-ELEM-OP-C2S-ADJUST-ATTRIBUTE` | §6.1.3 | Tables 170–172; Adjustment Type is §11.1, Tables 428–429 |
| Delete Attribute | `KMIPKIT-ELEM-OP-C2S-DELETE-ATTRIBUTE` | §6.1.13 | Tables 202–204 |
| Get Attributes | `KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES` | §6.1.20 | Tables 223–225 |
| Get Attribute List | `KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTE-LIST` | §6.1.21 | Tables 226–228 |
| Modify Attribute | `KMIPKIT-ELEM-OP-C2S-MODIFY-ATTRIBUTE` | §6.1.34 | Tables 265–267 |
| Set Attribute | `KMIPKIT-ELEM-OP-C2S-SET-ATTRIBUTE` | §6.1.51 | Tables 322–324 |

Shared attribute and request-value definitions follow §§4, 5.1, and 5.5–5.7, including Table 161 (Attribute Reference), Table 162 (Current Attribute), and Table 163 (New Attribute). Shared message and result behavior follows the applicable structures in §§8.1–8.6 and 9.1–9.2, 9.5–9.9, 9.12–9.13, 9.16, and 9.19–9.21. The pinned KMIP 2.1 Specification and the checked-in normative catalog are authoritative; Usage Guide examples are informative.

The implementation PR must assign every applicable catalog clause, requirement, and test case to this specification, its code, and an executable test. No operation is considered covered solely because a generic TTLV value can carry its payload.

## Product boundaries and exclusions

- Encoding is TTLV only. This is a Rust protocol/client slice using the accepted typed request, response, message, batch, and transport foundations.
- The scope is limited to the seven operations above. It does not implement object lifecycle, object transfer, cryptographic processing, other client operations, or server-initiated operations.
- C ABI, Java, and Python parity and high-level builders are tracked in the later 1.0 API work. This specification does not claim those language surfaces are complete.
- KMIPKit enforces client-side prohibitions that are explicit and identifiable from the pinned KMIP standard, including known standard Read-Only attributes, operation-specific restrictions, and the §4.60 Vendor Attribute rule when Vendor Identification is present in a supplied value. It does not infer mutability for other unknown vendor/future names or simulate server policy that depends on remote state; those outcomes remain server-authoritative and their KMIP status, reason, and message are preserved.
- Standard and vendor Attribute Name text remains representable exactly. Values using individually assigned tags or tags accepted by the KMIPKIT-0004/ADR-0010 §11.56 allocation gate remain lossless through generic TTLV handling; a Raw Tag that fails that gate cannot become an encodable Item. The implementation MUST NOT synthesize a tag from an unrecognized name. Unknown Enumeration values remain representable in the generic model, but an outbound Adjustment Type MUST be Increment (1), Decrement (2), Negate (3), or an extension value in `0x80000000–0x8FFFFFFF`; reserved values MUST be rejected before transmission.
- Each explicit invocation makes at most one request exchange. No automatic retry, background polling, local attribute mutation, or cryptographic operation is added.
- No change to the protocol/version, TLS policy, transport boundaries, immutable OASIS source copies, or public language scope is included.

## User Scenarios & Testing

### User Story 1 — Read managed-object attributes (Priority: P1)

As a KMIPKit caller, I can request attribute values or the names of available attributes for a managed object, so I can inspect the server-held metadata without losing repeated values or changing server state.

**Why this priority**: Read operations provide the basis for callers to inspect object state and verify later writes.

**Independent Test**: A deterministic fake transport returns Get Attributes responses covering selected, omitted, and repeated values, plus Get Attribute List responses covering the full repeated name list; typed results preserve all wire values and order.

**Acceptance Scenarios**:

1. **Given** Get Attributes names one or more unique Attribute References, **When** the server returns matching attributes, **Then** every instance of each named attribute is returned and an absent requested attribute is omitted.
2. **Given** Get Attributes has no Attribute Reference, **When** the server returns the object attributes, **Then** all returned attributes are represented without narrowing their TTLV values.
3. **Given** none of the requested Get Attributes names exists, **When** a successful response is decoded, **Then** the response still contains its required Unique Identifier and no fabricated attribute value.
4. **Given** Get Attribute List is requested with its optional Unique Identifier and no other request selector, **When** a successful response is decoded, **Then** the server's full list of attribute names is represented as required Attribute Reference entries, including every repeated response entry in wire order, as required by the §6.1.21 prose and Table 227.
5. **Given** the server rejects a read request, **When** the result is processed, **Then** its operation-specific Result Reason is preserved and the request is not retried.

### User Story 2 — Add, adjust, modify, set, or delete an attribute (Priority: P1)

As a KMIPKit caller, I can choose the KMIP operation whose semantics match my intended attribute change, so adding a value, adjusting it, replacing an existing instance, setting a single-valued attribute, and deleting values remain distinct requests.

**Why this priority**: Callers must express their intended state transition explicitly; collapsing these operations could replace or delete values contrary to KMIP semantics.

**Independent Test**: Table-driven fake-transport tests encode each operation's exact request shape, verify its response, and check that each operation returns its operation-specific errors without local state changes.

**Acceptance Scenarios**:

1. **Given** Add Attribute targets an existing attribute, **When** the request is executed, **Then** the client sends the new value as a distinct instance and does not model it as replacing an existing value.
2. **Given** Adjust Attribute selects an assigned Adjustment Type and optional Adjustment Value, **When** the request is encoded, **Then** both fields and their types follow §6.1.3 and §11.1; an omitted Adjustment Value uses the operation-defined parameter default (Increment/Decrement: 1 for numeric types, 1 second for Date Time, and 1 microsecond for Date Time Extended); the client does not calculate a replacement value locally.
3. **Given** a caller supplies an unknown Adjustment Type extension value, **When** it lies in `0x80000000–0x8FFFFFFF`, **Then** the generic value remains intact for the request; if it is outside the assigned or extension range, the client rejects it before transmission as reserved.
4. **Given** Adjust Attribute targets an object with no value for the referenced attribute, **When** the server applies the request, **Then** its starting value is 0 for numeric types and intervals, false for Boolean, and any other type produces an error, as specified in §6.1.3.
5. **Given** Modify Attribute includes a Current Attribute, **When** the request is encoded, **Then** it identifies the exact old value and the New Attribute value remains distinct.
6. **Given** Modify Attribute omits Current Attribute, **When** the target attribute is multi-instance, **Then** the client still sends the request as specified and reports the server's result without selecting or modifying an instance locally.
7. **Given** Set Attribute targets an attribute with zero or one instance, **When** the request is executed, **Then** the caller's New Attribute value is transmitted unchanged; the server applies the add-or-modify behavior.
8. **Given** Set Attribute targets a multi-instance attribute, **When** the server rejects the request, **Then** the client preserves the returned error and does not choose an instance.
9. **Given** Delete Attribute specifies an Attribute Reference and no Current Attribute, **When** the request is encoded, **Then** omission is preserved so the server can apply the normative all-instances deletion behavior.
10. **Given** Delete Attribute omits both Current Attribute and Attribute Reference, **When** the request is encoded, **Then** the client preserves both omissions and exposes the server's result without inventing a selector or applying local state.
11. **Given** a mutation targets a known standard attribute that the pinned definition marks Read-Only or prohibits for that operation, **When** execution is requested, **Then** KMIPKit returns a local validation error with delivery state `NotSent` and performs no transport exchange.
12. **Given** a mutation targets an unknown/vendor attribute or depends on server/object state not represented in the request, **When** the operation is otherwise structurally valid and does not match the explicit §4.60 Vendor Attribute exception below, **Then** KMIPKit does not invent an access rule or issue a preflight request; it sends at most one exchange and preserves the server result.
13. **Given** Add Attribute or Modify Attribute supplies a New Attribute named `Usage Limits`, **When** KMIPKit validates the request, **Then** it rejects the request locally with delivery state `NotSent`: §7.40 Table 392 requires the Count member in a Usage Limits value, and §4.59 prohibits clients from setting or modifying that Count through Add Attribute or Modify Attribute.
14. **Given** an Add, Modify, Set, or Delete request supplies a Vendor Attribute value whose Vendor Identification is `y`, **When** KMIPKit can inspect that value in the request, **Then** it rejects the corresponding mutation locally as prohibited by §4.60 and reports `NotSent`. For Modify it checks both supplied Current Attribute and New Attribute values; for Delete it can check a supplied Current Attribute value. Adjust and reference-only Delete requests carry no Vendor Identification, so KMIPKit does not infer `y` from an Attribute Reference and preserves the server result.
15. **Given** a supplied Vendor Attribute value has Vendor Identification `x` or another value not covered by the server-created `y` rule, **When** it is otherwise structurally valid, **Then** KMIPKit applies no prohibition based on the `y` rule, sends at most one exchange, and preserves the server result without inferring ownership or writability.

### User Story 3 — Preserve attribute identity and values (Priority: P1)

As a KMIPKit caller, I can use standard, vendor, repeated, and future attribute values without losing their names, tags, types, or order, so protocol data remains usable as the standard and vendors evolve.

**Why this priority**: Attribute values carry application metadata and may use extension tags unknown to this release.

**Independent Test**: Generic TTLV round-trip tests include recognized and unrecognized Attribute Names and values, repeated attributes, and unknown enum values in Adjustment Type; the encoded result retains the original names, values, and ordering.

**Acceptance Scenarios**:

1. **Given** an Attribute Name is unknown to KMIPKit and its value uses an allocated tag or an accepted §11.56 extension tag, **When** it is represented through generic TTLV, **Then** its exact name and complete tagged value remain available after a round trip; a reserved Raw Tag cannot be encoded as an Item.
2. **Given** an Attribute Name is repeated where the operation permits repeated values, **When** a response is decoded, **Then** every entry remains distinct and in wire order.
3. **Given** a caller constructs an unknown Adjustment Type extension value in generic TTLV, **When** it is in the OASIS extension range, **Then** its raw value remains available for encoding; reserved values outside assigned and extension ranges are rejected before transmission. Adjustment Type is request-only and is not expected in the Adjust Attribute response.

## Edge Cases

- Unique Identifier is omitted in an operation whose request table permits omission; the request remains omitted so the KMIP ID Placeholder behavior applies.
- Get Attributes contains the same Attribute Reference twice; the client rejects this invalid request locally as required by §6.1.20.
- Get Attribute List's request contains only the optional Unique Identifier; its successful response must contain at least one Attribute Reference as required by §6.1.21 Table 227.
- Delete Attribute omits both optional selectors; the client preserves that request shape and returns the server result without guessing a selection rule.
- A Current Attribute does not identify an existing attribute value; the server's operation error is preserved.
- A mutation targets a standard attribute marked non-modifiable/non-deletable by its source-defined attribute rules; the client rejects it before transport. Other vendor/unknown names and state-dependent policy failures are returned by the server and preserved without retry, subject to the inspectable §4.60 Vendor Attribute exception above.
- Add Attribute and Modify Attribute reject every New Attribute named `Usage Limits`: §7.40 Table 392 makes Count a required member, and §4.59 prohibits setting or modifying Count through these two operations. This rule is not extended to Adjust, Delete, or Set absent a source requirement.
- Under §4.60 and Table 150, a supplied Vendor Attribute value with Vendor Identification `y` is rejected for Add, Modify, Set, or a Delete carrying Current Attribute. Adjust and reference-only Delete cannot expose the identifier in the request; no identifier is inferred and the server result is preserved.
- Adjust Attribute is applied to an absent value: the server assumes 0 for numeric types and intervals, false for Boolean, and returns an error for other types; a multi-instance value also produces the operation-defined error.
- Adjustment Value is omitted: Increment/Decrement use the §11.1 Table 428 parameter defaults (1 for numeric types, 1 second for Date Time, 1 microsecond for Date Time Extended); no client-side arithmetic is performed.
- A successful read omits a named attribute or returns multiple instances; absence is not synthesized and repeated instances are not collapsed.
- A response is malformed, exceeds configured decoder limits, or contains an unknown value; malformed/over-limit input is rejected before unsafe allocation, allocated unknown tags remain lossless, and Reserved tags cannot enter the public generic Item model.
- The request may have been sent before a transport failure; the existing delivery-state contract reports whether it was not sent, possibly sent, or response reception began.

## Requirements

### Functional Requirements

- **KMIPKIT-0016-FR-001**: The client MUST provide typed requests and responses for Add Attribute, Adjust Attribute, Delete Attribute, Get Attributes, Get Attribute List, Modify Attribute, and Set Attribute, using the request and response structures in their cited OASIS tables.
- **KMIPKIT-0016-FR-002**: The client MUST preserve each operation's required, optional, and repeatable payload fields and their specified order/cardinality. Current Attribute, New Attribute, Attribute Reference, and Attribute MUST remain semantically distinct.
- **KMIPKIT-0016-FR-003**: Add Attribute MUST preserve the submitted value as a distinct request value and MUST NOT model it as changing existing values. Before transmission, the client MUST reject a target standard attribute that the pinned source marks Read-Only or otherwise prohibits for Add Attribute. If the New Attribute is `Usage Limits`, the client MUST reject it before transmission because §7.40 Table 392 requires Count and §4.59 prohibits setting Count through Add Attribute. If it is Vendor Attribute with Vendor Identification `y`, the client MUST reject it under §4.60 and Table 150. It MUST preserve server results for duplicate-value and other policies not statically determined by these rules.
- **KMIPKIT-0016-FR-004**: Adjust Attribute MUST represent assigned Adjustment Type values and extension values in the §11.1 Table 429 `8XXXXXXX` range, preserve unknown raw values, and reject reserved Adjustment Type values before transmission. It MUST transmit the caller's optional Adjustment Value without computing an adjusted value locally. When Adjustment Value is omitted, Increment/Decrement use Table 428 parameter defaults: 1 for numeric types, 1 second for Date Time, and 1 microsecond for Date Time Extended. If the target attribute has no existing value, the server assumes 0 for numeric types and intervals, false for Boolean, and raises an error for other types. Before transmission, the client MUST reject a target standard attribute unconditionally marked non-modifiable by the pinned source; conditional object-state restrictions remain server-authoritative when their status is not available to the client. The §4.60 Vendor Attribute restriction remains server-authoritative for Adjust because the request carries only an Attribute Reference and does not expose Vendor Identification.
- **KMIPKIT-0016-FR-005**: Delete Attribute MUST represent both optional Current Attribute and Attribute Reference fields independently. When Current Attribute is omitted and Attribute Reference is supplied, the client MUST preserve that omission so the server can delete all instances as specified. For a known standard attribute, the client MUST reject deletion when its pinned §4 rules prohibit client deletion or it is required to always have a value. When Current Attribute supplies a Vendor Attribute with Vendor Identification `y`, the client MUST reject deletion under §4.60 and Table 150; an Attribute Reference alone does not expose Vendor Identification and its result remains server-authoritative. If both optional selectors are omitted, the client MUST preserve both omissions and surface the server's result without inventing a selector; it MUST NOT perform a local deletion.
- **KMIPKIT-0016-FR-006**: Get Attributes MUST support zero or more Attribute References, MUST reject a repeated identical Attribute Reference, and MUST preserve the response rule that all instances of a selected name are returned while absent attributes are omitted. With no reference, it MUST represent the returned full attribute set.
- **KMIPKIT-0016-FR-007**: Get Attribute List MUST encode only its optional Unique Identifier request field from Table 226. Since no Attribute Reference selector is defined in the request table, the server returns all attribute names as stated in §6.1.21; a successful response MUST contain the required Attribute Reference value(s) from Table 227, and repeated returned names MUST be preserved in wire order.
- **KMIPKIT-0016-FR-008**: Modify Attribute MUST represent the optional Current Attribute and required New Attribute. It MUST transmit the caller's exact selector and replacement value; it MUST NOT locally select among multiple instances or create a missing attribute. Before transmission, it MUST reject standard attributes marked non-modifiable or operation-prohibited by the pinned source. If the New Attribute is `Usage Limits`, it MUST reject the request because §7.40 Table 392 requires Count and §4.59 prohibits setting or modifying Count through Modify Attribute. It MUST also reject a supplied Current Attribute or New Attribute Vendor Attribute value whose Vendor Identification is `y`, as prohibited by §4.60 and Table 150.
- **KMIPKIT-0016-FR-009**: Set Attribute MUST represent the required New Attribute and leave the add-or-modify decision to the server. It MUST NOT select among multiple existing instances or alter the caller's value. Before transmission, it MUST reject a target standard attribute marked non-modifiable by the pinned source and a Vendor Attribute with Vendor Identification `y` under §4.60 and Table 150.
- **KMIPKIT-0016-FR-010**: Every applicable operation-specific Result Reason in Tables 169, 172, 204, 225, 228, 267, and 324 MUST remain available to callers through the shared result model.
- **KMIPKIT-0016-FR-011**: Exact Attribute Name text and complete TTLV values using individually assigned tags or tags accepted by KMIPKIT-0004/ADR-0010's §11.56 allocation gate MUST remain representable and round-trip losslessly. Unknown Enumeration values MUST remain representable in generic TTLV, but a typed outbound Adjustment Type MUST be assigned or in the `0x80000000–0x8FFFFFFF` extension range; Reserved tags and enum values MUST NOT be emitted. The client MUST NOT synthesize tags or values for unknown attributes.
- **KMIPKIT-0016-FR-012**: Each explicit operation invocation MUST make at most one request exchange. KMIPKit MUST NOT automatically retry a request and MUST preserve the existing request delivery-state and pending-result contracts.
- **KMIPKIT-0016-FR-013**: Malformed or over-limit responses MUST be rejected according to the configured decoder limits before unbounded allocation. A response containing a Tag classified as Reserved MUST be rejected before constructing the public generic `Item` tree, as the accepted KMIPKit policy in ADR-0011 requires; this is a project policy consistent with the KMIP 2.1 Chapter 11 prohibition on using Reserved Tag Values and §11.56 classifications, and is not an assertion that OASIS independently mandates receiver rejection. Logs and errors MUST NOT include raw KMIP message bodies or secret-bearing attribute values.
- **KMIPKIT-0016-FR-014**: Normative requirements in this specification MUST be linked to exact OASIS source sections, catalog entries, implementation locations, and executable verification before implementation review.
- **KMIPKIT-0016-FR-015**: The client MUST apply an authoritative, source-backed mutation policy before transmission. It MUST reject an operation when the supplied request and exact source-backed policy prove the mutation is unconditionally prohibited, including standard Read-Only attributes and operation-specific prohibitions in §§6.1.2, 6.1.3, 6.1.13, and 6.1.51. Policy validation MUST inspect the New Attribute name for Add/Set, both supplied Current Attribute and New Attribute for Modify, the Attribute Reference name for Adjust, and every supplied Current Attribute and Attribute Reference name for Delete; when Delete has no selector, no target name is inferred. For Vendor Attribute values (the §4.60 structure in Table 150), it MUST inspect Vendor Identification in every supplied value-bearing field and reject client mutation of a server-created value identified by `y`. Adjust and reference-only Delete carry no Vendor Attribute value, so the client MUST NOT infer `y` from an Attribute Reference and MUST preserve the server result. For Add or Modify whose New Attribute name is `Usage Limits`, the client MUST reject the request: §7.40 Table 392 requires the Count member, and §4.59 prohibits clients from setting or modifying Count through these operations. A local rejection MUST report delivery state `NotSent` and MUST perform no transport exchange. The policy data MUST retain exact source references and verbatim attribute-rule source text for all 62 named attribute-rule tables and the separate §4.60 Vendor Attribute rule; qualified values MUST remain distinguishable from unconditional prohibitions. Restrictions whose result depends on remote object state are not guessed or prefetched; the client preserves the server result. Unknown/vendor names are not inferred to be writable or Read-Only beyond the explicit §4.60 `Vendor Identification=y` rule; KMIPKit preserves their exact names and values and returns server decisions without extra exchanges.

### Normative Traceability

| Requirement | Normative source | Verification required |
| --- | --- | --- |
| FR-001, FR-002 | OASIS KMIP Specification v2.1 §4, §§5.1, 5.5–5.7; KMIPKIT-REQ-SPEC-4-001-001; KMIPKIT-REQ-SPEC-4.60-001 (Table 150 Vendor Attribute encoding); operation tables listed above | Table-based request/response codec and typed execution tests for every operation, including distinct multi-instance values and the Table 150 Vendor Attribute encoding vector |
| FR-003 | §6.1.2, Tables 167–169; §§4, 4.28, 4.30, 4.59–4.60; §7.40 Table 392 | Add vectors, local `NotSent` rejection for source-prohibited attributes including Usage Limits and Vendor Attribute `y`, and server result preservation |
| FR-004 | §6.1.3, Tables 170–172; §4; §4.60; §11.1, Tables 428–429 | Assigned/extension enum validation, local standard-attribute rejection, parameter/absent-value defaults, and server-result preservation for rules not inspectable from Attribute Reference |
| FR-005 | §6.1.13, Tables 202–204; §§4, 4.60, 5.5–5.6 | Both selector omissions, all-instance deletion, local required/read-only rejection, inspectable Vendor Attribute `y` rejection, and server-result behavior when uninspectable/state-dependent |
| FR-006 | §6.1.20, Tables 223–225 | Tests for absent/duplicate references, all attributes, missing attributes, repeated instances, and errors |
| FR-007 | §6.1.21 prose and Tables 226–228 | Tests for UID-only request shape, full-name result, required response reference cardinality, repeated returned names, and errors |
| FR-008 | §6.1.34, Tables 265–267; §§4, 4.59–4.60; §7.40 Table 392; §§5.6–5.7 | Exact old/new values, local standard/Vendor Attribute/Usage Limits rejection, and remaining server results |
| FR-009 | §6.1.51, Tables 322–324; §§4, 4.60 | Absent/single/multiple instance behavior, local standard and Vendor Attribute `y` rejection, and operation errors |
| FR-010 | Operation-specific error tables cited above; §§9.1–9.2 | Tests preserve Result Status, Result Reason, and Result Message for each operation |
| FR-011 | §§4, 5.1, 5.5–5.7, 11.1 Table 429; KMIPKIT-0004/ADR-0010 allocation policy | Round-trip allocated/extension tags and unknown values; reject Reserved tags and Adjustment Type values outside assigned/extension ranges |
| FR-012 | §§8.1–8.6, 9.19–9.21; accepted KMIPKIT-0007 and KMIPKIT-0009 contracts | Fake-transport tests assert one exchange and delivery/pending outcomes |
| FR-013 | KMIPKit security constraints in AGENTS.md §8 and accepted TTLV decoder limits | Malformed and over-limit response tests; redaction assertions |
| FR-014 | All cited sections and catalog elements in Normative Scope | Traceability checker plus reviewed implementation/test matrix |
| FR-015 | §§4.1–4.63 attribute rules; §§4.28, 4.30, 4.59–4.60 (Table 150); §§6.1.2, 6.1.3, 6.1.13, 6.1.51; §7.40 Table 392 | Table-driven policy tests for standard attribute metadata, Usage Limits Count, and Vendor Attribute `y`; fake transport proves local failures are `NotSent` with zero exchanges; unknown names and state-dependent/uninspectable cases preserve server results |

## Success Criteria

### Measurable Outcomes

- **KMIPKIT-0016-SC-001**: All seven operations have typed request/response representations matching the required and optional fields and cardinalities in their cited OASIS tables.
- **KMIPKIT-0016-SC-002**: Tests exercise every operation's normal result, all applicable operation-specific error reasons representable by fixtures, and malformed payload cases without collapsing errors into a generic failure.
- **KMIPKIT-0016-SC-003**: Attribute update tests distinguish add, adjust, delete, modify, and set semantics and prove that the client never mutates server state locally or invents attribute values.
- **KMIPKIT-0016-SC-004**: Read tests preserve omitted Get Attributes values, repeated values, required Get Attribute List response references, and wire order exactly as specified.
- **KMIPKIT-0016-SC-005**: Unknown Attribute Names, allocated/accepted extension tags, and generic Enumeration values round-trip without loss; outbound typed Adjustment Type rejects Reserved values outside its assigned and extension ranges.
- **KMIPKIT-0016-SC-006**: Every applicable normative requirement is traced to its exact source, implementation, and executable verification with 100% coverage before the PR is ready for review.
- **KMIPKIT-0016-SC-007**: A deterministic fake transport confirms at most one exchange per invocation, preserves request delivery status, and demonstrates that no request is automatically retried.
- **KMIPKIT-0016-SC-008**: Every statically prohibited standard-attribute mutation and each inspectable Vendor Attribute mutation with Vendor Identification `y` fails with delivery state `NotSent` before transport; unknown names, Vendor Attribute values outside the `y` rule, uninspectable Vendor Attribute references, and remote-state-dependent cases do not trigger a preflight exchange and preserve server results.

## Key Entities

- **Attribute Reference**: A name selecting an attribute, distinct from an attribute value.
- **Current Attribute**: The existing name/value pair used to identify the instance being changed or deleted.
- **New Attribute**: The caller-supplied name/value pair to add, replace, or set.
- **Adjustment Type and Value**: The requested server-side adjustment operation and optional parameter; the client does not calculate the resulting value.
- **Attribute Result**: The returned Unique Identifier and, for reads, attribute values or names associated with the managed object.

## Assumptions

- KMIPKIT-0004 generic TTLV values and KMIPKIT-0005 encoding provide lossless structures for unknown values on allocated/accepted extension tags; Raw Tags failing allocation checks and reserved Adjustment Type values are not encodable.
- KMIPKIT-0006 message/batch, KMIPKIT-0007 client execution, and KMIPKIT-0009 pending-result contracts remain authoritative and are extended without changing their accepted semantics.
- The `AttributeEntry` representation specified by KMIPKIT-0014 is reused for complete name/value preservation; KMIPKIT-0016 does not change that shared model.
- The operation families and ordering follow the Phase D decomposition; 0016 is limited to the seven attribute operations, with cross-language parity assigned to Phase E.
- The server may reject an operation or attribute due to implementation, object state, or policy. Client operation support does not imply server support.

## Clarification Record

Protocol semantics for the seven explicitly listed operations are resolved by the cited normative sources. Before requesting approval of this specification PR, its author must close the Phase B inventory mapping for the seven operation elements, all applicable client requirements, and source-linked OASIS test cases, recording any reviewed exclusion in `specification/catalog/kmip-2.1.json` and regenerating `specification/catalog/coverage-report.md`. Phase B is the approved KMIPKIT-0002 inventory in `specs/002-normative-inventory/`; its catalog and generated report are the canonical assignment artifacts. This pre-approval work is included in this specification PR, so approval does not depend on a post-approval task. The project-wide family count remains provisional until Phase B assigns all applicable operations. API language parity, other operation families, and server-initiated operations remain explicitly excluded.
