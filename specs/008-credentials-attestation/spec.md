# Feature Specification: KMIP 2.1 Credentials and Attestation

**Feature Branch**: `feature/KMIPKIT-0008-credentials-readiness`<br>
**Created**: 2026-10-05<br>
**Status**: Draft — implementation remains gated on T004 and independent review of this exact specification and its reviewer checklist. The accepted KMIPKIT-0005/0006/0007 foundations and ADR-0012 lifecycle boundary have been verified. OD-001 preserves open DISC-041; OD-002 records the Table 412 minimum-presence interpretation; OD-003 limits Timestamp-monotonicity claims; OD-005 applies only to future Authentication selection and does not block these in-memory models. OD-004 is resolved by scope; OD-006 is resolved by the catalog's informative classification.<br>
**Input**: Roadmap item `KMIPKIT-0008-credentials-attestation`.

## Scope and normative sources

This feature defines lossless typed Rust models and validation for KMIP 2.1 `Credential`, `Authentication`, `Nonce`, and the request-header `Attestation Capable Indicator`. It covers the seven Credential Type values in OASIS Table 442, the field schemas in Tables 410–416, the authentication container in Table 403, and the server-provided Nonce in Table 419. Known protocol values receive typed views; unknown enum values and extension payloads remain representable without loss.

This is a client-side model and request-header capability specification. It does not decide which authentication mechanism a server requires, claim conformance to a KMIP Profile, verify device identity or global uniqueness, generate a password hash, create or verify attestation evidence, or generate a Nonce. Profiles and mechanisms remain assigned to KMIPKIT-0010. It reuses the existing private `Client::execute` request builder to emit the Attestation Capable Indicator; it adds no writer, permit, Authentication selection API, Credential payload, transport, retry, persistence, or language binding. This feature never adds or enables a production path that sends credentials. A separate, later client feature must own any approved secret-bearing send path and its candidate-callsite/owner-through-transport lifecycle test; approvals or evidence from this feature or KMIPKIT-0007 cannot substitute for that gate.

The normative authority is the immutable pinned OASIS KMIP Specification v2.1 at `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`. Exact citations below identify the clauses and structural tables. The pinned catalog currently links no official Test Case IDs to these requirements, and the 203 XML fixture references recorded in `KMIPKIT-DISC-036` are unavailable locally; acceptance tests are therefore derived tests, not claimed official vectors.

| Stable requirement / source | Treatment in this feature |
|---|---|
| `KMIPKIT-REQ-SPEC-9.3-001-001`, `KMIPKIT-REQ-SPEC-9.3-001-002`; Specification §9.3, Table 402 | The existing `Client::execute` builder emits True because the public Rust credential API can construct a structurally valid Attestation Credential. The generic view preserves omitted inbound indicators as absent with effective False. This does not claim that KMIPKit creates caller evidence or can satisfy server policy. |
| `KMIPKIT-REQ-SPEC-9.4-001-001`, `KMIPKIT-REQ-SPEC-9.4-001-002`, `KMIPKIT-REQ-SPEC-9.4-002`; §9.4, Table 403 | Authentication is optional; when present it contains one or more Credentials; Credential is repeatable and order-preserving. An empty Authentication is rejected. |
| `KMIPKIT-REQ-SPEC-9.4-001-003`; §9.4 and §1.2 | The pinned sentence uses lowercase “must,” while §1.2 defines uppercase RFC 2119 keywords. The accepted catalog records the clause as server-only and unassigned, while retaining the strength candidate under open `KMIPKIT-DISC-041`; it assigns no client implementation or verification. This feature does not test or enforce “all Credentials satisfied” as a client duty. The catalog correction and independent source review are recorded by KMIPKIT-0002 T045–T047; no explicit catalog-owner sign-off is recorded. |
| `KMIPKIT-REQ-SPEC-9.11-001`; §9.11 | The source describes Credential as used for client identification and says it MAY be used for authentication as indicated by KMIP Profiles. The accepted catalog summary now separates general identification from profile-indicated optional authentication and links Credential reciprocally. Authentication mechanisms and profile claims remain with KMIPKIT-0010. |
| `KMIPKIT-REQ-SPEC-9.11-004-001`, `KMIPKIT-REQ-SPEC-9.11-004-002`, `KMIPKIT-REQ-SPEC-9.11-004-003`; §9.11, Table 412 | Preserve all six Device fields. Interpret “the client SHALL provide at least one field” as requiring at least one member of the Table 412 Device Credential Value structure; the four-identifier uniqueness rule is separate. Do not impose non-empty text or claim local/global uniqueness. |
| `KMIPKIT-REQ-SPEC-9.11-006`; §9.11, Table 413 | Attestation Credential requires Nonce and Attestation Type and supplies at least one of Attestation Measurement or Attestation Assertion. The source does not explicitly prohibit supplying both; this specification allows both. |
| `KMIPKIT-REQ-SPEC-9.11-010-001`, `KMIPKIT-REQ-SPEC-9.11-010-002`; §9.11, Table 415 | Preserve a caller-supplied hashed-password byte string and Timestamp; the source requires the Timestamp to monotonically increase and defaults Hashing Algorithm to SHA-256. State/clock ownership for monotonicity is unresolved (OD-003). The optional algorithm field remains omitted when absent; its effective default is SHA-256. KMIPKit does not calculate the hash. |
| §9.11, Tables 410–416; §11.11, Table 442 | Validate typed shapes and field order for Username and Password, Device, Attestation, One Time Password, Hashed Password, Ticket, and Extensions. The Extensions value remains opaque generic TTLV unless another normative schema is established. Preserve unrecognized enum values. |
| §9.14, Table 419 | Nonce ID and Nonce Value are preserved as server-provided bytes. An Attestation Credential consumes the Nonce object returned by the server; the client does not generate or alter it. |

