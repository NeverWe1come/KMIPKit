# Feature Specification: KMIP 2.1 Key derivation and transfer

**Feature Branch**: feature/KMIPKIT-0037-key-material-transfer
**Created**: 2026-10-10
**Status**: Approved for implementation — 2026-10-11
**Approval evidence**: The maintainer approved specifications 022–028 for future implementation in a direct instruction on 2026-10-11. This approval does not start implementation.
**Input**: Specify the remaining client-initiated KMIP 2.1 operations in this family for the 1.0.0 release.

## Normative scope

OASIS Key Management Interoperability Protocol Specification Version 2.1 (OASIS Standard, 14 December 2020) is authoritative. Its pinned local copy is under specification/oasis/kmip-2.1/upstream. The Test Cases and Profiles documents provide evidence where applicable; the Usage Guide is informative.

| Operation | Catalog ID | Section | Request table | Response table | Error table |
| --- | --- | --- | ---: | ---: | ---: |
| Derive Key | KMIPKIT-ELEM-OP-C2S-DERIVE-KEY | §6.1.14 | 205 | 206 | 207 |
| Export | KMIPKIT-ELEM-OP-C2S-EXPORT | §6.1.18 | 217 | 218 | 219 |
| Import | KMIPKIT-ELEM-OP-C2S-IMPORT | §6.1.25 | 238 | 239 | 240 |
| Join Split Key | KMIPKIT-ELEM-OP-C2S-JOIN-SPLIT-KEY | §6.1.27 | 244 | 245 | 246 |

## Operation errors

Each operation error table is cited below with its Table number. Result Reason alternatives are preserved as listed in the pinned source; duplicate names reflect duplicate entries in the source table. The common result and transport-delivery contract remains applicable.

| Operation | OASIS error table | Result Status | Result Reason values |
| --- | ---: | --- | --- |
| Derive Key | 207 | Operation Failed | Bad Cryptographic parameters, Cryptographic Failure, Incompatible Cryptographic Usage Mask, Invalid Attribute, Invalid Field, Invalid Message, Invalid Object Type, Key Value Not Present, Non Unique Name Attribute, Object Not Found, Unsupported Cryptographic Parameters, Wrong Key Lifecycle State, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |
| Export | 219 | Operation Failed | Bad Cryptographic Parameters, Encoding Option Error, Encoding Option Error, Incompatible Cryptographic Usage Mask, Invalid Object Type, Key Compression Type Not Supported, Key Format Type Not Supported, Key Value Not Present, Key Wrap Type Not Supported, Object Not Found, Wrapping Object Archived, Wrapping Object Destroyed, Wrapping Object Not Found, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |
| Import | 240 | Operation Failed | Attribute Read Only, Attribute Single Valued, Encoding Option Error, Invalid Attribute, Invalid Attribute Value, Invalid Field, Non Unique Name Attribute, Object Already Exists, Server Limit Exceeded, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Protection Storage Unavailable, Response Too Large |
| Join Split Key | 246 | Operation Failed | Attribute Read Only, Attribute Single Valued, Bad Cryptographic Parameters, Cryptographic Failure, Cryptographic Failure, Invalid Attribute, Invalid Attribute Value, Invalid Object Type, Non Unique Name Attribute, Object Not Found, Server Limit Exceeded, Unsupported Cryptographic Parameters, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Protection Storage Unavailable, Response Too Large |

## Product boundaries and dependencies

The feature uses the approved TTLV codec, typed messages/batches, client execution, TLS/HTTPS transports, object model, credentials, and public-language bindings. It adds no local cryptographic algorithm, server-initiated operation, encoding, transport, or implicit server-policy choice. A caller selects each operation explicitly. One call sends at most one request. The existing typed-batch contract preserves protocol Pending and correlation bytes when the caller enables asynchronous results. Unknown and vendor values remain lossless. Raw KMIP bodies and secret values never appear in logs or errors. Rust, C, Java, and Python must offer equivalent 1.0.0 capability.

## User Scenarios & Testing

### User Story 1 — Derive Key (Priority: P1)

