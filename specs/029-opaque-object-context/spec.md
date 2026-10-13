# Feature Specification: Explicit Opaque Object cryptographic context for Register

**Feature Branch**: `feature/KMIPKIT-0044-opaque-object-context`

**Created**: 2026-10-13

**Status**: Approved for implementation under the maintainer's autonomous roadmap instruction dated 2026-10-13

**Input**: Implement the KMIP 2.1 client-context choice that determines whether an Opaque Object is handled as a Managed Cryptographic Object for Register attributes.

## Normative scope

The authoritative source is the pinned OASIS Key Management Interoperability Protocol Specification Version 2.1 (OASIS Standard, 14 December 2020), which is immutable under `specification/oasis/kmip-2.1/upstream/`.

| Source | Normative effect in this feature |
| --- | --- |
| §2.3, Opaque Object | An Opaque Object MAY be a Managed Cryptographic Object depending on the client context of usage; when so treated, it is handled in the same manner as a Managed Cryptographic Object for attributes. |
| §2.3, Table 4 | Defines the Opaque Object as a Structure. Its child fields Opaque Data Type (Enumeration) and Opaque Data Value (Byte String) are each marked REQUIRED=Yes. Register §6.1.43/Table 291 requires the Any Object item. |
| §11.34, Table 468 | Defines Opaque Object as Object Type Enumeration value 8. |
| §11.35, Table 469 | Defines the Opaque Data Type extension Enumeration range (`8XXXXXXX`); the guide example uses a value from this range and leaves extension semantics to its owner. |
| §6.1.43, Register, Table 291 | Defines the Register request payload and its Attributes and object fields. |
| §6.1.43, Table 293 | Defines attributes required when registering a Managed Cryptographic Object, including Cryptographic Usage Mask and the paired Cryptographic Algorithm and Cryptographic Length rules. |
| §4.17 | Defines Cryptographic Usage Mask. Relevant only to preserve and validate the supplied attribute representation; this feature does not enforce key-use authorization during later operations. |

This feature owns **KMIPKIT-REQ-SPEC-2.3-002** and **KMIPKIT-REQ-SPEC-2.3-003**. The §2.3 MAY remains optional: callers may explicitly select Managed Cryptographic Object context, while the default remains generic Opaque Object handling. When registering any Opaque Object, the client also supplies the required Table 4 Opaque Data Type and Opaque Data Value children; this requirement is traced to §2.3/Table 4 and Register §6.1.43/Table 291. This feature does not claim an OASIS certification or an official test-case pass; the pinned source does not link a requirement-specific official Test Cases ID.

## User Scenarios & Testing

### User Story 1 — Register a generic Opaque Object (Priority: P1)

As a KMIPKit caller registering an Opaque Object that is not used as cryptographic material, I can keep it in generic opaque context without supplying cryptographic attributes.

**Why this priority**: Existing Opaque Object callers must continue to register non-cryptographic opaque data without being forced into an unrelated crypto model.

**Independent Test**: Build a Register request with Object Type Opaque Object and a structurally valid Opaque Object, omit cryptographic attributes, and verify the normal request is encoded and sent exactly once.

**Acceptance Scenarios**:

1. Given an Opaque Object with no explicit cryptographic context, when the caller builds a Register request without cryptographic attributes, then local validation accepts it and emits the caller's attributes unchanged.
2. Given an Opaque Object with caller-supplied attributes in generic context, when the Register request is encoded, then tags, values, and order are preserved.
3. Given Object Type Opaque Object with a missing or mistagged Opaque Object item, missing Opaque Data Type or Opaque Data Value, or a child encoded using the wrong TTLV type, when the request is validated, then it fails locally before transport.
4. Given any non-Opaque Object Type, when the caller selects Opaque Object context, then request construction fails locally before transport and reports delivery as NotSent.

### User Story 2 — Register an Opaque Object as managed cryptographic material (Priority: P1)

As a KMIPKit caller storing cryptographic material in an Opaque Object representation, I can explicitly select cryptographic context so KMIPKit applies the Register requirements for Managed Cryptographic Objects.

**Why this priority**: The Opaque Object representation intentionally leaves its payload opaque to the server and client. The caller's usage context is the authoritative signal for applying crypto attribute rules without parsing or guessing from payload bytes.

**Independent Test**: Select Managed Cryptographic Object context for an Opaque Object, supply the required Table 293 attributes, and verify the exact attributes are encoded; then omit each required attribute in turn and verify local rejection with zero transport writes and NotSent delivery evidence.

**Acceptance Scenarios**:

