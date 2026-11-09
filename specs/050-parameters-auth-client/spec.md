# Feature Specification: KMIPKIT-0050 Parameters, Request Authentication, and Direct Client Operations

**Feature Branch**: `feature/KMIPKIT-0050-parameters-auth-client`
**Created**: 2026-11-05
**Status**: Approved for implementation by the maintainer's direct approval of the Phase 5 plan
**Input**: Approved Phase 5 plan, KMIPKIT-0050 row

## Normative scope

OASIS Key Management Interoperability Protocol Specification Version 2.1 (OASIS Standard, 14 December 2020) is authoritative. Use the pinned source `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html` and the reviewed `specification/catalog/kmip-2.1.json`. Relevant normative sources are:

| Subject | OASIS source | Catalog traceability |
|---|---|---|
| Cryptographic Parameters Structure members, types, alternatives, and table order | §4.16, Table 59; §10.1.2 | `KMIPKIT-ELEM-ATTRIBUTE-CRYPTOGRAPHIC-PARAMETERS`, the 18 stable member IDs in `traceability.md`, and tag elements under §11.56. Table 59 order is verified against the pinned source checksum in `research.md`. |
| Conditional IV Length and Tag Length requirements | §4.16; §11.6, Table 436 | `KMIPKIT-REQ-SPEC-4.16-002`, `KMIPKIT-REQ-SPEC-4.16-003` |
| Request Message and Request Header layout; one header per message and ordered batch items | §§8.1–8.3, Tables 394–396 | Request Message, Request Header, Authentication, and Request Batch Item catalog elements |
| Optional Authentication, non-empty Credential list, repeatable Credential field | §9.4 | `KMIPKIT-REQ-SPEC-9.4-001-001`, `KMIPKIT-REQ-SPEC-9.4-001-002`, `KMIPKIT-REQ-SPEC-9.4-002` |
| Direct typed operation calls | Each applicable client-to-server operation in §6.1 and asynchronous operation in §6.1.42–§6.1.45 | Corresponding `KMIPKIT-ELEM-OP-C2S-*` elements in the normative catalog; operation payload ownership remains with the existing approved operation specifications |
| Corrected KMIP 3.0 KEM response models inherited from KMIPKIT-0071 | Pinned KMIP 3.0 Working Draft §6.1.15, Table 297 (Decapsulate Response Payload); §6.1.22, Table 318 (Encapsulate Response Payload); §11.58, Table 605 (tag assignments) | KMIP 3.0 models in `crates/kmipkit-protocol/src/kem.rs`; this targeted correction is authorized by the Phase 5 correction plan and does not change the 2.1 product default |
| KMIP 3.0 request/response models and direct methods added by FR-016/017 | Pinned KMIP 3.0 Working Draft §6.1.9 Tables 277–278, §6.1.10 Tables 280–281, §6.1.13 Tables 290–291, §6.1.14 Tables 293–294, §6.1.15 Tables 296–297, §6.1.22 Tables 317–318, §6.1.41 Tables 374–375, and §11.37 Table 584 | `crates/kmipkit-protocol/src/kmip30_operations.rs`, `crates/kmipkit-client/src/execute.rs`, and `traceability.md`; source identity and digest are recorded in `specification/oasis/kmip-3.0/SOURCES.md`; support remains opt-in draft support |

This feature adds the Rust high-level façade over the KMIP 2.1 client operations and the seven explicitly authorized KMIP 3.0 Working Draft operations in FR-016/FR-017. KMIP 2.1 remains the 1.0 release baseline; the KMIP 3.0 additions are opt-in draft support and do not establish KMIP 3.0 profile conformance. C, Java, and Python parity for these draft additions is completed in KMIPKIT-0053. Authentication is represented at Request Header scope as specified by OASIS. A single Request Header applies to every Batch Item in its Request Message; there is no per-Batch-Item Authentication field.

## User Scenarios & Testing

### User Story 1 — Construct and inspect Cryptographic Parameters (Priority: P1)

As a Rust caller, I can build and inspect the typed Cryptographic Parameters structure without manually assembling tag/value pairs, while preserving unknown future and vendor fields.

**Why this priority**: Cryptographic operations and key-wrapping metadata currently accept a generic `Structure`; typed fields make valid caller-selected parameters easier to construct and review.

