# Feature Specification: KMIP 2.1 Credentials and Attestation

**Feature Branch**: `feature/KMIPKIT-0008-credentials-attestation`<br>
**Created**: 2026-10-05<br>
**Status**: Draft — implementation remains gated on approval of this exact specification and its reviewer checklist, accepted KMIPKIT-0005/0006 foundations, the KMIPKIT-0007 request/authentication handoff, the secret-bearing wire boundary and lifecycle decision, and the open decisions below.<br>
**Input**: Roadmap item `KMIPKIT-0008-credentials-attestation`.

## Scope and normative sources

This feature defines lossless typed Rust models and validation for KMIP 2.1 `Credential`, `Authentication`, `Nonce`, and the request-header `Attestation Capable Indicator`. It covers the seven Credential Type values in OASIS Table 442, the field schemas in Tables 410–416, the authentication container in Table 403, and the server-provided Nonce in Table 419. Known protocol values receive typed views; unknown enum values and extension payloads remain representable without loss.

This is a client-side model and construction specification. It does not decide which authentication mechanism a server requires, claim conformance to a KMIP Profile, verify device identity or global uniqueness, generate a password hash, create or verify attestation evidence, or generate a Nonce. Profiles and mechanisms remain assigned to KMIPKIT-0010. It does not add a server, transport, retry, credential persistence, secret-bearing transmission path, or language bindings. This feature never adds or enables a production path that sends credentials. A separate, later client feature must own any approved secret-bearing send path and its candidate-callsite/owner-through-transport lifecycle test; approvals or evidence from this feature or KMIPKIT-0007 cannot substitute for that gate.

The normative authority is the immutable pinned OASIS KMIP Specification v2.1 at `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`. Exact citations below identify the clauses and structural tables. The pinned catalog currently links no official Test Case IDs to these requirements, and the 203 XML fixture references recorded in `KMIPKIT-DISC-036` are unavailable locally; acceptance tests are therefore derived tests, not claimed official vectors.

| Stable requirement / source | Treatment in this feature |
|---|---|
| `KMIPKIT-REQ-SPEC-9.3-001-001`, `KMIPKIT-REQ-SPEC-9.3-001-002`; Specification §9.3, Table 402 | The request indicator reflects whether KMIPKit can construct a structurally valid Attestation Credential. Its omitted protocol value is false. The model does not claim that KMIPKit creates the caller's measurement/assertion or can satisfy any server's attestation policy. |
| `KMIPKIT-REQ-SPEC-9.4-001-001`, `KMIPKIT-REQ-SPEC-9.4-001-002`, `KMIPKIT-REQ-SPEC-9.4-002`; §9.4, Table 403 | Authentication is optional; when present it contains one or more Credentials; Credential is repeatable and order-preserving. An empty Authentication is rejected. |
| `KMIPKIT-REQ-SPEC-9.4-001-003`; §9.4 | The source says multiple Credentials “must ALL be satisfied,” a server authentication-process duty. The catalog currently misclassifies it as a client requirement. This feature does not claim client verification or enforcement of server authentication; catalog correction is tracked as an open inventory item and no client conformance test is assigned to this server duty. |
| `KMIPKIT-REQ-SPEC-9.11-001`; §9.11 | The source describes Credential as used for client identification and says it MAY be used for authentication as indicated by KMIP Profiles. The catalog summary incorrectly makes both uses profile-conditional. Correct interpretation and applicability are recorded here; authentication mechanisms and profile claims remain with KMIPKIT-0010. |
| `KMIPKIT-REQ-SPEC-9.11-004-001`, `KMIPKIT-REQ-SPEC-9.11-004-002`, `KMIPKIT-REQ-SPEC-9.11-004-003`; §9.11, Table 412 | Preserve Device fields and an optional shared secret/password. The source's “at least one field” has an unresolved field-set ambiguity, and uniqueness cannot be established from client-local data; see OD-002. No global uniqueness check is specified. |
| `KMIPKIT-REQ-SPEC-9.11-006`; §9.11, Table 413 | Attestation Credential requires Nonce and Attestation Type and supplies at least one of Attestation Measurement or Attestation Assertion. The source does not explicitly prohibit supplying both; this specification allows both. |
| `KMIPKIT-REQ-SPEC-9.11-010-001`, `KMIPKIT-REQ-SPEC-9.11-010-002`; §9.11, Table 415 | Preserve a caller-supplied hashed-password byte string and Timestamp; the source requires the Timestamp to monotonically increase and defaults Hashing Algorithm to SHA-256. State/clock ownership for monotonicity is unresolved (OD-003). The optional algorithm field remains omitted when absent; its effective default is SHA-256. KMIPKit does not calculate the hash. |
| §9.11, Tables 410–416; §11.11, Table 442 | Validate typed shapes and field order for Username and Password, Device, Attestation, One Time Password, Hashed Password, Ticket, and Extensions. The Extensions value remains opaque generic TTLV unless another normative schema is established. Preserve unrecognized enum values. |
| §9.14, Table 419 | Nonce ID and Nonce Value are preserved as server-provided bytes. An Attestation Credential consumes the Nonce object returned by the server; the client does not generate or alter it. |