As a KMIPKit caller, I can send the §6.1.14 Derive Key request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 205 request and returns a Table 206 success and Table 207 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Derive Key request, when encoded and sent, then exactly one operation with the Table 205 fields reaches the transport.
2. Given a valid Table 206 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 207 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.
5. Given both explicit Derivation Data and a Secret Data object Unique Identifier, when the request is sent, then KMIPKit does not pre-reject it and exposes the server error result required by §7.12.

**Specific rules**: Never substitute the ID Placeholder for source identifiers; preserve repeated identifiers and Derivation Data, explicit method and parameters, length, algorithm, and Derive Key usage permission. Preserve caller-selected Hashing Algorithm values; do not choose a non-default hash implicitly.

**Payload schema** (OASIS Specification v2.1 §6.1.14, Tables 205–206):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Object Type | Yes | Determines the type of object to be created. |
| Request | Unique Identifier | Yes. MAY be repeated | Determines the object or objects to be used to derive a new key. Note that the current value of the ID Placeholder SHALL NOT be used in place of a Unique Identifier in this operation. |
| Request | Derivation Method | Yes | An Enumeration object specifying the method to be used to derive the new key. |
| Request | Derivation Parameters | Yes | A Structure object containing the parameters needed by the specified derivation method; see the §7.12/Table 364 detail below. |
| Request | Attributes | Yes | Specifies desired attributes to be associated with the new object; the length and algorithm SHALL always be specified for the creation of a symmetric key. |
| Response | Unique Identifier | Yes | The Unique Identifier of the newly derived key or Secret Data object. |

**Derivation Parameters structure** (OASIS Specification v2.1 §7.12, Table 364):

| Field | REQUIRED | Description or condition | Catalog trace |
| --- | --- | --- | --- |
| Derivation Parameters | Yes | Structure containing parameters for the selected derivation method. | KMIPKIT-ELEM-OPERATION-STRUCTURE-7-12-DERIVATION-PARAMETERS |
| Cryptographic Parameters | No, depends on the PRF | Identifies the PRF or its mode. For HASH, clients are REQUIRED to indicate the hash algorithm; for AES in CBC mode, clients are REQUIRED to indicate Block Cipher Mode. For HMAC, the server ignores Cryptographic Parameters because the derivation key attributes identify the PRF. | KMIPKIT-ELEM-STRUCTURE-MEMBER-7-12-CRYPTOGRAPHIC-PARAMETERS; KMIPKIT-REQ-SPEC-7.12-003 |
| Initialization Vector | No, depends on PRF and mode | Include when required by the PRF/mode; the source assumes an empty IV when none is provided. | KMIPKIT-ELEM-STRUCTURE-MEMBER-7-12-INITIALIZATION-VECTOR |
| Derivation Data | Yes unless a Secret Data object Unique Identifier supplies it; the source says “May be repeated” | Carries data to encrypt, hash, or HMAC. The model and generic TTLV path preserve repeated occurrences. | KMIPKIT-ELEM-STRUCTURE-MEMBER-7-12-DERIVATION-DATA; KMIPKIT-REQ-SPEC-7.12-002 |
| Salt | Yes for PBKDF2 | PBKDF2 salt. | KMIPKIT-ELEM-STRUCTURE-MEMBER-7-12-SALT |
| Iteration Count | Yes for PBKDF2 | PBKDF2 iteration count. | KMIPKIT-ELEM-STRUCTURE-MEMBER-7-12-ITERATION-COUNT |

For derivation input, the client MAY supply Derivation Data or identify a Secret Data object by Unique Identifier. If it supplies both, the server SHALL return an error (OASIS §7.12; KMIPKIT-REQ-SPEC-7.12-005-001 and KMIPKIT-REQ-SPEC-7.12-005-002). This is a server result obligation: KMIPKit sends the caller’s fields and surfaces the response instead of imposing a local rejection.

For HKDF, the specified Managed Object supplies Input Key Material, Salt carries the salt, Derivation Data carries optional info, and the Cryptographic Length attribute sets output length. SHA-256 is the default. The lowercase “optional” and “may” in this description are not RFC 2119 keywords under §1.2, so they create no separate normative client obligation. KMIPKit still represents Hashing Algorithm within Cryptographic Parameters as the described override option in the Table 364 structure (KMIPKIT-DEC-011); a non-default value is never selected implicitly.

