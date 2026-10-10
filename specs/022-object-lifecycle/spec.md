# Feature Specification: KMIP 2.1 Object lifecycle and certificates

**Feature Branch**: feature/KMIPKIT-0036-object-lifecycle
**Created**: 2026-10-10
**Status**: Approved for implementation — 2026-10-11
**Approval evidence**: The maintainer approved specifications 022–028 for future implementation in a direct instruction on 2026-10-11. This approval does not start implementation.
**Input**: Specify the remaining client-initiated KMIP 2.1 operations in this family for the 1.0.0 release.

## Normative scope

OASIS Key Management Interoperability Protocol Specification Version 2.1 (OASIS Standard, 14 December 2020) is authoritative. Its pinned local copy is under specification/oasis/kmip-2.1/upstream. The Test Cases and Profiles documents provide evidence where applicable; the Usage Guide is informative.

| Operation | Catalog ID | Section | Request table | Response table | Error table |
| --- | --- | --- | ---: | ---: | ---: |
| Certify | KMIPKIT-ELEM-OP-C2S-CERTIFY | §6.1.6 | 179 | 180 | 181 |
| Check | KMIPKIT-ELEM-OP-C2S-CHECK | §6.1.7 | 183 | 184 | 185 |
| Obtain Lease | KMIPKIT-ELEM-OP-C2S-OBTAIN-LEASE | §6.1.35 | 268 | 269 | 270 |
| Re-certify | KMIPKIT-ELEM-OP-C2S-RE-CERTIFY | §6.1.45 | 300 | 301 | 302 |
| Re-key | KMIPKIT-ELEM-OP-C2S-RE-KEY | §6.1.46 | 305 | 306 | 307 |
| Re-key Key Pair | KMIPKIT-ELEM-OP-C2S-RE-KEY-KEY-PAIR | §6.1.47 | 310 | 311 | 312 |
| Re-Provision | KMIPKIT-ELEM-OP-C2S-RE-PROVISION | §6.1.48 | 313 | 314 | 315 |
| Register | KMIPKIT-ELEM-OP-C2S-REGISTER | §6.1.43 | 291 | 292 | 294 |
| Revoke | KMIPKIT-ELEM-OP-C2S-REVOKE | §6.1.44 | 295 | 296 | 297 |

## Operation errors

Each operation error table is cited below with its Table number. Result Reason alternatives are preserved as listed in the pinned source; duplicate names reflect duplicate entries in the source table. The common result and transport-delivery contract remains applicable.

| Operation | OASIS error table | Result Status | Result Reason values |
| --- | ---: | --- | --- |
| Certify | 181 | Operation Failed | Invalid CSR, Invalid Object Type, Item Not Found, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Protection Storage Unavailable, Response Too Large |
| Check | 185 | Operation Failed | Illegal Object Type, Incompatible Cryptographic Usage Mask, Object Not Found, Usage Limit Exceeded, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |
| Obtain Lease | 270 | Operation Failed | Object Not Found, Usage Limit Exceeded, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |
| Re-certify | 302 | Operation Failed | Invalid CSR, Invalid Message, Invalid Object Type, Object Not Found, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Protection Storage Unavailable, Response Too Large |
| Re-key | 307 | Operation Failed | Cryptographic Failure, Invalid Field, Invalid Message, Invalid Object Type, Key Value Not Present, Object Not Found, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Protection Storage Unavailable, Response Too Large |
| Re-key Key Pair | 312 | Operation Failed | Cryptographic Failure, Invalid Field, Invalid Message, Invalid Object Type, Key Value Not Present, Object Not Found, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Private Protection Storage Unavailable, Public Protection Storage Unavailable, Response Too Large |
| Re-Provision | 315 | Operation Failed | Cryptographic Failure, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |
| Register | 294 | Operation Failed | Attribute Read Only, Attribute Single Valued, Bad Password, Encoding Option Error, Invalid Attribute, Invalid Attribute Value, Invalid Object Type, Non Unique Name Attribute, Server Limit Exceeded, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Protection Storage Unavailable, Response Too Large |
| Revoke | 297 | Operation Failed | Invalid Field, Invalid Object Type, Object Not Found, Wrong Key Lifecycle State, Attestation Failed, Attestation Required, Feature Not Supported, Invalid Field, Invalid Message, Operation Not Supported, Permission Denied, Response Too Large |