Catalog element references include `KMIPKIT-ELEM-MESSAGE-FIELD-9-3-ATTESTATION-CAPABLE-INDICATOR`; `KMIPKIT-ELEM-MESSAGE-FIELD-9-14-NONCE`, `KMIPKIT-ELEM-MESSAGE-FIELD-9-14-NONCE-ID`, and `KMIPKIT-ELEM-MESSAGE-FIELD-9-14-NONCE-VALUE`; and `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-CREDENTIAL-CREDENTIAL-TYPE` and `KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-CREDENTIAL-CREDENTIAL-VALUE`. Variant member IDs are listed in [data-model.md](data-model.md). Table 442's IDs are `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-USERNAME-AND-PASSWORD-00000001`, `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-DEVICE-00000002`, `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-ATTESTATION-00000003`, `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-ONE-TIME-PASSWORD-00000004`, `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-HASHED-PASSWORD-00000005`, `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-TICKET-00000006`, and `KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-EXTENSIONS-8XXXXXXX`. The generated catalog's missing per-requirement element links and its two inaccurate requirement summaries are recorded as inventory corrections to address through the catalog workflow, not by editing generated output.

## Clarification record

The approved roadmap, architecture, pinned OASIS source, and upstream feature handoffs settle the non-ambiguous boundaries without a new product choice:

- The protocol model owns typed structures and structural validation. The common message model remains owned by KMIPKIT-0006; request execution and the point where Authentication is selected for a request remain owned by KMIPKIT-0007.
- Architecture permits client-default KMIP credentials and replacement at request or batch scope. One Time Password and Ticket values can be scoped to one request. Exact precedence and representation at the KMIPKIT-0007 call boundary must be agreed before implementation (OD-005); do not silently alter 0007's approved contract.
- Credential data, OTPs, tickets, and attestation values are sensitive. Secret Debug/Display/error output is redacted, and KMIPKit-owned secret memory is zeroized. Runtime copies retained by Java/Python are documented later with bindings.
- This library transports cryptographic material and is not a local cryptographic algorithm provider. Hashed Password bytes and attestation evidence are caller supplied. No local hashing, evidence generation, attestation verification, or Nonce generation is included.
- The Attestation Capable Indicator reports the library's ability to construct the protocol Attestation Credential structure from caller-provided fields. It is not a claim of evidence generation or server acceptance. If no indicator is encoded, the effective protocol value is false.
- Unknown Credential Type, Hashing Algorithm, Attestation Type, and vendor values must remain representable. Unknown Extensions Credential content remains generic TTLV; no vendor schema is invented.
- Authentication profile applicability and any support claim are owned by KMIPKIT-0010. Base-spec model support does not imply authentication success or profile conformance.

### Open decisions and implementation gates

