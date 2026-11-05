# KMIPKIT-0049 Normative Traceability

**Source**: `KMIPKIT-SRC-spec`, the pinned OASIS KMIP Specification v2.1 in
`specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`; catalog records
are in `specification/catalog/kmip-2.1.json`.

The table below lists the operative stable catalog requirement IDs in §§2.1–
3.12. `Test path` identifies the planned executable check before implementation;
the implementation-evidence record will replace planned references with exact
test names and code locations after Green.

| OASIS requirement ID | Section | KMIPKIT requirement | Planned implementation and verification |
|---|---:|---|---|
| KMIPKIT-REQ-SPEC-2.3-001 | 2.3 | FR-001, FR-005 | `managed_object.rs`; opaque context compatibility tests |
| KMIPKIT-REQ-SPEC-2.3-002 | 2.3 | FR-001, FR-008 | Opaque Object tests; preserve local context separation |
| KMIPKIT-REQ-SPEC-2.3-003 | 2.3 | FR-001, FR-004 | Opaque Object required-field and negative tests |
| KMIPKIT-REQ-SPEC-2.4-001-001 | 2.4 | FR-001, FR-005 | PGP Key public-only/public-private preservation tests |
| KMIPKIT-REQ-SPEC-2.4-001-002 | 2.4 | FR-001 | PGP Key Version round-trip test |
| KMIPKIT-REQ-SPEC-2.4-002-001 | 2.4 | FR-001, FR-008 | Opaque PGP Key Block preservation tests |
| KMIPKIT-REQ-SPEC-2.4-002-002 | 2.4 | FR-008 | Not applicable: the standard condition is a PGP-aware client; KMIPKit does not parse/decompose PGP. Verified by preserving opaque Key Block and recording the boundary. |
| KMIPKIT-REQ-SPEC-2.7-001 | 2.7 | FR-002, FR-004 | Secret Data wrapped/plain Key Value tests |
| KMIPKIT-REQ-SPEC-2.8-001-001 | 2.8 | FR-001 | Split Key round-trip and part-distribution metadata tests |
| KMIPKIT-REQ-SPEC-2.8-001-002 | 2.8 | FR-004 | Split Key Identifier range tests |
| KMIPKIT-REQ-SPEC-2.8-003 | 2.8 | FR-004 | Prime Field Size conditional requiredness tests |
| KMIPKIT-REQ-SPEC-3.1-001 | 3.1 | FR-002, FR-003 | Key Compression Type known/future value tests |
| KMIPKIT-REQ-SPEC-3.1-002 | 3.1 | FR-004 | Wrapped Key Block requires Key Wrapping Data tests |
| KMIPKIT-REQ-SPEC-3.1-004-001 | 3.1 | FR-004 | Cryptographic Algorithm omission only with source information tests |
| KMIPKIT-REQ-SPEC-3.1-004-002 | 3.1 | FR-004 | Algorithm presence requires Cryptographic Length tests |
| KMIPKIT-REQ-SPEC-3.1-005-001 | 3.1 | FR-004 | Cryptographic Length omission rule tests |
| KMIPKIT-REQ-SPEC-3.1-005-002 | 3.1 | FR-004 | Length presence requires Cryptographic Algorithm tests |
| KMIPKIT-REQ-SPEC-3.1-006 | 3.1 | FR-004 | Unwrapped Key Block forbids Key Wrapping Data tests |
| KMIPKIT-REQ-SPEC-3.2-001 | 3.2 | FR-002, FR-003 | Key Material and optional Attributes tests |
| KMIPKIT-REQ-SPEC-3.3-001-001 | 3.3 | FR-002, FR-005 | Wrapping metadata construction tests |
| KMIPKIT-REQ-SPEC-3.3-001-002 | 3.3 | FR-002, FR-004 | Optional wrapping metadata omission tests |
| KMIPKIT-REQ-SPEC-3.3-002 | 3.3 | FR-004 | Required IV/counter/nonce cases from each wrapping method tested; no cryptographic inference |
| KMIPKIT-REQ-SPEC-3.3-003 | 3.3 | FR-004 | `No Encoding` forbids Key Value Attributes negative test |
| KMIPKIT-REQ-SPEC-3.3-004 | 3.3 | FR-004 | At least one encryption or MAC/signature key-information structure test |
| KMIPKIT-REQ-SPEC-3.3-006 | 3.3 | FR-004 | Absent Encoding Option preserves TTLV encoding test |
| KMIPKIT-REQ-SPEC-3.3-009 | 3.3 | FR-002, FR-005 | Required MAC/signature key identifier tests |
| KMIPKIT-REQ-SPEC-3.1-007 | 3.1 | FR-002 | Key Format Type required field test |
| KMIPKIT-REQ-SPEC-3.2-003 | 3.2 | FR-003 | Key Material required field test |
| KMIPKIT-REQ-SPEC-3.3-010 | 3.3 | FR-002 | Wrapping Method required field test |
| KMIPKIT-REQ-SPEC-3.3-011 | 3.3 | FR-002 | Key information Unique Identifier required field test |
| KMIPKIT-REQ-SPEC-3.4-002 | 3.4 | FR-003 | Transparent Symmetric Key Key field test |
| KMIPKIT-REQ-SPEC-3.5-002 | 3.5 | FR-003 | DSA private P field test |
| KMIPKIT-REQ-SPEC-3.5-003 | 3.5 | FR-003 | DSA private Q field test |
| KMIPKIT-REQ-SPEC-3.5-004 | 3.5 | FR-003 | DSA private G field test |
| KMIPKIT-REQ-SPEC-3.5-005 | 3.5 | FR-003 | DSA private X field test |
| KMIPKIT-REQ-SPEC-3.6-002 | 3.6 | FR-003 | DSA public P field test |
| KMIPKIT-REQ-SPEC-3.6-004 | 3.6 | FR-003 | DSA public G field test |
| KMIPKIT-REQ-SPEC-3.6-005 | 3.6 | FR-003 | DSA public Y field test |
| KMIPKIT-REQ-SPEC-3.7-002 | 3.7 | FR-003, FR-004 | RSA private alternative field-set tests |
| KMIPKIT-REQ-SPEC-3.7-003 | 3.7 | FR-003 | RSA private Modulus field test |
| KMIPKIT-REQ-SPEC-3.8-002 | 3.8 | FR-003 | RSA public Modulus field test |
| KMIPKIT-REQ-SPEC-3.8-003 | 3.8 | FR-003 | RSA public exponent field test |
| KMIPKIT-REQ-SPEC-3.9-002 | 3.9 | FR-003 | DH private P field test |
| KMIPKIT-REQ-SPEC-3.9-003 | 3.9 | FR-003 | DH private G field test |
| KMIPKIT-REQ-SPEC-3.9-004 | 3.9 | FR-003 | DH private X field test |
| KMIPKIT-REQ-SPEC-3.10-002 | 3.10 | FR-003 | DH public P field test |
| KMIPKIT-REQ-SPEC-3.10-003 | 3.10 | FR-003 | DH public G field test |
| KMIPKIT-REQ-SPEC-3.10-004 | 3.10 | FR-003 | DH public Y field test |
| KMIPKIT-REQ-SPEC-3.11-002 | 3.11 | FR-003 | EC private Recommended Curve field test |
| KMIPKIT-REQ-SPEC-3.11-003 | 3.11 | FR-003 | EC private D field test |
| KMIPKIT-REQ-SPEC-3.12-002 | 3.12 | FR-003 | EC public Recommended Curve field test |
| KMIPKIT-REQ-SPEC-3.12-003 | 3.12 | FR-003 | EC public Q String field test |
| KMIPKIT-REQ-SPEC-6.1.43-005 | 6.1.43 | FR-006 | Register fake-transport vector with typed wrapped Key Value and Attributes |

## Catalog records not treated as operative requirements

The catalog contains table-heading records whose summary explicitly says the
`REQUIRED` heading is metadata, not an operative protocol requirement. They
are accounted for in the gap matrix's requiredness columns and schema tests,
but are not reported as independent normative behavior. Requirements whose
only applicable behavior belongs to NIST key-type attributes or Re-Provision
are outside this feature and remain assigned to their existing scope.