Catalog element references include `KMIPKIT-ELEM-MESSAGE-FIELD-9-3-ATTESTATION-CAPABLE-INDICATOR`; `KMIPKIT-ELEM-MESSAGE-FIELD-9-14-NONCE`, `KMIPKIT-ELEM-MESSAGE-FIELD-9-14-NONCE-ID`, and `KMIPKIT-ELEM-MESSAGE-FIELD-9-14-NONCE-VALUE`; and `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-CREDENTIAL-CREDENTIAL-TYPE` and `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-CREDENTIAL-CREDENTIAL-VALUE`. Variant member IDs are listed in [data-model.md](data-model.md). Table 442's IDs are `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-USERNAME-AND-PASSWORD-00000001`, `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-DEVICE-00000002`, `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-ATTESTATION-00000003`, `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-ONE-TIME-PASSWORD-00000004`, `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-HASHED-PASSWORD-00000005`, `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-TICKET-00000006`, and `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-EXTENSIONS-8XXXXXXX`. KMIPKIT-0002 T045–T047 corrected the catalog summaries and element links in the accepted release. The remaining lowercase-keyword question is open as `KMIPKIT-DISC-041`; do not change catalog inputs or generated output from this feature.

## Clarification record

The approved roadmap, architecture, pinned OASIS source, and upstream feature handoffs settle the non-ambiguous boundaries without a new product choice:

- The protocol model owns typed structures and structural validation. The common message model remains owned by KMIPKIT-0006; request execution and the point where Authentication is selected for a request remain owned by KMIPKIT-0007.
- Architecture discusses client-default KMIP credentials and request/batch replacement, but this feature does not implement Authentication selection. One Time Password and Ticket values can be modeled for request-scoped use. The explicit KMIPKIT-0007 handoff must settle inherited defaults, request/batch replacement, omission, precedence, and one Request Header Authentication applying to the whole batch (OD-005). This affects future execution integration only; the standalone in-memory Credential and Authentication models do not require an execution API.
- Credential data, OTPs, tickets, and attestation values are sensitive. Secret Debug/Display/error output is redacted, and KMIPKit-owned secret memory is zeroized. Runtime copies retained by Java/Python are documented later with bindings.
- This library transports cryptographic material and is not a local cryptographic algorithm provider. Hashed Password bytes and attestation evidence are caller supplied. No local hashing, evidence generation, attestation verification, or Nonce generation is included.
- The Attestation Capable Indicator reports the library's ability to construct the protocol Attestation Credential structure from caller-provided fields. It is not a claim of evidence generation or server acceptance. If no indicator is encoded, the effective protocol value is false.
- Unknown Credential Type, Hashing Algorithm, Attestation Type, and vendor values must remain representable. Unknown Extensions Credential content remains generic TTLV; no vendor schema is invented.
- Authentication profile applicability and any support claim are owned by KMIPKIT-0010. Base-spec model support does not imply authentication success or profile conformance.