## Product boundaries and dependencies

The feature uses the approved TTLV codec, typed messages/batches, client execution, TLS/HTTPS transports, object model, credentials, and public-language bindings. It adds no local cryptographic algorithm, server-initiated operation, encoding, transport, or implicit server-policy choice. A caller selects each operation explicitly. One call sends at most one request. The existing typed-batch contract preserves protocol Pending and correlation bytes when the caller enables asynchronous results. Unknown and vendor values remain lossless. Raw KMIP bodies and secret values never appear in logs or errors. Rust, C, Java, and Python must offer equivalent 1.0.0 capability.

## User Scenarios & Testing

### User Story 1 — Certify (Priority: P1)

As a KMIPKit caller, I can send the §6.1.6 Certify request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 179 request and returns a Table 180 success and Table 181 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Certify request, when encoded and sent, then exactly one operation with the Table 179 fields reaches the transport.
2. Given a valid Table 180 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 181 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: One certificate per request. Certificate Request Type is required with Certificate Request; when no request object is supplied, identify the public key and specify Certificate Type as §6.1.6 requires.

**Payload schema** (OASIS Specification v2.1 §6.1.6, Tables 179–180):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Unique Identifier | No | The Unique Identifier of the Public Key or the Certificate Request being certified. If omitted and Certificate Request is not present, then the ID Placeholder value is used by the server as the Unique Identifier. |
| Request | Certificate Request Type | No | An Enumeration object specifying the type of certificate request. It is REQUIRED if the Certificate Request is present. |
| Request | Certificate Request Value | No | A Byte String object with the certificate request. |
| Request | Attributes | No | Specifies desired object attributes. |
| Request | Protection Storage Masks | No | Specifies all permissible Protection Storage Mask selections for the new object |
| Response | Unique Identifier | Yes | The Unique Identifier of the generated Certificate object. |

---

### User Story 2 — Check (Priority: P1)

As a KMIPKit caller, I can send the §6.1.7 Check request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 183 request and returns a Table 184 success and Table 185 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Check request, when encoded and sent, then exactly one operation with the Table 183 fields reaches the transport.
2. Given a valid Table 184 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 185 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Expose the recommended ordered batch with Batch Order Option true and Stop or Undo continuation; do not silently alter the caller batch.

**Payload schema** (OASIS Specification v2.1 §6.1.7, Tables 183–184):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Unique Identifier | No | Determines the object being checked. If omitted, then the ID Placeholder value is used by the server as the Unique Identifier. |
| Request | Usage Limits Count | No | Specifies the number of Usage Limits Units to be protected to be checked against server policy. |
| Request | Cryptographic Usage Mask | No | Specifies the Cryptographic Usage for which the client intends to use the object. |
| Request | Lease Time | No | Specifies a Lease Time value that the Client is asking the server to validate against server policy. |
| Response | Unique Identifier | Yes, unless a failure, | The Unique Identifier of the object. |
| Response | Usage Limits Count | No | Returned by the Server if the Usage Limits value specified in the Request Payload is larger than the value that the server policy allows. |
| Response | Cryptographic Usage Mask | No | Returned by the Server if the Cryptographic Usage Mask specified in the Request Payload is rejected by the server for policy violation. |
| Response | Lease Time | No | Returned by the Server if the Lease Time value in the Request Payload is larger than a valid Lease Time that the server MAY grant. |

---

### User Story 3 — Obtain Lease (Priority: P1)

As a KMIPKit caller, I can send the §6.1.35 Obtain Lease request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 268 request and returns a Table 269 success and Table 270 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Obtain Lease request, when encoded and sent, then exactly one operation with the Table 268 fields reaches the transport.
2. Given a valid Table 269 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 270 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Preserve Lease Time and Last Change Date; no further use after expiry until a new lease, except the zero-time case in §6.1.35.

