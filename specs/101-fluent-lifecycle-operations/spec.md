# Feature Specification: Fluent Lifecycle and Lookup Operations (KMIPKIT-0101)

**Branch**: `feature/KMIPKIT-0101-fluent-lifecycle-operations`  
**Base**: PR #183 head `8b4724ca5` after rebase
**Status**: Approved within the maintainer's direct authorization of the complete KMIPKIT-0093 plan.  
**Owning identifier**: KMIPKIT-0101

## Purpose

Provide operation-specific Rust fluent requests for common object lookup and
lifecycle operations that already have typed protocol models and client
execution routes. This delivery covers Locate, Activate, Archive, Recover,
Revoke, Re-key, Re-key Key Pair, and Re-certify. It extends the existing
`Client` path and does not create a second encoder, transport, or result model.

## Sources and dependencies

- OASIS KMIP v2.1 §6.1.1 Activate.
- OASIS KMIP v2.1 §6.1.4 Archive.
- OASIS KMIP v2.1 §6.1.28 Locate.
- OASIS KMIP v2.1 §6.1.42 Recover.
- OASIS KMIP v2.1 §6.1.44 Revoke.
- OASIS KMIP v2.1 §6.1.45 Re-certify.
- OASIS KMIP v2.1 §6.1.46 Re-key.
- OASIS KMIP v2.1 §6.1.47 Re-key Key Pair.
- KMIPKIT-0093-FR-004–008, KMIPKIT-0094 operation inventory, and the existing
  request and response types in `kmipkit-protocol`.
- Accepted ADR-0012, ADR-0014, ADR-0018, and ADR-0022 govern typed execution,
  extension insertion, response ownership, and protocol-version validation.

## Functional requirements

- **KMIPKIT-0101-FR-001**: `RequestBuilder` MUST provide operation-specific
  constructors for the eight operations in this slice. Each builder MUST use
  the corresponding existing typed request and response route.
- **KMIPKIT-0101-FR-002**: Each builder MUST expose semantic setters for the
  fields present in its existing request model, preserve caller-supplied
  values and order, and offer `validate`, `build_request`,
  `to_batch_item`, and one-exchange `execute` operations. Required values MUST
  be validated locally by the existing typed request validator. Optional
  identifiers MUST remain omittable where the typed model allows omission; the
  Locate constructor MUST require the caller's `AttributeSet`, including when
  the caller intentionally supplies an empty set. No optional filter,
  identifier, reason, offset, or attribute may be invented.
- **KMIPKIT-0101-FR-003**: Locate MUST accept a prebuilt `AttributeSet` and
  operation-specific paging and object-selection filters. Attribute type,
  value, multiplicity, and version checks MUST use existing catalog and typed
  validation rules. Re-key builders MUST preserve key-specific attribute
  groups and protection masks without moving values between groups.
- **KMIPKIT-0101-FR-004**: Every builder MUST convert to the matching existing
  `ClientRequest` variant. `execute` MUST use the existing `Client` exchange
  path and return its typed batch result, including Success, Pending, Failure,
  delivery state, and correlation where present. It MUST perform at most one
  exchange and MUST NOT Poll or retry automatically.
- **KMIPKIT-0101-FR-005**: Exchange-local `RequestOptions` and Request Header
  asynchronous indicator MUST remain separate. No builder may bypass
  protocol-version validation, alter authentication, send arbitrary TTLV, or
  disclose request payloads through formatting or errors.
- **KMIPKIT-0101-FR-006**: Each public declaration MUST be recorded in
  `specification/api/public-api.json`, generated through the repository tool,
  and mapped to this requirement in traceability. Existing APIs remain
  source-compatible.

## Acceptance criteria

1. One compile-time and focused behavior test exercises construction for each
   of the eight operations, with missing-required-field and invalid-field
   cases where the typed model defines such requirements.
2. Each builder's request variant is checked against its operation, and
   `to_batch_item` is proven to remain local.
3. Controlled exchange tests cover typed Success and at least one Pending and
   Failure route, exact asynchronous correlation, one exchange, and no
   automatic follow-up. Version-incompatible requests fail before transport.
4. Secret-bearing or caller-sensitive request data does not appear in
   `Debug`, `Display`, or error text.
5. Manifest generation, API export audit, Rust format, Clippy, focused tests,
   client suite, traceability, coverage, and the exact-head PR CI pass.

## Exclusions

Attribute retrieval and mutation operations (KMIP operation values 0x0B–0x0F
and 0x30–0x31), cryptographic operations, key import/export and derivation,
KMIP 3.0 operations, and server-to-client listener objects remain in later
KMIPKIT-0093 deliveries. This specification adds no protocol assignment and
makes no interoperability or profile-conformance claim.