**Independent Test**: Construct each Table 59 member, serialize it, parse it back, and compare tag, type, value, order, and unknown descendants.

**Acceptance Scenarios**:

1. Given typed parameter values supplied in any setter order, when converted to TTLV, then known members appear once and in Table 59 order.
2. Given a valid generic Table 59 `Structure` containing unknown fields, when parsed through the typed model, then the original tree remains lossless and readable through the generic TTLV view.
3. Given malformed member type, duplicate known singleton, or invalid known-field order, when parsed, then a sanitized error is returned without exposing parameter values.
4. Given CTR, CCM, or GCM parameters covered by the existing local rules, when required IV Length or GCM Tag Length is absent, then applicable validation fails; operation-level omission and unknown future enumeration values remain representable.

### User Story 2 — Supply Authentication for one request message (Priority: P1)

As a Rust caller, I can attach a validated Authentication value to exactly one execution batch, including a single-item request, without storing credentials in long-lived client configuration.

**Why this priority**: The public credential models exist, but the execution path does not yet let a caller submit them.

**Independent Test**: Execute a single item and a multi-item batch through the fake transport; inspect the captured Request Header, credential order, body redaction, request delivery state, and one-exchange behavior.

**Acceptance Scenarios**:

1. Given no Authentication on the batch, when executed, then the header omits Authentication and the client performs no credential lookup or implicit authentication.
2. Given a valid Authentication with one or more Credentials, when attached to a batch, then exactly one Authentication structure is encoded in the Request Header and applies to all ordered Batch Items in that message.
3. Given repeated Credential fields in caller order, when encoded, then their order and generic TTLV contents are preserved.
4. Given an empty or malformed generic Authentication Structure, when the caller converts it to typed `Authentication`, then construction fails with a payload-free error before it can be attached to a batch or reach transport. The safe typed API cannot represent empty Authentication.
5. Given an execution error, diagnostic formatting and debug output never contain credential data, raw request bytes, or secret values; KMIPKit-owned secret memory follows the existing zeroization contract.

### User Story 3 — Call every client-initiated operation directly (Priority: P1)

As a Rust caller, I can invoke each of the 56 supported client-initiated operation methods directly on `Client`, while preserving the existing generic batch API and asynchronous outcomes.

**Why this priority**: The current client has direct methods for many operations but leaves gaps that force callers back to manually wrapping the typed request in a batch.

**Independent Test**: A compile-time consumer names every direct `Client` method; fake-transport tests compare each newly added wrapper with the equivalent one-item `ClientBatch` request and response.

**Acceptance Scenarios**:

1. Given a typed request for any of the 56 client-initiated operations, when passed to its direct method, then exactly one request is sent and the same typed result, delivery state, and asynchronous status as the existing batch path are returned.
2. Given any direct method, when the caller selects per-exchange options, then a matching `_with_options` method accepts the same request plus `RequestOptions`; timeout options pass through unchanged and no retry, Poll, or extra exchange occurs implicitly.
3. Given an Authentication-bearing single-item batch, when executed, then it uses the same Request Header path and security guarantees as an authenticated multi-item batch.
4. Given a direct method for an operation already exposed today, when this feature is implemented, then its source-level signature and existing behavior remain available.

### Edge Cases

- Future Enumeration and vendor values remain raw-preserving; the model does not reject values it cannot name.
- Unknown Cryptographic Parameters members remain accessible as generic TTLV and do not count as duplicate known fields.
- Authentication stays absent unless explicitly supplied. There is no client-wide default, implicit selection, credential cache, or precedence between stored and per-call credentials.
- Request-level Authentication is shared by the message batch because KMIP defines it in the single Request Header; per-item credential selection is out of scope.
- Direct operation methods preserve any returned Pending status and correlation value without polling. The Asynchronous Indicator is a Request Header field; callers that need to select it use the existing `ClientBatch` API, which also supports Authentication and ordered multi-item requests.
- Authentication validation errors, transport errors, and `Client` formatting disclose no secret or body contents.
- Authentication and parameters do not alter TLS 1.3/mTLS configuration, select cryptographic algorithms, or assert server policy acceptance.

## Requirements

