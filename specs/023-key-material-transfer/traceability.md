# Traceability: KMIP 2.1 Key Derivation and Transfer

This design-time map links all 18 stable client requirement IDs and each request, response, and conditional Derivation Parameters row to implementation tasks and verification. Exact source sections and table numbers are in [spec.md](spec.md). Implementation must replace task links with code/test references and regenerate the normative catalog with the pinned tool. A task mapping is not evidence that implementation or conformance tests have passed.

## Stable normative requirement IDs

| Requirement ID | Source and obligation/capability | Implementation and verification |
|---|---|---|
| KMIPKIT-REQ-SPEC-6.1.14-001-001 | OASIS KMIP 2.1 §6.1.14: source object Cryptographic Usage Mask includes Derive Key. | T006–T008: source-use validation and negative cases from caller-known request data; no remote lookup. |
| KMIPKIT-REQ-SPEC-6.1.14-001-002 | §6.1.14: specify Cryptographic Length for each derived key or Secret Data object. | T006–T008: required output length cases. |
| KMIPKIT-REQ-SPEC-6.1.14-001-003 | §6.1.14: when deriving a key, specify Cryptographic Length and Cryptographic Algorithm. | T006–T008: symmetric-key missing-length/algorithm cases. |
| KMIPKIT-REQ-SPEC-6.1.14-001-004 | §6.1.14: MAY derive multiple keys or IVs in Secret Data by specifying total length. | T006–T008: typed capability and round-trip/encoding case. |
| KMIPKIT-REQ-SPEC-6.1.14-002 | §6.1.14: MAY identify derivation inputs, method, and parameters. | T006–T008: typed capability and Table 205 encoding case. |
| KMIPKIT-REQ-SPEC-6.1.14-006-001 | §6.1.14: MAY repeat source Unique Identifier. | T006–T008: ordered repeated-field encoding/round-trip case. |
| KMIPKIT-REQ-SPEC-6.1.14-006-002 | §6.1.14: SHALL NOT substitute ID Placeholder for Derive Key Unique Identifier. | T006–T008: negative preflight/encoding case; verify no placeholder substitution. |
| KMIPKIT-REQ-SPEC-6.1.14-007-001 | §6.1.14: SHALL specify desired new-object Attributes. | T006–T008: required Attributes case. |
| KMIPKIT-REQ-SPEC-6.1.14-007-002 | §6.1.14: symmetric-key Attributes SHALL include Cryptographic Length and Algorithm. | T006–T008: missing-member negative cases. |
| KMIPKIT-REQ-SPEC-7.12-003 | §7.12: REQUIRED PRF/mode parameters, including HASH algorithm and AES-CBC block mode. | T006–T008: method-dependent Cryptographic Parameters validation and cases. |
| KMIPKIT-REQ-SPEC-7.12-005-001 | §7.12: MAY provide derivation input as Derivation Data or by Secret Data Unique Identifier. | T006–T008: both input forms remain expressible and are transmitted as supplied. |
| KMIPKIT-REQ-SPEC-6.1.25-001 | §6.1.25: MUST supply Import Attribute values rather than rely on server-generated values. | T012–T014: required caller-supplied Attributes cases. |
| KMIPKIT-REQ-SPEC-6.1.25-004 | §6.1.25: SHALL set Replace Existing true when intending replacement for the same Unique Identifier. | T012–T014: explicit replacement true/false/absent cases. |
| KMIPKIT-REQ-SPEC-6.1.25-006 | §6.1.25: MAY send imported object and Attributes wrapped. | T012–T014: wrapped request capability and conditional Key Wrap Type cases. |
| KMIPKIT-REQ-SPEC-6.1.27-001 | §6.1.27: SHALL include at least the referenced Split Key Threshold number of Unique Identifier entries. | T015–T017: caller-known per-occurrence context; count below/equal/above maximum; NotSent on failure. |
| KMIPKIT-REQ-SPEC-6.1.27-002-001 | §6.1.27: MAY specify resulting Object Type. | T015–T017: typed optional field encoding/round-trip. |
| KMIPKIT-REQ-SPEC-6.1.27-002-002 | §6.1.27: MAY specify Secret Data Type for Secret Data result. | T015–T017: typed optional field encoding/round-trip. |
| KMIPKIT-REQ-SPEC-6.1.27-005 | §6.1.27: MAY provide repeated Unique Identifiers for split keys. | T015–T017: duplicate values and repeated occurrences retained in order and counted as transmitted. |

## Source clauses outside the client requirement ledger

These catalog entries appear in the specification but are not additional active client requirement IDs:

| Requirement ID | Source role and disposition | Implementation and verification |
|---|---|---|
| KMIPKIT-REQ-SPEC-7.12-002 | §7.12 sentence-case “May be repeated” extraction, retired under KMIPKIT-DEC-011; repetition remains represented in the Table 364 schema without a separate normative client obligation. | T006–T008 preserve and round-trip repeated Derivation Data occurrences. |
| KMIPKIT-REQ-SPEC-7.12-005-002 | §7.12 server-only SHALL: the server returns an error if both Derivation Data and a Secret Data object Unique Identifier are supplied. KMIPKit sends both caller-provided fields and surfaces the server result; it does not pre-reject locally. | T006–T008 include the both-inputs server-error result case from spec.md acceptance scenario 5. |

These entries do not change the count of 18 stable client requirement IDs.

## Operation payload table rows

Tasks below include Red tests against exact table shape, Green model/dispatch, and a separate Refactor commit. “Success/error” includes success payload decoding and the operation error result table while preserving common open result values and delivery evidence.

