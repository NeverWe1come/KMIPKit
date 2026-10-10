# Feature Specification: KMIP 2.1 Encrypt and Decrypt Operations

**Feature Branch**: `feature/KMIPKIT-0019-cryptographic-operations`
**Created**: 2026-10-09
**Status**: Authorized for autonomous implementation under the maintainer's standing direct instruction (2026-10-09). This records implementation authorization, not a claim of separate line-by-line review. KMIPKIT-DISC-045 and KMIPKIT-DISC-046 remain open and bounded as specified below.
**Input**: KMIPKit roadmap Phase D: implement the Encrypt and Decrypt operation family for a KMIP 2.1 client.

## Normative scope

This specification covers exactly two client-to-server KMIP 2.1 operations:

| Operation | Catalog element | Normative source | Request/response/error tables | OASIS test-case records |
| --- | --- | --- | --- | --- |
| Decrypt | `KMIPKIT-ELEM-OP-C2S-DECRYPT` | OASIS KMIP Specification v2.1 §6.1.11 | Tables 196–198 | `KMIPKIT-TEST-CN01-2-100` |
| Encrypt | `KMIPKIT-ELEM-OP-C2S-ENCRYPT` | §6.1.17 | Tables 214–216 | `KMIPKIT-TEST-CN01-2-99`, `KMIPKIT-TEST-CN01-2-100`, `KMIPKIT-TEST-CN01-2-101` |

Shared field definitions are in §§4.16 and 4.58; multipart and byte-string fields are in §§7.3, 7.4, 7.8, 7.9, 7.14, and 7.17. Common request, response, batch, and result behavior follows the applicable contracts in §§8.1–8.6 and 9.1–9.2, 9.5–9.9, 9.12–9.13, 9.16, and 9.19–9.21. The pinned KMIP 2.1 Specification and the checked-in catalog are authoritative. Test Cases are evidence only; the Usage Guide is informative.

This specification maps two operation elements, six shared operation-structure elements, the Cryptographic Parameters attribute element, 21 applicable client requirements, and three distinct OASIS Test Case records. Four related server-only requirements remain in the normative catalog but are outside the client implementation count. Common batch requirements are inherited from KMIPKIT-0006 and KMIPKIT-0007 and are linked in traceability.md. All three official XML fixtures are pinned in `specification/oasis/kmip-2.1/fixtures/` with source URLs and SHA-256 values in `SOURCES.md`; collectively they supply Encrypt and Decrypt operation items for fixture-derived tests. Catalog operation-to-case mappings follow the pinned HTML descriptions. The §2.100 HTML describes both operations but its linked XML has no Decrypt item; the §2.101 HTML describes Encrypt only while its XML includes Decrypt items. KMIPKIT-DISC-046 records this evidence mismatch, and tests use only operation messages actually present in each XML file.

The assigned protocol elements are:

| Catalog element | OASIS source | Feature use |
| --- | --- | --- |
| `KMIPKIT-ELEM-OP-C2S-ENCRYPT` | §6.1.17 | Encrypt request and response |
| `KMIPKIT-ELEM-OP-C2S-DECRYPT` | §6.1.11 | Decrypt request and response |
| `KMIPKIT-ELEM-ATTRIBUTE-CRYPTOGRAPHIC-PARAMETERS` | §4.16 | Optional parameters and recognized conditions |
| `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-3-AUTHENTICATED-ENCRYPTION-ADDITIONAL-DATA` | §7.3 | Initial multipart input |
| `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-4-AUTHENTICATED-ENCRYPTION-TAG` | §7.4 | Decrypt input and Encrypt output |
| `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-8-CORRELATION-VALUE` | §7.8 | Multipart continuation |
| `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-9-DATA` | §7.9 | Request Data encodings and response bytes |
| `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-14-FINAL-INDICATOR` | §7.14 | Caller-controlled final part |
| `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-17-INIT-INDICATOR` | §7.17 | Caller-controlled initial part |

The exact source-to-test mappings for these elements and all 21 assigned client requirements are in [traceability.md](traceability.md).

## Product boundaries and exclusions