| ID | Question to resolve before implementation | Current safe treatment |
|---|---|---|
| OD-001 | Catalog records `KMIPKIT-REQ-SPEC-9.4-001-003` as client-to-server and summarizes §9.11 `KMIPKIT-REQ-SPEC-9.11-001` inaccurately. What catalog fields and review evidence should correct those classifications and element links? | Record the source-correct interpretation in this draft; do not test the server duty as a client requirement. Correct only the normative catalog input and regenerate its report through the pinned tool. |
| OD-002 | In §9.11, “The client SHALL provide at least one field” follows the four named unique device identifiers; Table 412 also has Device Identifier and Password, all optional. Which fields satisfy this rule? The uniqueness scope/data is also not locally observable. | Preserve each field. Do not count a particular field toward the at-least-one validation or claim a uniqueness check until the catalog/source disposition is reviewed. |
| OD-003 | What owns the monotonically increasing Hashed Password Timestamp across requests, and what comparison scope/clock behavior is required? | Require an explicit caller-supplied Timestamp and preserve it exactly. No implicit clock, persistent state, comparison, or hash operation is designed until a reviewed decision closes this gate. |
| OD-004 | If a future feature introduces secret-bearing encoding and sending, which exact approved boundary and candidate-callsite owner-through-transport lifecycle evidence must it satisfy? | This feature adds no production writer callsite or send path. Any later path must be separately specified, approved, and tested by the feature that introduces its candidate callsite, in line with KMIPKIT-0005/0007 and ADR-0012. |
| OD-005 | How does the request execution API represent inherited client defaults, explicit per-request/batch replacement, and omission of Authentication? | Keep the model independent of a guessed 0007 API. Freeze a contract only after an explicit 0007 handoff is accepted. |
| OD-006 | Is an OTP strictly single-use at the library API boundary, or only request-scoped per architecture? The base text says it may only be used for a single authentication in lowercase, and the catalog currently treats that wording as informative (`KMIPKIT-CLAUSE-SPEC-9.11-008`). | Support request-scoped values as project architecture allows; do not claim a normative client-side single-use enforcement until catalog review resolves the classification. |

These are explicit gates, not `NEEDS CLARIFICATION` placeholders or permission to choose an unsupported interpretation. The feature stays Draft while any item affecting its implementation remains open.

## User scenarios and acceptance

### User Story 1 — Build a typed Authentication value (Priority: P1)

A Rust caller constructs an Authentication value from one or more known Credential variants, or keeps a credential value opaque when the type is an extension. The client can validate structure without deciding whether a server or profile will accept it.

**Independent Test**: Table-driven model tests build and convert each of the seven Table 442 variants, preserve repeated Credential order, reject an empty Authentication, and round-trip unknown Credential Type values without exposing any raw value in diagnostics.

**Acceptance Scenarios**:

1. **Given** no Authentication value, **when** a typed message is built, **then** Authentication remains absent and is not replaced with an empty structure.
2. **Given** an Authentication value, **when** it is validated, **then** it contains at least one Credential and preserves repeated Credentials and their order.
3. **Given** a known Credential Type, **when** its value is converted, **then** only the members and types in its OASIS table are accepted and field order is canonical.
4. **Given** an unknown Credential Type or opaque Extensions value, **when** it is decoded and encoded through generic TTLV, **then** the raw enum and subtree are preserved exactly without invented schema.
5. **Given** the base protocol's “all credentials must be satisfied” text, **when** a client model validates Authentication, **then** it checks only the non-empty structure; server-side satisfaction is not claimed as a client check.

### User Story 2 — Construct Device, password, OTP, Ticket, and Attestation credentials (Priority: P1)

A caller supplies credential data explicitly, including secret values and server-provided attestation Nonce data. KMIPKit validates representable structure while leaving external identity, authentication, hashing, and evidence generation to their owners.

**Independent Test**: Derived tests cover each table's required and optional fields, malformed types, device-field ambiguity gates, password algorithm absence/default behavior, timestamp preservation, nonce byte preservation, both attestation-data fields, and redacted secret diagnostics. This feature adds no production credential send path under any OD-004 disposition; a future send path requires a separate approved feature and candidate-callsite/owner-through-transport lifecycle test.

**Acceptance Scenarios**:

