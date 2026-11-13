# KMIP 3.0 Fluent Operation Builders (KMIPKIT-0116)

**Parent contract**: KMIPKIT-0093, approved scope KMIP 3.0 operations in the
1.x product line.
**Status**: Approved by the maintainer's explicit authorization to implement
the complete KMIPKIT-0093 plan, including KMIP 3.0 within 1.x.
**Base**: KMIPKIT-0115 Java/Python lifecycle facade head.

## Purpose

Complete the high-level surface for the seven KMIP 3.0 client-to-server
operations already assigned in the approved KMIPKIT-0093 operation matrix.
Applications will set semantic fields, build an existing typed request, and
execute through the existing client without writing raw TTLV or a manual
batch. This adds no operations or assignments beyond that approved matrix.

## Normative sources and existing owners

- Pinned KMIP 3.0 Working Draft (13 July 2026): Create Credential §6.1.9,
  Table 277; Create Group §6.1.10, Table 280; Create User §6.1.13,
  Table 290; Deactivate §6.1.14, Table 293; Decapsulate §6.1.15,
  Table 296; Encapsulate §6.1.22, Table 317; Obliterate §6.1.41,
  Table 374; operation codes §11.37, Table 584.
- Existing typed owners: `kmipkit_protocol::{CreateCredentialRequest,
  CreateGroupRequest, CreateUserRequest, DeactivateRequest,
  DecapsulateRequest, EncapsulateRequest, ObliterateRequest}` and matching
  responses.
- KMIPKIT-0069, ADR-0022, KMIPKIT-0071, and KMIPKIT-0093 govern version,
  Working Draft, direction, and API scope.

## Functional requirements

- **KMIPKIT-0116-FR-001**: Rust, C, Java, and Python MUST expose equivalent
  operation-specific builders for Create Credential, Create Group, Create
  User, Deactivate, Decapsulate, Encapsulate, and Obliterate. Constructors and
  setters MUST accept semantic values matching the existing typed request
  models, including typed credential alternatives and ordered `AttributeSet`
  inputs where applicable. No builder may require callers to construct raw
  operation TTLV or a manual batch.
- **KMIPKIT-0116-FR-002**: Each builder MUST expose local validation, batch-item
  conversion, and one-exchange execution through existing typed client
  execution. It MUST honor the caller's explicitly selected KMIP 3.0 version,
  existing validation, request options, asynchronous indicator, typed result,
  correlation, and delivery-state behavior. It MUST NOT select KMIP 3.0
  implicitly, issue a retry, or Poll.
- **KMIPKIT-0116-FR-003**: Create Credential MUST accept the existing closed
  typed credential alternatives. Encapsulate and Decapsulate MUST retain the
  existing typed `CryptographicParameters`, optional attributes, and byte
  ownership rules. Decapsulate MUST NOT expose the server-managed shared
  secret; Encapsulate MAY expose only the protocol's returned identifier and
  public `Data`.
- **KMIPKIT-0116-FR-004**: Result access MUST preserve the actual typed fields
  and common result metadata, with absent wire fields remaining absent. Errors,
  `Debug`, logs, and default formatting MUST redact request and response
  payloads and sensitive identifiers according to existing contracts.
- **KMIPKIT-0116-FR-005**: C MUST use opaque handles, fixed-width status codes,
  explicit ownership, and existing error categories. Java and Python MUST use
  idiomatic method names and existing JNI/CFFI lifetime rules. Python MUST
  provide PEP 561 type information. Existing public entry points remain
  source and ABI compatible.
- **KMIPKIT-0116-FR-006**: The public API manifest, capability matrix,
  generated exports, bilingual user guides, examples, and traceability MUST
  describe exactly these seven operations and their explicit 3.0 version gate.

## Acceptance

- All seven operations can be built, validated, converted to one batch item,
  and executed in each language using existing typed request/response paths.
- Local negative tests reject missing or wrong-type semantic inputs and
  reject an unselected or incompatible protocol version before transport.
- Controlled client tests prove one exchange, correct 3.0 operation values,
  field ordering, typed Success/Pending/Failure, no implicit retry, and
  absence of decapsulation shared-secret response data.
- C ABI, JNI, CFFI, Python stubs, source/ABI compatibility, manifest
  generation, T102, export audit, traceability, formatting, lint, coverage,
  and full CI pass on the exact PR head.

## Boundaries

- The 3.0 edition remains the pinned Working Draft. This API does not claim
  final-standard compliance, blanket KMIP 3.0 support, server capability, or
  external-KMS interoperability.
- Builders use the already approved typed models and transport. No generic
  message-send path, arbitrary transport, or new encoding is introduced.
- Protocol 2.1 remains the default; the application explicitly selects 3.0.
- The manifest keeps the 56-entry `client_1_0` KMIP 2.1 operation inventory
  unchanged. These seven opt-in routes are recorded under the separate
  `kmip30_fluent_operations` capability and linked into the versioned fluent
  operation inventory, preserving the distinction between 1.0 default
  operations and the explicitly selected 3.0 Working Draft routes.