### Decision dispositions and implementation gates

| ID | Status | Question or disposition | Current treatment |
|---|---|---|---|
| OD-001 | Open — source classification and owner sign-off | The pinned §9.4 multiple-Credential sentence uses lowercase “must,” while §1.2 defines uppercase RFC 2119 keywords. KMIPKIT-0002 T045–T047 corrected the catalog representation: the candidate is server-only and unassigned, the §9.11-001 summary separates identification from profile-indicated authentication, and reciprocal element links were reviewed. The pinned lowercase-keyword force remains open as `KMIPKIT-DISC-041`; the catalog evidence records independent source review and parent QA, but no explicit catalog-owner sign-off. | Do not test or enforce “all Credentials satisfied” as a client duty. Preserve `KMIPKIT-DISC-041` and the current catalog disposition; do not reinterpret the lowercase word in this feature. This feature does not edit catalog inputs or generated output. The corrected records and evidence are in `specification/catalog/kmip-2.1.json`, `specification/catalog/coverage-report.md`, and `specification/catalog/review-evidence.md`. |
| OD-002 | Resolved by §9.11 and Table 412 | The source requires the client to provide at least one field after defining the Device Credential Value structure in Table 412. This specification treats any of its six members as a qualifying present field; it does not infer a non-empty string rule. The separate uniqueness sentence identifies four fields and is not used to narrow the minimum-presence rule. | Reject a Device Credential Value with no Table 412 members; preserve all six members and accept presence independently of text length. Do not claim or implement local/global uniqueness enforcement. |
| OD-003 | Open | What owns the cross-request Hashed Password Timestamp rule, its comparison scope, and clock behavior? | Require and preserve caller-supplied Timestamp and hashed bytes; expose SHA-256 as the effective Hashing Algorithm default when omitted while preserving field absence. Do not calculate hashes or implement/claim monotonicity checking or tests until owner, comparison scope, and clock behavior are reviewed. |
| OD-004 | Resolved by scope | Does KMIPKIT-0008 need a Credential writer or secret-bearing send path? | No. Credential and Authentication values remain in-memory; the only execute-path change is the non-secret Attestation Capable Indicator in the existing Request Header builder. This feature adds no Credential writer, Authentication selection, or secret-bearing send path. Any future send path requires its own approved feature and candidate-callsite/owner-through-transport lifecycle evidence. |
| OD-005 | Open — execution integration only | How must KMIPKIT-0007 represent inherited defaults, request/batch replacement, explicit omission, precedence, and one Request Header Authentication applying to the entire batch? | Settle these behaviors through an explicit KMIPKIT-0007 handoff before execution integration. This does not require an execution API for the standalone in-memory Credential or Authentication models. |
| OD-006 | Resolved — informative source text | Does the lowercase OTP “may” sentence create a normative library-wide single-use/replay requirement? | No. The catalog classifies `KMIPKIT-CLAUSE-SPEC-9.11-008` as `informative_context` and assigns no requirement ID. Do not add library-wide replay/single-use state or claim normative client enforcement. Request-scoped use follows architecture; execution selection remains under OD-005. This is distinct from KMIPKIT-0007 OD-006, which concerns secret-send lifecycle-test ownership. |

OD-001, OD-003, and OD-005 remain open with the limits recorded above. They do not authorize unreviewed catalog reclassification, a cross-request monotonicity claim, or Authentication execution integration. They do not expand this feature beyond its in-memory credential scope. OD-002's bounded minimum-presence interpretation is resolved above. The feature remains Draft until T004 and independent review of the exact specification/checklist are complete.

## User scenarios and acceptance

### User Story 1 — Build a typed Authentication value (Priority: P1)

A Rust caller constructs an Authentication value from one or more known Credential variants, or keeps a credential value opaque when the type is an extension. The client can validate structure without deciding whether a server or profile will accept it.

