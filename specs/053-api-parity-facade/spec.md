# Feature Specification: Four-Language API Parity and Final Facade

**Feature Branch**: `feature/KMIPKIT-0053-parity-facade`

**Created**: 2026-11-06

**Status**: Approved for implementation by the maintainer's Phase 5 plan.

**Input**: User-approved Phase 5 plan assigning KMIPKIT-0053 to reconcile public APIs and deliver equivalent Rust, C, Java, and Python capabilities after KMIPKIT-0047 through KMIPKIT-0052.

## User Scenarios & Testing

### User Story 1 - Use typed attributes in every supported language (Priority: P1)

An application developer can construct, inspect, and add the KMIP 2.1 direct attributes supported by KMIPKit through the Rust, C, Java, or Python APIs, while retaining the generic TTLV path for repeated, unknown, future, and vendor data.

**Why this priority**: Typed attributes connect managed objects and operations to interoperable, inspectable values.

**Independent Test**: A real consumer in every language round-trips all 62 directly encoded attribute types applicable to AttributeSet, CurrentAttribute, and NewAttribute, including all source-defined value categories and invalid tag/type combinations.

**Acceptance Scenarios**:

1. **Given** any applicable catalogued attribute, **When** a consumer builds or reads it, **Then** the value type, raw value, and tag match the KMIP 2.1 source and Rust reference model.
2. **Given** repeated attributes or an unknown/vendor Item, **When** the consumer inserts and reads the AttributeSet, **Then** order and values remain available without rejecting data solely because it is unknown.

### User Story 2 - Build and inspect managed objects safely (Priority: P1)

An application developer can build and inspect all nine typed managed objects, Key Block, Key Value, Key Material, and wrapping metadata from each supported language, while keeping unknown data lossless and secret ownership explicit.

**Why this priority**: Managed objects and keys are central request and response values for KMIP clients.

**Independent Test**: Four-language consumers exercise each object type, plaintext and wrapped Key Value, each supported Key Material representation, typed wrapping fields, unknown extension round-trips, redacted diagnostics, and close/release behavior.

**Acceptance Scenarios**:

1. **Given** a typed object or key structure, **When** a consumer converts it to generic TTLV and back, **Then** the direct object tag, child order, item types, and values match the Rust model.
2. **Given** secret-bearing key material, **When** the owning handle is closed or released, **Then** KMIPKit-owned copies are zeroized and automatic Debug/error formatting does not reveal the value.

### User Story 3 - Configure cryptographic parameters and message authentication (Priority: P1)

An application developer can build all supported Cryptographic Parameters and attach one caller-owned Authentication value to an individual request message through any supported language.

**Why this priority**: These values affect operation semantics and must not be silently selected or stored as global credentials.

**Independent Test**: Consumers round-trip every Table 59 parameter and execute one- and multi-item batches with Authentication present and absent, checking exact Request Header order and one-message scope.

**Acceptance Scenarios**:

1. **Given** any known or future raw Enumeration in Cryptographic Parameters, **When** the client builds the structure, **Then** it preserves the raw value and emits known fields in KMIP order.
2. **Given** an Authentication value on a batch, **When** the request is encoded, **Then** it appears once in the Request Header for all Batch Items and is never stored as client-global configuration.

### User Story 4 - Validate requests consistently before sending (Priority: P1)

An application developer can inspect a structured validation report and rely on all four language clients to reject locally checkable normative request violations before transport writes.

**Why this priority**: The existing Rust validator must not be bypassed by C, Java, or Python callers.

**Independent Test**: For representative invalid typed requests in each language, verify identical issue metadata, `NotSent`, zero bytes written, no secret values in diagnostics, and unchanged wire bytes for valid requests.

**Acceptance Scenarios**:

1. **Given** a typed request with a locally provable KMIP 2.1 violation, **When** execution is attempted through any language, **Then** the shared Rust validation path rejects it before transport and reports `NotSent`.
2. **Given** an unknown tag, future value, or condition that depends on server state, **When** validation runs, **Then** it preserves the data and does not reject it solely because local knowledge is incomplete.

### User Story 5 - Use the complete client and response API in each language (Priority: P1)

An application developer can construct and execute every in-scope client-initiated operation, obtain its typed result and asynchronous outcome, and inspect generic response TTLV consistently in Rust, C, Java, and Python.

**Why this priority**: A 1.0 client requires functional parity across its supported language bindings, including lossless response inspection.