**Payload schema** (OASIS Specification v2.1 §6.1.35, Tables 268–269):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Unique Identifier | No | Determines the object for which the lease is being obtained. If omitted, then the ID Placeholder value is used by the server as the Unique Identifier. |
| Response | Unique Identifier | Yes | The Unique Identifier of the object. |
| Response | Lease Time | Yes | An interval (in seconds) that specifies the amount of time that the object MAY be used until a new lease needs to be obtained. |
| Response | Last Change Date | Yes | The date and time indicating when the latest change was made to the contents or any attribute of the specified object. |

---

### User Story 4 — Re-certify (Priority: P1)

As a KMIPKit caller, I can send the §6.1.45 Re-certify request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 300 request and returns a Table 301 success and Table 302 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Re-certify request, when encoded and sent, then exactly one operation with the Table 300 fields reaches the transport.
2. Given a valid Table 301 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 302 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: One certificate per request; support optional Offset, certificate request, attributes, masks, and the date/attribute rules in Tables 298–299. Under §6.1.45, the server SHALL create replacement/replaced links and update the public-key Certificate link.

**Payload schema** (OASIS Specification v2.1 §6.1.45, Tables 300–301):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Unique Identifier | No | The Unique Identifier of the Certificate being renewed. If omitted, then the ID Placeholder value is used by the server as the Unique Identifier. |
| Request | Certificate Request Unique Identifier | No | The Unique Identifier of the Certificate Request. |
| Request | Certificate Request Type | No | An Enumeration object specifying the type of certificate request. It is REQUIRED if the Certificate Request is present. |
| Request | Certificate Request Value | No | A Byte String object with the certificate request. |
| Request | Offset | No | An Interval object indicating the difference between the Initial Date of the new certificate and the Activation Date of the new certificate. |
| Request | Attributes | No | Specifies desired object attributes. |
| Request | Protection Storage Masks | No | Specifies all permissible Protection Storage Mask selections for the new object |
| Response | Unique Identifier | Yes | The Unique Identifier of the new certificate. |

---

### User Story 5 — Re-key (Priority: P1)

As a KMIPKit caller, I can send the §6.1.46 Re-key request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 305 request and returns a Table 306 success and Table 307 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Re-key request, when encoded and sent, then exactly one operation with the Table 305 fields reaches the transport.
2. Given a valid Table 306 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 307 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Support the symmetric-key replacement Offset and attribute rules in Tables 303–304; return the replacement identifier. Under §6.1.46, the server SHALL create Replacement Object and Replaced Key links.

**Payload schema** (OASIS Specification v2.1 §6.1.46, Tables 305–306):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Unique Identifier | No | Determines the existing Symmetric Key being re-keyed. If omitted, then the ID Placeholder value is used by the server as the Unique Identifier. |
| Request | Offset | No | An Interval object indicating the difference between the Initial Date and the Activation Date of the replacement key to be created. |
| Request | Attributes | No | Specifies desired object attributes. |
| Request | Protection Storage Masks | No | Specifies all permissible Protection Storage Mask selections for the new object |
| Response | Unique Identifier | Yes | The Unique Identifier of the newly-created replacement Symmetric Key. |

---

### User Story 6 — Re-key Key Pair (Priority: P1)

As a KMIPKit caller, I can send the §6.1.47 Re-key Key Pair request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 310 request and returns a Table 311 success and Table 312 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Re-key Key Pair request, when encoded and sent, then exactly one operation with the Table 310 fields reaches the transport.
2. Given a valid Table 311 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 312 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Support separate common/private/public attributes and masks and the rules in Tables 308–309; return both replacement identifiers. Under §6.1.47, the server SHALL create replacement and replaced links for both key objects.