### Functional Requirements

- **KMIPKIT-0050-FR-001**: The Rust protocol API MUST expose a typed Cryptographic Parameters model for every defined member in KMIP v2.1 §4.16 Table 59, with raw-preserving enumeration accessors and typed Boolean, Integer, and Byte String values.
- **KMIPKIT-0050-FR-002**: The typed model MUST emit its known members once and in Table 59 order regardless of builder call order, and MUST preserve unknown members and values through generic TTLV access.
- **KMIPKIT-0050-FR-003**: The typed model MUST reject known-member type errors, repeated known singleton fields, and known-field ordering violations with sanitized errors while preserving unknown fields where structurally valid.
- **KMIPKIT-0050-FR-004**: Existing Cryptographic Parameters APIs that accept generic TTLV MUST remain source-compatible; the typed model MUST be usable by every existing operation and key-wrapping setter that accepts these parameters.
- **KMIPKIT-0050-FR-005**: Existing §4.16 local rules for required IV Length and GCM Tag Length MUST remain enforced only when supplied parameters make the condition locally knowable; operation-level omission and unknown mode values MUST remain representable.
- **KMIPKIT-0050-FR-006**: A caller MUST be able to attach an already validated `Authentication` value to one `ClientBatch`/Request Message, including a one-item request, and MUST be able to omit Authentication explicitly by using the existing default batch construction.
- **KMIPKIT-0050-FR-007**: Authentication MUST be encoded once in Request Header order and shared by all Batch Items in that Request Message. KMIPKit MUST NOT introduce per-item Authentication, a persistent client credential setting, implicit credential selection, or a credential cache.
- **KMIPKIT-0050-FR-008**: Authentication supplied for execution MUST contain at least one Credential and preserve repeatable Credential source order. The typed protocol constructor MUST reject empty or malformed generic Authentication before it can be attached to a batch; any execution preflight failure before transport MUST report `NotSent`.
- **KMIPKIT-0050-FR-009**: Authentication and credential values MUST remain redacted from `Debug`, `Display`, logs, and errors; KMIPKit-owned secret copies and encoded request storage MUST follow the repository's secret-lifetime and zeroization rules.
- **KMIPKIT-0050-FR-010**: `Client` MUST provide direct typed methods for all 56 client-initiated `ClientOperation` values. Every direct method that performs an exchange MUST have a corresponding `_with_options` form; the inventory MUST identify missing methods and option-form gaps and verify each against the same typed batch path.
- **KMIPKIT-0050-FR-011**: Direct methods MUST preserve one-exchange semantics, caller-selected options, operation results, delivery evidence, and asynchronous Pending/correlation behavior; they MUST NOT retry or poll implicitly.
- **KMIPKIT-0050-FR-012**: The generic batch API and all existing direct method signatures MUST remain available; this feature MUST NOT expose raw transport exchange, add server operations, change TLS policy, or implement server-side authentication.
- **KMIPKIT-0050-FR-013**: Each normative requirement MUST link to the pinned OASIS clause, implementation path, and executable test; project API decisions MUST be traced separately and MUST NOT be represented as OASIS requirements.
- **KMIPKIT-0050-FR-014**: The KMIP 3.0 `EncapsulateResponse` model MUST serialize the §6.1.22 Table 318 fields `Unique Identifier` and `Data` in order; it MUST NOT include the shared secret. The `DecapsulateResponse` model MUST serialize only the `Unique Identifier` required by §6.1.15 Table 297. Their public model shape MUST NOT expose a shared-secret response field.
- **KMIPKIT-0050-FR-015**: Debug formatting for the KMIP 3.0 Encapsulate and Decapsulate response models MUST NOT disclose shared-secret material. Encapsulated `Data` remains available as the protocol's public ciphertext output; this requirement does not classify that `Data` as secret.
- **KMIPKIT-0050-FR-016**: The KMIP 3.0 client API MUST provide typed request and response payload models and direct `Client` methods, including `_with_options` forms, for Create Credential, Create Group, Create User, Deactivate, Decapsulate, Encapsulate, and Obliterate. Payload members and order MUST follow the pinned KMIP 3.0 Working Draft §6.1.9 Tables 277–278, §6.1.10 Tables 280–281, §6.1.13 Tables 290–291, §6.1.14 Tables 293–294, §6.1.15 Tables 296–297, §6.1.22 Tables 317–318, and §6.1.41 Tables 374–375. Create Credential MUST accept only the object alternatives defined by Table 277 and their referenced system-object encodings. The client MUST preserve the existing batch execution path, delivery evidence, and no-retry behavior.
- **KMIPKIT-0050-FR-017**: The KMIP 3.0 operation codes MUST follow KMIP 3.0 §11.37, Table 584 exactly: Create Group=`0x3C`, Obliterate=`0x3D`, Create User=`0x3E`, Create Credential=`0x3F`, Deactivate=`0x40`, Encapsulate=`0x41`, and Decapsulate=`0x42`. They MUST be accepted only when the effective protocol version is KMIP 3.0 and rejected under KMIP 2.0/2.1. The client MUST NOT infer support for reserved or unassigned values such as `0x43`. KEM request/response models MUST keep the shared secret server-managed; only Encapsulate's encapsulated output `Data` may be returned as specified by Table 318.

