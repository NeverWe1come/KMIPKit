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

Require and preserve caller-supplied hashed bytes, username, and Timestamp; allow an optional raw Hashing Algorithm value. Its effective default is SHA-256 while field absence remains intact. Do not compute the OASIS hash formula locally. Do not create an implicit clock or persistence layer, or implement/claim monotonicity checking or tests, until OD-003 review settles the owner, comparison scope, and clock behavior.

**Rationale**: §9.11 states the formula, monotonic timestamp requirement, and SHA-256 default; Table 415 marks Hashing Algorithm optional. Product definition excludes local cryptographic algorithms.

**Alternatives considered**: hash a supplied password locally (violates product boundary); always emit Hashing Algorithm (changes an optional field's wire presence); silently accept timestamps as monotonic without state/evidence (unverifiable).

### Secret handling and transmission

Redact all Credential formatting and diagnostics. Zeroize KMIPKit-owned secret allocations under the repository's explicit secret type/ownership rules, reusing the pinned workspace `zeroize` dependency if approved by the implementation review. OD-004 is resolved by scope: KMIPKIT-0008 is permanently in-memory for its scope and never adds a production credential writer or send path. Any later send path belongs to a separate approved client feature that owns the candidate callsite and owner-through-transport lifecycle test.

**Rationale**: `AGENTS.md` §8, constitution IV, and `docs/architecture/public-api.md` require redaction and zeroization; the existing architecture explicitly gates secret-bearing encoding.

**Alternatives considered**: allow secret-bearing generic writer use (violates the gate); log raw protocol trees for diagnostics (violates redaction); claim zeroization of foreign runtime copies (not controllable by Rust).

### Device and profile behavior

Represent and preserve every Table 412 field and model the shared secret/Password as optional. The §9.11 prose names Device Serial Number, Network Identifier, Machine Identifier, and Media Identifier for uniqueness; Table 412 additionally defines optional Device Identifier and Password. Client-local data cannot prove global uniqueness. Keep empty/minimum-field validation gated until OD-002 review settles the covered set; do not claim local or global uniqueness enforcement. Authentication applicability and mechanism selection are deferred to KMIPKIT-0010.

**Rationale**: §9.11 enumerates four unique-identifier fields but Table 412 additionally has Device Identifier and Password; all table fields are optional. Uniqueness is not locally observable. The source explicitly points mechanism choices to KMIP Profiles.

**Alternatives considered**: treat every optional field, including Password, as an identifier; assume Device Identifier is in the uniqueness set; impose a client-global identity registry; claim an authentication profile from base-model support.

## Inventory findings requiring review

1. The pinned §9.4 multiple-Credential sentence uses lowercase `must`; §1.2 defines uppercase RFC 2119 keywords. The catalog currently records `KMIPKIT-REQ-SPEC-9.4-001-003` as `MUST` / client / `client_to_server`. Do not test or enforce “all Credentials satisfied” as a client duty. Its exact source casing and catalog classification remain open for catalog-owner review.
2. The catalog summary for `KMIPKIT-REQ-SPEC-9.11-001` incorrectly makes both identification and authentication profile-conditional. §9.11 describes client identification and says Credential MAY be used for authentication as indicated by KMIP Profiles. The summary and its element links require reviewed correction.
3. The exact OD-001 corrections and review evidence are a separate catalog workflow item, not work on the KMIPKIT-0008 catalog input. That item must review: (a) the lowercase source keyword and removal of client normative/client-to-server attribution for `KMIPKIT-REQ-SPEC-9.4-001-003`; (b) the §9.11-001 summary separating client identification from profile-indicated optional authentication; and (c) the associated element links against §9.11 and Table 410. Catalog-owner evidence must record the source/reason and accepted catalog representation. No catalog JSON or generated output is changed by this feature task.
4. §9.11 names Device Serial Number, Network Identifier, Machine Identifier, and Media Identifier in the uniqueness prose, while Table 412 also has optional Device Identifier and Password. Keep every field representable and preserved; do not claim local/global uniqueness. Only empty/minimum-field validation remains gated under OD-002.
5. The lowercase OTP sentence says “may” and the catalog already classifies `KMIPKIT-CLAUSE-SPEC-9.11-008` as `informative_context`, with no requirement ID. OD-006 is resolved: do not add library-wide replay/single-use state or normative client-side enforcement. Request-scoped use follows architecture; request execution selection is OD-005. Keep KMIPKIT-0008 OD-006 distinct from KMIPKIT-0007 OD-006, which concerns secret-send test ownership.
6. §9.11 defines Hashed Password computation and monotonic Timestamp but not the state owner, comparison scope, or clock behavior. Require and preserve the caller's Timestamp and hashed bytes and expose effective SHA-256 when omitted; do not calculate hashes or claim a monotonicity check/test until OD-003 review.
7. OD-004 is resolved by scope: KMIPKIT-0008 remains permanently in-memory and introduces no production credential writer/send path. Any future path requires its own approved feature and candidate-callsite/owner-through-transport lifecycle evidence.
8. OD-005 remains open only for execution integration. An explicit KMIPKIT-0007 handoff must settle inherited defaults, request/batch replacement, omission, precedence, and one Request Header Authentication applying to the entire batch. Standalone in-memory Credential and Authentication models do not need an execution API.

## Dependencies

- KMIPKIT-0005 owns TTLV wire encoding and its separately gated secret-memory/writer lifecycle contract.
- KMIPKIT-0006 owns common headers/messages and currently holds Authentication opaquely.
- KMIPKIT-0007 owns the request selection/execution boundary; only execution integration needs its accepted Authentication handoff, including batch-wide header application.
- KMIPKIT-0010 owns profile-specific authentication applicability and all profile claims.
- The release branch currently contains draft 0005/0006/0007 specifications; their exact acceptance/merge and code gates must be rechecked before implementation.