**Payload schema** (OASIS Specification v2.1 §6.1.47, Tables 310–311):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Private Key Unique Identifier | No | Determines the existing Asymmetric key pair to be re-keyed. If omitted, then the ID Placeholder is substituted by the server. |
| Request | Offset | No | An Interval object indicating the difference between the Initial Date and the Activation Date of the replacement key pair to be created. |
| Request | Common Attributes | No | Specifies desired attributes that apply to both the Private and Public Key Objects. |
| Request | Private Key Attributes | No | Specifies attributes that apply to the Private Key Object. |
| Request | Public Key Attributes | No | Specifies attributes that apply to the Public Key Object. |
| Request | Common Protection Storage Masks | No | Specifies all Protection Storage Mask selections that are permissible for the new Private Key and new Public Key objects |
| Request | Private Protection Storage Masks | No | Specifies all Protection Storage Mask selections that are permissible for the new Private Key object. |
| Request | Public Protection Storage Masks | No | Specifies all Protection Storage Mask selections that are permissible for the new Public Key object. |
| Response | Private Key Unique Identifier | Yes | The Unique Identifier of the newly created replacement Private Key object. |
| Response | Public Key Unique Identifier | Yes | The Unique Identifier of the newly created replacement Public Key object. |

---

### User Story 7 — Re-Provision (Priority: P1)

As a KMIPKit caller, I can send the §6.1.48 Re-Provision request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 313 request and returns a Table 314 success and Table 315 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Re-Provision request, when encoded and sent, then exactly one operation with the Table 313 fields reaches the transport.
2. Given a valid Table 314 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 315 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Support certificate-request, certificate, and parameterless server-credential paths. Table 315 has an errant RNG Retrieve caption in the Re-Provision error section.

**Payload schema** (OASIS Specification v2.1 §6.1.48, Tables 313–314):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Certificate Request | No | The certificate request to be signed |
| Request | Certificate | No | The certificate to replace the existing certificate |
| Response | Unique Identifier | No | The Certificate or Private Key unique identifier |

---

### User Story 8 — Register (Priority: P1)

As a KMIPKit caller, I can send the §6.1.43 Register request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 291 request and returns a Table 292 success and Table 294 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Register request, when encoded and sent, then exactly one operation with the Table 291 fields reaches the transport.
2. Given a valid Table 292 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 294 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Require type, attributes, and object; apply conditional algorithm, length, and signature-attribute rules from §6.1.43.

**Payload schema** (OASIS Specification v2.1 §6.1.43, Tables 291–292):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Object Type | Yes | Determines the type of object being registered. |
| Request | Attributes | Yes | Specifies desired object attributes to be associated with the new object. |
| Request | Any Object (Section 2) | Yes | The object being registered. The object and attributes MAY be wrapped. |
| Request | Protection Storage Masks | No | Specifies all permissible Protection Storage Mask selections for the new object |
| Response | Unique Identifier | Yes | The Unique Identifier of the newly registered object. |

---

### User Story 9 — Revoke (Priority: P1)

As a KMIPKit caller, I can send the §6.1.44 Revoke request and inspect the typed result while controlling its parameters and sensitive values.

**Why this priority**: This operation belongs to the complete 1.0.0 client-initiated inventory.

**Independent Test**: A fake transport captures one Table 295 request and returns a Table 296 success and Table 297 failure; it checks field presence, order, result metadata, and no retry.

**Acceptance Scenarios**:

1. Given a valid Revoke request, when encoded and sent, then exactly one operation with the Table 295 fields reaches the transport.
2. Given a valid Table 296 success, when decoded, then every present field is inspectable and absent fields remain absent.
3. Given a Table 297 failure, malformed payload, or transport error, when the call completes, then status/reason or error and delivery state remain available without retry.
4. Given Pending in a caller-enabled typed batch, when the response arrives, then Pending and the exact correlation bytes remain available without an implicit Poll.

**Specific rules**: Compromise Occurrence Date SHOULD accompany key/CA compromise and SHALL NOT accompany other reasons.

**Payload schema** (OASIS Specification v2.1 §6.1.44, Tables 295–296):