- Encoding is TTLV only. This is a Rust protocol/client slice extending the existing typed request, response, message, batch, and transport contracts.
- The scope is limited to Encrypt and Decrypt. Hashing, MAC, signing, key derivation, key renewal, and other KMIP operations remain separate families.
- KMIPKit sends requests to a server. It does not perform encryption or decryption locally and does not choose algorithms, key sizes, modes, padding, IVs, or other cryptographic parameters implicitly.
- The caller controls each single-part or multipart exchange. KMIPKit does not retain multipart state, synthesize a Correlation Value, retry a request, poll automatically, or infer missing cryptographic choices.
- The 1.0 language scope for this feature is Rust. C ABI, Java, Python, high-level builders, and API parity remain in the later 1.0 API work.
- Server-side Usage Limits allocation remains the server's responsibility under §6.1.17. The client does not allocate or emulate server object state.
- Server support and policy vary. The client preserves the server's result and does not claim that any server supports these operations or a conformance profile.
- No change to protocol version, TLS policy, transport boundaries, immutable OASIS source copies, or release scope is included.

## User Scenarios & Testing

### User Story 1 — Encrypt data with an explicit KMIP key (Priority: P1)

As a KMIPKit caller, I can ask the KMIP server to encrypt caller-supplied data with a selected Managed Cryptographic Object and explicitly supplied parameters, so the key remains under server control and KMIPKit does not act as a cryptographic provider.

**Why this priority**: Encrypt is a primary client cryptographic operation and exercises the server-side key reference, parameter, data, and response contracts.

**Independent Test**: A deterministic fake transport receives a typed Encrypt request once and returns single-part and multipart response vectors; the client preserves the selected key identity, opaque bytes, optional generated IV, authentication tag, result, and response order.

**Acceptance Scenarios**:

1. **Given** a single-part Encrypt request with a Managed Cryptographic Object identifier and Data, **When** it is encoded and sent, **Then** the request follows Table 214 and the returned payload follows Table 215.
2. **Given** the caller omits Cryptographic Parameters or IV/Counter/Nonce, **When** the object attributes or algorithm satisfy the conditions in §6.1.17, **Then** KMIPKit preserves the omission instead of inventing values.
3. **Given** multipart encryption with a Correlation Value and Init/Final Indicators, **When** each explicit part is sent, **Then** the client transmits the caller's part fields without creating hidden state or retrying.
4. **Given** the server generates a random IV or completes authenticated encryption, **When** it returns the applicable IV/Counter/Nonce or Authenticated Encryption Tag, **Then** the typed response preserves those bytes exactly.
5. **Given** the server returns an operation failure or Pending status, **When** the result is decoded, **Then** the server result remains observable without automatic retry or polling. Local request-delivery classification remains attached only to local client errors under the shared client execution contract.

### User Story 2 — Decrypt data with an explicit KMIP key (Priority: P1)

As a KMIPKit caller, I can ask the KMIP server to decrypt caller-supplied data with a selected Managed Cryptographic Object and any required authenticated-encryption inputs, so plaintext handling stays explicit and tied to the server result.

**Why this priority**: Decrypt complements Encrypt and is required to use KMIPKit for the complete request/response lifecycle of the two operations.

**Independent Test**: A deterministic fake transport receives typed Decrypt requests for single-part and multipart inputs and returns success, failure, and Pending results; tests verify exact response data, key identifier, correlation, and error preservation.

**Acceptance Scenarios**:

1. **Given** a single-part Decrypt request with Data, **When** it is sent, **Then** it follows Table 196 and its response follows Table 197.
2. **Given** a request omits Cryptographic Parameters, **When** the Managed Cryptographic Object provides them, **Then** the request remains valid without client-side parameter synthesis.
3. **Given** multipart Decrypt supplies Authenticated Encryption Additional Data or Authenticated Encryption Tag, **When** the request is an initial part, **Then** the values are carried on that initial request as required by §6.1.11.
4. **Given** the server rejects an identifier, parameter, key state, or correlation value, **When** the result is returned, **Then** the operation-specific Result Reason is preserved and the request is not automatically retried.

### User Story 3 — Preserve cryptographic bytes and diagnostics safely (Priority: P1)

As a KMIPKit caller, I can inspect operation metadata and results without diagnostics exposing request data, decrypted output, authentication material, or a raw KMIP message body.

**Why this priority**: Encrypt and Decrypt inputs and outputs can contain sensitive plaintext, ciphertext, or authentication values; safe diagnostics are required for production use.