### Key Entities

- **Cryptographic Parameters**: Ordered TTLV Structure with the 18 known Table 59 members plus losslessly retained unknown members.
- **Authentication**: Optional Request Header Structure containing one or more ordered Credential structures.
- **ClientBatch**: One Request Message's ordered typed Batch Items and shared caller-selected Request Header fields.
- **Direct operation method**: A named `Client` method that accepts an existing typed request and performs the same single exchange as a one-item typed batch.

## Success Criteria

### Measurable Outcomes

- **KMIPKIT-0050-SC-001**: A Rust consumer can name direct methods for all 56 `ClientOperation` values and their `_with_options` variants, including every currently missing method, and compile without using a raw operation name or untyped request value.
- **KMIPKIT-0050-SC-002**: Tests cover all 18 known Cryptographic Parameters members and show builder-order-independent Table 59 output, typed round-trip, unknown-member preservation, and malformed-type/duplicate/order rejection.
- **KMIPKIT-0050-SC-003**: Protocol tests show empty/malformed generic Authentication cannot become a typed value; fake-transport tests show exactly one Authentication header for valid single-item and multi-item batches, no implicit retry, and accurate delivery state for failures before or during transport.
- **KMIPKIT-0050-SC-004**: Debug, Display, and error assertions for Authentication and client execution contain no credential values or request body bytes.
- **KMIPKIT-0050-SC-005**: A Rust consumer can construct each of the seven KMIP 3.0 operation requests, invoke direct and `_with_options` client methods, and observe exact operation codes and request payload order; fake-transport tests reject these operations before any write under KMIP 2.0/2.1.
- **KMIPKIT-0050-SC-006**: Existing tests and Rust APIs using generic `Structure` parameters or `ClientBatch` continue to pass unchanged.

## Assumptions

- The maintainer's direct approval of the Phase 5 plan authorizes 0050 to complete the client execution path for Authentication, superseding the earlier 0008/0038 planning hold on adding a credential-send path.
- KMIP v2.1 §8.2 defines Authentication at Request Header scope. The recommended project API therefore stores it only on the one-shot `ClientBatch`; the same shape supports one-item calls without adding global Client state or a second competing precedence model.
- The existing typed Credential/Authentication models and TLS client authentication remain distinct: KMIP Authentication is an optional KMIP Request Header value, while mTLS authenticates the secure channel.
- The 1.0 adapter parity for C, Java, and Python remains assigned to KMIPKIT-0053; this feature completes the Rust API only.
- The 56-operation inventory is the existing `ClientOperation` enum, including Poll, Cancel, Process, and Query Async Requests, with server-initiated operations excluded.

## Out of Scope

- Implementing credential variants, attestation evidence generation/verification, or changing secret model ownership.
- Authentication inheritance, default credentials, per-operation credential policy, retries, caching, or server login state.
- Adding operations outside the seven KMIP 3.0 operations explicitly authorized in the approval-record addendum.
- Implementing FFI and Java/Python wrappers; these are reconciled in KMIPKIT-0053.
- Claiming profile conformance, server acceptance, formal certification, or interoperability.
<!-- retry-142 -->
<!-- retry-142-2 -->