**Independent Test**: Table-driven model tests build and convert each of the seven Table 442 variants, preserve repeated Credential order, reject an empty Authentication, and round-trip unknown Credential Type values without exposing any raw value in diagnostics.

**Acceptance Scenarios**:

1. **Given** no Authentication value, **when** a typed message is built, **then** Authentication remains absent and is not replaced with an empty structure.
2. **Given** an Authentication value, **when** it is validated, **then** it contains at least one Credential and preserves repeated Credentials and their order.
3. **Given** a known Credential Type, **when** known members are converted, **then** their OASIS-defined types and canonical order are validated; unrecognized children from an existing generic TTLV tree remain preserved and accessible without typed interpretation.
4. **Given** an unknown Credential Type or opaque Extensions value, **when** it is decoded and encoded through generic TTLV, **then** the raw enum and subtree are preserved exactly without invented schema.
5. **Given** the base protocol's “all credentials must be satisfied” text, **when** a client model validates Authentication, **then** it checks only the non-empty structure; server-side satisfaction is not claimed as a client check.

### User Story 2 — Construct Device, password, OTP, Ticket, and Attestation credentials (Priority: P1)

A caller supplies credential data explicitly, including secret values and server-provided attestation Nonce data. KMIPKit validates representable structure while leaving external identity, authentication, hashing, and evidence generation to their owners.

**Independent Test**: Derived tests cover each table's required and optional fields, malformed types, device-field ambiguity gates, password algorithm absence/default behavior, timestamp preservation, nonce byte preservation, both attestation-data fields, and redacted secret diagnostics. KMIPKIT-0008 remains in-memory and adds no production credential send path; a future send path requires a separate approved feature and candidate-callsite/owner-through-transport lifecycle test.

**Acceptance Scenarios**:

1. **Given** Username and Password or One Time Password Credential data, **when** the value is built, **then** required usernames and OTPs are checked and optional Password fields remain distinguishable from absence.
2. **Given** Hashed Password Credential data, **when** the value is built, **then** the caller-provided hashed bytes and Date Time Extended Timestamp are preserved, omitted Hashing Algorithm has effective SHA-256 semantics without forcing that optional field onto the wire, and the library performs no cryptographic hashing.
3. **Given** a server Nonce and attestation data, **when** an Attestation Credential is built, **then** Nonce ID and value are unchanged, Attestation Type is present, and at least one of Measurement or Assertion is present; both are preserved if supplied.
4. **Given** a Ticket or Extensions value, **when** converted, **then** its nested structure or opaque generic TTLV is preserved without interpreting opaque vendor fields.
5. **Given** any secret-bearing value, **when** debug formatting, display formatting, validation, or errors are observed, **then** the secret bytes do not appear; owned secret memory follows the approved zeroization model.
6. **Given** any secret-bearing in-memory value, **when** an application uses it through this feature, **then** this feature has no production send path that can serialize or transmit it; any later send path is owned by a separate approved feature and lifecycle test.

### User Story 3 — Report attestation construction capability truthfully (Priority: P1)

A client header advertises whether its supported public credential model can construct a structurally valid Attestation Credential using caller-provided Nonce and evidence fields.

**Independent Test**: A deterministic `kmipkit-client` fake-transport test captures the existing outbound Request Header and verifies the indicator is true when the released Rust API can construct a structurally valid Attestation Credential. It also verifies Authentication remains absent and no Credential payload is emitted. Existing protocol-view tests retain coverage for an externally supplied omitted indicator and its effective false value.

**Acceptance Scenarios**:

1. **Given** the released typed Rust API can construct a valid Attestation Credential structure, **when** `Client::execute` builds a request, **then** its existing private builder emits the indicator as true, without claiming that KMIPKit generated or verified its evidence.
2. **Given** an externally supplied Request Header omits the indicator, **when** the message is inspected, **then** its effective value is false and its field absence remains preserved. The client does not expose an override that contradicts its shipped credential capability.
3. **Given** an operation requires attestation while the indicator is false or absent, **when** the request is handled by a server, **then** the client's conformance claim is limited to reporting its indicator; §9.3's failure response is a server duty and is not tested as a client action here.

## Requirements

### Functional Requirements

