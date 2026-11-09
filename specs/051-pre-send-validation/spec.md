# Feature Specification: KMIPKIT-0051 Pre-Send Validation

**Feature Branch**: `feature/KMIPKIT-0051-pre-send-validation`

**Created**: 2026-11-05

**Status**: Approved for implementation under the maintainer-approved Phase 5 plan

**Input**: User-approved Phase 5 plan: provide `validate()` for requests and structures with known schemas; reject locally detectable normative violations in `Client::execute` before transport writes and report `NotSent`.

## User Scenarios & Testing

### User Story 1 - Inspect a request before execution (Priority: P1)

An application developer wants to validate a typed KMIP request or a generic TTLV structure against an explicitly known schema before sending it, so malformed values can be corrected locally.

**Why this priority**: The caller should be able to inspect local protocol errors without connecting to a server or discovering them through a remote rejection.

**Independent Test**: Construct valid and invalid typed requests and schema-identified structures, call the public validation API, and assert deterministic diagnostics with safe field paths and exact KMIP 2.1 references.

**Acceptance Scenarios**:

1. **Given** a typed request satisfying its known schema, **When** the caller validates it, **Then** the report contains no error-severity issues.
2. **Given** a known field with an invalid TTLV type, order, cardinality, or requiredness, **When** the caller validates the request, **Then** the report identifies the field path and rule without including its value.
3. **Given** a generic `Structure` and no schema identifier, **When** the caller validates it, **Then** only generic TTLV structure constraints are checked and no operation or model schema is guessed from tag numbers.

### User Story 2 - Prevent invalid requests from reaching transport (Priority: P1)

An application developer wants the typed client to reject locally detectable normative request violations before any network write, so the server never receives a request that KMIPKit can prove invalid from caller-supplied data.

**Why this priority**: Pre-send rejection protects request correctness and preserves clear delivery evidence for non-idempotent operations.

**Independent Test**: Send invalid typed requests through the existing scripted transport and assert the returned error reports `NotSent`, with zero exchange calls and zero bytes written.

**Acceptance Scenarios**:

1. **Given** a request violating a locally checkable rule, **When** `Client::execute` is called, **Then** it returns a sanitized validation/protocol error with `NotSent` before invoking the transport.
2. **Given** a valid request, **When** it is executed, **Then** its existing serialized request bytes and one-exchange behavior are unchanged.
3. **Given** a constraint that depends on server capabilities, object state, authorization, or vendor policy, **When** the request is locally validated, **Then** KMIPKit does not reject it solely on that unknown server-side condition.

### User Story 3 - Preserve extensibility and safe diagnostics (Priority: P2)

An application developer wants unknown values and extension data to remain available while validation explains only what it can verify, so newer or vendor-specific data is not silently rewritten or disclosed.

**Why this priority**: KMIPKit is an extensible protocol client; validation must not turn incomplete local knowledge into data loss or false conformance claims.

**Independent Test**: Validate structures containing unknown tags and future enumeration values, verify their values and ordering remain unchanged, and search all validation output and formatted errors for secret/input bytes.

**Acceptance Scenarios**:

1. **Given** unknown tags or future raw values in input accepted by an open generic/extension path, **When** validation runs, **Then** those items remain preserved and validation does not classify them as invalid solely because they are unknown.
2. **Given** a diagnostic involving credential, key, or request payload data, **When** it is formatted, **Then** it contains only safe codes, paths, severities, and normative references, never the value or raw body.

## Edge Cases

- A structure's fields are supplied in an order different from the call order of its builder; the final typed serialization still follows the order in the KMIP structure description.
- The same tag appears multiple times: validation applies repeatability from that exact schema, not a global singleton assumption.
- A conditional field rule depends on an optional value present in the request; if the condition is locally decidable, validation applies it, otherwise it records no rejection.
- A generic structure has no schema identity; validation must not infer one from matching tags.
- Unknown or vendor tags occur inside a known structure; they are preserved and not rejected solely for being unknown, while known fields are still checked.
- The request is valid locally but rejected by the server because of policy, capabilities, authorization, or object state; the server result remains authoritative.
- A failure occurs during request serialization after validation but before transport; existing delivery reporting remains `NotSent` and no exchange is attempted.

## Requirements

### Normative Source Boundary