**Independent Test**: A generated and hand-written consumer matrix covers all 56 in-scope operations, direct and batch entry points, operation options, typed result access, asynchronous correlation, Response Header, and optional raw Response Payload.

**Acceptance Scenarios**:

1. **Given** each of the 56 supported client operation identities, **When** the caller invokes it from each language, **Then** it maps to the same Rust operation request and response behavior.
2. **Given** a response with absent, empty, repeated, unknown, or nested vendor TTLV, **When** the caller reads the result, **Then** payload-presence semantics, item order, values, and ownership follow KMIPKIT-0052.

### User Story 6 - Integrate a documented package from one language dependency (Priority: P2)

An application developer can use KMIPKit from the documented package surface, with runnable examples and a consistent package version, without reaching into a lower-level transport API.

**Why this priority**: Consistent packaging and examples make the implemented capabilities usable and reviewable.

**Independent Test**: Clean external consumers build from each package using its documented dependency declaration and execute the language's basic client, typed model, validation, and raw response examples.

**Acceptance Scenarios**:

1. **Given** a clean Rust, C, Java, or Python consumer, **When** it follows the canonical quickstart, **Then** it compiles using the documented KMIPKit package entry point and only the documented dependencies.
2. **Given** the public API manifest and generated outputs, **When** the pinned generator and export audits run, **Then** every route has an exact equivalent or narrowly justified exclusion and no generated diff remains.

## Edge Cases

- A Rust public trait method has no meaningful foreign-language entry point; it requires an exact per-route exclusion with evidence, not a broad language-level waiver.
- A typed attribute's source value category is Structure or Unique Identifier rather than a primitive scalar.
- Repeated attributes, unknown attributes, future enumeration values, unknown mask bits, and vendor extensions must remain accessible.
- Secret-bearing objects are retained by more than one consumer handle; closing one view must not invalidate another live owner or retain the bytes after the last owner closes.
- Malformed or stale opaque handles, null pointers, invalid lengths, and failed conversions must produce stable, redacted errors and release any partially created owned handle.
- Server policy, authorization, capabilities, and current object state remain server-authoritative; local validation cannot infer them.
- One-shot Authentication applies to the request message and its Batch Items, not to a persistent Client or individual operation.
- Pending and Failure results may have different payload-presence rules; APIs must preserve the behavior recorded by KMIPKIT-0052.

## Requirements

### Functional Requirements