**Independent Test**: Tests exercise public Debug/Display and error paths with recognizable sentinel bytes and prove those bytes are not present in formatted output, while wire and typed-value round trips preserve the original octets.

**Acceptance Scenarios**:

1. **Given** Data, Authenticated Encryption Additional Data, or Authenticated Encryption Tag is present, **When** an owned KMIPKit value is dropped, **Then** its owned byte storage is zeroized under the existing TTLV value contract.
2. **Given** any request, response, protocol error, or client result contains operation bytes, **When** Debug, Display, logs, or errors are produced, **Then** the bytes and raw message body are redacted.
3. **Given** an unknown result value or extension is received, **When** the result is decoded and re-encoded, **Then** its raw value is preserved without assigning new standardized semantics.

## Edge Cases

- Data validation follows the explicit multipart matrix in data-model.md. The typed client accepts the single-request Init=true/Final=true form when Data is present. While KMIPKIT-DISC-045 remains open, it rejects that form before transmission only when Data is omitted. This support boundary does not select whether OASIS requires or permits omission; generic TTLV structure handling remains governed by its own contract.
- A missing Unique Identifier is permitted only when the request uses a valid ID Placeholder under the existing batch contract.
- Omitted Cryptographic Parameters and IV/Counter/Nonce are preserved when the server may obtain them from object attributes or generate them. KMIPKit validates only conditions visible in caller-supplied values; it cannot inspect server object attributes.
- IV/Counter/Nonce may be omitted only where the cited operation/algorithm rules allow omission or server generation; the client never generates one implicitly.
- AAD and AEAD Tag placement in multipart requests follows the initial-part rules in §§6.1.11 and 6.1.17; the client does not move these values between caller requests.
- Correlation Value, Init Indicator, and Final Indicator remain caller-controlled multipart inputs. The implementation validates their TTLV type and cardinality without inventing server stream state.
- Duplicate, missing, or wrong-type required fields are rejected before transmission or response acceptance as appropriate; optional and repeated fields follow the exact cited tables.
- Server failures may use any Result Reason permitted by shared Message Data Structures; Tables 198 and 216 are not exhaustive. Unknown future Result Reasons remain lossless.
- A response above the configured decoder message-size limit is rejected before decoder entry; a server-provided Maximum Response Size does not replace the local limit.
- Pending responses remain explicit caller-visible outcomes. KMIPKit does not automatically call Poll or start another operation exchange.

## Requirements

### Functional Requirements