1. Given Object Type Opaque Object with explicit Managed Cryptographic Object context, when a Register request is validated, then Cryptographic Usage Mask, Cryptographic Algorithm, and Cryptographic Length are required as direct attributes because Table 4's Opaque Object has no Key Block in which Algorithm or Length could be encapsulated.
2. Given only one of Cryptographic Algorithm and Cryptographic Length, when request validation runs, then it rejects the request locally.
3. Given both required crypto attributes and a usage mask, when the request is encoded, then the caller's attribute values and order are unchanged.
4. Given an invalid request, when the high-level Register call is invoked, then the transport receives zero writes and the returned delivery state is NotSent.
5. Given a valid request, when the server handles it, then KMIPKit sends the normal Table 291 Register payload; no private context marker is added to the wire and the server remains authoritative for acceptance.
6. Given a valid Opaque Object, when its child fields are encoded, then Opaque Data Type uses Enumeration and Opaque Data Value uses Byte String, with unknown data-type values and unrecognized child items retained in caller order.

### Edge Cases

- Unknown Opaque Data Type values, unrecognized Opaque Object child items, and unknown attribute values remain preserved under the existing lossless model.
- The feature does not inspect, interpret, or infer cryptographic properties from Opaque Data Value bytes.
- Certificate-only Table 293 fields, including Certificate Length and Digital Signature Algorithm, are not made required for an Opaque Object.
- Selecting Managed Cryptographic Object context does not authorize later Encrypt, Decrypt, Sign, or other cryptographic operations; per-key use authorization is outside this feature.
- Caller-provided attributes are not synthesized, normalized, reordered, or removed.


## Product boundaries and dependencies

This feature changes only caller-selected, local Register attribute handling for Opaque Objects. It adds no wire field, operation, server behavior, cryptographic algorithm, payload parser, automatic attribute lookup, retry, or inferred key-use policy. Existing behavior for every non-Opaque Object Type remains unchanged.

Every supported 1.0 language binding must expose equivalent capability and behavior; the public API manifest remains authoritative for generated binding surfaces.

## Requirements

### Functional Requirements

- **FR-001**: The Register API MUST let the caller explicitly select whether Object Type Opaque Object is handled in generic opaque context or as a Managed Cryptographic Object for attributes.
- **FR-002**: If no context is selected, an Opaque Object MUST retain generic opaque handling and MUST NOT acquire additional cryptographic attribute requirements.
- **FR-003**: If Managed Cryptographic Object context is selected for an Opaque Object, Register preflight MUST enforce the applicable §6.1.43 Table 293 attribute requirements: Cryptographic Usage Mask is present, and Cryptographic Algorithm and Cryptographic Length are both present because the §2.3/Table 4 Opaque Object structure contains no Key Block.
- **FR-004**: Cryptographic Algorithm and Cryptographic Length MUST be present together. Caller values MUST be retained exactly; KMIPKit MUST NOT infer or synthesize missing values from opaque payload bytes.
- **FR-005**: Selecting an Opaque Object context for a non-Opaque Object Type MUST fail before transport; the resulting delivery evidence MUST be NotSent and the transport MUST receive zero writes.
- **FR-006**: Valid Register requests MUST preserve caller-supplied attribute tags, values, duplicates where structurally allowed, and order, and MUST use the ordinary §6.1.43 Table 291 wire shape without a private context marker.
- **FR-007**: The caller's context choice MUST be available with equivalent behavior in every supported language binding before 1.0.0.
- **FR-008**: Tests and traceability MUST map KMIPKIT-REQ-SPEC-2.3-002, KMIPKIT-REQ-SPEC-2.3-003, KMIPKIT-REQ-SPEC-6.1.43-001, KMIPKIT-REQ-SPEC-6.1.43-007, KMIPKIT-REQ-SPEC-6.1.43-009-001, KMIPKIT-REQ-SPEC-6.1.43-009-002, KMIPKIT-REQ-SPEC-6.1.43-010-001, and KMIPKIT-REQ-SPEC-6.1.43-010-002 to exact source sections, implementation, and positive and negative verification.
- **FR-009**: English and Spanish documentation MUST explain that caller context changes client-side attribute handling only; server policy remains authoritative and later cryptographic use is outside this feature.
- **FR-010**: For Object Type Opaque Object, the required Any Object item MUST use the Opaque Object tag and a Structure value per §6.1.43/Table 291, §2.3/Table 4, and §11.34/Table 468. It MUST contain Opaque Data Type as an Enumeration and Opaque Data Value as a Byte String; unknown Enumeration values and unrecognized child items MUST be preserved in caller order. Example Enumeration values MUST be selected from a defined KMIP value or the §11.35/Table 469 extension range.

### Normative requirement ledger