| Side | Field | REQUIRED | Description or condition |
| --- | --- | --- | --- |
| Request | Unique Identifier | No | Determines the object being revoked. If omitted, then the ID Placeholder value is used by the server as the Unique Identifier. |
| Request | Revocation Reason | Yes | Specifies the reason for revocation. |
| Request | Compromise Occurrence Date | No | SHOULD be specified if the Revocation Reason is 'key compromise' or ‘CA compromise’ and SHALL NOT be specified for other Revocation Reason enumerations . |
| Response | Unique Identifier | Yes | The Unique Identifier of the object. |

---

### Edge Cases

- Missing required or incompatible conditional request fields fail before transmission with NotSent delivery evidence.
- Optional absence, repeated field order, unknown values, and opaque structures remain distinct; server results are not fabricated.
- KMIP failure results and malformed network input are handled through the shared result/error and TTLV resource-limit contracts.
- Tickets, credentials, keys, seeds, and raw KMIP bodies never appear in diagnostics.

## Requirements

### Functional Requirements

- **FR-001**: Expose typed request/response models and explicit client calls for Certify, Check, Obtain Lease, Re-certify, Re-key, Re-key Key Pair, Re-Provision, Register, Revoke using the common batch and delivery-state contract.
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
| KMIPKIT-REQ-SPEC-6.1.35-001-001 | §6.1.35 | MAY | When Lease Time is zero, the client may use the object without a lease time limit. |
| KMIPKIT-REQ-SPEC-6.1.35-001-002 | §6.1.35 | SHALL NOT | After a lease expires, do not use the cryptographic object until a new lease is obtained. |
| KMIPKIT-REQ-SPEC-6.1.35-002 | §6.1.35 | MAY | A client may compare Last Change Date with its prior value to decide whether cached attributes need refreshing. |
| KMIPKIT-REQ-SPEC-6.1.35-005 | §6.1.35 | MAY | A client may use the object for the returned Lease Time interval before obtaining another lease. |
| KMIPKIT-REQ-SPEC-6.1.43-001 | §6.1.43 | SHALL | A Register request must specify object type and attributes in an Attributes object. |
| KMIPKIT-REQ-SPEC-6.1.43-005 | §6.1.43 | MAY | A client may send the registered object and its attributes wrapped. |
| KMIPKIT-REQ-SPEC-6.1.43-007 | §6.1.43 | SHALL | When registering a Managed Cryptographic Object, include its required attributes. |
| KMIPKIT-REQ-SPEC-6.1.43-009-001 | §6.1.43 | MAY | Cryptographic Algorithm may be omitted only when encoded in Key Block; it does not apply to Secret Data. |
| KMIPKIT-REQ-SPEC-6.1.43-009-002 | §6.1.43 | SHALL | If Cryptographic Algorithm is present, also include Cryptographic Length. |
| KMIPKIT-REQ-SPEC-6.1.43-010-001 | §6.1.43 | MAY | Cryptographic Length may be omitted only when encoded in Key Block; it does not apply to Secret Data. |
| KMIPKIT-REQ-SPEC-6.1.43-010-002 | §6.1.43 | SHALL | If Cryptographic Length is present, also include Cryptographic Algorithm. |
| KMIPKIT-REQ-SPEC-6.1.43-011 | §6.1.43 | MAY | Digital Signature Algorithm may be omitted only when encoded in the Certificate object. |
| KMIPKIT-REQ-SPEC-6.1.44-001 | §6.1.44 | SHOULD | For key-compromise or CA-compromise revocation, the client should supply Compromise Occurrence Date. |
| KMIPKIT-REQ-SPEC-6.1.44-003-001 | §6.1.44 | SHOULD | Specify Compromise Occurrence Date for key-compromise or CA-compromise reasons. |
| KMIPKIT-REQ-SPEC-6.1.44-003-002 | §6.1.44 | SHALL NOT | Do not specify Compromise Occurrence Date for other revocation reasons. |
| KMIPKIT-REQ-SPEC-6.1.45-001 | §6.1.45 | SHALL | A Re-certify request must renew only one certificate. |
| KMIPKIT-REQ-SPEC-6.1.45-002-001 | §6.1.45 | MAY | A client may omit Certificate Request and identify the public key by Unique Identifier. |
| KMIPKIT-REQ-SPEC-6.1.45-002-002 | §6.1.45 | SHALL | If request fields and Certificate Type in Attributes are omitted, use the existing certificate's type. |
| KMIPKIT-REQ-SPEC-6.1.45-004 | §6.1.45 | SHOULD | A client should perform Re-certify no more than once on a given existing certificate. |
| KMIPKIT-REQ-SPEC-6.1.45-006-001 | §6.1.45 | MAY | A client may set Offset to determine the date difference for the new certificate. |
| KMIPKIT-REQ-SPEC-6.1.45-006-002 | §6.1.45 | SHALL | If Offset is omitted, retain the existing certificate's Activation Date and Deactivation Date. |
| KMIPKIT-REQ-SPEC-6.1.45-008 | §6.1.45 | REQUIRED | Certificate Request Type is required when Certificate Request is present. |
| KMIPKIT-REQ-SPEC-6.1.46-001 | §6.1.46 | SHOULD | A client should perform Re-key no more than once on a given key. |
| KMIPKIT-REQ-SPEC-6.1.46-004-001 | §6.1.46 | MAY | A client may set Offset to determine the date difference for the replacement key. |
| KMIPKIT-REQ-SPEC-6.1.46-004-002 | §6.1.46 | SHALL | If Offset is omitted, lifecycle dates are copied from the existing key. |
| KMIPKIT-REQ-SPEC-6.1.47-001 | §6.1.47 | SHOULD | A client should perform Re-key Key Pair no more than once on a given key pair. |
| KMIPKIT-REQ-SPEC-6.1.47-004-001 | §6.1.47 | MAY | A client may set Offset to determine the date difference for the replacement key pair. |
| KMIPKIT-REQ-SPEC-6.1.47-004-002 | §6.1.47 | SHALL | If Offset is omitted, lifecycle dates are copied from the existing key pair. |
| KMIPKIT-REQ-SPEC-6.1.48-001 | §6.1.48 | SHALL | A client requesting Re-Provision must provide a certificate signing request, a certificate, or no parameters for server-created credentials. |
| KMIPKIT-REQ-SPEC-6.1.48-004 | §6.1.48 | MAY | A client may omit all parameters for server-created credentials, then retrieve the private key with Get. |
| KMIPKIT-REQ-SPEC-6.1.48-006 | §6.1.48 | SHALL | A client that needs new credentials must call Re-Provision. |
| KMIPKIT-REQ-SPEC-6.1.6-001 | §6.1.6 | SHALL | A Certify request must request no more than one certificate. |
| KMIPKIT-REQ-SPEC-6.1.6-002-001 | §6.1.6 | MAY | A client may omit the Certificate Request object. |
| KMIPKIT-REQ-SPEC-6.1.6-002-002 | §6.1.6 | SHALL | If Certificate Request is omitted, the request must identify the public key by Unique Identifier. |
| KMIPKIT-REQ-SPEC-6.1.6-002-003 | §6.1.6 | SHALL | If both Certificate Request Type and Certificate Request are omitted, specify Certificate Type in Attributes. |
| KMIPKIT-REQ-SPEC-6.1.6-003 | §6.1.6 | MAY | A client may retrieve the generated certificate with Get in the same batch using the ID Placeholder. |
| KMIPKIT-REQ-SPEC-6.1.6-007 | §6.1.6 | REQUIRED | Certificate Request Type is required when Certificate Request is present. |
| KMIPKIT-REQ-SPEC-6.1.7-001 | §6.1.7 | SHOULD | A client should use Check in a batch, generally after locating or creating an object and before Get. |
| KMIPKIT-REQ-SPEC-6.1.7-002-001 | §6.1.7 | SHOULD | A client should set Batch Order Option to true in a batch containing Check. |
| KMIPKIT-REQ-SPEC-6.1.7-002-002 | §6.1.7 | SHOULD | A client should use Stop or Undo as the Batch Error Continuation Option in a batch containing Check. |
| KMIPKIT-REQ-SPEC-6.1.7-003 | §6.1.7 | MAY | A client may include the usage amount it considers necessary in Usage Limits Count. |
| KMIPKIT-REQ-SPEC-6.1.7-004-001 | §6.1.7 | MAY | A client may specify intended cryptographic uses in Cryptographic Usage Mask. |
| KMIPKIT-REQ-SPEC-6.1.7-004-002 | §6.1.7 | MAY | The Check mask may differ from the mask used in a preceding Locate. |
| KMIPKIT-REQ-SPEC-6.1.7-005 | §6.1.7 | MAY | A client may specify a desired lease duration in Check. |