- **FR-001**: The model MUST represent optional Authentication and MUST reject a present Authentication containing zero Credentials. Source: `KMIPKIT-REQ-SPEC-9.4-001-001` and `KMIPKIT-REQ-SPEC-9.4-001-002`, §9.4 and Table 403.
- **FR-002**: The model MUST preserve one or more repeatable Credential structures in source order. Source: `KMIPKIT-REQ-SPEC-9.4-002`, §9.4 and Table 403.
- **FR-003**: The model MUST provide typed views for all seven Credential Type values and Table 410's required Credential Type and Credential Value members. It MUST preserve raw unknown enum values and opaque value trees. Source: §9.11, Tables 410–416 and §11.11, Table 442; KMIPKit lossless-model policy.
- **FR-004**: Known credential variants MUST validate required members, encoding types, and OASIS field order using Tables 411–416. Unknown values MUST remain available through generic TTLV without being misinterpreted as a known variant.
- **FR-005**: The model MUST represent all six Table 412 Device fields and MUST reject a Device Credential Value with none of those fields present. Presence is distinct from text content: this requirement does not impose non-empty strings. The model MUST NOT claim to verify local or global uniqueness. Source: `KMIPKIT-REQ-SPEC-9.11-004-001`, `KMIPKIT-REQ-SPEC-9.11-004-002`, and `KMIPKIT-REQ-SPEC-9.11-004-003`, §9.11 and Table 412.
- **FR-006**: Attestation Credential MUST preserve required Nonce and Attestation Type, and require at least one of Measurement or Assertion while preserving both if provided. Nonce ID and Nonce Value MUST be caller-sourced server values preserved exactly. Source: `KMIPKIT-REQ-SPEC-9.11-006`, §§9.11 and 9.14, Tables 413 and 419.
- **FR-007**: Hashed Password Credential MUST preserve required Username, caller-supplied Timestamp, and Hashed Password bytes; support optional Hashing Algorithm; expose SHA-256 as the effective default when omitted; and perform no hashing. Do not implement or claim monotonicity checking or tests until OD-003 review settles the owner, comparison scope, and clock behavior. Source: `KMIPKIT-REQ-SPEC-9.11-010-001` and `KMIPKIT-REQ-SPEC-9.11-010-002`, §9.11 and Table 415.
- **FR-008**: When the shipped Rust credential API can create a structurally valid Attestation Credential from caller input, the existing `Client::execute` request builder MUST emit Attestation Capable Indicator=True. The generic message view MUST continue to preserve an omitted field and report its effective protocol default as False. The client MUST NOT expose an override that contradicts its capability, and the indicator MUST NOT imply evidence generation or verification. This change MUST reuse the existing single writer and permit path and MUST NOT add Authentication or a Credential payload. Source: `KMIPKIT-REQ-SPEC-9.3-001-001` and `KMIPKIT-REQ-SPEC-9.3-001-002`, §9.3 and Table 402.
- **FR-009**: Authentication mechanism selection and per-profile applicability MUST remain outside this feature; the model MUST NOT claim profile support. Source: §9.11 and KMIP Profile references under §9.4; KMIPKIT-0010 owns profile-specific decisions.
- **FR-010**: Secret-bearing data MUST be redacted from Debug, Display, errors, and logs; KMIPKit-owned secret memory MUST be zeroized under the approved ownership contract. This feature MUST NOT add a production credential send path; any later path requires a separate feature and lifecycle test.
- **FR-011**: All variant conversion MUST preserve unknown fields and values in generic TTLV where structurally allowed, including unknown credential, algorithm, and attestation enum values.
- **FR-012**: This feature MUST NOT implement hash functions, attestation evidence generation or verification, Nonce generation, transport, automatic retries, persistence, or a server.
- **FR-013**: Execution integration MUST not infer Authentication selection behavior without an accepted KMIPKIT-0007 handoff that settles inherited defaults, request/batch replacement, explicit omission, precedence, and one Request Header Authentication applying to the whole batch. OASIS §§8.1–8.3 (Tables 394–396) define one Request Header per Request Message, place optional Authentication in that header, and allow repeated Request Batch Items; shared application across the message batch is the structural implication of that layout. OASIS does not define KMIPKit's default/replacement/omission/precedence policy. OD-005 does not gate or require an execution API for the standalone in-memory Credential and Authentication models.
- **FR-014**: Normative traceability MUST map every in-scope OASIS requirement to exact source clause/catalog IDs, planned implementation locations, and executable derived tests. Project policies MUST be traced separately to their canonical project decision/source and tests, without attributing them to OASIS. Server-only duties, unresolved source/catalog classification, deferred profile work, and transmission gates MUST be explicitly listed and excluded from unsupported client conformance claims.