| Requirement ID | Source | Keyword | Feature treatment |
| --- | --- | --- | --- |
| KMIPKIT-REQ-SPEC-2.3-002 | OASIS KMIP v2.1 §2.3 | MAY | Expose the caller's choice to treat an Opaque Object as a Managed Cryptographic Object for attribute handling. |
| KMIPKIT-REQ-SPEC-2.3-003 | OASIS KMIP v2.1 §2.3, Table 4; §6.1.43, Table 291 | REQUIRED | Require the Opaque Data Type and Opaque Data Value children with their defined TTLV types inside the required Register Any Object item. |
| KMIPKIT-REQ-SPEC-6.1.43-001 | OASIS KMIP v2.1 §6.1.43, Table 291 | SHALL | A Register request must specify Object Type and Attributes in an Attributes object. |
| KMIPKIT-REQ-SPEC-6.1.43-007 | OASIS KMIP v2.1 §6.1.43, Table 293 | SHALL | When context treats the Opaque Object as a Managed Cryptographic Object, include required applicable attributes, including Cryptographic Usage Mask. |
| KMIPKIT-REQ-SPEC-6.1.43-009-001 | OASIS KMIP v2.1 §6.1.43, Table 293 | MAY | Cryptographic Algorithm may be omitted only when encoded in Key Block; Opaque Object has no Key Block, so this omission does not apply. |
| KMIPKIT-REQ-SPEC-6.1.43-009-002 | OASIS KMIP v2.1 §6.1.43, Table 293 | SHALL | If Cryptographic Algorithm is present, Cryptographic Length is also present. |
| KMIPKIT-REQ-SPEC-6.1.43-010-001 | OASIS KMIP v2.1 §6.1.43, Table 293 | MAY | Cryptographic Length may be omitted only when encoded in Key Block; Opaque Object has no Key Block, so this omission does not apply. |
| KMIPKIT-REQ-SPEC-6.1.43-010-002 | OASIS KMIP v2.1 §6.1.43, Table 293 | SHALL | If Cryptographic Length is present, Cryptographic Algorithm is also present. |

### Key Entities

- **Opaque Object context**: Caller-selected interpretation controlling local attribute handling for an Opaque Object during Register. The default is generic opaque context.
- **Register attributes**: Ordered caller-supplied attributes serialized in the normal KMIP Attributes structure.
- **Opaque Object value**: Opaque Data Type and Opaque Data Value whose bytes are not interpreted by this feature.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Generic Opaque Object Register requests without cryptographic attributes remain valid when the caller selects no crypto context.
- **SC-002**: Every explicit crypto-context Register request missing Cryptographic Usage Mask, Cryptographic Algorithm, or Cryptographic Length fails locally with NotSent and zero transport writes.
- **SC-003**: Valid requests preserve caller attributes exactly and contain no context-specific wire field.
- **SC-004**: Every supported language binding exposes equivalent caller choice and passes the same positive and negative scenarios before 1.0.0.
- **SC-005**: KMIPKIT-REQ-SPEC-2.3-002, KMIPKIT-REQ-SPEC-2.3-003, KMIPKIT-REQ-SPEC-6.1.43-001, KMIPKIT-REQ-SPEC-6.1.43-007, KMIPKIT-REQ-SPEC-6.1.43-009-001, KMIPKIT-REQ-SPEC-6.1.43-009-002, KMIPKIT-REQ-SPEC-6.1.43-010-001, and KMIPKIT-REQ-SPEC-6.1.43-010-002 have 100% source-to-spec-to-code-to-test traceability.
- **SC-006**: A wrong Opaque Object tag or missing/mistyped Table 4 child is rejected before transport; valid fields retain their exact values.

## Assumptions and decisions

- The source's §2.3 client-context MAY is implemented as an explicit, caller-controlled choice; the client does not infer context from Opaque Data Type, Opaque Data Value, other attributes, or server responses.
- §2.3's same-manner attribute handling applies the Register Table 293 requirements when the client selects Managed Cryptographic Object context, despite generic Opaque Object exemptions in §§4.13, 4.15, and 4.17. Those general attribute tables do not apply as an override when the caller selects the explicit §2.3 Managed Cryptographic Object context for Register. This does not implement later cryptographic-use authorization.
- Table 293's Key Block omission allowance cannot be used for the Table 4 Opaque Object structure because it has no Key Block field; Opaque Data Value is an uninterpreted Byte String and is not inferred to contain a Key Block. The paired Algorithm/Length constraint still applies. Certificate-only requirements remain inapplicable.
- Table 4 requires both Opaque Data Type and Opaque Data Value child rows; the enclosing Opaque Object row has a blank requiredness cell. Register's Table 291 requires the Any Object payload item.
- The existing request delivery contract classifies local validation failures as NotSent.
- The public API manifest is the source of truth for generated binding surfaces; implementation sequencing must follow its accepted contract.
- The official OASIS Test Cases document does not list a direct test case for KMIPKIT-REQ-SPEC-2.3-002; feature tests are derived from the cited normative specification tables and are not represented as official-case passes.

### Exclusions

- KMIP operations other than client-initiated Register.
- Client-side authorization of subsequent cryptographic operations against Cryptographic Usage Mask.
- Parsing, validating, or converting opaque data values.
- Inventing cryptographic attributes or querying the server implicitly.
- Adding a context marker or any other field to the KMIP wire request.
- Changing Opaque Object Custom Attribute storage/retrieval behavior assigned to KMIPKIT-0010.
- Server-initiated operations, JSON/XML, transports, or language adapters beyond Rust, C, Java, and Python.