### Key Entities

- Operation request: typed fields and preserved structurally valid extensions.
- Operation response: typed fields and preserved unknown data.
- KMIP result: status, reason, permitted message, Pending correlation, and delivery evidence.
- Secret value: ticket, credential, key, seed, or other sensitive bytes subject to redaction and owned-memory zeroization.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All 9 operations have typed request/response, an explicit client call, generic TTLV access, and Rust/C/Java/Python parity before 1.0.0.
- **SC-002**: All Table-defined fields and error forms and all 44 catalog client requirement IDs have positive/relevant negative verification and exact source-to-test links.
- **SC-003**: Fake-transport cases confirm one exchange and preserved Success, Failure, Pending, unknown values, malformed response behavior, and delivery state.
- **SC-004**: Required repository coverage, formatting, lint, security, generated-artifact, and two-independent-server integration gates pass before release.

## Assumptions and source decisions

- The accepted common message, batch, TTLV, TLS, security, and binding contracts remain authoritative. This specification adds only the named operations.
- Server-side obligations are not silently reclassified as client requirements; server policy may reject valid requests.
- Pinned official XML fixtures are incomplete. An unavailable fixture is documented rather than reported as a passing conformance vector.
- Other unresolved source discrepancies must receive an explicit disposition before implementation of affected behavior.