For operation schemas and request validation, use the pinned OASIS KMIP 2.1 source in `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`. The principal cross-cutting sources are §8.1 Table 394, §8.2 Table 395, §8.3 Table 396, §9.4, §9.5, §9.13, §10.1.2, §10.1.3, and §11.56 Table 487 for tag assignments. Operation-specific rules come from the relevant §6.1 operation subsections and their Request Payload tables; nested field types and structures come from their defining OASIS tables. For the already-approved multi-version pre-send gate, use the corresponding pinned 2.0, 2.1, and 3.0 sources identified by KMIPKIT-0069 through KMIPKIT-0071. KMIP 2.0 attribute mutation restrictions use KMIP 2.0 §5 and KMIPKIT-0070 REQ-070-003; KMIP 3.0 tag assignments use §11.58 Table 605 in the pinned 3.0 draft. Cryptographic Parameters rules already implemented under KMIPKIT-0050 remain governed by KMIP 2.1 §4.16 Table 59 and are not broadened by this feature.

OASIS specifies structure fields and their encoding order, data types, requiredness, and conditional rules. It does not prescribe a library method named `validate()`, a client-side pre-send policy, or KMIPKit's `NotSent` delivery-state enum; these are project API and transport decisions recorded separately in `plan.md` and `traceability.md`.

### Functional Requirements

- **KMIPKIT-0051-FR-001**: The Rust API MUST expose structured validation for each supported typed client request and for generic TTLV structures when the caller supplies an explicit schema identity.
- **KMIPKIT-0051-FR-002**: A generic `Structure` without a schema identity MUST be checked only for generic TTLV model/wire constraints; validation MUST NOT infer an operation schema from tags or values.
- **KMIPKIT-0051-FR-003**: For each known schema, validation MUST check locally decidable field type, field order, requiredness, cardinality, and conditional rules exactly as defined by the corresponding OASIS KMIP 2.1 structure description.
- **KMIPKIT-0051-FR-004**: Repeatability and conditionality MUST be taken from the individual schema; no rule may treat all fields as singleton or all optional fields as required.
- **KMIPKIT-0051-FR-005**: The public validation report MUST identify each issue with a stable code, structure-relative path, severity, and exact OASIS document/section/table reference, and MUST omit field values and raw TTLV bytes.
- **KMIPKIT-0051-FR-006**: Existing `Client::execute` paths MUST apply the same locally decidable normative request validation before invoking transport for every in-scope client-initiated operation.
- **KMIPKIT-0051-FR-007**: If pre-send validation fails, execution MUST return a sanitized failure with `RequestDeliveryState::NotSent` and MUST perform zero transport exchanges and zero writes.
- **KMIPKIT-0051-FR-008**: A locally valid request MUST retain its existing wire representation, operation result mapping, delivery semantics, timeout options, and one-exchange/no-implicit-retry behavior.
- **KMIPKIT-0051-FR-009**: Validation MUST NOT reject a request solely because of unknown server capabilities, server-side state, authorization policy, credential acceptance, or vendor policy.
- **KMIPKIT-0051-FR-010**: Unknown tags, future enum values, unknown bitmask bits, and extensions already accepted by the generic or explicitly extensible model path MUST remain losslessly preserved; unknown data MUST NOT be rejected solely for being unknown.
- **KMIPKIT-0051-FR-011**: Validation MUST be read-only with respect to caller-owned request data and MUST NOT serialize or clone secret-bearing values merely to produce diagnostics.
- **KMIPKIT-0051-FR-012**: The requirement inventory MUST cover all 56 client-initiated operation values supported by KMIPKIT-0050, including asynchronous control operations. It MUST record **Set Endpoint Role** (§6.1.54, Table 331) as a client-initiated KMIP operation explicitly deferred to 1.1 by the approved product scope; it MUST NOT misclassify it as server-initiated. Server-initiated operations in §6.2 are outside this feature.
- **KMIPKIT-0051-FR-013**: Every normative validation rule MUST have a stable traceability identifier (`KMIPKIT-0051-R###`) linked to the exact OASIS source, implementation path, and executable test; project API decisions MUST be separately identified.
- **KMIPKIT-0051-FR-014**: Public APIs MUST be additive; existing typed request, generic TTLV, and batch execution entry points MUST remain source-compatible.
- **KMIPKIT-0051-FR-015**: When the audited implementation emits or expects a field tag that conflicts with an exact pinned KMIP 2.1 assignment, the same feature MUST correct that tag using a test-first regression whose expected value is taken independently from the production constant; the traceability record MUST identify every affected request serializer and response reader.
- **KMIPKIT-0051-FR-016**: Import validation MUST check the required members, TTLV types, and table order of PGP Key, Secret Data, and Split Key objects as defined in KMIP v2.1 §2.4 Table 5, §2.7 Table 8, and §2.8 Table 9.
- **KMIPKIT-0051-FR-017**: Split Key validation MUST require Prime Field Size when Split Key Method is Polynomial Sharing Prime Field and MUST reject a Key Part Identifier below 1 or above Split Key Parts, as specified in KMIP v2.1 §2.8 Table 9 and its normative prose.
- **KMIPKIT-0051-FR-018**: Before transport, `Client::execute` MUST apply the existing `VersionGate` to every tag in each serialized Request Payload and to every tag-form attribute mutation using the effective request version. KMIP 2.0 rotate-only mutation restrictions follow the pinned KMIP 2.0 source and KMIPKIT-0070 REQ-070-003; versioned tags and operations follow KMIPKIT-0069 REQ-069-004 and exact assignments in the corresponding pinned OASIS version catalogs, including KMIP 3.0 §11.58 Table 605. Rejection MUST return `NotSent` with zero exchanges and writes. Unknown tags and opaque vendor extension contents remain open as defined in FR-010.
- **KMIPKIT-0051-FR-019**: The public `NestedStructureSchema` MUST expose explicit identities for the KMIP 2.1 PGP Key (§2.4, Table 5), Secret Data (§2.7, Table 8), and Split Key (§2.8, Table 9) structures. Selecting one MUST apply that structure's required fields, TTLV types, order, cardinality, and locally decidable conditional rules without inferring a schema from the input tags.