---

### User Story 2 — Export (Priority: P1)

As a KMIPKit caller, I can send the §6.1.18 Export request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 217 request and returns a Table 218 success and Table 219 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Export request, when encoded and sent, then exactly one operation with the Table 217 fields reaches the transport.
2. Given a valid Table 218 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 219 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Explicitly select optional key format, wrapping, and compression; preserve the returned object and attributes as sensitive data.

**Payload schema** (OASIS Specification v2.1 §6.1.18, Tables 217–218):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Unique Identifier | No | Determines the object being requested. If omitted, then the IDPlaceholder value is used by the server as the Unique Identifier. |
| Request | Key Format Type | No | Determines the key format type to be returned. |
| Request | Key Wrap Type | No | Determines the Key Wrap Type of the returned key value. |
| Request | Key Compression Type | No | Determines the compression method for elliptic curve public keys. |
| Request | Key Wrapping Specification | No | Specifies keys and other information for wrapping the returned object. |
| Response | Object Type | Yes | Type of object |
| Response | Unique Identifier | Yes | The Unique Identifier of the object. |
| Response | Attributes | Yes | All of the object’s Attributes. |
| Response | Any Object (Section 2) | Yes | The object value being returned, in the same manner as the Get operation. |

---

### User Story 3 — Import (Priority: P1)

As a KMIPKit caller, I can send the §6.1.25 Import request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 238 request and returns a Table 239 success and Table 240 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Import request, when encoded and sent, then exactly one operation with the Table 238 fields reaches the transport.
2. Given a valid Table 239 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 240 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Replacement is explicit; preserve conditional Key Wrap Type, object, attributes, and wrapped representation.

**Payload schema** (OASIS Specification v2.1 §6.1.25, Tables 238–239):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Unique Identifier | Yes | The Unique Identifier of the object to be imported |
| Request | Object Type | Yes | Determines the type of object being imported. |
| Request | Replace Existing | No | A Boolean. If specified and true then any existing object with the same Unique Identifier SHALL be replaced by this operation. If absent or false and an object exists with the same Unique Identifier then an error SHALL be returned. |
| Request | Key Wrap Type | If and only if the key object is wrapped. | If Not Wrapped then the server SHALL unwrap the object before storing it, and return an error if the wrapping key is not available. Otherwise the server SHALL store the object as provided. |
| Request | Attributes | Yes | Specifies object attributes to be associated with the new object. |
| Request | Any Object (Section 2) | Yes | The object being imported. The object and attributes MAY be wrapped. |
| Response | Unique Identifier | Yes | The Unique Identifier of the newly imported object. |

---

### User Story 4 — Join Split Key (Priority: P1)

As a KMIPKit caller, I can send the §6.1.27 Join Split Key request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 244 request and returns a Table 245 success and Table 246 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Join Split Key request, when encoded and sent, then exactly one operation with the Table 244 fields reaches the transport.
2. Given a valid Table 245 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 246 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Preserve repeated Split Key identifiers and enforce the known Split Key Threshold minimum.

**Payload schema** (OASIS Specification v2.1 §6.1.27, Tables 244–245):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Object Type | Yes | Determines the type of object to be created. |
| Request | Unique Identifier | Yes, MAY be repeated | Determines the Split Keys to be combined to form the object returned by the server. The minimum number of identifiers is specified by the Split Key Threshold field in each of the Split Keys. |
| Request | Secret Data Type | No | Determines which Secret Data type the Split Keys form. |
| Request | Attributes | No | Specifies desired object attributes. |
| Request | Protection Storage Masks | No | Specifies all permissible Protection Storage Mask selections for the new object |
| Response | Unique Identifier | Yes | The Unique Identifier of the object obtained by combining the Split Keys. |

---

### Edge Cases

- Missing required or incompatible conditional request fields fail before transmission with NotSent delivery evidence.
- Optional absence, repeated field order, unknown values, and opaque structures remain distinct; server results are not fabricated.
- KMIP failure results and malformed network input are handled through the shared result/error and TTLV resource-limit contracts.
- Tickets, credentials, keys, seeds, and raw KMIP bodies never appear in diagnostics.