### Traceability disposition

| Requirement / policy | Source | Planned verification | Status |
|---|---|---|---|
| `KMIPKIT-REQ-SPEC-9.3-001-001`, `KMIPKIT-REQ-SPEC-9.3-001-002` | Specification §9.3, Table 402 | fake-transport capture of True from the existing `Client::execute` builder; parsed omitted indicator remains absent/effective False | In scope |
| `KMIPKIT-REQ-SPEC-9.4-001-001`, `KMIPKIT-REQ-SPEC-9.4-001-002`, `KMIPKIT-REQ-SPEC-9.4-002` | §9.4, Table 403 | absent/present/empty and repeated-order model tests | In scope |
| `KMIPKIT-REQ-SPEC-9.4-001-003` | §9.4 authentication prose and §1.2 keyword definitions | no client verification/enforcement test; catalog trace points to `role=server`, `scope_state=server_only`, `status=unassigned` | Server authentication-process duty; lowercase-keyword classification remains open as `KMIPKIT-DISC-041`; no explicit catalog-owner sign-off is recorded |
| `KMIPKIT-REQ-SPEC-9.11-001` | §9.11 prose; profile reference | model identification use; authentication applicability deferred to 0010 | Catalog summary and reciprocal Credential link confirmed by KMIPKIT-0002 T046; no profile claim |
| `KMIPKIT-REQ-SPEC-9.11-004-001`, `KMIPKIT-REQ-SPEC-9.11-004-002`, `KMIPKIT-REQ-SPEC-9.11-004-003` | §9.11 prose, Table 412 | require at least one present Table 412 member; preserve all six fields and their types; do not impose text non-emptiness or claim uniqueness | In scope; uniqueness is not established from client-local data |
| `KMIPKIT-REQ-SPEC-9.11-006` | §9.11 prose, Table 413 | absent/both measurement and assertion plus required fields | In scope |
| `KMIPKIT-REQ-SPEC-9.11-010-001`, `KMIPKIT-REQ-SPEC-9.11-010-002` | §9.11 prose, Table 415 | caller Timestamp and hashed-byte preservation; omitted and explicit algorithm | No hash calculation or monotonicity check/test until OD-003 review |
| `KMIPKIT-ELEM-MESSAGE-FIELD-9-14-NONCE-ID`, `KMIPKIT-ELEM-MESSAGE-FIELD-9-14-NONCE-VALUE` | §9.14, Table 419; §9.11, Table 413 | exact byte preservation of server-provided Nonce | In scope |
| Secret redaction and zeroization | `AGENTS.md` §8; constitution IV; accepted project policy | sentinel checks for every secret wrapper and safe drop/lifecycle test owned by the applicable callsite feature | This feature is in-memory only and introduces no production send path |
| Unknown values and Extensions | `AGENTS.md` §9; generic TTLV contract | property/round-trip tests over raw enum and opaque subtree | In scope |
| Authentication message scope and selection policy | OASIS §§8.1–8.3, Tables 394–396 (single message header with optional Authentication and repeatable batch items; whole-batch scope is a structural implication); KMIPKIT project policy | keep the value standalone in memory; future execution contract tests must specify selection at the accepted 0007 boundary | Execution integration only is gated by OD-005; in-memory models remain standalone |

## Edge cases

- Authentication absent versus present with no entries.
- Multiple credentials of the same or different types; preserve order and do not enforce the server's “ALL satisfied” duty.
- Unknown standard/vendor Credential Type, Hashing Algorithm, or Attestation Type; opaque Extensions payload.
- Device Credential containing any single Table 412 member, none, or combinations; the empty structure is rejected, and field presence is distinct from text length.
- Attestation with neither, either, or both data fields; missing Nonce ID/value or Attestation Type; empty versus non-empty byte values according to table-level structural rules.
- Hashed Password algorithm absent, explicitly SHA-256, a different known value, or unknown raw value; caller Timestamp/hash-byte preservation. Do not add monotonicity checking or tests until OD-003 review settles owner, scope, and clock behavior.
- Secret sentinel placed in every variant to verify formatting, validation and errors remain redacted.
- Verify that no approval state enables a credential send through this feature: it adds no credential writer or production send path.