- **KMIPKIT-0019-FR-001**: The client MUST expose distinct typed Encrypt and Decrypt request and successful-response models. Request Data MUST preserve the three encodings permitted by §7.9 (Byte String, Enumeration, or Integer); when response Data is present, it MUST preserve the Byte String encoding specified by Tables 197 and 215. Response Data remains optional. All fields MUST retain the exact operation-table order and generic source tree.
- **KMIPKIT-0019-FR-002**: Encrypt and Decrypt requests MUST represent Unique Identifier, Cryptographic Parameters, Data, and IV/Counter/Nonce according to their respective operation tables. The operation model preserves optional Unique Identifier; the client batch layer permits omission only for a structurally eligible ID Placeholder sequence under KMIPKIT-0006. The client does not predict whether a prior operation will succeed at the server and preserves the server's per-item result.
- **KMIPKIT-0019-FR-003**: The client MUST preserve caller-supplied Cryptographic Parameters and MUST NOT select or synthesize values. When supplied parameters explicitly select a variable-IV Block Cipher Mode, IV Length MUST be present; when they explicitly select GCM, Tag Length MUST be present (§4.16). When parameters are omitted or depend on server-held object attributes, KMIPKit MUST NOT infer object state and MUST pass the server result through.
- **KMIPKIT-0019-FR-004**: Request Data, Correlation Value, Init Indicator, and Final Indicator MUST follow Tables 196/214 and §§6.1, 7.8, 7.9, 7.14, and 7.17. A middle multipart request with neither Init nor Final true MUST contain Data; an initial or final part MAY omit Data under §6.1. A single-part request with both Init and Final true and no Correlation Value is explicitly permitted by §6.1 and MUST be accepted when Data is present, as required by Tables 196/214. While KMIPKIT-DISC-045 remains open, the typed client MUST return a local validation error before transmission only when that single-part form omits Data. KMIPKit MUST NOT create hidden multipart state or correlation values.
- **KMIPKIT-0019-FR-005**: Successful Encrypt and Decrypt responses MUST preserve all present Table 197/215 payload values. Response Data is optional in both tables and is a Byte String; Encrypt also preserves optional IV/Counter/Nonce, Correlation Value, and Authenticated Encryption Tag. A success response MUST contain Unique Identifier; non-success and Pending response shapes MUST follow the shared client outcome contract and MUST NOT be rejected for lacking a success payload.
- **KMIPKIT-0019-FR-006**: For multipart Encrypt and Decrypt, supplied Authenticated Encryption Additional Data MUST be present on the initial request; for multipart Decrypt, a supplied Authenticated Encryption Tag MUST also be present on the initial request.
- **KMIPKIT-0019-FR-007**: Successful responses MUST preserve the required Unique Identifier and all present operation payload values. Completed failures MUST preserve KMIP Result Status, any Result Reason permitted by the shared Message Data Structures (including reasons absent from Tables 198/216), and any Result Message. Unknown future reason values MUST remain lossless. Pending remains the separate shared PendingOutcome, with its Asynchronous Correlation Value distinguished from multipart Correlation Value.
- **KMIPKIT-0019-FR-008**: One caller invocation MUST perform at most one request exchange. Delivery classification, Pending handling, decoder limits, and no-automatic-retry behavior MUST reuse the existing client contracts; no automatic Poll or cryptographic operation is permitted.
- **KMIPKIT-0019-FR-009**: KMIPKit-owned operation byte values MUST be redacted from diagnostics and zeroized according to the existing opaque TTLV `Value` ownership contract. No raw KMIP body, plaintext, ciphertext, AAD, AEAD Tag, or private key material may be logged or included in errors.
- **KMIPKIT-0019-FR-010**: The client MUST NOT implement encryption/decryption algorithms locally, allocate server Usage Limits, or claim server/profile support without the required external evidence. When the caller knows a target object is archived, the caller MUST explicitly complete Recover before issuing Encrypt or Decrypt; KMIPKit MUST NOT infer remote object state or recover automatically.
- **KMIPKIT-0019-FR-011**: Every applicable client source clause, stable client catalog requirement, operation element, shared operation-structure element, and available OASIS test-case record MUST be assigned to this feature. Each applicable client normative requirement MUST link to implementation paths and executable verification before implementation review. Related server-only requirements remain recorded in the normative catalog and are excluded from client feature ownership and acceptance counts. Shared batch requirements are inherited with their owning feature and evidence identified.
- **KMIPKIT-0019-FR-012**: The implementation MUST include positive and negative protocol tests, deterministic fake-transport tests, byte-preservation tests, and diagnostic-redaction tests. It MUST execute all 28 in-scope Encrypt/Decrypt request-response pairs in the three pinned official XML fixtures as fixture-derived operation-item tests (10 Encrypt pairs in §2.99, 10 Encrypt pairs in §2.100, and 4 Encrypt plus 4 Decrypt pairs in §2.101). The fixture adapter MUST pair messages in source order, retain case and step identity, filter to in-scope Encrypt/Decrypt messages before symbol validation, substitute only documented deterministic values for $NOW, $UNIQUE_IDENTIFIER_0, and $CORRELATION_VALUE in those selected messages, and fail on unknown symbols there or skipped in-scope pairs. It MUST NOT reject the fixtures based on symbols used only by out-of-scope operations. These partial fixture tests MUST NOT be reported as complete official Test Case passes; additional table-derived tests MUST be labeled separately.

### Normative Traceability