## Requirements

### Functional Requirements

- **FR-001**: Expose typed request/response models and explicit client calls for Derive Key, Export, Import, Join Split Key using the common batch and delivery-state contract.
- **FR-002**: Validate payload structure and applicable client-side MUST/SHALL/SHALL NOT requirements that can be checked from local request data. Do not turn server-side requirements into client preconditions or infer remote object state or policy. Preserve unknown tags, enum values, bitmask bits, and extensions instead of rejecting them solely because KMIPKit does not recognize them.
- **FR-003**: Preserve Success, Failure, Pending, Result Reason, permitted Result Message, correlation, unknown values, and extension data losslessly. Never retry or poll automatically.
- **FR-004**: Expose equivalent Rust, C, Java, and Python capability by 1.0.0; retain generic structurally valid TTLV access alongside typed forms.
- **FR-005**: Redact secrets and raw bodies; zeroize KMIPKit-owned secret memory and document Java/Python runtime-copy limits.
- **FR-006**: Implement every applicable client requirement below, negatively verify MUST NOT/SHALL NOT, and record any SHOULD deviation for review.
- **FR-007**: Do not claim profile support, official test-vector passes, interoperability, or certification without corresponding pinned evidence.
- **FR-008**: Map every mandatory or conditional payload-table row to a stable catalog element or requirement ID and to verification; assign a stable spec-scoped trace ID when no suitable catalog ID exists.

### Client normative requirement ledger

Stable IDs and summaries below come from the checked-in normative catalog. The implementation PR must link each ID to code and verification and regenerate traceability with the pinned repository tool. Source sections are exact; payload tables above remain binding for operations with no separate client-clause records.

| Requirement ID | OASIS Specification section | Keyword | Client obligation or capability |
| --- | --- | --- | --- |
| KMIPKIT-REQ-SPEC-6.1.14-001-001 | §6.1.14 | SHALL | Use only a source object whose Cryptographic Usage Mask includes the Derive Key bit. |
| KMIPKIT-REQ-SPEC-6.1.14-001-002 | §6.1.14 | SHALL | Specify Cryptographic Length for each derived key or Secret Data object. |
| KMIPKIT-REQ-SPEC-6.1.14-001-003 | §6.1.14 | SHALL | When deriving a key, specify both Cryptographic Length and Cryptographic Algorithm. |
| KMIPKIT-REQ-SPEC-6.1.14-001-004 | §6.1.14 | MAY | A client may derive multiple keys or IVs in a Secret Data result by specifying the total length. |
| KMIPKIT-REQ-SPEC-6.1.14-002 | §6.1.14 | MAY | A client may identify derivation inputs, select a derivation method, and provide its parameters. |
| KMIPKIT-REQ-SPEC-6.1.14-006-001 | §6.1.14 | MAY | A client may provide repeated Unique Identifiers for derivation inputs. |
| KMIPKIT-REQ-SPEC-6.1.14-006-002 | §6.1.14 | SHALL NOT | Do not substitute the ID Placeholder for a Unique Identifier in Derive Key. |
| KMIPKIT-REQ-SPEC-6.1.14-007-001 | §6.1.14 | SHALL | Specify the desired attributes for the new object. |
| KMIPKIT-REQ-SPEC-6.1.14-007-002 | §6.1.14 | SHALL | For a symmetric key, specify both length and algorithm. |
| KMIPKIT-REQ-SPEC-7.12-003 | §7.12 | REQUIRED | For derivation methods using a PRF, provide its PRF or mode in Cryptographic Parameters; HASH requires the hash algorithm and AES-CBC requires Block Cipher Mode. |
| KMIPKIT-REQ-SPEC-7.12-005-001 | §7.12 | MAY | Allow Derivation Data to be supplied explicitly or by identifying a Secret Data object. |
| KMIPKIT-REQ-SPEC-6.1.25-001 | §6.1.25 | MUST | Supply the Import attribute values rather than rely on server-generated values. |
| KMIPKIT-REQ-SPEC-6.1.25-004 | §6.1.25 | SHALL | Set Replace Existing to true when the request intends to replace an object with the same Unique Identifier. |
| KMIPKIT-REQ-SPEC-6.1.25-006 | §6.1.25 | MAY | A client may send the imported object and its attributes wrapped. |
| KMIPKIT-REQ-SPEC-6.1.27-001 | §6.1.27 | SHALL | Include at least the Split Key Threshold number of Unique Identifiers in Combine. |
| KMIPKIT-REQ-SPEC-6.1.27-002-001 | §6.1.27 | MAY | A client may specify the resulting Object Type. |
| KMIPKIT-REQ-SPEC-6.1.27-002-002 | §6.1.27 | MAY | For a Secret Data result, a client may specify Secret Data Type. |
| KMIPKIT-REQ-SPEC-6.1.27-005 | §6.1.27 | MAY | A client may provide repeated Unique Identifiers for Split Keys to combine. |