## Key entities

- **Credential**: A Credential Type and a type-specific or opaque Credential Value.
- **Authentication**: An optional ordered non-empty list of Credential values.
- **Credential Value**: A known Username and Password, Device, Attestation, One Time Password, Hashed Password, Ticket, or opaque Extensions payload.
- **Secret Value**: Caller-provided sensitive text or bytes with redacted formatting and explicit KMIPKit-owned zeroization behavior.
- **Nonce**: Server-assigned Nonce ID and server-created Nonce Value, preserved for use in attestation.
- **Attestation Capability**: A derived boolean about the API's ability to construct the protocol structure, not an evidence-generation claim.
- **Deferred Authentication selection**: Inherited client defaults and request/batch replacement are not models implemented by KMIPKIT-0008. Their exact request-boundary representation is deferred under OD-005 to a future execution-integration feature.

## Success criteria

- **SC-001**: A test matrix covers all seven assigned Credential Type values, all table-required fields, optional fields, field order, wrong field type, and invalid missing-field cases; every case cites its exact OASIS clause/table.
- **SC-002**: Deterministic round-trip tests preserve repeated Credential order, unknown raw enum values, unknown allowed fields, and opaque Extensions trees exactly.
- **SC-003**: Secret sentinel tests show zero secret bytes in Debug, Display, validation errors, and logs for every secret-bearing variant; memory-lifecycle tests meet the accepted ownership/zeroization contract.
- **SC-004**: A fake-transport capture proves the existing `Client::execute` header emits True when the public Rust API constructs Attestation Credential, while Authentication and Credential payload remain absent. Parsed omitted indicators remain absent with effective False; Nonce bytes are preserved without local generation.
- **SC-005**: Requirement traceability is 100% for all applicable in-scope requirements; server duties, unavailable official vectors, source/catalog inconsistencies, and open decisions have explicit dispositions.
- **SC-006**: KMIPKIT-0008 remains in-memory for its scope and introduces no production path that transmits secret-bearing credential bytes. A separate approved feature must own any future send path and its candidate-callsite/owner-through-transport lifecycle evidence.

## Assumptions and exclusions

- KMIP version is 2.1 and wire encoding is TTLV only for 1.0.
- C, Java, and Python parity and generated public bindings are completed under the public API/binding specifications; this feature first stabilizes the Rust model contract.
- OASIS test fixtures are unavailable locally; derived tests are not official conformance vectors.
- One OASIS Request Header contains the optional Authentication value for a Request Message that may contain repeated Batch Items; treating the header Authentication as message-wide follows the structure. KMIPKit default and request/batch override behavior remains project policy and awaits a 0007 handoff under OD-005.
- The feature does not add cryptographic algorithms, credential refresh, secrets persistence, async APIs, JSON/XML, server-initiated operations, or any server-side authentication behavior.

## Dependency gates

Before implementation begins, reviewers must verify the exact accepted revisions of KMIPKIT-0005 (wire codec and separately gated secret lifecycle), KMIPKIT-0006 (message model and Authentication field location), and KMIPKIT-0007 (request selection and execution integration). OD-001's catalog corrections are present in the accepted release and their remaining source-classification question is tracked as `KMIPKIT-DISC-041`; OD-002 resolves the minimum Device field presence; OD-003 limits the timestamp claim; OD-005 blocks future Authentication selection, not these in-memory models or the non-secret indicator. OD-004 is resolved by scope and OD-006 by the existing informative catalog classification. Keep KMIPKIT-0008 OD-006 distinct from KMIPKIT-0007 OD-006, which concerns ownership of a future secret-bearing request lifecycle test under ADR-0012. The OTP wording is `KMIPKIT-CLAUSE-SPEC-9.11-008`. The exact final specification revision and updated checklist must receive independent review and delegated maintainer authorization before coding. Profile-specific decisions remain with KMIPKIT-0010. The feature adds no credential send path; it only reuses the existing request writer for the capability indicator.
