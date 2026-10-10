# Feature Specification: KMIP 2.1 Hash, MAC, and Signature Operations

**Feature Branch**: `feature/KMIPKIT-0021-hash-mac-signature-operations`

**Created**: 2026-10-09

**Status**: Approved (PR #70 merged into `release/1.0.0` on 2026-10-09; merge commit `be5c73c2198daab4b6877dc319f728f927399feb`)

**Input**: KMIPKit roadmap Phase D: specify the client-initiated Hash, MAC, MAC Verify, Sign, and Signature Verify operations for KMIP 2.1 over TTLV.

## Normative scope

This specification covers five client-to-server operations from the pinned OASIS KMIP Specification v2.1:

| Operation | Catalog element | Request, response, and error tables |
| --- | --- | --- |
| Hash | `KMIPKIT-ELEM-OP-C2S-HASH` | §6.1.24, Tables 235–237 |
| MAC | `KMIPKIT-ELEM-OP-C2S-MAC` | §6.1.32, Tables 259–261 |
| MAC Verify | `KMIPKIT-ELEM-OP-C2S-MAC-VERIFY` | §6.1.33, Tables 262–264 |
| Sign | `KMIPKIT-ELEM-OP-C2S-SIGN` | §6.1.55, Tables 334–336 |
| Signature Verify | `KMIPKIT-ELEM-OP-C2S-SIGNATURE-VERIFY` | §6.1.56, Tables 337–339 |

The shared fields use their normative definitions: Cryptographic Parameters §4.16; Data §7.9; Correlation Value §7.8; Init Indicator §7.17; Final Indicator §7.14; MAC Data §7.20; Signature Data §7.38; and Validity Indicator §11.61. Cryptographic Parameters must preserve all fields and values defined by §4.16, including values KMIPKit does not interpret. Enumeration values remain lossless under the common public-value policy.

Hash requests require Cryptographic Parameters containing the selected Hashing Algorithm (Table 235). Data is present for single-part requests and absent for multi-part requests. The response table specifies Data for single-part operations and no Data for multi-part operations (Table 236). The client supplies the algorithm explicitly; KMIPKit does not hash data locally or select an algorithm.

The MAC and Sign request tables mark Unique Identifier and Cryptographic Parameters optional. The field descriptions allow the server to resolve an omitted Unique Identifier from the ID Placeholder and describe Cryptographic Parameters for the requested method; where an object has no associated parameters and its algorithm requires them, the server returns Operation Failed (Tables 259 and 334). MAC requests carry Data for single-part and omit it for multi-part calls. Sign requests carry Data for single-part unless Digested Data is supplied, omit Data for multi-part calls, and may carry Digested Data (Tables 259 and 334). MAC responses expose MAC Data; Sign responses expose Signature Data, with response field occurrence as printed in Tables 260 and 335. Server-only key resolution and parameter-failure behavior are not implemented by this client feature.

MAC Verify and Signature Verify request tables mark Unique Identifier and Cryptographic Parameters optional. Their field descriptions define ID Placeholder lookup and server parameter-failure handling; those are server duties. MAC Verify may include original Data for algorithms that need it and carries MAC Data for single-part requests, with MAC Data absent for multi-part requests. Signature Verify may include original Data or Digested Data and carries Signature Data for single-part requests, with Signature Data absent for multi-part requests (Tables 262 and 337). The response tables say Validity Indicator is present for single-part and absent for multi-part responses (Tables 263 and 338); the prose in §§6.1.33 and 6.1.56 instead requires it on the final multi-part response and prohibits it on non-final responses. Both sources agree on absence in non-final parts, while the final-part rule conflicts. This conflict is recorded as `KMIPKIT-DISC-048`. The client exposes a received indicator without resolving conformance for a final response; standard values are `Valid`, `Invalid`, and `Unknown` (§11.61). Signature Verify also permits optional recovered Data where the signature algorithm supports recovery (Table 338). An invalid or unknown verification result is an operation result, not a fabricated transport or protocol error.

The catalog's client permissions applicable to this feature are the six `MAY` requirements in §§6.1.32, 6.1.33, 6.1.55, and 6.1.56. Existing records that describe a server `SHALL` or misstate a table description as an independent client obligation will be corrected or retired while retaining their stable IDs. Hash has no standalone client-keyword requirement in §6.1.24; its request and response field contracts are traced to Tables 235–236 through stable feature requirements and the Hash operation element. All five operation elements and their operation enumeration values are assigned to this feature. Exact requirement and element assignments, implementation plans, and verification plans are listed in [traceability.md](traceability.md).

Official test-case links are recorded in the catalog and repeated with their availability in `traceability.md`. Test Cases v2.1 §§2.102–2.107 and the ECDSA Sign case in §2.37 inform test design where linked. A missing fixture is neither a test failure nor a pass. No profile, interoperability, certification, or official test-suite conformance claim is made by this specification.

## Product boundaries and exclusions

- The feature is limited to KMIP 2.1, TTLV, client-initiated calls, the accepted request/batch model, and existing raw TTLV/TLS and TTLV/HTTPS transports.
- The deliverable is an additive Rust protocol model and client API. C ABI, JNI, CFFI, JSON, XML, new transports, and server-initiated operations are out of scope. This feature-level Rust scope does not satisfy or waive release-wide Rust/C/Java/Python capability parity for 1.0; the corresponding bindings remain tracked by their own 1.0 specifications.
- KMIPKit transports cryptographic material and requests cryptographic operations from a KMIP server. It does not implement local hashing, MAC generation, signature generation, or verification algorithms, and does not implicitly choose algorithms, key sizes, parameters, key usage, or protection policy.
- Caller input is represented losslessly. The API does not claim that the chosen key or algorithm is enabled or authorized by a server.
- Server-side obligations, including key lookup through ID Placeholder, key usage accounting, operation execution, generated output, and response-field production, remain server behavior. The client decodes and exposes received responses without claiming that their sender conformed.
- Each explicit operation call performs at most one request exchange. Multipart exchanges are explicit caller-controlled invocations using the returned Correlation Value; KMIPKit performs no implicit continuation, polling, or retry.
- This feature does not change the common transport, TLS, batch, delivery-state, secret-redaction, or TTLV-limit contracts. It does not edit pinned OASIS source copies.

## User Scenarios & Testing

### User Story 1 — Request a hash from a KMIP server (Priority: P1)

As a KMIPKit caller, I can request a hash using an explicitly supplied hashing algorithm and either a single request or explicit multipart calls, so the server performs the operation using its managed cryptographic service.

**Why this priority**: Hash is the only unkeyed operation in this family and provides the smallest path to validate operation-specific data and multipart handling.

**Independent Test**: Fake-transport tests capture a single-part Hash and an explicit multi-part sequence, assert exact operation payloads, and return single-part, intermediate, final, and error responses without invoking local cryptography.

**Acceptance Scenarios**:

1. **Given** a single-part Hash call, **When** the request is encoded, **Then** it contains explicit Cryptographic Parameters with a Hashing Algorithm and Data.
2. **Given** a multi-part Hash call, **When** an intermediate part is sent, **Then** every request part retains required Cryptographic Parameters, omits Data as required for multi-part use, and carries caller-controlled Correlation Value, Init Indicator, and Final Indicator presence according to the shared multipart contract.
3. **Given** a successful single-part Hash response, **When** it is decoded, **Then** required Data and any optional Correlation Value from Table 236 are exposed; for a multi-part response, Data is absent and any Correlation Value is exposed. The client does not calculate a local result.
4. **Given** a server or transport failure, **When** the call completes, **Then** the shared result and delivery state are preserved and the request is not retried.

### User Story 2 — Request a MAC or signature from a KMIP server (Priority: P1)

As a KMIPKit caller, I can request a MAC or signature with an explicit managed key and caller-specified cryptographic parameters, so cryptographic work stays with the server and the request does not hide key or algorithm choices.

**Why this priority**: MAC and Sign are core client operations for the lifecycle of server-held keys and cryptographic material.

**Independent Test**: Fake-transport tests cover each required/optional request field, explicit and ID Placeholder key selection, parameters present or omitted under the specified condition, single-part and multipart data, output fields, errors, and no retry.

**Acceptance Scenarios**:

1. **Given** a MAC or Sign request, **When** it is sent, **Then** optional Unique Identifier and Cryptographic Parameters retain their caller-supplied presence; MAC has Data for a single-part request and omits it for multipart; Sign has Data unless single-part Digested Data is supplied and omits Data for multipart; optional Correlation Value, Init Indicator, and Final Indicator are preserved according to Tables 259 and 334.
2. **Given** Cryptographic Parameters are omitted, **When** the request is built, **Then** omission is represented only as the caller's explicit choice under the OASIS condition that the managed object's attributes provide the parameters; KMIPKit does not infer those attributes locally.
3. **Given** the caller uses an ID Placeholder for a permitted operation, **When** the request is encoded, **Then** the Unique Identifier is omitted and the shared batch ID Placeholder semantics are preserved.
4. **Given** a successful MAC or Sign response, **When** it is decoded, **Then** exactly one Unique Identifier is required; MAC Data or Signature Data is required for single-part and absent for multipart as applicable, and optional Correlation Value is preserved without logging, formatting, or transforming any bytes.

### User Story 3 — Verify a MAC or signature remotely (Priority: P1)

As a KMIPKit caller, I can ask the server to verify supplied MAC or signature material and inspect the returned Validity Indicator, so I can distinguish valid, invalid, and unknown results without treating invalidity as a protocol failure.

**Why this priority**: Verification completes the operation family and exposes a security decision made by the server-held key service.

**Independent Test**: Fake-transport tests exercise the three Validity Indicator values, optional original data and parameters, ID Placeholder key selection, single-part and multipart requests, optional recovered data, server failures, unknown enumeration values, and delivery state.

**Acceptance Scenarios**:

1. **Given** a MAC Verify or Signature Verify request, **When** it is sent, **Then** optional Unique Identifier, Cryptographic Parameters, original Data, and (for Signature Verify) Digested Data retain their caller-supplied presence; MAC Data or Signature Data is required for single-part and absent for multipart; optional Correlation Value, Init Indicator, and Final Indicator are preserved according to Tables 262 and 337. No local verification is performed.
2. **Given** a successful MAC Verify or Signature Verify response, **When** it is decoded, **Then** exactly one Unique Identifier is required; Validity Indicator is required for single-part, omitted on non-final multipart (and a received indicator is a sanitized typed shape error), and either present or absent on final multipart while `KMIPKIT-DISC-048` remains open; optional Correlation Value is preserved. Valid, Invalid, or Unknown is exposed as an operation result, not an exception solely because it is Invalid or Unknown.
3. **Given** a Signature Verify response includes recovered Data, **When** its algorithm supports recovery, **Then** that optional response field is preserved; when it is absent, no data is fabricated.
4. **Given** an unknown future or vendor Validity Indicator value, **When** the response is decoded, **Then** the wire value remains accessible under the shared future-value policy.

### Edge Cases

- Single-part requests include the operation-specific input field required by their request table; multipart requests omit that field when the table specifies “No for multi-part.”
- Successful MAC, MAC Verify, Sign, and Signature Verify responses MUST contain exactly one Unique Identifier as required by Tables 260, 263, 335, and 338. Typed decoding reports a sanitized response-shape error for a missing, repeated, or invalid Unique Identifier while the generic response remains available. Other response fields stay absent when not received; repeated fields, when permitted, retain received order.
- A missing key identifier is represented only through the existing ID Placeholder mechanism where OASIS permits it; no local key ID is invented.
- Omitted Cryptographic Parameters remain omitted on the wire; the client does not inspect key attributes to fill them.
- Cryptographic Parameters that do not match the requested method, or that omit algorithm-required values, are transmitted only as explicitly supplied input; KMIPKit does not claim the server must accept them. The result status and reason are preserved.
- Validity Indicator `Invalid` and `Unknown` remain successful protocol responses unless the KMIP response itself indicates an operation failure.
- Validity Indicator on a final multipart MAC Verify or Signature Verify response is accepted either present or absent while the source conflict is open; on a non-final part it is a typed response-shape violation, with raw TTLV retained for generic inspection.
- A server response containing unsupported but structurally valid fields or future values is retained by typed/generic conversion.
- Transport failure cannot trigger automatic retries because a cryptographic operation may have been received or executed.

## Requirements

### Functional Requirements

- **FR-001**: The Rust client MUST expose explicit entry points for Hash, MAC, MAC Verify, Sign, and Signature Verify through the existing client execution and batch contracts.
- **FR-002**: Hash requests MUST encode Cryptographic Parameters with the caller-supplied Hashing Algorithm and MUST represent Data as required for single-part and multi-part requests by §6.1.24, Table 235.
- **FR-003**: Hash responses MUST expose Data and Correlation Value according to §6.1.24, Table 236, preserving absence and received values; Data is absent for multi-part responses as stated by that table.
- **FR-004**: MAC and Sign requests MUST expose the optional Unique Identifier, optional Cryptographic Parameters, Data, optional Digested Data for Sign, Correlation Value, Init Indicator, and Final Indicator with occurrence governed by §§6.1.32 and 6.1.55, Tables 259 and 334, and shared multipart contracts.
- **FR-005**: The operation request models MUST preserve an omitted Unique Identifier as omitted. They MUST NOT perform local ID Placeholder resolution; the server-side resolution described in Tables 259, 262, 334, and 337 remains server behavior.
- **FR-006**: MAC, MAC Verify, Sign, and Signature Verify MUST preserve omitted or caller-supplied Cryptographic Parameters without silently changing or choosing their values. The six source-backed client permissions to omit Cryptographic Parameters or include original verification Data are linked in `traceability.md`.
- **FR-007**: MAC Verify and Signature Verify requests MUST represent their optional original Data and Digested Data fields, optional Cryptographic Parameters and Unique Identifier, and operation input MAC Data or Signature Data according to §§6.1.33 and 6.1.56, Tables 262 and 337.
- **FR-008**: MAC Verify and Signature Verify responses MUST expose any received Validity Indicator and standard values `Valid`, `Invalid`, and `Unknown`; Signature Verify MUST also preserve optional recovered Data where returned (§11.61; §6.1.56, Table 338). For a final multipart verification response, the typed response MUST accept either presence or absence of Validity Indicator and MUST NOT claim server conformance for either form while `KMIPKIT-DISC-048` remains open. For a non-final multipart response, both sources prohibit Validity Indicator; the typed path MUST report the shared response-shape error, while generic TTLV remains available for lossless inspection.
- **FR-009**: All five typed operation models MUST preserve structurally valid unknown enum values, extension values, unrecognized fields, and generic TTLV data under the existing lossless model contract.
- **FR-010**: The operations MUST preserve common KMIP result status, result reason, permitted result message, batch correlation, and request delivery state.
- **FR-011**: Each explicit invocation MUST perform at most one request exchange; multipart progression is explicit and caller-controlled; the client MUST NOT retry automatically.
- **FR-012**: Public diagnostics MUST follow existing redaction rules and MUST NOT include credentials, keys, cryptographic input/output bytes, raw KMIP bodies, or TLS private keys.
- **FR-013**: The feature MUST NOT perform local cryptographic operations or implicitly choose algorithms, key sizes, parameters, usage, or protection policy.
- **FR-014**: The public Rust API MUST have English and Spanish user-guide documentation with executable examples for all five operations, caller-supplied cryptographic choices, multipart calls, verification results, and the open response-shape discrepancy.
- **FR-015**: Successful MAC, MAC Verify, Sign, and Signature Verify response payloads MUST contain exactly one valid Unique Identifier as required by Tables 260, 263, 335, and 338. Typed decoding MUST return a sanitized response-shape error when it is missing, repeated, or malformed, while preserving generic response access.

### Key Entities

- **Cryptographic Parameters**: A lossless §4.16 structure supplied by the caller or omitted as permitted by operation semantics.
- **Operation input**: Data, Digested Data, MAC Data, or Signature Data used as operation request material, with single-part or multi-part cardinality from the operation table.
- **Key selector**: An optional Unique Identifier. KMIPKit sends an omitted identifier as omitted; any ID Placeholder resolution remains server behavior in the existing batch contract.
- **Operation response**: Hash output, MAC output, signature output, or verification result returned by the server, with absent/repeated fields retained.
- **Validity Indicator**: The `Valid`, `Invalid`, or `Unknown` result enumeration from §11.61, including future and extension values through the shared value policy.
- **Correlation Value**: The server-returned value carried by the caller in later explicit multipart operations.
- **Delivery State**: Existing `NotSent`, `PossiblySent`, or `ResponseStarted` evidence for the exchange.

## Success Criteria

- **SC-001**: All five operation elements, operation enumeration values, the operation-specific data fields cataloged to KMIPKIT-0021, relevant Validity Indicator values, and every applicable client requirement are assigned to KMIPKIT-0021 in the normative catalog; generated coverage outputs pass consistency checks. Digested Data remains assigned to the shared tag-registry feature KMIPKIT-0004 and its Sign/Signature Verify behavior is traced by FR-004 and FR-007. Hash table contracts are covered by feature FRs without inventing catalog `REQ` IDs for table cells.
- **SC-002**: Focused model and fake-transport tests cover all five operations, single-part and multipart forms, required and optional request/response fields, errors, unknown values, delivery states, and no retry.
- **SC-003**: Typed-to-TTLV and TTLV-to-typed round trips preserve all operation-specific fields and structurally valid future values without invoking local cryptography.
- **SC-004**: Official OASIS test mappings and fixture availability are documented accurately; no missing fixture is reported as passed or failed, and no conformance claim exceeds available evidence. `KMIPKIT-DISC-048` remains open unless a pinned erratum or accepted project decision resolves it.
- **SC-005**: Changed protocol/model code meets the repository coverage and quality gates in the later implementation PR; this specification PR claims no implementation coverage.
- **SC-006**: English and Spanish user-guide examples compile and exercise the public Rust operation APIs without performing local cryptography or exposing sensitive values.

## Assumptions

- The accepted KMIP message, batch, TTLV, result, client execution, ID Placeholder, secret, and transport contracts remain authoritative.
- The feature is the next non-overlapping client operation family after KMIPKIT-0019 and KMIPKIT-0020, and is tracked as KMIPKIT-0021 / feature number 021.
- OASIS normative request/response tables control occurrence rules. Usage Guide prose and examples are informative and cannot change those rules.
- Key attribute inspection, algorithm availability, key usage limits, cryptographic execution, and response production are server responsibilities.
- Official test-case links do not imply a linked fixture is present or that a profile is supported.

## Clarification Record

No user clarification is needed to define this bounded client feature. Table field requiredness is taken directly from Tables 235–237, 259–264, and 334–339; six client `MAY` permissions are mapped to their existing stable IDs; Hash table contracts use feature FRs rather than invented catalog requirement IDs. The only unresolved source question is the final multipart Validity Indicator conflict in `KMIPKIT-DISC-048`; the feature tolerates both final forms and makes no conformance claim until the source is resolved.