- **KMIPKIT-0053-FR-001**: Rust, C, Java, and Python MUST expose equivalent usable capabilities for every 1.0 feature assigned to this specification. Every declaration in the approved candidate inventories MUST be mapped to equivalent public routes or receive an exact, evidence-backed exclusion; no general language or module exclusion is permitted.
- **KMIPKIT-0053-FR-002**: The language bindings MUST expose the 62 directly encoded typed attributes accepted by KMIPKIT-0048 for AttributeSet, CurrentAttribute, and NewAttribute while preserving generic TTLV insertion, repetition, unknown values, and source-defined Unique Identifier alternatives.
- **KMIPKIT-0053-FR-003**: The language bindings MUST expose the nine typed managed objects, Key Block, Key Value, all in-scope Key Material representations, and Key Wrapping Data accepted by KMIPKIT-0049 without performing cryptographic algorithms or format autodetection.
- **KMIPKIT-0053-FR-004**: The language bindings MUST expose the typed Cryptographic Parameters and typed setter behavior accepted by KMIPKIT-0050, preserving unknown Enumeration values, extensions, and generic Structure APIs.
- **KMIPKIT-0053-FR-005**: The language bindings MUST expose caller-owned Authentication on an individual request message with Request Header scope and KMIP order; no credential may be stored as global Client configuration.
- **KMIPKIT-0053-FR-006**: C, Java, and Python MUST call the existing Rust schema-aware request validator for the KMIPKIT-0051 validation capability. An invalid locally checkable request MUST remain `NotSent` and MUST write zero bytes; a generic Structure without a schema identity MUST NOT be assigned a guessed schema.
- **KMIPKIT-0053-FR-007**: Rust, C, Java, and Python MUST expose equivalent operations for the 56-operation KMIPKit 1.0 client scope, including request options and asynchronous outcomes; Set Endpoint Role and server-initiated operations remain excluded under the approved product scope.
- **KMIPKIT-0053-FR-008**: The bindings MUST retain the typed response and decoded-tree inspection semantics of KMIPKIT-0052, including optional payload presence, header access, arbitrary-depth vendor extensions, and explicit foreign ownership/release.
- **KMIPKIT-0053-FR-009**: Language-specific APIs MAY use idiomatic naming and ownership patterns, but each mapping MUST preserve observable value, error, delivery, redaction, and lifetime behavior of its Rust reference route.
- **KMIPKIT-0053-FR-010**: KMIPKit-owned secret copies MUST be zeroized on release. Foreign runtime limitations and copy behavior MUST be documented; no new logging, Debug, exception, or error path may disclose credentials, keys, or raw KMIP bodies.
- **KMIPKIT-0053-FR-011**: Every new public C symbol MUST use the `kmipkit_` prefix, fixed-width values, opaque handles where applicable, panic containment, documented ownership, and stable error mapping. Unsafe Rust MUST remain inside `kmipkit-ffi` with SAFETY explanations.
- **KMIPKIT-0053-FR-012**: The reviewed API manifest MUST map every newly observed public declaration and capability to exact language equivalents or exact exclusions. Generated API declarations, bindings, parity fixtures, and documentation MUST be produced only by the pinned generator.
- **KMIPKIT-0053-FR-013**: English and Spanish user guides MUST include executable consumer examples for typed attributes, managed objects/key material, parameters/authentication, validation, client operations, and response TTLV. Official package versions MUST remain synchronized.
- **KMIPKIT-0053-FR-014**: The Rust high-level facade MUST remain usable with `kmipkit` as the only KMIPKit Cargo dependency and MUST NOT expose low-level byte transport exchange through the facade.
- **KMIPKIT-0053-FR-015**: Every normative source-dependent rule MUST point to its exact pinned KMIP 2.1 clause/table and stable requirement ID. This specification MUST preserve the existing protocol behavior of KMIPKIT-0047 through KMIPKIT-0052 and MUST NOT claim formal conformance, certification, or interoperability from API parity alone.
- **KMIPKIT-0053-FR-016**: The 934 exact Rust validation candidates synchronized from KMIPKIT-0009, KMIPKIT-0043, and KMIPKIT-0044 MUST be reconciled individually. The 12 lower-level aliases with direct four-language counterparts MUST map to `request_validation`; every other candidate MUST have an exact, evidence-backed route exclusion and no blanket Rust exemption.
- **KMIPKIT-0053-FR-017**: C, Java, and Python schema identifiers MUST accept every `RequestPayloadSchema` and `NestedStructureSchema` identity represented by the Rust model, plus `Unsupported`. `SetEndpointRole` remains an accepted identity whose descriptor is intentionally unsupported within the 1.0 product scope.
- **KMIPKIT-0053-FR-018**: C, Java, and Python MUST expose usable request routes for the seven opt-in KMIP 3.0 Working Draft operations assigned by KMIPKIT-0050-FR-016, and an immutable client protocol-version selector to activate them. The selector MUST default to KMIP 2.1 and survive configuration copies such as `withWireEncoding`; the KMIP 1.0 operation inventory MUST remain exactly 56 `client_1_0` operations. A separate versioned API manifest MUST record these draft routes and MUST NOT claim KMIP 3.0 profile conformance or certification.

### Normative Source References

- OASIS KMIP Specification v2.1, exact attribute and conditional tables listed in `specs/048-typed-attributes/gap-matrix.md` and its `traceability.md`.
- OASIS KMIP Specification v2.1, managed object, Key Block, Key Material, and wrapping tables listed in `specs/049-managed-objects/gap-matrix.md` and its `traceability.md`.
- OASIS KMIP Specification v2.1 §4.16 Table 59 (Cryptographic Parameters); §8.2 Table 395 (Request Header Authentication position and scope); §8.3 Table 396 (Request Batch Item); operation-specific §6.1 request/response tables for the 56 supported operations.
- For the separately scoped KMIP 3.0 draft routes in FR-018, use the exact pinned Working Draft references and source digest recorded by KMIPKIT-0050: §6.1.9 Tables 277–278, §6.1.10 Tables 280–281, §6.1.13 Tables 290–291, §6.1.14 Tables 293–294, §6.1.15 Tables 296–297, §6.1.22 Tables 317–318, §6.1.41 Tables 374–375, and §11.37 Table 584. These references define the opt-in payloads and operation codes; the KMIP 2.1 manifest and 1.0 profile scope remain unchanged.
- OASIS KMIP Specification v2.1 §10.1.2 (Structure ordering), relevant field tables and requirement mapping in `specs/051-pre-send-validation/traceability.md`.
- OASIS KMIP Specification v2.1 §8.5 Table 398 and §8.6 Table 399 (response header and payload presence); §9.13 and §11.56 (vendor extension allocation); nested examples in `specs/052-response-navigation/traceability.md`.
- The immutable source is `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`; project API, language mapping, package, and ownership choices are project decisions, not additional OASIS requirements.