### Key Entities

- **Validation Schema**: The explicit identity for a known KMIP Structure or operation Request Payload, with its field order, TTLV types, cardinality, requiredness, and local conditional rules.
- **Validation Issue**: A redacted diagnostic containing a stable issue code, field path, severity, and normative source reference, with no input value.
- **Validation Report**: The deterministic collection of issues for one request or schema-identified structure.
- **Delivery Evidence**: The existing `RequestDeliveryState` describing whether a request was not sent, possibly sent, or had begun receiving a response.

## Success Criteria

### Measurable Outcomes

- **KMIPKIT-0051-SC-001**: All 56 client-initiated operations have an explicit validation inventory entry, tested schema coverage, or an exact documented exclusion with a source-backed reason; no operation is silently omitted.
- **KMIPKIT-0051-SC-002**: Tests exercise known-schema order, type, requiredness, cardinality, and every locally decidable conditional rule in the implemented inventory.
- **KMIPKIT-0051-SC-003**: Every tested invalid pre-send request returns `NotSent` with zero transport exchanges and zero bytes written.
- **KMIPKIT-0051-SC-004**: Valid request fixtures retain byte-for-byte identical TTLV encoding before and after validation integration.
- **KMIPKIT-0051-SC-005**: Automated checks find no credential, key material, raw request bytes, or input values in validation reports, error formatting, or Debug output.
- **KMIPKIT-0051-SC-006**: Normative validation requirements have 100% traceability to pinned OASIS references, code, and executable tests.
- **KMIPKIT-0051-SC-007**: Malformed PGP Key, Secret Data, and Split Key Import requests are rejected before transport with `NotSent`; the Split Key tests cover both conditional requiredness and the normative Key Part Identifier bounds.
- **KMIPKIT-0051-SC-008**: Request payload tags and attribute mutations outside the effective KMIP version are rejected before transport; version 2.1 remains the default, and unknown extension data is not rejected solely for being unknown.
- **KMIPKIT-0051-SC-009**: A caller can select explicit PGP Key, Secret Data, and Split Key nested schemas and receive the same required-field and locally decidable condition findings as the managed-object pre-send path.

## Assumptions

- The maintainer-approved Phase 5 plan authorizes this implementation scope and its branch to be based on the pushed KMIPKIT-0050 branch.
- KMIP v2.1 §6.1 contains 57 client-initiated operation subsections. The 1.0 product scope supports the 56 operations implemented by KMIPKIT-0050 and defers the client-initiated Set Endpoint Role operation in §6.1.54 to 1.1. This product-scope exclusion is distinct from the §6.2 server-initiated operations, which are also out of scope.
- KMIP 2.1 is the source of truth for operation schemas and remains the default client version. The already-approved multi-version boundary uses the pinned 2.0 and 3.0 sources only for their version-specific tag, operation, and mutation gates. Where a source does not prescribe local behavior, use a deterministic, read-only validator that reports only what can be proven from the supplied request.
- The validator covers typed Rust client APIs in this specification; C, Java, and Python parity is reconciled by KMIPKIT-0053 as already planned.
- Existing multipart, profile, message-shape, Cryptographic Parameters, Authentication, and model-local checks are reused or reconciled rather than duplicated with conflicting semantics.
- A successful local validation is not a claim that a server supports or will accept the request.

## Out of Scope

- Server-initiated operations, a KMIP server implementation, or 1.1 behavior.
- Server capability discovery, authorization decisions, key policy, object state, or vendor-specific policy enforcement.
- Changing the wire encoder, transport framing, TLS policy, request retry behavior, or response validation.
- Claiming Baseline Client conformance, formal certification, or interoperability from local validation.
- C, Java, or Python API parity work assigned to KMIPKIT-0053.
