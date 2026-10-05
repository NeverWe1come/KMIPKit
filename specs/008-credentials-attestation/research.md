# Research: KMIPKIT-0008 Credentials and Attestation

## Source and scope

- Primary source: pinned `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`; checksum is recorded in `specification/oasis/kmip-2.1/CHECKSUMS.sha256`. Source clauses are §9.3/Table 402, §9.4/Table 403, §9.11/Tables 410–416, §9.14/Table 419, and §11.11/Table 442. Only local immutable source copies were consulted.
- Applicable credentials: Username and Password (1), Device (2), Attestation (3), One Time Password (4), Hashed Password (5), Ticket (6), and Extensions (8XXXXXXX). All other raw values must remain round-trippable through generic TTLV.
- No requirement-specific official Test Case IDs are linked in the catalog; the 203 XML fixtures recorded in `KMIPKIT-DISC-036` are unavailable. Acceptance tests must be explicitly labeled derived.

## Decisions

### Credential and Authentication models

Use an ordered `Authentication` collection with at least one `Credential` when present. Represent every named variant from Tables 411–416 and table-required fields. Keep the Extensions value and unknown enum cases opaque. Generic TTLV remains the lossless fallback, including unknown children and vendor values.

**Rationale**: §9.4 makes Authentication optional and requires one or more Credentials when present; Table 403 permits Credential repetition. The public API policy requires future/vendor values to survive round trips. OASIS gives no schema for vendor Extensions credentials.

**Alternatives considered**: reject repeated entries (contradicts Table 403); reject unknown enum values (violates preservation policy); invent an Extensions schema (unsupported by source).

### Attestation and Nonce

Attestation Credential carries caller-provided Nonce, Attestation Type, and at least one of Attestation Measurement or Assertion. Permit both evidence fields because the prose requires data in either field but does not explicitly prohibit both. Preserve Nonce ID/Value exactly from the server's Nonce object. Do not generate or verify evidence.

**Rationale**: §9.11 and Table 413 require Nonce and Attestation Type; §9.14/Table 419 defines the server-assigned and server-created Nonce values. The library is not a local cryptographic provider.

**Alternatives considered**: generate a Nonce or attestation evidence locally (outside product boundary); make evidence exclusive (source does not prohibit both); drop unknown Attestation Type values (lossy).

### Hashed Password

Represent caller-supplied hashed bytes, username, and Timestamp; allow an optional raw Hashing Algorithm value. Its effective default is SHA-256 while field absence remains intact. Do not compute the OASIS hash formula locally. Do not create an implicit clock or persistence layer to claim monotonicity before OD-003 resolves ownership and comparison scope.

**Rationale**: §9.11 states the formula, monotonic timestamp requirement, and SHA-256 default; Table 415 marks Hashing Algorithm optional. Product definition excludes local cryptographic algorithms.

**Alternatives considered**: hash a supplied password locally (violates product boundary); always emit Hashing Algorithm (changes an optional field's wire presence); silently accept timestamps as monotonic without state/evidence (unverifiable).

### Secret handling and transmission

Redact all Credential formatting and diagnostics. Zeroize KMIPKit-owned secret allocations under the repository's explicit secret type/ownership rules, reusing the pinned workspace `zeroize` dependency if approved by the implementation review. This feature is in-memory only and never adds a production credential send path, regardless of OD-004. Any later send path belongs to a separate approved client feature that owns the candidate callsite and owner-through-transport lifecycle test.

**Rationale**: `AGENTS.md` §8, constitution IV, and `docs/architecture/public-api.md` require redaction and zeroization; the existing architecture explicitly gates secret-bearing encoding.

**Alternatives considered**: allow secret-bearing generic writer use (violates the gate); log raw protocol trees for diagnostics (violates redaction); claim zeroization of foreign runtime copies (not controllable by Rust).

### Device and profile behavior

Preserve all Table 412 fields and model the shared secret/password as optional. Do not validate uniqueness against an external namespace. Do not decide which field fulfills §9.11's “at least one field” until OD-002 closes. Authentication applicability and mechanism selection are deferred to KMIPKIT-0010.

**Rationale**: §9.11 enumerates four unique-identifier fields but Table 412 additionally has Device Identifier and Password; all table fields are optional. Uniqueness is not locally observable. The source explicitly points mechanism choices to KMIP Profiles.

**Alternatives considered**: treat every optional field, including Password, as an identifier; assume Device Identifier is in the uniqueness set; impose a client-global identity registry; claim an authentication profile from base-model support.

## Inventory findings requiring review

1. `KMIPKIT-REQ-SPEC-9.4-001-003` currently has `role=client`, `direction=client_to_server`, and `scope_state=client_1_0`, although §9.4 says all supplied credentials must be satisfied by the authentication process. The client can preserve a list but cannot claim server-side satisfaction. A reviewed catalog update is required before requirement assignment.
2. `KMIPKIT-REQ-SPEC-9.11-001` catalog summary says both identification and authentication are profile-conditional. §9.11 describes client identification generally and says authentication MAY follow the selected KMIP Profile. Correct the summary/scope through reviewed catalog workflow.
3. `KMIPKIT-REQ-SPEC-9.11-004-002` says supplied device identifiers must be unique, but client-side enforcement scope is not defined. Treat the uniqueness as identity semantics; no global client validator is possible.
4. The lowercase OTP “may only be used for a single authentication” wording is currently classified as informative by the catalog (`KMIPKIT-CLAUSE-SPEC-9.11-008`). Architecture permits request-scoped OTPs. Do not promote the lowercase wording to a client MUST absent catalog review.
5. §9.11 defines Hashed Password computation and monotonic Timestamp but not the state owner/comparison scope; the feature must not imply that caller-supplied timestamps are verified monotonic.

## Dependencies

- KMIPKIT-0005 owns TTLV wire encoding and its gated private secret-bearing writer boundary.
- KMIPKIT-0006 owns common headers/messages and currently holds Authentication opaquely.
- KMIPKIT-0007 owns the request selection/execution boundary; this feature needs its accepted Authentication request handoff before API integration.
- KMIPKIT-0010 owns profile-specific authentication applicability and all profile claims.
- The release branch currently contains draft 0005/0006/0007 specifications; their exact acceptance/merge and code gates must be rechecked before implementation.