1. **Given** Username and Password or One Time Password Credential data, **when** the value is built, **then** required usernames and OTPs are checked and optional Password fields remain distinguishable from absence.
2. **Given** Hashed Password Credential data, **when** the value is built, **then** the caller-provided hashed bytes and Date Time Extended Timestamp are preserved, omitted Hashing Algorithm has effective SHA-256 semantics without forcing that optional field onto the wire, and the library performs no cryptographic hashing.
3. **Given** a server Nonce and attestation data, **when** an Attestation Credential is built, **then** Nonce ID and value are unchanged, Attestation Type is present, and at least one of Measurement or Assertion is present; both are preserved if supplied.
4. **Given** a Ticket or Extensions value, **when** converted, **then** its nested structure or opaque generic TTLV is preserved without interpreting opaque vendor fields.
5. **Given** any secret-bearing value, **when** debug formatting, display formatting, validation, or errors are observed, **then** the secret bytes do not appear; owned secret memory follows the approved zeroization model.
6. **Given** any state of OD-004, **when** an application attempts to use secret-bearing values, **then** this feature has no production send path that can serialize or transmit them; any later send path is owned by a separate feature and lifecycle test.

### User Story 3 — Report attestation construction capability truthfully (Priority: P1)

A client header advertises whether its supported public credential model can construct a structurally valid Attestation Credential using caller-provided Nonce and evidence fields.

**Independent Test**: Tests derive the indicator from the released credential construction capability and verify true, false, and omitted behavior independently from any server operation or evidence generator.

**Acceptance Scenarios**:

1. **Given** the released typed builder accepts a valid Attestation Credential structure, **when** the client constructs its header, **then** the indicator is true, without claiming that KMIPKit generated or verified its evidence.
2. **Given** the client cannot construct that structure, **when** the header is built, **then** the indicator is false or absent; an absent value has the effective OASIS default false.
3. **Given** an operation requires attestation while the indicator is false or absent, **when** the request is handled by a server, **then** the client's conformance claim is limited to reporting its indicator; §9.3's failure response is a server duty and is not tested as a client action here.

## Requirements

### Functional Requirements

- **FR-001**: The model MUST represent optional Authentication and MUST reject a present Authentication containing zero Credentials. Source: `KMIPKIT-REQ-SPEC-9.4-001-001` and `KMIPKIT-REQ-SPEC-9.4-001-002`, §9.4 and Table 403.
- **FR-002**: The model MUST preserve one or more repeatable Credential structures in source order. Source: `KMIPKIT-REQ-SPEC-9.4-002`, §9.4 and Table 403.
- **FR-003**: The model MUST provide typed views for all seven Credential Type values and Table 410's required Credential Type and Credential Value members. It MUST preserve raw unknown enum values and opaque value trees. Source: §9.11, Tables 410–416 and §11.11, Table 442; KMIPKit lossless-model policy.
- **FR-004**: Known credential variants MUST validate required members, encoding types, and OASIS field order using Tables 411–416. Unknown values MUST remain available through generic TTLV without being misinterpreted as a known variant.
- **FR-005**: The model MUST represent all Table 412 Device fields and optional shared secret/password. The empty-value and minimum-field validator MUST remain gated on OD-002; the model MUST NOT claim to verify global uniqueness. Source: `KMIPKIT-REQ-SPEC-9.11-004-001`, `KMIPKIT-REQ-SPEC-9.11-004-002`, and `KMIPKIT-REQ-SPEC-9.11-004-003`, §9.11 and Table 412.
- **FR-006**: Attestation Credential MUST preserve required Nonce and Attestation Type, and require at least one of Measurement or Assertion while preserving both if provided. Nonce ID and Nonce Value MUST be caller-sourced server values preserved exactly. Source: `KMIPKIT-REQ-SPEC-9.11-006`, §§9.11 and 9.14, Tables 413 and 419.
- **FR-007**: Hashed Password Credential MUST preserve required Username, Timestamp, and Hashed Password bytes; support optional Hashing Algorithm; expose SHA-256 as the effective default when it is omitted; and perform no hashing. Monotonic timestamp enforcement remains gated on OD-003. Source: `KMIPKIT-REQ-SPEC-9.11-010-001` and `KMIPKIT-REQ-SPEC-9.11-010-002`, §9.11 and Table 415.
- **FR-008**: The Attestation Capable Indicator MUST be true exactly when the shipped Rust credential API can create a structurally valid Attestation Credential from caller input, false otherwise, and false by effective default when omitted. It MUST NOT imply generation or verification of attestation evidence. Source: `KMIPKIT-REQ-SPEC-9.3-001-001` and `KMIPKIT-REQ-SPEC-9.3-001-002`, §9.3 and Table 402.
- **FR-009**: Authentication mechanism selection and per-profile applicability MUST remain outside this feature; the model MUST NOT claim profile support. Source: §9.11 and KMIP Profile references under §9.4; KMIPKIT-0010 owns profile-specific decisions.
- **FR-010**: Secret-bearing data MUST be redacted from Debug, Display, errors, and logs; KMIPKit-owned secret memory MUST be zeroized under the approved ownership contract. This feature MUST NOT add a production credential send path; any later path requires a separate feature and lifecycle test.
- **FR-011**: All variant conversion MUST preserve unknown fields and values in generic TTLV where structurally allowed, including unknown credential, algorithm, and attestation enum values.
- **FR-012**: This feature MUST NOT implement hash functions, attestation evidence generation or verification, Nonce generation, transport, automatic retries, persistence, or a server.
- **FR-013**: The request layer MUST not infer default/request override behavior without an accepted KMIPKIT-0007 handoff; architecture permits client defaults and request/batch replacement, while exact API representation remains OD-005.
- **FR-014**: Normative traceability MUST map every in-scope OASIS requirement to exact source clause/catalog IDs, planned implementation locations, and executable derived tests. Project policies MUST be traced separately to their canonical project decision/source and tests, without attributing them to OASIS. Server-only duties, unresolved source/catalog classification, deferred profile work, and transmission gates MUST be explicitly listed and excluded from unsupported client conformance claims.