| Catalog requirement | OASIS source | Feature requirement / owner | Planned verification |
| --- | --- | --- | --- |
| KMIPKIT-REQ-SPEC-6.1-001-001 | §6.1 | FR-001 / KMIPKIT-0019 | Typed client accepts the operation it implements |
| KMIPKIT-REQ-SPEC-6.1-001-002 | §6.1 | FR-001 / KMIPKIT-0019 | Typed success, failure, and Pending response behavior |
| KMIPKIT-REQ-SPEC-6.1-003-002 | §6.1; §§8.1–8.6 | FR-002 / KMIPKIT-0019; ordered batch construction inherited from KMIPKIT-0006 | Client tests reject locally detectable ineligible placeholder shapes, and preserve the server's per-item result when an eligible later item follows a prior server failure |
| KMIPKIT-REQ-SPEC-6.1-005 | §6.1 | FR-010 / KMIPKIT-0019; Recover implementation in KMIPKIT-0018 | Fake-client test and bilingual example explicitly execute Recover before Encrypt for a caller-known archived object |
| KMIPKIT-REQ-SPEC-4.16-001-001, KMIPKIT-REQ-SPEC-4.16-001-002 | §4.16 | FR-003 / KMIPKIT-0019 | Omission and unknown/object-type-specific parameter preservation |
| KMIPKIT-REQ-SPEC-4.16-002 | §4.16 | FR-003 / KMIPKIT-0019 | Variable-IV mode requires IV Length when supplied parameters expose that mode |
| KMIPKIT-REQ-SPEC-4.16-003 | §4.16 | FR-003 / KMIPKIT-0019 | GCM requires Tag Length when supplied parameters expose GCM |
| KMIPKIT-REQ-SPEC-6.1.11-001-001, KMIPKIT-REQ-SPEC-6.1.11-001-002 | §6.1.11; Table 196 | FR-002, FR-003 / KMIPKIT-0019 | Decrypt omission and exact-preservation tests |
| KMIPKIT-REQ-SPEC-6.1.11-004-001, KMIPKIT-REQ-SPEC-6.1.11-004-002 | §6.1.11; Table 196 | FR-003 / KMIPKIT-0019 | Preserve supplied parameters; server failure passthrough if server-held parameters are unavailable |
| KMIPKIT-REQ-SPEC-6.1.11-005, KMIPKIT-REQ-SPEC-6.1.11-006 | §6.1.11; Table 196 | FR-006 / KMIPKIT-0019 | Multipart initial-part placement tests for AAD and AEAD Tag |
| KMIPKIT-REQ-SPEC-6.1.17-001-001, KMIPKIT-REQ-SPEC-6.1.17-001-002 | §6.1.17; Table 214 | FR-002, FR-003 / KMIPKIT-0019 | Encrypt omission and exact-preservation tests |
| KMIPKIT-REQ-SPEC-6.1.17-005-001, KMIPKIT-REQ-SPEC-6.1.17-005-002 | §6.1.17; Table 214 | FR-003 / KMIPKIT-0019 | Preserve supplied parameters; server failure passthrough if server-held parameters are unavailable |
| KMIPKIT-REQ-SPEC-6.1.17-006 | §6.1.17; Table 214 | FR-006 / KMIPKIT-0019 | Multipart initial-part AAD placement |
| KMIPKIT-REQ-SPEC-7.8-001 | §7.8 | FR-004 / KMIPKIT-0019 | Continuation/final requests include the first server Correlation Value |
| KMIPKIT-REQ-SPEC-6.1-001-003 | §6.1; §§8.1–8.6 | FR-001 / KMIPKIT-0019; shared batch mechanics inherited from KMIPKIT-0006 | Client fake-transport test places Encrypt/Decrypt with another operation in one ordered batch and checks item outcomes remain associated |

Source clauses §§8.1–8.6, 9.1–9.2, 9.5–9.9, 9.12–9.13, 9.16, and 9.19–9.21 are inherited from KMIPKIT-0006, KMIPKIT-0007, and KMIPKIT-0009 as applicable; traceability.md names the specific owner and evidence paths. The Result Reason values shown in Tables 198 and 216 are not exhaustive under §6.1; shared Message Data Structures values and unknown raw values remain supported.