### Accepted reading for replacement links

KMIPKIT-DEC-010 treats the one-replacement cardinality in §11.28/Table 464 as a server-side invariant for this client scope. Sections 6.1.45–6.1.47 explicitly require the server to create replacement links; KMIPKit does not add a preflight validation requirement. Section 6.1.47 uses “Replacement Key” and “Replaced Key”, while §11.28/Table 465 uses “Replacement Object Link” and “Replaced Object Link”. That OASIS naming difference is preserved in the decision record and does not change upstream text.
### Test-case inventory

| Operation | Catalog case IDs |
| --- | --- |
| Certify | No direct catalog case ID |
| Check | No direct catalog case ID |
| Obtain Lease | No direct catalog case ID |
| Re-certify | No direct catalog case ID |
| Re-key | KMIPKIT-TEST-CN01-2-76, KMIPKIT-TEST-CN01-2-77, KMIPKIT-TEST-CN01-2-78, KMIPKIT-TEST-CN01-2-79, KMIPKIT-TEST-CN01-2-80, KMIPKIT-TEST-CN01-2-81, KMIPKIT-TEST-CN01-2-82, KMIPKIT-TEST-CN01-2-83, KMIPKIT-TEST-CN01-2-84, KMIPKIT-TEST-CN01-2-85, KMIPKIT-TEST-CN01-2-86, KMIPKIT-TEST-CN01-2-87 |
| Re-key Key Pair | No direct catalog case ID |
| Re-Provision | No direct catalog case ID |
| Register | No direct catalog case ID |
| Revoke | No direct catalog case ID |

### Maintainer-approved interpretations (2026-10-10)

These are accepted KMIPKit interpretations, not official OASIS errata. Their decision records are in the checked-in normative catalog; pinned upstream copies remain unchanged.

| Operation | Discrepancy | Disposition |
| --- | --- | --- |
| Re-Provision | KMIPKIT-DISC-040 | Support certificate-request, certificate, and parameterless server-credential paths. Table 315 has an errant RNG Retrieve caption in the Re-Provision error section. |
| Unique Identifier | KMIPKIT-DISC-025 | The lowercase “may” in §11.58 is descriptive text, not a new normative client keyword. Explicit operation-specific identifier and batch rules remain in force. Decision: KMIPKIT-DEC-009. |