## Success Criteria

### Measurable Outcomes

- **KMIPKIT-0053-SC-001**: All approved Rust declaration candidates and in-scope capabilities are either verified with exact four-language mapping evidence or listed as narrowly justified exclusions; zero declarations remain blocked without an owner and reason.
- **KMIPKIT-0053-SC-002**: Every one of the 62 applicable typed attributes and nine managed object types can be constructed, read, and converted to generic TTLV from real Rust, C, Java, and Python consumers.
- **KMIPKIT-0053-SC-003**: All Table 59 Cryptographic Parameters, one-message Authentication behavior, and all 56 in-scope operation identities pass four-language mapping and behavior tests.
- **KMIPKIT-0053-SC-004**: Representative invalid requests through C, Java, and Python report the same Rust validation issue metadata, `NotSent`, and zero transport writes; valid request bytes remain unchanged.
- **KMIPKIT-0053-SC-005**: Secret canaries are absent from every tested automatic diagnostic and Debug/error path; owned KMIPKit copies follow the release/zeroization contract.
- **KMIPKIT-0053-SC-006**: All generated outputs pass `tools/api_manifest/generate.py --check`, and Rust plus C/Java/Python export audits have zero missing, unobserved, mismatched, or unresolved declarations.
- **KMIPKIT-0053-SC-007**: The English and Spanish guides' examples compile/run against synchronized package versions, and a Rust consumer depends on `kmipkit` as its only KMIPKit crate.
- **KMIPKIT-0053-SC-008**: All 934 synchronized validation declarations are resolved as exactly 12 mapped Rust aliases and 922 exact exclusions; a finite inventory test proves every Rust request and nested schema identity has a matching foreign schema-string parser entry.

## Assumptions

- The maintainer's direct approval of the complete Phase 5 plan approves this scope and the ordered 0047→0053 sequence. The maintained plan establishes OASIS KMIP 2.1 as the source of truth and permits recommended project choices where the standard does not prescribe an API shape.
- Implementation initially started from the pushed KMIPKIT-0052 head, and draft PR #133 first targeted `feature/KMIPKIT-0052-typed-responses`. The current canonical release-synchronized PR is review-ready #150, based on KMIPKIT-0052 PR #149 (which is based on PR #143). PR #133 was closed as superseded because GitHub would not change the base of a stacked PR. The #150 release-alias branch initially shared implementation head `47e6aaee4` with #133; it includes the earlier #149 update through merge commit `8407ed352` and final T022 evidence through merge commit `0b2169cd0`.
- KMIPKIT-0047 through KMIPKIT-0052 contain the Rust reference implementations and approved semantics. This feature adds/reconciles foreign bindings and does not alter those Rust contracts except for source-compatible fixes required to expose a missing equivalent behavior.
- A language API may represent a capability through an idiomatic typed wrapper instead of mirroring Rust method names, provided the data, wire output, errors, validation, ownership, and generic fallback are equivalent and the manifest records that mapping.
- The 62 direct attribute types and the 56-operation client scope are inherited from the source-checked records in KMIPKIT-0048 and KMIPKIT-0050; Certificate Attributes has no direct TTLV tag and Set Endpoint Role is deferred to 1.1.
- Coverage and remote CI are temporarily deferred under the maintainer's direct instruction about GitHub billing; they remain release gates and are not waived by this specification.

## Out of Scope

- Server-initiated operations, Set Endpoint Role, a general KMIP server, or any change to the approved 1.0 protocol/transport/language boundaries.
- New wire formats, protocol operations, algorithm execution, automatic algorithm selection, client-global credential storage, or altered server policy decisions.
- Changing the approved Rust API unless needed for source-compatible parity and separately traced in this specification.
- Claiming formal profile conformance, certification, FIPS status, or interoperability without the required external evidence.

## Clarifications

No unresolved clarification remains. The maintainer-approved Phase 5 plan fixes the language set, scope, ordering, required capabilities, security invariants, and stack; the pinned OASIS Standard supplies protocol semantics.