The Encrypt server-side Usage Limits allocation clause is server-only. This feature does not emulate that allocation; it preserves the server's result.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All request and response fields defined by Tables 196, 197, 214, and 215 have typed representation and exact round-trip tests, including optional fields and multipart values.
- **SC-002**: Negative tests reject malformed or missing required single-part fields and invalid field types without logging sensitive values.
- **SC-003**: Fake-transport tests prove one exchange per call, preserve response/result/error data, and do not automatically retry or poll.
- **SC-004**: Diagnostic tests prove recognizable payload sentinel bytes never appear in Debug, Display, or error output; owned TTLV byte values continue to use their existing zeroizing drop behavior.
- **SC-005**: The catalog links every in-scope source requirement, operation, and test-case record to this feature; every applicable requirement links onward to code and executable tests before implementation review.
- **SC-006**: Before implementation review, tests execute all 28 in-scope Encrypt/Decrypt request-response pairs from the pinned XML fixtures: 10 in KMIPKIT-TEST-CN01-2-99, 10 in KMIPKIT-TEST-CN01-2-100, and 8 in KMIPKIT-TEST-CN01-2-101. Filter to these operation items before symbol validation; record item-level results and deterministic substitutions for $NOW, $UNIQUE_IDENTIFIER_0, and $CORRELATION_VALUE. Do not label them complete official Test Case passes. Complete workflow execution belongs to the 1.0 interoperability gate after every operation in each workflow is implemented and supported.
- **SC-007**: English and Spanish Rust API examples compile and demonstrate caller-supplied parameters and explicit multipart continuation without exposing payload bytes.

## Key Entities

- **Managed Cryptographic Object Identifier**: The Unique Identifier naming the server-held key; a valid ID Placeholder may be used under the existing message/batch contract.
- **Cryptographic Parameters**: Caller-supplied operation settings or settings obtained by the server from the managed object's attributes.
- **Operation Data**: Opaque caller-provided input or server-returned output bytes; the client does not interpret them as a local cryptographic operation.
- **Multipart Correlation**: Server-issued Correlation Value and caller-controlled Init/Final Indicators for explicit follow-up parts.
- **Authenticated Encryption Inputs and Output**: Optional AAD and Decrypt authentication Tag inputs plus the Encrypt response authentication Tag.
- **KMIP Operation Result**: Result Status, optional Result Reason and Result Message, and the operation-specific response payload.

## Assumptions

- KMIPKIT-0005 TTLV, KMIPKIT-0006 message/batch, KMIPKIT-0007 client execution, and the current KMIPKIT-0009 Pending-result behavior remain the shared wire foundation; this specification does not revise their contracts.
- The configured local message-size limit is enforced independently of any server-advertised response size.
- A server can reject a request based on implementation support, object state, or policy; that does not by itself mean the client payload model is nonconformant.
- Exact field rules, values, and operation errors come from the pinned OASIS v2.1 source and catalog, not server examples or the Usage Guide.
- No approved KMIP v2.1 erratum is pinned in the repository; this specification uses the pinned OASIS Standard identified by its checked-in source manifest.

## Clarification Record

KMIPKIT-DISC-045 remains open for the one-request Init=true/Final=true Data-omission conflict. The typed client accepts this form when Data is present; while the discrepancy remains open, it returns a local validation error before transmission only when Data is omitted. This support boundary does not select Data requiredness under either reading. KMIPKIT-DISC-046 records a test-evidence source defect: Test Cases §2.100 describes Encrypt and Decrypt, but its linked XML has no Decrypt item; §2.101 describes Encrypt only, but its XML includes Decrypt items. The catalog retains HTML-based case associations while executable fixture evidence uses only messages present in each XML. The Table 59 header extraction is retained only as an excluded source-clause audit record because “REQUIRED” is a column heading and the Cryptographic Parameters row has no requiredness value. The catalog also summarizes §4.16-001-002 without adding a client-side object-type gate. The server-only ID Placeholder requirements in Tables 196 and 214 remain cataloged but are not counted as applicable client requirements; the client omission capability is covered by the §6.1 MAY requirement.

The remaining scope is bounded: two client operations; server-side cryptographic execution and Usage Limits; caller-driven multipart requests; and no local cryptographic algorithms. The three assigned official OASIS XML fixtures are pinned. This feature executes only its in-scope Encrypt/Decrypt items from those multi-operation workflows and records them as fixture-derived operation-item evidence, not complete official-case passes. Complete workflow results remain a 1.0 interoperability gate; fixture presence or partial item results do not establish profile conformance or a complete case pass. OASIS Test Cases §2.100 describes both Encrypt and Decrypt, while the pinned `TC-STREAM-ENC-2-21.xml` contains no Decrypt operation item. The catalog links Decrypt to the case based on its explicit HTML description; fixture-derived tests use only the Encrypt items actually present in the XML, and no missing Decrypt item or complete-case result is inferred.