### Traceability disposition

| Requirement / policy | Source | Planned verification | Status |
|---|---|---|---|
| `KMIPKIT-REQ-SPEC-9.3-001-001`, `KMIPKIT-REQ-SPEC-9.3-001-002` | Specification §9.3, Table 402 | Indicator truth table against builder capability; omitted effective false | In scope |
| `KMIPKIT-REQ-SPEC-9.4-001-001`, `KMIPKIT-REQ-SPEC-9.4-001-002`, `KMIPKIT-REQ-SPEC-9.4-002` | §9.4, Table 403 | absent/present/empty and repeated-order model tests | In scope |
| `KMIPKIT-REQ-SPEC-9.4-001-003` | §9.4 authentication prose | no client verification test; correct catalog role/disposition through reviewed catalog update | Server duty; catalog correction open |
| `KMIPKIT-REQ-SPEC-9.11-001` | §9.11 prose; profile reference | model identification use; authentication applicability deferred to 0010 | In scope with catalog summary correction open |
| `KMIPKIT-REQ-SPEC-9.11-004-001`, `KMIPKIT-REQ-SPEC-9.11-004-002`, `KMIPKIT-REQ-SPEC-9.11-004-003` | §9.11 prose, Table 412 | all individual field shapes, optional secret, and selected minimum-field rule | Minimum-field/uniqueness semantics gated by OD-002 |
| `KMIPKIT-REQ-SPEC-9.11-006` | §9.11 prose, Table 413 | absent/both measurement and assertion plus required fields | In scope |
| `KMIPKIT-REQ-SPEC-9.11-010-001`, `KMIPKIT-REQ-SPEC-9.11-010-002` | §9.11 prose, Table 415 | timestamp and hashed-byte preservation; omitted and explicit algorithm | Monotonic owner gated by OD-003 |
| §9.14 Nonce fields | §9.14, Table 419; 9.11/Table 413 | exact byte preservation of server-provided Nonce | In scope |
| Secret redaction and zeroization | `AGENTS.md` §8; constitution IV; accepted project policy | sentinel checks for every secret wrapper and safe drop/lifecycle test owned by the applicable callsite feature | This feature is in-memory only and introduces no production send path |
| Unknown values and Extensions | `AGENTS.md` §9; generic TTLV contract | property/round-trip tests over raw enum and opaque subtree | In scope |
| Credentials default and replacement | `docs/architecture/transport-security.md` §Credentials | contract test at accepted 0007 boundary | Gated by OD-005 |

## Edge cases