| OASIS source row | Row requirement/condition | Implementation and verification |
|---|---|---|
| §6.1.14 Table 205 — Object Type | Required. | T006–T008 |
| §6.1.14 Table 205 — Unique Identifier | Required; may repeat; ID Placeholder must not be substituted. | T006–T008; stable IDs 6.1.14-006-001/-002 |
| §6.1.14 Table 205 — Derivation Method | Required. | T006–T008 |
| §6.1.14 Table 205 — Derivation Parameters | Required structure. | T006–T008; §7.12 Table 364 rows below |
| §6.1.14 Table 205 — Attributes | Required; key length and algorithm conditions apply. | T006–T008; stable IDs 6.1.14-007-001/-002 |
| §6.1.14 Table 206 — Unique Identifier | Required on success. | T006–T008 |
| §6.1.14 Table 207 — operation errors | Preserve server Operation Failed result reasons, including unknown/future values via common result path. | T006–T008 |
| §7.12 Table 364 — Derivation Parameters | Required structure containing derivation-method parameters. | T006–T008 |
| §7.12 Table 364 — Cryptographic Parameters | Conditional on PRF/mode; HASH and AES-CBC specifics per §7.12. | T006–T008; stable ID KMIPKIT-REQ-SPEC-7.12-003 |
| §7.12 Table 364 — Initialization Vector | Conditional on PRF/mode; preserve absence/value. | T006–T008 |
| §7.12 Table 364 — Derivation Data | Required unless Secret Data Unique Identifier supplies input; repeated representation is preserved. | T006–T008; stable ID KMIPKIT-REQ-SPEC-7.12-005-001 |
| §7.12 Table 364 — Salt | Required for PBKDF2. | T006–T008 |
| §7.12 Table 364 — Iteration Count | Required for PBKDF2. | T006–T008 |
| §6.1.18 Table 217 — Unique Identifier | Optional; absence preserves server-side ID Placeholder behavior. | T009–T011 |
| §6.1.18 Table 217 — Key Format Type | Optional explicit caller choice. | T009–T011 |
| §6.1.18 Table 217 — Key Wrap Type | Optional explicit caller choice. | T009–T011 |
| §6.1.18 Table 217 — Key Compression Type | Optional explicit caller choice. | T009–T011 |
| §6.1.18 Table 217 — Key Wrapping Specification | Optional explicit caller choice. | T009–T011 |
| §6.1.18 Table 218 — Object Type | Required on success. | T009–T011 |
| §6.1.18 Table 218 — Unique Identifier | Required on success. | T009–T011 |
| §6.1.18 Table 218 — Attributes | Required on success; sensitive values remain redacted. | T009–T011 |
| §6.1.18 Table 218 — Any Object | Required on success; opaque/sensitive object representation preserved. | T009–T011 |
| §6.1.18 Table 219 — operation errors | Preserve server Operation Failed result reasons via common result path. | T009–T011 |
| §6.1.25 Table 238 — Unique Identifier | Required. | T012–T014 |
| §6.1.25 Table 238 — Object Type | Required. | T012–T014 |
| §6.1.25 Table 238 — Replace Existing | Optional; true requests replacement; absent/false leaves duplicate handling to server. | T012–T014; stable ID KMIPKIT-REQ-SPEC-6.1.25-004 |
| §6.1.25 Table 238 — Key Wrap Type | Conditional if and only if the key object is wrapped; preserve wrapped form. | T012–T014 |
| §6.1.25 Table 238 — Attributes | Required caller-supplied values. | T012–T014; stable ID KMIPKIT-REQ-SPEC-6.1.25-001 |
| §6.1.25 Table 238 — Any Object | Required; wrapped object/Attributes remain as supplied. | T012–T014; stable ID KMIPKIT-REQ-SPEC-6.1.25-006 |
| §6.1.25 Table 239 — Unique Identifier | Required on success. | T012–T014 |
| §6.1.25 Table 240 — operation errors | Preserve server Operation Failed result reasons via common result path. | T012–T014 |
| §6.1.27 Table 244 — Object Type | Required. | T015–T017 |
| §6.1.27 Table 244 — Unique Identifier | Required; may repeat; entry count and exact order/repetition are preserved. | T015–T017; stable IDs KMIPKIT-REQ-SPEC-6.1.27-001/-005 |
| §6.1.27 Table 244 — Secret Data Type | Optional. | T015–T017; stable ID KMIPKIT-REQ-SPEC-6.1.27-002-002 |
| §6.1.27 Table 244 — Attributes | Optional. | T015–T017 |
| §6.1.27 Table 244 — Protection Storage Masks | Optional; preserve all caller-selected permissible masks. | T015–T017 |
| §6.1.27 Table 245 — Unique Identifier | Required on success. | T015–T017 |
| §6.1.27 Table 246 — operation errors | Preserve server Operation Failed result reasons via common result path. | T015–T017 |

## Shared behavior and evidence

| Contract | Source | Tasks |
|---|---|---|
| One explicit exchange, no retry, Pending/correlation and per-item delivery state | Approved common batch/client/result contract; operation acceptance scenarios in spec.md | T004–T005, T006–T017 |
| Unknown tags/enums/bitmasks/extensions and ordered repeated fields remain lossless | Constitution; OASIS tables cited above | T004–T017 |
| Sensitive values and raw bodies are redacted; KMIPKit-owned secrets are zeroized | Constitution security invariants | T004–T005, T006–T017 |
| Rust/C/Java/Python public capability uses canonical manifest; no handwritten generated declarations | KMIPKIT-0043 tasks T046, T047, T050, T052 | T018 |
| Catalog and generated coverage report updated by pinned generators | Constitution generated-artifact rules | T020–T021 |

No official fixture pass, profile support, interoperability result, certification, or implementation completion is claimed by this design-time mapping.
