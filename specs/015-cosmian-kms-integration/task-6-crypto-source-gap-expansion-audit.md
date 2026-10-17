# Task 6 cryptographic source-gap expansion audit

Date: 2026-10-17. Offline inventory update only. No production code, test executor, live KMIP request, Compose, Docker, or network operation was changed or run in this scoped update.

## Provenance and classification

- Pinned Cosmian KMS source: tag `5.28.0`, target commit `4ab3ee65829a903abb71f1298f400657fcee7ec9`.
- Pinned normative source: OASIS KMIP Specification v2.1 OS, immutable local copy `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`.
- The added matrix rows cite the operation clauses/tables: Create §6.1.8 Tables 186–188; Create Key Pair §6.1.9 Tables 189–192; Decrypt §6.1.11 Tables 196–198; Encrypt §6.1.17 Tables 214–216; Sign §6.1.55 Tables 334–336; Signature Verify §6.1.56 Tables 337–339. Cryptographic Algorithm names/values are checked against §11.12 Table 443.
- Cosmian source-only numeric values MLKEM_512/768/1024, MLDSA_44/65/87, and SLHDSA_SHA2/SHAKE variants map to `0x00000039` through `0x0000004A` in `crate/kmip/src/kmip_2_1/kmip_types.rs:190-207` at the pinned commit. Those names and low numeric values are absent from the pinned OASIS 2.1 Cryptographic Algorithm enumeration. Their matrix rows therefore use `expected_outcome=source_documentation_discrepancy` and `wire_enum_values.*.classification=source_only_numeric_probe`; they are not represented as standard KMIP values or conformance cases. All added rows remain `observed_result.classification=not_run`.

## Added source-confirmed row groups

The inventory adds 146 not-run rows to `tests/integration/cosmian/capability-matrix.json` and the cryptographic case-ID list:

| Operation | Rows | Evidence and scope |
|---|---:|---|
| Create SymmetricKey | 26 | 13 algorithms × 8 and 8192 bits: AES, ChaCha20, ChaCha20Poly1305, HMAC-SHA-1/224/384/512, SHA3-224/256/384/512, SHAKE-128/256. Pinned common source constructor `crate/server/src/core/kms/other_kms_methods.rs:104-231` documents 8..8192-bit generation. These are key-generation endpoints; they do not assert that every length is usable by the corresponding cipher/hash/MAC operation. |
| Create Key Pair | 18 | Direct source-only MLKEM (512/768/1024), MLDSA (44/65/87), and 12 SLHDSA variants. Source dispatch: `crate/server/src/core/operations/create_key_pair.rs:436-501`; ML-KEM generation mapping: `crate/crypto/src/crypto/kem.rs:32-90`. |
| Create Key Pair, ConfigurableKEM | 6 | Post-only MLKEM-512/768 and P256/CURVE25519 hybrid × MLKEM-512/768. Outer ConfigurableKEM is vendor extension `0x88800003`; nested MLKEM numeric probes are separately marked `source_only_numeric_probe`. No MLKEM-1024 ConfigurableKEM row is added because the pinned KEM map documents only MLKEM-512/768 for that profile. |
| Encrypt / Decrypt | 6 | Direct MLKEM 512/768/1024 encapsulation and decapsulation source probes. Encrypt response semantics are shared secret in Data plus encapsulation in IVCounterNonce, not arbitrary plaintext encryption. Source: `crate/server/src/core/operations/encrypt.rs:603-625` and `crate/server/src/core/operations/decrypt.rs:358-387`. |
| Sign | 30 | 15 MLDSA/SLHDSA algorithms × raw Data or DigestedData field selection. Pinned one-shot PQC branch: `crate/server/src/core/operations/sign.rs:154-177`. |
| Signature Verify | 60 | 15 MLDSA/SLHDSA algorithms × raw/prehashed field selector × PublicKey/PrivateKey object. Pinned one-shot branch accepts either Data or DigestedData and extracts verification material from either key object: `crate/server/src/core/operations/signature_verify.rs:105-151`. |

The raw/prehashed distinction for PQC rows only selects Data versus DigestedData bytes accepted by the source path. It does not claim standardized prehash, context, or streaming semantics.

## Observed Create Key Pair outcomes

All 24 source-discrepancy Create Key Pair probes returned typed success. KMIPKit validated both the private and public Unique Identifier fields, so these cases count as observed KMIPKit–Cosmian interoperability. The cases remain marked as source/documentation discrepancies because their algorithm values are outside the pinned KMIP 2.1 Cryptographic Algorithm enumeration; they do not establish KMIP 2.1 conformance for those values. Two additional ConfigurableKEM curve-only profiles were rejected and remain failures.

## Conservative exclusions

- No extra HMAC-SHA-256 Create endpoints: both 8-bit and 8192-bit rows already existed.
- FPE_FF1 and THREE_DES are not in this common Create constructor branch; their separate special branches were not extrapolated into this group.
- HMAC-SHA3 is absent from the pinned Create match, even though HMAC-SHA3 algorithms appear in other source paths; no HMAC-SHA3 Create rows were added.
- No ConfigurableKEM MLKEM-1024 profile or unlisted pre-quantum/hybrid combination was inferred.
- No streaming PQC Sign/Verify rows or standardized PQC prehash claims were inferred from the one-shot source branch.
- Runtime provider, non-FIPS build, and server configuration behavior remain unobserved.

The 42 existing symmetric Create rows that incorrectly cited `allowed_dsa_key_sizes_bits` now point to the pinned shared symmetric Create constructor source-catalog entry. No other existing case outcome or ID was changed.

## Red / Green evidence

- Red: after inserting the 146 source rows while retaining the old ID list, an offline JSON/ID parity check reported `manifest=1466, IDs=1320, missing=146, extra=0` and failed as expected.
- Red commit (DCO): `5ef540d73ba2bb695dfb1a1b1bf5e4e9607aa3b7` (`test: expand cryptographic source inventory`).
- Green: added the 146 manifest case IDs in sorted unique order. The offline Python inventory check validated JSON parsing, whole-manifest ID uniqueness, the exact per-operation additions, not-run/outcome classifications, source-catalog reference resolution, preservation of every baseline crypto ID, and exact manifest/list parity. Result: `GREEN offline inventory validation passed`; `cases=6429 crypto=1466 ids=1466 added=146`; group counts were Create 26, Create Key Pair 24, Encrypt 3, Decrypt 3, Sign 30, Signature Verify 60. Added outcomes were 26 `typed_success` expected rows and 120 `source_documentation_discrepancy` probes, all observed `not_run`.
- Green command: run a Python JSON/inventory validator over `tests/integration/cosmian/capability-matrix.json` and `crates/kmipkit-client/tests/cosmian_kms/cryptographic-case-ids.txt`; it returned exit code 0 with the counts above. No Rust tests, live Compose, Docker, or network tests were run.
- Green commit (DCO): recorded after committing the synchronized IDs and this audit note.