- Authentication absent versus present with no entries.
- Multiple credentials of the same or different types; preserve order and do not enforce the server's “ALL satisfied” duty.
- Unknown standard/vendor Credential Type, Hashing Algorithm, or Attestation Type; opaque Extensions payload.
- Device Credential containing only Device Identifier, only Password, none, or combinations; acceptance of the minimum-field cases waits for OD-002.
- Attestation with neither, either, or both data fields; missing Nonce ID/value or Attestation Type; empty versus non-empty byte values according to table-level structural rules.
- Hashed Password algorithm absent, explicitly SHA-256, a different known value, or unknown raw value; timestamp byte/value preservation and non-monotonic sequence awaiting OD-003.
- Secret sentinel placed in every variant to verify formatting, validation and errors remain redacted.
- Verify that no approval state enables a credential send through this feature: it adds no credential writer or production send path.

## Key entities

- **Credential**: A Credential Type and a type-specific or opaque Credential Value.
- **Authentication**: An optional ordered non-empty list of Credential values.
- **Credential Value**: A known Username and Password, Device, Attestation, One Time Password, Hashed Password, Ticket, or opaque Extensions payload.
- **Secret Value**: Caller-provided sensitive text or bytes with redacted formatting and explicit KMIPKit-owned zeroization behavior.
- **Nonce**: Server-assigned Nonce ID and server-created Nonce Value, preserved for use in attestation.
- **Attestation Capability**: A derived boolean about the API's ability to construct the protocol structure, not an evidence-generation claim.
- **Authentication Selection**: Inherited client default or explicit request/batch replacement; exact request-boundary representation is gated by OD-005.

## Success criteria

- **SC-001**: A test matrix covers all seven assigned Credential Type values, all table-required fields, optional fields, field order, wrong field type, and invalid missing-field cases; every case cites its exact OASIS clause/table.
- **SC-002**: Deterministic round-trip tests preserve repeated Credential order, unknown raw enum values, unknown allowed fields, and opaque Extensions trees exactly.
- **SC-003**: Secret sentinel tests show zero secret bytes in Debug, Display, validation errors, and logs for every secret-bearing variant; memory-lifecycle tests meet the accepted ownership/zeroization contract.
- **SC-004**: The Attestation Capable Indicator passes true/false/omitted cases tied to the exact public builder capability; Nonce bytes are preserved without local generation.
- **SC-005**: Requirement traceability is 100% for all applicable in-scope requirements; server duties, unavailable official vectors, source/catalog inconsistencies, and open decisions have explicit dispositions.
- **SC-006**: This feature introduces no production path that transmits secret-bearing credential bytes under any OD-004 disposition. A separate approved feature must own any future send path and its candidate-callsite/owner-through-transport lifecycle evidence.

## Assumptions and exclusions

- KMIP version is 2.1 and wire encoding is TTLV only for 1.0.
- C, Java, and Python parity and generated public bindings are completed under the public API/binding specifications; this feature first stabilizes the Rust model contract.
- OASIS test fixtures are unavailable locally; derived tests are not official conformance vectors.
- Authentication defaults and request/batch overrides follow the approved architecture only after 0007 accepts an exact interface handoff.
- The feature does not add cryptographic algorithms, credential refresh, secrets persistence, async APIs, JSON/XML, server-initiated operations, or any server-side authentication behavior.

## Dependency gates

Before implementation begins, reviewers must verify the exact revisions and acceptance state of KMIPKIT-0005 (wire codec and secret-bearing writer boundary), KMIPKIT-0006 (message model and Authentication field location), and KMIPKIT-0007 (request selection and execution integration). Resolve this specification's OD-001 through OD-006 separately from KMIPKIT-0007 OD-006, which concerns ownership of a future secret-bearing request lifecycle test under ADR-0012. The OTP wording in this specification's OD-006 is cataloged at `KMIPKIT-CLAUSE-SPEC-9.11-008`. The exact final specification revision and updated checklist must receive independent review and human approval after all catalog and interface changes. All six decisions in this specification must be resolved or explicitly dispositioned without weakening a normative `MUST`; profile-specific decisions remain with KMIPKIT-0010. This feature remains in-memory only and must never add a production credential send path.