### Key Entities

- Operation request: typed fields and preserved structurally valid extensions.
- Operation response: typed fields and preserved unknown data.
- KMIP result: status, reason, permitted message, Pending correlation, and delivery evidence.
- Secret value: ticket, credential, key, seed, or other sensitive bytes subject to redaction and owned-memory zeroization.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All 4 operations have typed request/response, an explicit client call, generic TTLV access, and Rust/C/Java/Python parity before 1.0.0.
- **SC-002**: All Table-defined fields and error forms and all 18 catalog client requirement IDs have positive/relevant negative verification and exact source-to-test links.
- **SC-003**: Fake-transport cases confirm one exchange and preserved Success, Failure, Pending, unknown values, malformed response behavior, and delivery state.
- **SC-004**: Required repository coverage, formatting, lint, security, generated-artifact, and two-independent-server integration gates pass before release.

## Assumptions and source decisions

- The accepted common message, batch, TTLV, TLS, security, and binding contracts remain authoritative. This specification adds only the named operations.
- Server-side obligations are not silently reclassified as client requirements; server policy may reject valid requests.
- Pinned official XML fixtures are incomplete. An unavailable fixture is documented rather than reported as a passing conformance vector.
- Other unresolved source discrepancies must receive an explicit disposition before implementation of affected behavior.

### Maintainer-approved interpretations (2026-10-10)

These project interpretations do not alter the pinned OASIS source or assert official errata. The maintainer approved this project interpretation in the current task. The normative catalog decision record KMIPKIT-DEC-011 records the §7.12 interpretations; pinned upstream copies remain unchanged.

| Topic | Discrepancy | Applied interpretation |
| --- | --- | --- |
| Unique Identifier | KMIPKIT-DISC-025 | The lowercase “may” in §11.58 is descriptive text, not a new normative client keyword. Explicit operation-specific identifier and batch rules remain in force. |
| Derive Key / HKDF | KMIPKIT-DISC-021, KMIPKIT-DISC-049 | Follow §1.2: lowercase HKDF “optional” and “may” are descriptive. Preserve the Hashing Algorithm override option described in §7.12/Table 364 without claiming an independent normative obligation. Table 364’s sentence-case “May be repeated” is also not an uppercase MAY; KMIPKit preserves the described repetition without counting it as a client requirement. The uppercase §7.12 MAY/SHALL for derivation data remain normative, with the error returned by the server. |
### Test-case inventory

| Operation | Catalog case IDs |
| --- | --- |
| Derive Key | KMIPKIT-TEST-CN01-2-14, KMIPKIT-TEST-CN01-2-15, KMIPKIT-TEST-CN01-2-16, KMIPKIT-TEST-CN01-2-17, KMIPKIT-TEST-CN01-2-18, KMIPKIT-TEST-CN01-2-19 |
| Export | KMIPKIT-TEST-CN01-2-43, KMIPKIT-TEST-CN01-2-44, KMIPKIT-TEST-CN01-2-45, KMIPKIT-TEST-CN01-2-46, KMIPKIT-TEST-CN01-2-47 |
| Import | KMIPKIT-TEST-CN01-2-43, KMIPKIT-TEST-CN01-2-44, KMIPKIT-TEST-CN01-2-45, KMIPKIT-TEST-CN01-2-46, KMIPKIT-TEST-CN01-2-47 |
| Join Split Key | No direct catalog case ID |
